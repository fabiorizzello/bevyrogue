use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

use bevy::log;
use bevy::prelude::World;

use crate::combat::{
    api::{
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
    /// stalling. Call `resume_cue()` to unlatch, then `step()` to advance normally.
    /// Never returned by `run_to_completion` (auto-resumed) or HeadlessAuto.
    AwaitingCue,
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
                    _ => panic!("loop_stack frame points at non-Loop beat `{enclosing}`"),
                }
            };

            if !just_resumed {
                let params = self.make_params(world, regs, mode, pending);
                let beat_targets =
                    crate::combat::api::runner_common::fire_beat(&cur_beat, hop_index, params);
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
            let should_exit = crate::combat::api::runner_common::eval_predicate(
                exit_when,
                &exit_evt,
                &mut params,
            );
            if should_exit {
                self.loop_stack.pop();
                // F1: first passing edge from the enclosing Loop beat becomes the next cursor.
                let mut params = self.make_params(world, regs, mode, pending);
                let next =
                    crate::combat::api::runner_common::next_beat(enclosing, &exit_evt, &mut params);
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
                    panic!("Loop beat `{beat_id}` has an empty body — invalid timeline");
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
                        crate::combat::api::runner_common::fire_beat(&beat, 0, params);
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
                let next =
                    crate::combat::api::runner_common::next_beat(beat_id, &gate_evt, &mut params);
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
    /// The subsequent `step()` call will advance cursor/body_cursor normally
    /// without re-firing the stalled beat's hook.
    pub fn resume_cue(&mut self) {
        self.awaiting_cue = None;
        self.cue_just_resumed = true;
    }

    /// Drive the runner to completion, calling `step` repeatedly.
    ///
    /// `StepOutcome::AwaitingCue` is auto-resumed (Windowed cues are not awaited
    /// in batch mode), preserving identical S02 drive-to-completion semantics.
    /// Returns `Done` on normal finish, `Halted` on MAX_HOPS circuit-breaker.
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
    ) -> crate::combat::api::runner_common::RunnerParams<'a, 'w> {
        crate::combat::api::runner_common::RunnerParams {
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
mod tests {
    use super::*;
    use crate::combat::api::{
        clock::Clock,
        intent::CastId,
        registry::ExtRegistries,
        skill_ctx::{SkillCtx, SkillCtxMode},
        timeline::{Beat, BeatEdge, BeatEvent, BeatKind, CompiledTimeline, Presentation},
    };
    use std::{collections::VecDeque, num::NonZeroU32, sync::Arc};

    fn cast_id() -> CastId {
        CastId(NonZeroU32::new(2).unwrap())
    }

    const CASTER: UnitId = UnitId(1);
    const TARGET: UnitId = UnitId(2);

    // ── Test (a): linear 2-beat timeline fires both beats and returns Done ─────

    #[test]
    fn linear_cast_impact_reaches_done() {
        // Register a hook that tracks call count.
        use std::sync::atomic::{AtomicU32, Ordering};
        static HOOK_CALLS: AtomicU32 = AtomicU32::new(0);

        fn counting_hook(_evt: &BeatEvent, _ctx: &mut SkillCtx<'_>) {
            HOOK_CALLS.fetch_add(1, Ordering::Relaxed);
        }

        let mut regs = ExtRegistries::default();
        regs.hooks.register("count_hook", counting_hook);

        let timeline = Arc::new(CompiledTimeline {
            id: "linear_test",
            entry: "cast",
            beats: vec![
                Beat {
                    id: "cast",
                    kind: BeatKind::Cast,
                    hook: Some("count_hook"),
                    selector: None,
                    presentation: None,
                    payload: None,
                },
                Beat {
                    id: "impact",
                    kind: BeatKind::Impact,
                    hook: Some("count_hook"),
                    selector: None,
                    presentation: None,
                    payload: None,
                },
            ],
            edges: vec![BeatEdge {
                from: "cast",
                to: "impact",
                gate: None,
            }],
        });

        HOOK_CALLS.store(0, Ordering::Relaxed);
        let mut runner = BeatRunner::new(timeline, cast_id(), CASTER, TARGET);
        let mut world = World::new();
        let mut pending = VecDeque::new();

        let o1 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(o1, StepOutcome::Advanced);

        let o2 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(o2, StepOutcome::Done);

        assert_eq!(
            HOOK_CALLS.load(Ordering::Relaxed),
            2,
            "both beats should fire hooks"
        );
    }

    // ── Test (b): Loop with exit_when="always_true" exits after one body pass ──

    #[test]
    fn loop_with_always_true_exits_after_one_pass() {
        fn always_true(_evt: &BeatEvent, _ctx: &SkillCtx<'_>) -> bool {
            true
        }

        let mut regs = ExtRegistries::default();
        regs.predicates.register("always_true", always_true);

        let timeline = Arc::new(CompiledTimeline {
            id: "loop_exit_test",
            entry: "loop_beat",
            beats: vec![Beat {
                id: "loop_beat",
                kind: BeatKind::Loop {
                    body: vec![Beat {
                        id: "body_impact",
                        kind: BeatKind::Impact,
                        hook: None,
                        selector: None,
                        presentation: None,
                        payload: None,
                    }],
                    exit_when: "always_true",
                },
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            }],
            edges: vec![],
        });

        let mut runner = BeatRunner::new(timeline, cast_id(), CASTER, TARGET);
        let mut world = World::new();
        let mut pending = VecDeque::new();

        // Step 1: encounter Loop beat, push frame.
        let o1 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(o1, StepOutcome::Advanced);

        // Step 2: fire body[0] (Impact), eval exit_when → true, pop frame, cursor=None.
        let o2 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(o2, StepOutcome::LoopExited);

        // Step 3: cursor is None → Done.
        let o3 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(o3, StepOutcome::Done);
    }

    // ── Test (c): Loop with exit_when="never" → circuit-breaker at MAX_HOPS ───

    #[test]
    fn loop_with_never_exit_halts_at_circuit_breaker() {
        fn never(_evt: &BeatEvent, _ctx: &SkillCtx<'_>) -> bool {
            false
        }

        let mut regs = ExtRegistries::default();
        regs.predicates.register("never", never);

        let timeline = Arc::new(CompiledTimeline {
            id: "loop_halt_test",
            entry: "loop_beat",
            beats: vec![Beat {
                id: "loop_beat",
                kind: BeatKind::Loop {
                    body: vec![Beat {
                        id: "body_beat",
                        kind: BeatKind::Cast,
                        hook: None,
                        selector: None,
                        presentation: None,
                        payload: None,
                    }],
                    exit_when: "never",
                },
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            }],
            edges: vec![],
        });

        let mut runner = BeatRunner::new(timeline, cast_id(), CASTER, TARGET);
        let mut world = World::new();
        let mut pending = VecDeque::new();

        // max_steps must exceed MAX_HOPS (256) iterations. Each iteration = 1 step,
        // plus 1 step for loop entry, plus 1 for the Halted check = ~258 total.
        let outcome =
            runner.run_to_completion(&mut world, &regs, SkillCtxMode::Execute, &mut pending, 1000);
        assert_eq!(outcome, StepOutcome::Halted);
    }

    // ── Helpers for I3 / Windowed tests ───────────────────────────────────────

    /// Build a simple timeline: Cast (with presentation) → Impact (no presentation)
    fn presentation_timeline() -> Arc<CompiledTimeline> {
        Arc::new(CompiledTimeline {
            id: "presentation_test",
            entry: "cast",
            beats: vec![
                Beat {
                    id: "cast",
                    kind: BeatKind::Cast,
                    hook: Some("record_hook"),
                    selector: None,
                    presentation: Some(Presentation {
                        cue_id: "test_cue",
                        anim: None,
                        vfx: None,
                        sfx: None,
                    }),
                    payload: None,
                },
                Beat {
                    id: "impact",
                    kind: BeatKind::Impact,
                    hook: Some("record_hook"),
                    selector: None,
                    presentation: None,
                    payload: None,
                },
            ],
            edges: vec![BeatEdge {
                from: "cast",
                to: "impact",
                gate: None,
            }],
        })
    }

    // ── Test (d): HeadlessAuto regression — presentation-bearing timeline ──────

    #[test]
    fn headless_auto_presentation_reaches_done_no_awaiting_cue() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static HEADLESS_HOOK_CALLS: AtomicUsize = AtomicUsize::new(0);
        fn headless_hook(_evt: &BeatEvent, _ctx: &mut SkillCtx<'_>) {
            HEADLESS_HOOK_CALLS.fetch_add(1, Ordering::Relaxed);
        }

        let mut regs = ExtRegistries::default();
        regs.hooks.register("record_hook", headless_hook);

        HEADLESS_HOOK_CALLS.store(0, Ordering::Relaxed);
        let timeline = presentation_timeline();
        // HeadlessAuto is the default — no with_clock needed.
        let mut runner = BeatRunner::new(timeline, cast_id(), CASTER, TARGET);
        let mut world = World::new();
        let mut pending = VecDeque::new();

        let o1 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(
            o1,
            StepOutcome::Advanced,
            "HeadlessAuto must not stall on presentation"
        );

        let o2 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(o2, StepOutcome::Done);

        assert_eq!(
            HEADLESS_HOOK_CALLS.load(Ordering::Relaxed),
            2,
            "both hooks must fire under HeadlessAuto"
        );
    }

    // ── Test (e): Windowed stalls exactly once, hook fires exactly once ────────

    #[test]
    fn windowed_stalls_on_presentation_hook_fires_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static WINDOWED_HOOK_CALLS: AtomicUsize = AtomicUsize::new(0);
        fn windowed_hook(_evt: &BeatEvent, _ctx: &mut SkillCtx<'_>) {
            WINDOWED_HOOK_CALLS.fetch_add(1, Ordering::Relaxed);
        }

        let mut regs = ExtRegistries::default();
        regs.hooks.register("record_hook", windowed_hook);

        WINDOWED_HOOK_CALLS.store(0, Ordering::Relaxed);
        let timeline = presentation_timeline();
        let mut runner =
            BeatRunner::new(timeline, cast_id(), CASTER, TARGET).with_clock(Clock::Windowed);
        let mut world = World::new();
        let mut pending = VecDeque::new();

        // Step 1: fires "cast" hook, then stalls (presentation + Windowed).
        let o1 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(o1, StepOutcome::AwaitingCue, "first step must stall");
        assert_eq!(
            WINDOWED_HOOK_CALLS.load(Ordering::Relaxed),
            1,
            "hook must fire exactly once before stall"
        );

        // Step 2 without resume: still stalled.
        let o2 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(
            o2,
            StepOutcome::AwaitingCue,
            "still stalled before resume_cue"
        );
        assert_eq!(
            WINDOWED_HOOK_CALLS.load(Ordering::Relaxed),
            1,
            "hook must NOT fire again while stalled"
        );

        // Unlatch and advance.
        runner.resume_cue();

        // Step 3: cursor advances to "impact" (no re-fire of "cast").
        let o3 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(
            o3,
            StepOutcome::Advanced,
            "must advance past presentation beat after resume"
        );
        assert_eq!(
            WINDOWED_HOOK_CALLS.load(Ordering::Relaxed),
            1,
            "cast hook must not re-fire on advance"
        );

        // Step 4: fires "impact" hook → Done.
        let o4 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(o4, StepOutcome::Done);
        assert_eq!(
            WINDOWED_HOOK_CALLS.load(Ordering::Relaxed),
            2,
            "impact hook fires once"
        );
    }

    // ── Test (f): HeadlessAuto pending == Windowed pending (stream parity) ────

    #[test]
    fn headless_and_windowed_produce_identical_pending_stream() {
        use crate::combat::api::intent::Intent;
        use std::sync::atomic::{AtomicUsize, Ordering};
        static PARITY_HOOK_CALLS: AtomicUsize = AtomicUsize::new(0);

        fn parity_hook(evt: &BeatEvent, ctx: &mut SkillCtx<'_>) {
            PARITY_HOOK_CALLS.fetch_add(1, Ordering::Relaxed);
            // Enqueue a distinguishable intent per beat to verify stream ordering.
            ctx.enqueue(Intent::SetBlueprintState {
                actor: UnitId(evt.hop_index + 1),
                key: evt.beat_id.to_string(),
                value: 42,
                cast_id: evt.cast_id,
            });
        }

        let mut regs = ExtRegistries::default();
        regs.hooks.register("record_hook", parity_hook);

        let timeline = presentation_timeline();
        let mut world = World::new();

        // HeadlessAuto run.
        PARITY_HOOK_CALLS.store(0, Ordering::Relaxed);
        let mut pending_headless = VecDeque::new();
        let mut runner_headless = BeatRunner::new(Arc::clone(&timeline), cast_id(), CASTER, TARGET);
        runner_headless.run_to_completion(
            &mut world,
            &regs,
            SkillCtxMode::Execute,
            &mut pending_headless,
            100,
        );

        // Windowed run (auto-resumed via run_to_completion).
        PARITY_HOOK_CALLS.store(0, Ordering::Relaxed);
        let mut pending_windowed = VecDeque::new();
        let mut runner_windowed = BeatRunner::new(Arc::clone(&timeline), cast_id(), CASTER, TARGET)
            .with_clock(Clock::Windowed);
        runner_windowed.run_to_completion(
            &mut world,
            &regs,
            SkillCtxMode::Execute,
            &mut pending_windowed,
            100,
        );

        // Compare normalized intent streams.
        let headless_str: Vec<String> = pending_headless.iter().map(|i| format!("{i:?}")).collect();
        let windowed_str: Vec<String> = pending_windowed.iter().map(|i| format!("{i:?}")).collect();
        assert_eq!(
            headless_str, windowed_str,
            "HeadlessAuto and Windowed must produce identical Intent streams"
        );
    }
}
