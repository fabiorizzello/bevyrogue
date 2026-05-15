use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

use bevy::prelude::World;

use crate::combat::{
    api::{
        intent::{CastId, Intent},
        registry::ExtRegistries,
        skill_ctx::{SkillCtx, SkillCtxMode},
        timeline::{Beat, BeatEvent, BeatId, BeatKind, CompiledTimeline, SelectorCtx},
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
    Halted,
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
        }
    }

    /// Advance the FSM by one beat.
    ///
    /// - `Done` is returned when the timeline has no more beats.
    /// - `Halted` is returned when a loop exceeds `MAX_HOPS` (256).
    /// - `SkillCtxMode` is forwarded to hooks and predicates so S03 can flip
    ///   to `DryRun` without an API change.
    pub fn step(
        &mut self,
        world: &mut World,
        regs: &ExtRegistries,
        mode: SkillCtxMode,
        pending: &mut VecDeque<Intent>,
    ) -> StepOutcome {
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

            let beat_targets = self.fire_beat(&cur_beat, hop_index, world, regs, mode, pending);
            if matches!(cur_beat.kind, BeatKind::Impact) {
                self.last_beat_targets = beat_targets;
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
            let should_exit = self.eval_predicate(exit_when, &exit_evt, world, regs, mode);
            if should_exit {
                self.loop_stack.pop();
                // F1: first passing edge from the enclosing Loop beat becomes the next cursor.
                let next = self.next_beat(enclosing, &exit_evt, world, regs, mode);
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
                let beat_targets = self.fire_beat(&beat, 0, world, regs, mode, pending);
                if matches!(beat.kind, BeatKind::Impact) {
                    self.last_beat_targets = beat_targets;
                }
                let gate_evt = BeatEvent {
                    cast_id: self.cast_id,
                    beat_id,
                    hop_index: 0,
                    beat_targets: self.last_beat_targets.clone(),
                };
                let next = self.next_beat(beat_id, &gate_evt, world, regs, mode);
                self.cursor = next;
                if self.cursor.is_none() {
                    StepOutcome::Done
                } else {
                    StepOutcome::Advanced
                }
            }
        }
    }

    /// Drive the runner to completion, calling `step` repeatedly.
    ///
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
                _ => {}
            }
        }
        panic!(
            "BeatRunner::run_to_completion exceeded max_steps={max_steps}; possible infinite loop"
        );
    }

    // ─── Private helpers ──────────────────────────────────────────────────────

    /// Execute one beat: resolve selector (Impact only), fire hook, fold DealDamage hits.
    fn fire_beat(
        &mut self,
        beat: &Beat,
        hop_index: u32,
        world: &World,
        regs: &ExtRegistries,
        mode: SkillCtxMode,
        pending: &mut VecDeque<Intent>,
    ) -> Vec<UnitId> {
        // Selector — only Impact beats resolve targets.
        let beat_targets = if matches!(beat.kind, BeatKind::Impact) {
            if let Some(sel_id) = beat.selector {
                let sel = *regs.selectors.get(sel_id).unwrap_or_else(|| {
                    panic!(
                        "selector `{sel_id}` not registered \
                         (validate_timeline_refs catches this at App::finish)"
                    )
                });
                let sctx = SelectorCtx {
                    caster: self.caster,
                    primary_target: self.primary_target,
                    state: &(),
                };
                sel(&sctx)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Hook.
        if let Some(hook_id) = beat.hook {
            let f = *regs.hooks.get(hook_id).unwrap_or_else(|| {
                panic!(
                    "hook `{hook_id}` not registered \
                     (validate_timeline_refs catches this at App::finish)"
                )
            });
            let evt = BeatEvent {
                cast_id: self.cast_id,
                beat_id: beat.id,
                hop_index,
                beat_targets: beat_targets.clone(),
            };
            let prev_len = pending.len();
            {
                let mut ctx = SkillCtx::new(
                    self.caster,
                    self.primary_target,
                    self.cast_id,
                    mode,
                    regs,
                    world,
                    &mut self.cast_hit_set,
                    pending,
                );
                f(&evt, &mut ctx);
            }
            // F6 fix: fold newly enqueued DealDamage targets into cast_hit_set so
            // subsequent bounce / NoRepeat selectors skip already-hit units.
            for i in prev_len..pending.len() {
                if let Some(Intent::DealDamage { target, .. }) = pending.get(i) {
                    self.cast_hit_set.insert(*target);
                }
            }
        }

        beat_targets
    }

    /// Evaluate a registered predicate, providing a fresh (read-only) `SkillCtx`.
    ///
    /// Any intents the predicate erroneously enqueues are discarded (dummy queue).
    fn eval_predicate(
        &mut self,
        pred_id: &str,
        evt: &BeatEvent,
        world: &World,
        regs: &ExtRegistries,
        mode: SkillCtxMode,
    ) -> bool {
        let f = *regs.predicates.get(pred_id).unwrap_or_else(|| {
            panic!("predicate `{pred_id}` not registered")
        });
        let mut dummy: VecDeque<Intent> = VecDeque::new();
        let ctx = SkillCtx::new(
            self.caster,
            self.primary_target,
            self.cast_id,
            mode,
            regs,
            world,
            &mut self.cast_hit_set,
            &mut dummy,
        );
        f(evt, &ctx)
    }

    /// Pick the next beat by walking outgoing edges from `from` in declaration order.
    ///
    /// F1 fallback-edge rule: edges are tested left-to-right; the first edge whose
    /// gate predicate is absent (`None`) or returns `true` is selected. An
    /// unconditional edge placed last acts as the implicit fallback / default
    /// transition. Returns `None` when the timeline has no more beats.
    fn next_beat(
        &mut self,
        from: BeatId,
        evt: &BeatEvent,
        world: &World,
        regs: &ExtRegistries,
        mode: SkillCtxMode,
    ) -> Option<BeatId> {
        // Clone the Arc to obtain an independent reference; this avoids a borrow
        // conflict between iterating `self.timeline.edges` and calling `&mut self`
        // methods (eval_predicate) inside the loop body.
        let timeline = Arc::clone(&self.timeline);
        for edge in timeline.edges.iter().filter(|e| e.from == from) {
            let passes = match edge.gate {
                None => true,
                Some(pred_id) => self.eval_predicate(pred_id, evt, world, regs, mode),
            };
            if passes {
                return Some(edge.to);
            }
        }
        None
    }
}

// ─── Free helpers ─────────────────────────────────────────────────────────────

fn find_beat<'t>(timeline: &'t CompiledTimeline, id: BeatId) -> &'t Beat {
    timeline
        .beats
        .iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| panic!("beat `{id}` not found in timeline `{}`", timeline.id))
}

// ─── Inline unit tests ────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::api::{
        intent::CastId,
        registry::ExtRegistries,
        skill_ctx::{SkillCtx, SkillCtxMode},
        timeline::{Beat, BeatEdge, BeatKind, BeatEvent, CompiledTimeline},
    };
    use std::{
        collections::VecDeque,
        num::NonZeroU32,
        sync::{Arc, Mutex},
    };

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
                },
                Beat {
                    id: "impact",
                    kind: BeatKind::Impact,
                    hook: Some("count_hook"),
                    selector: None,
                    presentation: None,
                },
            ],
            edges: vec![BeatEdge { from: "cast", to: "impact", gate: None }],
        });

        HOOK_CALLS.store(0, Ordering::Relaxed);
        let mut runner = BeatRunner::new(timeline, cast_id(), CASTER, TARGET);
        let mut world = World::new();
        let mut pending = VecDeque::new();

        let o1 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(o1, StepOutcome::Advanced);

        let o2 = runner.step(&mut world, &regs, SkillCtxMode::Execute, &mut pending);
        assert_eq!(o2, StepOutcome::Done);

        assert_eq!(HOOK_CALLS.load(Ordering::Relaxed), 2, "both beats should fire hooks");
    }

    // ── Test (b): Loop with exit_when="always_true" exits after one body pass ──

    #[test]
    fn loop_with_always_true_exits_after_one_pass() {
        fn always_true(_evt: &BeatEvent, _ctx: &SkillCtx<'_>) -> bool { true }

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
                    }],
                    exit_when: "always_true",
                },
                hook: None,
                selector: None,
                presentation: None,
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
        fn never(_evt: &BeatEvent, _ctx: &SkillCtx<'_>) -> bool { false }

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
                    }],
                    exit_when: "never",
                },
                hook: None,
                selector: None,
                presentation: None,
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
}
