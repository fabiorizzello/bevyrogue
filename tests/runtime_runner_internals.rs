use bevy::prelude::World;
use bevyrogue::combat::runtime::runner::{BeatRunner, StepOutcome};
use bevyrogue::combat::runtime::{
    clock::Clock,
    intent::CastId,
    registry::ExtRegistries,
    skill_ctx::{SkillCtx, SkillCtxMode},
    timeline::{Beat, BeatEdge, BeatEvent, BeatKind, CompiledTimeline, Presentation},
};
use bevyrogue::combat::types::UnitId;
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

// ── Test (f): Windowed batch run awaits cues but preserves final parity ────

#[test]
fn headless_and_windowed_manual_resume_produce_identical_pending_stream() {
    use bevyrogue::combat::runtime::intent::Intent;
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
    let mut world_headless = World::new();

    // HeadlessAuto run.
    PARITY_HOOK_CALLS.store(0, Ordering::Relaxed);
    let mut pending_headless = VecDeque::new();
    let mut runner_headless = BeatRunner::new(Arc::clone(&timeline), cast_id(), CASTER, TARGET);
    let headless_outcome = runner_headless.run_to_completion(
        &mut world_headless,
        &regs,
        SkillCtxMode::Execute,
        &mut pending_headless,
        100,
    );
    assert_eq!(headless_outcome, StepOutcome::Done);

    // Windowed run: first batch stops at cue, second batch finishes after resume.
    PARITY_HOOK_CALLS.store(0, Ordering::Relaxed);
    let mut world_windowed = World::new();
    let mut pending_windowed = VecDeque::new();
    let mut runner_windowed = BeatRunner::new(Arc::clone(&timeline), cast_id(), CASTER, TARGET)
        .with_clock(Clock::Windowed);
    let first_outcome = runner_windowed.run_to_completion(
        &mut world_windowed,
        &regs,
        SkillCtxMode::Execute,
        &mut pending_windowed,
        100,
    );
    assert_eq!(first_outcome, StepOutcome::AwaitingCue);
    runner_windowed.resume_cue();
    let second_outcome = runner_windowed.run_to_completion(
        &mut world_windowed,
        &regs,
        SkillCtxMode::Execute,
        &mut pending_windowed,
        100,
    );
    assert_eq!(second_outcome, StepOutcome::Done);

    // Compare normalized intent streams.
    let headless_str: Vec<String> = pending_headless.iter().map(|i| format!("{i:?}")).collect();
    let windowed_str: Vec<String> = pending_windowed.iter().map(|i| format!("{i:?}")).collect();
    assert_eq!(
        headless_str, windowed_str,
        "HeadlessAuto and Windowed must produce identical final Intent streams"
    );
}
