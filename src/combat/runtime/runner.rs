use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

use bevy::log;
use bevy::prelude::World;

use crate::combat::{
    runtime::{
        clock::Clock,
        intent::{CastId, Intent},
        registry::ExtRegistries,
        runner_common::find_beat,
        skill_ctx::SkillCtxMode,
        timeline::{Beat, BeatEvent, BeatId, BeatKind, CompiledTimeline},
    },
    types::UnitId,
};

/// Maximum loop iterations before the circuit breaker fires.
const MAX_HOPS: u32 = 256;

/// Loop iteration state while the runner is executing a `BeatKind::Loop` body.
///
/// Single-level only for S02; S05 lifts the nested-loop restriction.
#[derive(Debug, Clone)]
pub struct LoopFrame {
    /// The Loop beat in the outer timeline that owns this body.
    pub enclosing_loop_beat: BeatId,
    /// Index into `body[..]` of the currently executing beat.
    pub body_cursor: usize,
    /// Iteration counter (0-based). Exposed to predicates via `BeatEvent.hop_index`.
    pub hop_index: u32,
    /// Predicate key: when it returns `true` after a full body pass, exit the loop.
    pub exit_when: &'static str,
}

/// Outcome of a single `BeatRunner::step` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    /// Beat executed; cursor advanced; timeline still running.
    Advanced,
    /// Loop body completed one full pass and the loop exited.
    LoopExited,
    /// No more beats; the timeline finished normally.
    Done,
    /// Loop iterated `MAX_HOPS` (256) times without `exit_when` returning `true`.
    /// A `bevy::log::warn!` is emitted with cast_id, timeline id, and hop count.
    Halted,
    /// Clock::Windowed only: a Presentation-bearing beat fired its hook and is now
    /// stalling. Call `resume_cue()` to unlatch, then `step()` or
    /// `run_to_completion()` again to advance normally.
    /// HeadlessAuto never returns this.
    AwaitingCue,
}

/// Read-only snapshot of the currently latched presentation barrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AwaitingCueInfo {
    pub beat_id: BeatId,
    pub cue_id: &'static str,
    pub animation_node: Option<&'static str>,
}

/// FSM engine that drives a `CompiledTimeline` beat-by-beat.
///
/// Callers advance the runner via repeated `step` calls, or drive it
/// to completion with `run_to_completion`. All mutations are emitted as
/// `Intent` values appended to the caller-owned `pending` queue — the runner
/// never mutates Bevy world state directly.
pub struct BeatRunner {
    timeline: Arc<CompiledTimeline>,
    cast_id: CastId,
    caster: UnitId,
    primary_target: UnitId,
    /// `None` once the timeline has finished.
    cursor: Option<BeatId>,
    /// Active loop frame stack. Depth is at most 1 for S02.
    loop_stack: Vec<LoopFrame>,
    /// Units hit by `Intent::DealDamage` during this cast.
    /// NoRepeat / bounce selectors read this via `SkillCtx::cast_hit_set`.
    cast_hit_set: HashSet<UnitId>,
    /// Targets resolved by the most recent `BeatKind::Impact` beat.
    /// F6 fix: updated after every hook fires so edge-gate predicates see
    /// running target lists rather than the empty `BeatEvent` default.
    last_beat_targets: Vec<UnitId>,
    /// Execution clock: HeadlessAuto (never stalls) or Windowed (stalls on Presentation).
    clock: Clock,
    /// Some(beat_id) when we fired a Presentation-bearing beat (Windowed) and are
    /// waiting for `resume_cue()`. `step()` returns `AwaitingCue` while this is set.
    awaiting_cue: Option<BeatId>,
    /// Set by `resume_cue()`. Tells the next `step()` to skip re-firing the
    /// already-fired presentation beat and advance cursor/body_cursor directly.
    cue_just_resumed: bool,
}

impl BeatRunner {
    pub fn new(
        timeline: Arc<CompiledTimeline>,
        cast_id: CastId,
        caster: UnitId,
        primary_target: UnitId,
    ) -> Self {
        let entry = timeline.entry;
        Self {
            timeline,
            cast_id,
            caster,
            primary_target,
            cursor: Some(entry),
            loop_stack: Vec::new(),
            cast_hit_set: HashSet::new(),
            last_beat_targets: Vec::new(),
            clock: Clock::HeadlessAuto,
            awaiting_cue: None,
            cue_just_resumed: false,
        }
    }

    /// Override the clock mode (builder pattern — keeps `new` signature stable).
    // Consumed by tests/timeline_two_clock_parity.rs.
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    /// Current cursor beat, or `None` when the timeline has finished.
    pub fn cursor(&self) -> Option<BeatId> {
        self.cursor
    }

    /// Entry beat for the timeline.
    pub fn entry(&self) -> BeatId {
        self.timeline.entry
    }

    pub fn timeline_id(&self) -> &'static str {
        self.timeline.id
    }

    pub fn awaiting_cue_info(&self) -> Option<AwaitingCueInfo> {
        let beat_id = self.awaiting_cue?;
        let beat = find_beat(&self.timeline, beat_id);
        let presentation = beat.presentation.as_ref()?;
        Some(AwaitingCueInfo {
            beat_id,
            cue_id: presentation.cue_id,
            animation_node: presentation.anim,
        })
    }

    /// Whether the runner is currently inside a `BeatKind::Loop` body.
    pub fn in_loop(&self) -> bool {
        !self.loop_stack.is_empty()
    }

    /// Advance the FSM by one beat.
    ///
    /// - `Done` is returned when the timeline has no more beats.
    /// - `Halted` is returned when a loop exceeds `MAX_HOPS` (256) with a warn!.
    /// - `AwaitingCue` is returned (Clock::Windowed only) when a Presentation-bearing
    ///   beat fired its hook and is now stalling. Call `resume_cue()` to unlatch.
    /// - `SkillCtxMode` is forwarded to hooks and predicates so S03 can flip
    ///   to `DryRun` without an API change.
    pub fn step(
        &mut self,
        world: &mut World,
        regs: &ExtRegistries,
        mode: SkillCtxMode,
        pending: &mut VecDeque<Intent>,
    ) -> StepOutcome {
        // Global stall gate: don't re-enter until resume_cue() clears the latch.
        if self.awaiting_cue.is_some() {
            return StepOutcome::AwaitingCue;
        }

        // Consume the resume flag at the top so both paths can read it.
        let just_resumed = self.cue_just_resumed;
        self.cue_just_resumed = false;

        // ── Loop body path ────────────────────────────────────────────────────
        if !self.loop_stack.is_empty() {
            // Copy all frame fields (all Copy types) before any &mut self calls.
            let enclosing;
            let body_cursor;
            let hop_index;
            let exit_when;
            {
                let frame = self.loop_stack.last().unwrap();
                enclosing = frame.enclosing_loop_beat;
                body_cursor = frame.body_cursor;
                hop_index = frame.hop_index;
                exit_when = frame.exit_when;
            }

            if hop_index >= MAX_HOPS {
                self.loop_stack.pop();
                log::warn!(
                    "BeatRunner circuit-breaker: cast_id={:?} timeline={} halted at hop_index={}",
                    self.cast_id,
                    self.timeline.id,
                    hop_index
                );
                return StepOutcome::Halted;
            }

            let cur_beat: Beat = {
                let tl = Arc::clone(&self.timeline);
                let lb = find_beat(&tl, enclosing);
                match &lb.kind {
                    BeatKind::Loop { body, .. } => body[body_cursor].clone(),
                    _ => unreachable!("loop_stack frame points at non-Loop beat `{enclosing}`"),
                }
            };

            if !just_resumed {
                let params = self.make_params(world, regs, mode, pending);
                let beat_targets =
                    crate::combat::runtime::runner_common::fire_beat(&cur_beat, hop_index, params);
                if matches!(cur_beat.kind, BeatKind::Impact) {
                    self.last_beat_targets = beat_targets;
                }

                // Windowed stall: presentation beat fired once, now latch.
                if cur_beat.presentation.is_some() && self.clock == Clock::Windowed {
                    self.awaiting_cue = Some(cur_beat.id);
                    return StepOutcome::AwaitingCue;
                }
            }

            let body_len: usize = {
                let tl = Arc::clone(&self.timeline);
                let lb = find_beat(&tl, enclosing);
                match &lb.kind {
                    BeatKind::Loop { body, .. } => body.len(),
                    _ => unreachable!(),
                }
            };

            if body_cursor + 1 < body_len {
                self.loop_stack.last_mut().unwrap().body_cursor += 1;
                return StepOutcome::Advanced;
            }

            // End of body pass — evaluate `exit_when`.
            let exit_evt = BeatEvent {
                cast_id: self.cast_id,
                beat_id: enclosing,
                hop_index,
                beat_targets: self.last_beat_targets.clone(),
            };
            let mut params = self.make_params(world, regs, mode, pending);
            let should_exit = crate::combat::runtime::runner_common::eval_predicate(
                exit_when,
                &exit_evt,
                &mut params,
            );
            if should_exit {
                self.loop_stack.pop();
                // F1: first passing edge from the enclosing Loop beat becomes the next cursor.
                let mut params = self.make_params(world, regs, mode, pending);
                let next = crate::combat::runtime::runner_common::next_beat(
                    enclosing,
                    &exit_evt,
                    &mut params,
                );
                self.cursor = next;
                return StepOutcome::LoopExited;
            } else {
                let frame = self.loop_stack.last_mut().unwrap();
                frame.body_cursor = 0;
                frame.hop_index += 1;
                return StepOutcome::Advanced;
            }
        }

        // ── Linear path ───────────────────────────────────────────────────────
        let Some(beat_id) = self.cursor else {
            return StepOutcome::Done;
        };

        let beat: Beat = {
            let tl = Arc::clone(&self.timeline);
            find_beat(&tl, beat_id).clone()
        };

        match &beat.kind {
            BeatKind::Loop { body, exit_when } => {
                // Loop beat entry: push a frame; body beats execute in subsequent steps.
                debug_assert!(
                    self.loop_stack.is_empty(),
                    "BeatRunner: nested Loop is not supported in S02 (S05 lifts this restriction)"
                );
                if body.is_empty() {
                    unreachable!(
                        "Loop beat `{beat_id}` has an empty body — validate_timeline_structure catches this before runtime"
                    );
                }
                self.loop_stack.push(LoopFrame {
                    enclosing_loop_beat: beat_id,
                    body_cursor: 0,
                    hop_index: 0,
                    exit_when: *exit_when,
                });
                StepOutcome::Advanced
            }
            _ => {
                if !just_resumed {
                    let params = self.make_params(world, regs, mode, pending);
                    let beat_targets =
                        crate::combat::runtime::runner_common::fire_beat(&beat, 0, params);
                    if matches!(beat.kind, BeatKind::Impact) {
                        self.last_beat_targets = beat_targets;
                    }

                    // Windowed stall: presentation beat fired once, now latch.
                    if beat.presentation.is_some() && self.clock == Clock::Windowed {
                        self.awaiting_cue = Some(beat_id);
                        return StepOutcome::AwaitingCue;
                    }
                }

                let gate_evt = BeatEvent {
                    cast_id: self.cast_id,
                    beat_id,
                    hop_index: 0,
                    beat_targets: self.last_beat_targets.clone(),
                };
                let mut params = self.make_params(world, regs, mode, pending);
                let next = crate::combat::runtime::runner_common::next_beat(
                    beat_id,
                    &gate_evt,
                    &mut params,
                );
                self.cursor = next;
                if self.cursor.is_none() {
                    StepOutcome::Done
                } else {
                    StepOutcome::Advanced
                }
            }
        }
    }

    /// Unlatch a `StepOutcome::AwaitingCue` stall (Clock::Windowed only).
    ///
    /// The subsequent `step()` or `run_to_completion()` call will advance
    /// cursor/body_cursor normally without re-firing the stalled beat's hook.
    /// Calling this when no cue is latched is a harmless no-op.
    pub fn resume_cue(&mut self) {
        if self.awaiting_cue.is_some() {
            self.awaiting_cue = None;
            self.cue_just_resumed = true;
        }
    }

    /// Drive the runner until it either finishes or hits a cue barrier.
    ///
    /// - `Clock::HeadlessAuto` keeps stepping until `Done`/`Halted`.
    /// - `Clock::Windowed` returns `AwaitingCue` as soon as a presentation beat
    ///   latches; callers must invoke `resume_cue()` and call this again.
    ///
    /// Panics if `max_steps` is exceeded (a safety net for bugs, not the loop guard).
    pub fn run_to_completion(
        &mut self,
        world: &mut World,
        regs: &ExtRegistries,
        mode: SkillCtxMode,
        pending: &mut VecDeque<Intent>,
        max_steps: usize,
    ) -> StepOutcome {
        for _ in 0..max_steps {
            let outcome = self.step(world, regs, mode, pending);
            match outcome {
                StepOutcome::Done | StepOutcome::Halted => return outcome,
                StepOutcome::AwaitingCue if self.clock == Clock::Windowed => {
                    return StepOutcome::AwaitingCue;
                }
                StepOutcome::AwaitingCue => {
                    self.resume_cue();
                }
                _ => {}
            }
        }
        panic!(
            "BeatRunner::run_to_completion exceeded max_steps={max_steps}; possible infinite loop"
        );
    }

    // ─── Private helpers ──────────────────────────────────────────────────────

    fn make_params<'a, 'w>(
        &'a mut self,
        world: &'w World,
        regs: &'a ExtRegistries,
        mode: SkillCtxMode,
        pending: &'a mut VecDeque<Intent>,
    ) -> crate::combat::runtime::runner_common::RunnerParams<'a, 'w> {
        crate::combat::runtime::runner_common::RunnerParams {
            timeline: &self.timeline,
            caster: self.caster,
            primary_target: self.primary_target,
            cast_id: self.cast_id,
            cast_hit_set: &mut self.cast_hit_set,
            world,
            regs,
            mode,
            pending,
        }
    }
}

// ─── Inline unit tests ────────────────────────────────────────────────────────
#[cfg(test)]
mod tests;
