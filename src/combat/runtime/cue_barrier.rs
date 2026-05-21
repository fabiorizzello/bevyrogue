use std::collections::VecDeque;

use bevy::{
    log::{debug, info, warn},
    prelude::{Resource, World},
};

use crate::combat::{
    runtime::{
        clock::Clock,
        intent::{CastId, Intent},
        runner::BeatRunner,
    },
    state::InFlightAction,
    types::SkillId,
};

/// Runtime-selected execution clock for timeline-backed actions.
///
/// `Clock` stays the runner-level enum; this resource lets the action pipeline
/// choose whether newly-started timelines should auto-complete (`HeadlessAuto`)
/// or suspend at presentation barriers (`Windowed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub struct TimelineClock(pub Clock);

/// Bounded frame budget for a windowed cue barrier before the runtime force-resumes.
pub const CUE_BARRIER_TIMEOUT_FRAMES: u32 = 180;

/// Inspectable snapshot of the currently-latched cue barrier.
///
/// Animation-specific fields are optional so headless tests and early windowed
/// plumbing can expose deterministic barrier state before the animation bridge
/// starts annotating node/frame information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueBarrierStatus {
    pub cast_id: CastId,
    pub skill_id: SkillId,
    pub timeline_id: &'static str,
    pub beat_id: &'static str,
    pub cue_id: &'static str,
    pub awaiting_release: bool,
    pub released: bool,
    pub timed_out: bool,
    pub waited_frames: u32,
    pub timeout_frames: u32,
    pub animation_node: Option<String>,
    pub animation_frame: Option<usize>,
    /// `Some(n)` when the barrier is inside a loop body at iteration `n`.
    /// `None` for linear (non-loop) presentation beats.
    pub hop_index: Option<u32>,
}

impl CueBarrierStatus {
    fn awaiting(
        cast_id: CastId,
        skill_id: SkillId,
        timeline_id: &'static str,
        beat_id: &'static str,
        cue_id: &'static str,
        hop_index: Option<u32>,
    ) -> Self {
        Self {
            cast_id,
            skill_id,
            timeline_id,
            beat_id,
            cue_id,
            awaiting_release: true,
            released: false,
            timed_out: false,
            waited_frames: 0,
            timeout_frames: CUE_BARRIER_TIMEOUT_FRAMES,
            animation_node: None,
            animation_frame: None,
            hop_index,
        }
    }

    fn mark_released(&mut self) {
        self.awaiting_release = false;
        self.released = true;
    }

    fn tick_wait(&mut self) {
        self.waited_frames = self.waited_frames.saturating_add(1);
    }

    fn mark_timed_out(&mut self) {
        self.mark_released();
        self.timed_out = true;
        self.waited_frames = self.waited_frames.max(self.timeout_frames);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CueReleaseResult {
    Released,
    TimedOut,
    DuplicateRelease,
    NoSuspendedTimeline,
    CueMismatch,
}

pub struct SuspendedTimeline {
    pub runner: BeatRunner,
    pub pending: VecDeque<Intent>,
    pub inflight: InFlightAction,
    pub cast_id: CastId,
    pub status: CueBarrierStatus,
    pub release_requested: bool,
    just_suspended: bool,
}

impl SuspendedTimeline {
    pub fn new(
        runner: BeatRunner,
        pending: VecDeque<Intent>,
        inflight: InFlightAction,
        cast_id: CastId,
    ) -> Self {
        let awaiting = runner
            .awaiting_cue_info()
            .expect("suspended timeline must carry awaiting cue info");
        let mut status = CueBarrierStatus::awaiting(
            cast_id,
            inflight.action.skill_id.clone(),
            runner.timeline_id(),
            awaiting.beat_id,
            awaiting.cue_id,
            awaiting.hop_index,
        );
        if let Some(animation_node) = awaiting.animation_node {
            status.animation_node = Some(animation_node.to_string());
        }
        Self {
            runner,
            pending,
            inflight,
            cast_id,
            status,
            release_requested: false,
            just_suspended: true,
        }
    }
}

#[derive(Default, Resource)]
pub struct SuspendedTimelineState {
    current: Option<SuspendedTimeline>,
    last_status: Option<CueBarrierStatus>,
    last_release_result: Option<CueReleaseResult>,
    last_message: Option<String>,
}

impl SuspendedTimelineState {
    pub fn active_status(&self) -> Option<&CueBarrierStatus> {
        self.current.as_ref().map(|current| &current.status)
    }

    pub fn last_status(&self) -> Option<&CueBarrierStatus> {
        self.last_status.as_ref()
    }

    pub fn last_release_result(&self) -> Option<CueReleaseResult> {
        self.last_release_result
    }

    pub fn last_message(&self) -> Option<&str> {
        self.last_message.as_deref()
    }

    pub fn suspend(&mut self, suspended: SuspendedTimeline) {
        let status = suspended.status.clone();
        let msg = format!(
            "timeline cue barrier awaiting cast_id={:?} skill_id={:?} timeline={} beat_id={} cue_id={} hop_index={} waited_frames={} timeout_frames={} anim_node={} anim_frame={}",
            status.cast_id,
            status.skill_id,
            status.timeline_id,
            status.beat_id,
            status.cue_id,
            status
                .hop_index
                .map(|h| h.to_string())
                .unwrap_or_else(|| "none".to_string()),
            status.waited_frames,
            status.timeout_frames,
            status.animation_node.as_deref().unwrap_or("none"),
            status
                .animation_frame
                .map(|frame| frame.to_string())
                .unwrap_or_else(|| "none".to_string()),
        );
        info!(target: "combat.timeline_barrier", "{msg}");
        self.last_status = Some(status);
        self.last_release_result = None;
        self.last_message = Some(msg);
        self.current = Some(suspended);
    }

    pub fn take_released(&mut self) -> Option<SuspendedTimeline> {
        match self.current.as_ref() {
            Some(current) if current.release_requested => self.current.take(),
            _ => None,
        }
    }

    pub fn note_completion(&mut self, cast_id: CastId, skill_id: &SkillId) {
        let tail = format!(
            "timeline cue barrier cleared after completion cast_id={cast_id:?} skill_id={skill_id:?}"
        );
        let msg = if self.last_release_result == Some(CueReleaseResult::TimedOut) {
            format!(
                "{} | post_timeout_outcome=completed",
                self.last_message.as_deref().unwrap_or(&tail)
            )
        } else {
            tail
        };
        info!(target: "combat.timeline_barrier", "{msg}");
        self.last_message = Some(msg);
        self.current = None;
    }

    pub fn note_failure(&mut self, cast_id: CastId, skill_id: &SkillId, reason: &str) {
        let tail = format!(
            "timeline cue barrier cleared after failure cast_id={cast_id:?} skill_id={skill_id:?} reason={reason}"
        );
        let msg = if self.last_release_result == Some(CueReleaseResult::TimedOut) {
            format!(
                "{} | post_timeout_outcome=failed reason={reason}",
                self.last_message.as_deref().unwrap_or(&tail)
            )
        } else {
            tail
        };
        warn!(target: "combat.timeline_barrier", "{msg}");
        self.last_message = Some(msg);
        self.current = None;
    }

    pub fn annotate_active_animation(&mut self, node: &str, frame: usize) {
        if let Some(current) = self.current.as_mut() {
            current.status.animation_node = Some(node.to_string());
            current.status.animation_frame = Some(frame);
        }
    }

    pub fn tick_timeout(&mut self) -> bool {
        let Some(current) = self.current.as_mut() else {
            return false;
        };
        if current.release_requested || !current.status.awaiting_release {
            return false;
        }
        if current.just_suspended {
            current.just_suspended = false;
            return false;
        }

        current.status.tick_wait();
        if current.status.waited_frames < current.status.timeout_frames {
            return false;
        }

        current.release_requested = true;
        current.status.mark_timed_out();
        let snapshot = current.status.clone();
        let msg = format!(
            "timeline cue barrier timed out: force-resuming cast_id={:?} skill_id={:?} timeline={} beat_id={} cue_id={} hop_index={} waited_frames={} timeout_frames={} anim_node={} anim_frame={}",
            snapshot.cast_id,
            snapshot.skill_id,
            snapshot.timeline_id,
            snapshot.beat_id,
            snapshot.cue_id,
            snapshot
                .hop_index
                .map(|h| h.to_string())
                .unwrap_or_else(|| "none".to_string()),
            snapshot.waited_frames,
            snapshot.timeout_frames,
            snapshot.animation_node.as_deref().unwrap_or("none"),
            snapshot
                .animation_frame
                .map(|frame| frame.to_string())
                .unwrap_or_else(|| "none".to_string()),
        );

        warn!(target: "combat.timeline_barrier", "{msg}");
        self.last_status = Some(snapshot);
        self.last_release_result = Some(CueReleaseResult::TimedOut);
        self.last_message = Some(msg);
        true
    }

    pub fn request_release(&mut self, requested_cue_id: &str) -> CueReleaseResult {
        let (result, snapshot, msg) = match self.current.as_mut() {
            None => {
                let msg = format!(
                    "timeline cue release ignored: no suspended timeline for cue_id={requested_cue_id}"
                );
                (CueReleaseResult::NoSuspendedTimeline, None, msg)
            }
            Some(current) if current.release_requested => {
                let outcome = if current.status.timed_out {
                    "timed_out"
                } else {
                    "duplicate_release"
                };
                let msg = format!(
                    "timeline cue release ignored: {outcome} cast_id={:?} skill_id={:?} beat_id={} cue_id={} requested_cue_id={requested_cue_id} waited_frames={} timeout_frames={}",
                    current.status.cast_id,
                    current.status.skill_id,
                    current.status.beat_id,
                    current.status.cue_id,
                    current.status.waited_frames,
                    current.status.timeout_frames,
                );
                (
                    if current.status.timed_out {
                        CueReleaseResult::TimedOut
                    } else {
                        CueReleaseResult::DuplicateRelease
                    },
                    Some(current.status.clone()),
                    msg,
                )
            }
            Some(current) if current.status.cue_id != requested_cue_id => {
                let msg = format!(
                    "timeline cue release ignored: cue mismatch cast_id={:?} skill_id={:?} beat_id={} expected_cue_id={} requested_cue_id={requested_cue_id}",
                    current.status.cast_id,
                    current.status.skill_id,
                    current.status.beat_id,
                    current.status.cue_id,
                );
                (
                    CueReleaseResult::CueMismatch,
                    Some(current.status.clone()),
                    msg,
                )
            }
            Some(current) => {
                current.release_requested = true;
                current.status.mark_released();
                let msg = format!(
                    "timeline cue release accepted cast_id={:?} skill_id={:?} beat_id={} cue_id={} waited_frames={} timeout_frames={} anim_node={} anim_frame={}",
                    current.status.cast_id,
                    current.status.skill_id,
                    current.status.beat_id,
                    current.status.cue_id,
                    current.status.waited_frames,
                    current.status.timeout_frames,
                    current.status.animation_node.as_deref().unwrap_or("none"),
                    current
                        .status
                        .animation_frame
                        .map(|frame| frame.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                );
                (
                    CueReleaseResult::Released,
                    Some(current.status.clone()),
                    msg,
                )
            }
        };

        match result {
            CueReleaseResult::Released => info!(target: "combat.timeline_barrier", "{msg}"),
            CueReleaseResult::DuplicateRelease | CueReleaseResult::NoSuspendedTimeline => {
                debug!(target: "combat.timeline_barrier", "{msg}")
            }
            CueReleaseResult::TimedOut | CueReleaseResult::CueMismatch => {
                warn!(target: "combat.timeline_barrier", "{msg}")
            }
        }

        self.last_release_result = Some(result);
        if let Some(snapshot) = snapshot {
            self.last_status = Some(snapshot);
        }
        self.last_message = Some(msg);

        result
    }
}

/// Request release of the currently suspended timeline barrier.
///
/// Future windowed animation systems can call this when a `ReleaseKernelCue`
/// fires. Duplicate calls and calls with no suspended timeline are explicit,
/// inspectable no-ops.
pub fn request_timeline_cue_release(world: &mut World, requested_cue_id: &str) -> CueReleaseResult {
    world.init_resource::<SuspendedTimelineState>();
    world
        .resource_mut::<SuspendedTimelineState>()
        .request_release(requested_cue_id)
}
