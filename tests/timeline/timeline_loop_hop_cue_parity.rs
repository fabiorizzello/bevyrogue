//! Per-hop kernel cue parity: loop body beats with `Presentation` cause one
//! `AwaitingCue` stall per hop in `Clock::Windowed`, while `Clock::HeadlessAuto`
//! drains straight through.
//!
//! Assertions:
//! - HeadlessAuto with N-hop loop reaches `Done` in one `run_to_completion` call
//!   and emits N intents.
//! - Windowed suspends exactly N times (one per hop); each requires one
//!   `resume_cue()` call; after N resumes it reaches `Done`.
//! - The final `Intent` stream is identical between both clock modes.
//! - `hop_index` in `AwaitingCueInfo` tracks the current loop iteration.
//! - Releasing more cues than hops is a harmless no-op.

use bevy::prelude::*;
use bevyrogue::combat::{
    runtime::{
        CastId, Intent,
        clock::Clock,
        registry::ExtRegistries,
        runner::{BeatRunner, StepOutcome},
        skill_ctx::{SkillCtx, SkillCtxMode},
        timeline::{Beat, BeatEdge, BeatEvent, BeatKind, CompiledTimeline, Presentation},
    },
    types::{DamageTag, UnitId},
};
use std::collections::VecDeque;
use std::sync::Arc;

const CASTER: UnitId = UnitId(1);
const TARGET: UnitId = UnitId(2);
const HOP_CUE: &str = "test/loop_hop";
const N_HOPS: u32 = 3;

/// Hook: emit a DealDamage intent per hop with `amount = hop_index + 1` so the
/// intent stream is hop-distinguishable and trivially verifiable.
fn emit_hop_intent(ev: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target: ctx.primary_target,
        amount: (ev.hop_index + 1) as i32,
        tag: DamageTag::Physical,
        cast_id: ctx.cast_id,
    });
}

/// Build the test timeline:
///   loop_beat (Loop) → body: [hop_body (Impact + Presentation)]
///   exit_when: hop_index >= N_HOPS - 1 (N_HOPS total iterations: 0 … N_HOPS-1)
///   No outgoing edges from loop_beat → cursor becomes None → Done.
fn build_timeline() -> Arc<CompiledTimeline> {
    Arc::new(CompiledTimeline {
        id: "hop_cue_parity",
        entry: "loop_beat",
        beats: vec![Beat {
            id: "loop_beat",
            kind: BeatKind::Loop {
                body: vec![Beat {
                    id: "hop_body",
                    kind: BeatKind::Impact,
                    hook: Some("test/hop_emit"),
                    selector: None,
                    presentation: Some(Presentation {
                        cue_id: HOP_CUE,
                        anim: None,
                        vfx: None,
                        sfx: None,
                    }),
                    payload: None,
                }],
                exit_when: "test/exit_after_n",
            },
            hook: None,
            selector: None,
            presentation: None,
            payload: None,
        }],
        edges: vec![],
    })
}

fn build_regs() -> ExtRegistries {
    let mut regs = ExtRegistries::default();
    regs.hooks.register("test/hop_emit", emit_hop_intent);
    regs.predicates
        .register("test/exit_after_n", |ev: &BeatEvent, _ctx| {
            ev.hop_index >= N_HOPS - 1
        });
    regs
}

fn normalized_intents(pending: &VecDeque<Intent>) -> Vec<String> {
    pending.iter().map(|i| format!("{i:?}")).collect()
}

#[test]
fn headless_auto_drains_all_hops_in_one_call() {
    let timeline = build_timeline();
    let regs = build_regs();
    let mut world = World::new();
    let mut pending: VecDeque<Intent> = VecDeque::new();
    let mut runner = BeatRunner::new(timeline, CastId::ROOT, CASTER, TARGET);

    let outcome =
        runner.run_to_completion(&mut world, &regs, SkillCtxMode::Execute, &mut pending, 256);

    assert_eq!(
        outcome,
        StepOutcome::Done,
        "HeadlessAuto must reach Done in one batch call"
    );
    assert_eq!(
        pending.len(),
        N_HOPS as usize,
        "HeadlessAuto must emit exactly {N_HOPS} intents (one per hop); got {:?}",
        normalized_intents(&pending)
    );
    assert_eq!(runner.cursor(), None, "cursor must be None after Done");
}

#[test]
fn windowed_suspends_once_per_hop_then_matches_headless() {
    let timeline = build_timeline();
    let regs = build_regs();

    // ── HeadlessAuto reference run ────────────────────────────────────────────
    let headless_intents = {
        let mut world = World::new();
        let mut pending: VecDeque<Intent> = VecDeque::new();
        let mut runner = BeatRunner::new(Arc::clone(&timeline), CastId::ROOT, CASTER, TARGET);
        runner.run_to_completion(&mut world, &regs, SkillCtxMode::Execute, &mut pending, 256);
        normalized_intents(&pending)
    };
    assert_eq!(
        headless_intents.len(),
        N_HOPS as usize,
        "reference run must produce {N_HOPS} intents"
    );

    // ── Windowed run ──────────────────────────────────────────────────────────
    let mut world = World::new();
    let mut pending: VecDeque<Intent> = VecDeque::new();
    let mut runner = BeatRunner::new(Arc::clone(&timeline), CastId::ROOT, CASTER, TARGET)
        .with_clock(Clock::Windowed);

    for hop in 0..N_HOPS {
        // Each call should stall on the hop body's presentation beat.
        let outcome =
            runner.run_to_completion(&mut world, &regs, SkillCtxMode::Execute, &mut pending, 256);
        assert_eq!(
            outcome,
            StepOutcome::AwaitingCue,
            "hop {hop}: Windowed must return AwaitingCue before resume"
        );

        // Re-entering without resume must not advance or duplicate.
        let redundant =
            runner.run_to_completion(&mut world, &regs, SkillCtxMode::Execute, &mut pending, 256);
        assert_eq!(
            redundant,
            StepOutcome::AwaitingCue,
            "hop {hop}: re-entering without resume must remain stalled"
        );
        assert_eq!(
            pending.len(),
            (hop + 1) as usize,
            "hop {hop}: intent count must be hop+1 before resume; no duplicates"
        );

        // Verify hop_index is exposed correctly.
        let info = runner
            .awaiting_cue_info()
            .expect("hop {hop}: awaiting_cue_info must be Some while stalled");
        assert_eq!(info.cue_id, HOP_CUE, "hop {hop}: cue_id must match HOP_CUE");
        assert_eq!(
            info.hop_index,
            Some(hop),
            "hop {hop}: hop_index in AwaitingCueInfo must be Some({hop})"
        );

        runner.resume_cue();
    }

    // After N resumes, the next run_to_completion must reach Done.
    let final_outcome =
        runner.run_to_completion(&mut world, &regs, SkillCtxMode::Execute, &mut pending, 256);
    assert_eq!(
        final_outcome,
        StepOutcome::Done,
        "Windowed must reach Done after all {N_HOPS} resumes"
    );
    assert_eq!(runner.cursor(), None, "cursor must be None after Done");

    // Intent stream must match HeadlessAuto.
    let windowed_intents = normalized_intents(&pending);
    assert_eq!(
        windowed_intents, headless_intents,
        "Windowed and HeadlessAuto must produce the same final Intent stream"
    );
}

#[test]
fn extra_resume_cue_after_done_is_no_op() {
    let timeline = build_timeline();
    let regs = build_regs();
    let mut world = World::new();
    let mut pending: VecDeque<Intent> = VecDeque::new();
    let mut runner = BeatRunner::new(Arc::clone(&timeline), CastId::ROOT, CASTER, TARGET)
        .with_clock(Clock::Windowed);

    // Drain all hops.
    for _ in 0..N_HOPS {
        runner.run_to_completion(&mut world, &regs, SkillCtxMode::Execute, &mut pending, 256);
        runner.resume_cue();
    }
    let done =
        runner.run_to_completion(&mut world, &regs, SkillCtxMode::Execute, &mut pending, 256);
    assert_eq!(done, StepOutcome::Done);
    let intent_count_at_done = pending.len();

    // Extra resume + run must not add intents or change state.
    runner.resume_cue();
    let after_extra =
        runner.run_to_completion(&mut world, &regs, SkillCtxMode::Execute, &mut pending, 256);
    assert_eq!(
        after_extra,
        StepOutcome::Done,
        "runner must remain Done after extra resume_cue()"
    );
    assert_eq!(
        pending.len(),
        intent_count_at_done,
        "extra resume_cue() must not add intents"
    );
}
