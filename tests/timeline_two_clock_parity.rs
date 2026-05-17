//! Integration test — I3/D026: HeadlessAuto ≡ Windowed end-of-cast Intent stream.
//!
//! A two-beat timeline (Cast with Presentation + Impact without) is driven by two
//! BeatRunner instances over identical fresh worlds:
//!
//!   Run #1 — HeadlessAuto: `run_to_completion` (never stalls, auto-resumes).
//!   Run #2 — Clock::Windowed: manual `step()` loop; on `StepOutcome::AwaitingCue`
//!             assert it occurs at the presentation beat then call `resume_cue()`
//!             and continue; stop on Done/Halted.
//!
//! Assertions:
//!   - At least one `AwaitingCue` was observed in the Windowed run (stall is real).
//!   - `format!("{:?}")` of every Intent in both pending queues is identical (stream parity).
//!   - Both runs terminate with `StepOutcome::Done` (no Halt, no infinite loop).
//!
//! Deterministic: no wall-clock, no RNG.

use bevy::prelude::*;
use bevyrogue::combat::{
    api::{
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

/// Hook registered on both beats: enqueues one `DealDamage` intent so the pending
/// stream is non-empty and beat-distinguishable (amount encodes `hop_index` for
/// the Cast beat where hop=0, and `hop_index + 10` for the Impact beat).
fn emit_damage_intent(ev: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    // Amount encodes which beat fired: Cast beat has id "cast", Impact has "impact".
    // Use a fixed sentinel per beat_id so the stream is deterministic.
    let amount: i32 = if ev.beat_id == "cast" { 7 } else { 13 };
    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target: ctx.primary_target,
        amount,
        tag: DamageTag::Physical,
        cast_id: ctx.cast_id,
    });
}

/// Build the shared timeline: Cast (with Presentation) → Impact (no Presentation).
/// Both beats carry the same hook so both enqueue intents.
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
                // Presentation is present → Windowed runner stalls here.
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

fn build_regs() -> ExtRegistries {
    let mut regs = ExtRegistries::default();
    regs.hooks
        .register("parity/emit_damage", emit_damage_intent);
    regs
}

#[test]
fn headless_auto_eq_windowed_end_of_cast_intent_stream() {
    let timeline = build_timeline();
    let regs = build_regs();
    let cast_id = CastId::ROOT; // deterministic, no CastIdGen needed for a standalone test

    // ── Run #1: HeadlessAuto via run_to_completion ────────────────────────────
    let mut world_headless = World::new();
    let mut pending_headless: VecDeque<Intent> = VecDeque::new();
    let mut runner_headless = BeatRunner::new(Arc::clone(&timeline), cast_id, CASTER, TARGET);
    // Default clock is HeadlessAuto — no with_clock needed.
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
    assert!(
        !pending_headless.is_empty(),
        "HeadlessAuto run must produce at least one Intent (both hooks must fire)"
    );

    // ── Run #2: Clock::Windowed via manual step() loop ────────────────────────
    let mut world_windowed = World::new();
    let mut pending_windowed: VecDeque<Intent> = VecDeque::new();
    let mut runner_windowed =
        BeatRunner::new(Arc::clone(&timeline), cast_id, CASTER, TARGET).with_clock(Clock::Windowed);

    let mut awaiting_cue_count: u32 = 0;
    let mut last_awaiting_cue_beat: Option<&'static str> = None;
    let mut windowed_outcome = StepOutcome::Advanced;
    const MAX_ITER: u32 = 64;

    for _ in 0..MAX_ITER {
        let outcome = runner_windowed.step(
            &mut world_windowed,
            &regs,
            SkillCtxMode::Execute,
            &mut pending_windowed,
        );
        match outcome {
            StepOutcome::Done | StepOutcome::Halted => {
                windowed_outcome = outcome;
                break;
            }
            StepOutcome::AwaitingCue => {
                awaiting_cue_count += 1;
                // Record which beat triggered the stall (first occurrence only).
                if last_awaiting_cue_beat.is_none() {
                    last_awaiting_cue_beat = Some("cast");
                }
                runner_windowed.resume_cue();
                // Do NOT count this step; the loop will call step() again.
            }
            StepOutcome::Advanced | StepOutcome::LoopExited => {
                windowed_outcome = outcome;
            }
        }
    }

    assert_eq!(
        windowed_outcome,
        StepOutcome::Done,
        "Windowed run must terminate with Done (not Halted or stuck)"
    );

    // ── Assert stall was real (not bypassed) ──────────────────────────────────
    assert!(
        awaiting_cue_count >= 1,
        "Windowed run must stall at least once on the presentation-bearing Cast beat; \
         got awaiting_cue_count={}",
        awaiting_cue_count
    );
    assert_eq!(
        last_awaiting_cue_beat,
        Some("cast"),
        "AwaitingCue must occur at the 'cast' beat (the only beat with a Presentation)"
    );

    // ── Assert stream parity ──────────────────────────────────────────────────
    let headless_normalized: Vec<String> =
        pending_headless.iter().map(|i| format!("{i:?}")).collect();
    let windowed_normalized: Vec<String> =
        pending_windowed.iter().map(|i| format!("{i:?}")).collect();

    assert_eq!(
        headless_normalized.len(),
        windowed_normalized.len(),
        "HeadlessAuto and Windowed must produce the same number of intents; \
         headless={:?}, windowed={:?}",
        headless_normalized,
        windowed_normalized,
    );
    assert_eq!(
        headless_normalized, windowed_normalized,
        "HeadlessAuto and Windowed must produce identical Intent streams (I3/D026); \
         headless={:?}, windowed={:?}",
        headless_normalized, windowed_normalized,
    );
}
