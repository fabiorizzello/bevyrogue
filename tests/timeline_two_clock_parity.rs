//! Integration test — I3/D026: HeadlessAuto and Windowed preserve the same
//! end-of-cast `Intent` stream while differing only in cue timing.
//!
//! The shared timeline has two presentation-bearing beats:
//!
//!   cast (Presentation) -> impact (Presentation)
//!
//! HeadlessAuto drives straight to `Done`. Windowed uses repeated
//! `run_to_completion()` calls, which now stop at each presentation barrier with
//! `StepOutcome::AwaitingCue`; the caller must invoke `resume_cue()` before the
//! runner advances. Assertions cover:
//!
//! - Windowed returns `AwaitingCue` at each presentation barrier.
//! - Re-entering `run_to_completion()` without `resume_cue()` does not advance or
//!   duplicate intents.
//! - After both manual resumes, the final normalized `Intent` stream matches the
//!   HeadlessAuto run exactly and terminates with `Done`.
//! - Calling `resume_cue()` with no awaiting beat is a harmless no-op.
//!
//! Deterministic: no wall-clock, no RNG.

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

/// Hook registered on both beats: enqueues one distinct `DealDamage` intent so
/// the pending stream is beat-distinguishable and easy to diff in failure text.
fn emit_damage_intent(ev: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    let amount: i32 = match ev.beat_id {
        "cast" => 7,
        "impact" => 13,
        other => panic!("unexpected beat_id in parity test: {other}"),
    };
    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target: ctx.primary_target,
        amount,
        tag: DamageTag::Physical,
        cast_id: ctx.cast_id,
    });
}

/// Build the shared timeline: both beats carry Presentation so Windowed must
/// handshake twice before the runner can terminate with `Done`.
fn build_timeline() -> Arc<CompiledTimeline> {
    Arc::new(CompiledTimeline {
        id: "parity_test",
        entry: "cast",
        beats: vec![
            Beat {
                id: "cast",
                kind: BeatKind::Cast,
                hook: Some("parity/emit_damage"),
                selector: None,
                presentation: Some(Presentation {
                    cue_id: "parity_cast_cue",
                    anim: None,
                    vfx: None,
                    sfx: None,
                }),
                payload: None,
            },
            Beat {
                id: "impact",
                kind: BeatKind::Impact,
                hook: Some("parity/emit_damage"),
                selector: None,
                presentation: Some(Presentation {
                    cue_id: "parity_impact_cue",
                    anim: None,
                    vfx: None,
                    sfx: None,
                }),
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

fn build_regs() -> ExtRegistries {
    let mut regs = ExtRegistries::default();
    regs.hooks
        .register("parity/emit_damage", emit_damage_intent);
    regs
}

fn normalized_intents(pending: &VecDeque<Intent>) -> Vec<String> {
    pending.iter().map(|intent| format!("{intent:?}")).collect()
}

#[test]
fn headless_auto_eq_windowed_manual_cue_handshake_end_of_cast_intent_stream() {
    let timeline = build_timeline();
    let regs = build_regs();
    let cast_id = CastId::ROOT; // deterministic, no CastIdGen needed for a standalone test

    // ── Run #1: HeadlessAuto reaches Done in one batch call ──────────────────
    let mut world_headless = World::new();
    let mut pending_headless: VecDeque<Intent> = VecDeque::new();
    let mut runner_headless = BeatRunner::new(Arc::clone(&timeline), cast_id, CASTER, TARGET);
    let headless_outcome = runner_headless.run_to_completion(
        &mut world_headless,
        &regs,
        SkillCtxMode::Execute,
        &mut pending_headless,
        64,
    );
    assert_eq!(
        headless_outcome,
        StepOutcome::Done,
        "HeadlessAuto run must terminate with Done"
    );
    let headless_normalized = normalized_intents(&pending_headless);
    assert_eq!(
        headless_normalized.len(),
        2,
        "HeadlessAuto should fire both presentation-bearing beats exactly once; got {headless_normalized:?}"
    );

    // ── Run #2: Windowed stops at each cue barrier until manually resumed ───
    let mut world_windowed = World::new();
    let mut pending_windowed: VecDeque<Intent> = VecDeque::new();
    let mut runner_windowed =
        BeatRunner::new(Arc::clone(&timeline), cast_id, CASTER, TARGET).with_clock(Clock::Windowed);

    let first_barrier = runner_windowed.run_to_completion(
        &mut world_windowed,
        &regs,
        SkillCtxMode::Execute,
        &mut pending_windowed,
        64,
    );
    assert_eq!(
        first_barrier,
        StepOutcome::AwaitingCue,
        "Windowed batch run must stop at the first presentation beat"
    );
    assert_eq!(
        runner_windowed.cursor(),
        Some("cast"),
        "first barrier should latch on the cast beat before cursor advances"
    );
    let after_first_barrier = normalized_intents(&pending_windowed);
    assert_eq!(
        after_first_barrier,
        vec![headless_normalized[0].clone()],
        "first barrier should enqueue only the cast beat intent once; got {after_first_barrier:?}"
    );

    let redundant_run = runner_windowed.run_to_completion(
        &mut world_windowed,
        &regs,
        SkillCtxMode::Execute,
        &mut pending_windowed,
        64,
    );
    assert_eq!(
        redundant_run,
        StepOutcome::AwaitingCue,
        "Windowed must remain stalled until resume_cue() is called"
    );
    assert_eq!(
        runner_windowed.cursor(),
        Some("cast"),
        "cursor must stay pinned to the stalled beat before resume_cue()"
    );
    assert_eq!(
        normalized_intents(&pending_windowed),
        after_first_barrier,
        "re-entering run_to_completion() without resume_cue() must not duplicate intents"
    );

    runner_windowed.resume_cue();

    let second_barrier = runner_windowed.run_to_completion(
        &mut world_windowed,
        &regs,
        SkillCtxMode::Execute,
        &mut pending_windowed,
        64,
    );
    assert_eq!(
        second_barrier,
        StepOutcome::AwaitingCue,
        "after resuming cast, Windowed must stop again at the impact presentation beat"
    );
    assert_eq!(
        runner_windowed.cursor(),
        Some("impact"),
        "second barrier should latch on the impact beat"
    );
    let after_second_barrier = normalized_intents(&pending_windowed);
    assert_eq!(
        after_second_barrier,
        headless_normalized,
        "second barrier should add the impact beat exactly once; got {after_second_barrier:?}"
    );

    let redundant_second_run = runner_windowed.run_to_completion(
        &mut world_windowed,
        &regs,
        SkillCtxMode::Execute,
        &mut pending_windowed,
        64,
    );
    assert_eq!(
        redundant_second_run,
        StepOutcome::AwaitingCue,
        "Windowed must not advance past the impact barrier without resume_cue()"
    );
    assert_eq!(
        normalized_intents(&pending_windowed),
        after_second_barrier,
        "impact barrier must not duplicate intents across repeated AwaitingCue returns"
    );

    runner_windowed.resume_cue();

    let windowed_outcome = runner_windowed.run_to_completion(
        &mut world_windowed,
        &regs,
        SkillCtxMode::Execute,
        &mut pending_windowed,
        64,
    );
    assert_eq!(
        windowed_outcome,
        StepOutcome::Done,
        "Windowed run must terminate with Done after both cues are resumed"
    );
    assert_eq!(
        runner_windowed.cursor(),
        None,
        "cursor must clear after the final resumed beat advances to timeline end"
    );

    let windowed_normalized = normalized_intents(&pending_windowed);
    assert_eq!(
        windowed_normalized, headless_normalized,
        "HeadlessAuto and Windowed must produce identical final Intent streams; \
         headless={headless_normalized:?}, windowed={windowed_normalized:?}",
    );
}

#[test]
fn resume_cue_without_awaiting_is_harmless() {
    let timeline = build_timeline();
    let regs = build_regs();
    let mut world = World::new();
    let mut pending: VecDeque<Intent> = VecDeque::new();
    let mut runner = BeatRunner::new(timeline, CastId::ROOT, CASTER, TARGET).with_clock(Clock::Windowed);

    runner.resume_cue();

    let outcome = runner.run_to_completion(
        &mut world,
        &regs,
        SkillCtxMode::Execute,
        &mut pending,
        64,
    );
    assert_eq!(
        outcome,
        StepOutcome::AwaitingCue,
        "resume_cue() with no suspended beat must be a no-op, not skip the first presentation beat"
    );
    assert_eq!(
        runner.cursor(),
        Some("cast"),
        "no-op resume_cue() must leave the runner awaiting the first cue"
    );
    assert_eq!(
        normalized_intents(&pending),
        vec![format!(
            "{:?}",
            Intent::DealDamage {
                source: CASTER,
                target: TARGET,
                amount: 7,
                tag: DamageTag::Physical,
                cast_id: CastId::ROOT,
            }
        )],
        "no-op resume_cue() must not suppress the cast beat hook"
    );
}
