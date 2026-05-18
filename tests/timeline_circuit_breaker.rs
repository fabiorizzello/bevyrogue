//! Integration test — F4 (S03): Loop whose exit_when never fires halts at MAX_HOPS=256.
//!
//! Exercises the real BeatRunner circuit-breaker end-to-end with a spawned world and
//! intent_applier wiring (same pattern as timeline_chain_bolt_port).
//!
//! Invariants verified:
//! - `run_to_completion` returns `StepOutcome::Halted` (no panic, no hang).
//! - `pending` contains exactly `MAX_HOPS` (256) `DealDamage` intents — the body
//!   fires once per hop for hop_index 0..=255, and the circuit-breaker trips at
//!   hop_index=256 BEFORE the 257th body execution.
//! - The breaker fires before `max_steps=1000` (no panic from the max_steps guard).
//!
//! NOTE: T01 (runner.rs) also emits `bevy::log::warn!` carrying cast_id, timeline id,
//! and hop_count when Halted is returned. Capturing that log signal is out of scope here;
//! `StepOutcome::Halted` is the observable contract at the integration boundary.

use bevy::prelude::*;
use bevyrogue::combat::{
    runtime::{
        CastIdGen, Intent, IntentQueue,
        applier::intent_applier,
        registry::ExtRegistries,
        runner::{BeatRunner, StepOutcome},
        skill_ctx::{SkillCtx, SkillCtxMode},
        timeline::{Beat, BeatEvent, BeatKind, CompiledTimeline, SelectorCtx},
    },
    events::{CombatEvent, CombatEventKind},
    team::Team,
    types::{Attribute, DamageTag, EvoStage, UnitId},
    unit::Unit,
};
use std::collections::VecDeque;
use std::sync::Arc;

/// MAX_HOPS as defined in `src/combat/api/runner.rs`. Kept as a local constant so
/// the assertion is self-documenting; it must stay in sync with the runner.
const MAX_HOPS: usize = 256;

fn setup_app() -> App {
    let mut app = App::new();
    app.init_resource::<IntentQueue>()
        .init_resource::<CastIdGen>()
        .add_message::<CombatEvent>()
        .add_systems(Update, intent_applier);
    app
}

fn spawn_unit(app: &mut App, id: UnitId, team: Team) {
    app.world_mut().spawn((
        Unit {
            id,
            name: format!("unit_{}", id.0),
            hp_max: 500,
            hp_current: 500,
            attribute: Attribute::Data,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        team,
    ));
}

/// Selector: always returns the single enemy target so the hook has a target to hit.
fn single_target_selector(sctx: &SelectorCtx<'_>) -> Vec<UnitId> {
    vec![sctx.primary_target]
}

/// Hook: enqueues one `DealDamage` per hop (fixed amount=1, no falloff needed).
/// The target is `primary_target` via `BeatEvent.beat_targets[0]`.
fn one_damage_per_hop(ev: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    let target = ev
        .beat_targets
        .first()
        .copied()
        .unwrap_or(ctx.primary_target);
    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target,
        amount: 1,
        tag: DamageTag::Physical,
        cast_id: ctx.cast_id,
    });
}

/// Predicate: always returns false — the loop exit condition is never satisfied.
/// Mirrors the inline `never` predicate in runner.rs unit tests, but as a named
/// integration fixture wired through the real `ExtRegistries` path.
fn never(_evt: &BeatEvent, _ctx: &SkillCtx<'_>) -> bool {
    false
}

#[test]
fn loop_never_exit_halts_at_max_hops() {
    let mut app = setup_app();

    let caster_id = UnitId(1);
    let target_id = UnitId(2);

    spawn_unit(&mut app, caster_id, Team::Ally);
    spawn_unit(&mut app, target_id, Team::Enemy);

    let cast_id = app.world_mut().resource_mut::<CastIdGen>().next();

    let mut regs = ExtRegistries::default();
    regs.selectors
        .register("cb/single_target", single_target_selector);
    regs.hooks
        .register("cb/one_damage_per_hop", one_damage_per_hop);
    regs.predicates.register("cb/never", never);

    // Loop timeline: a single Impact body beat with a hook that enqueues 1 DealDamage
    // per hop, and an exit_when that never fires.
    let timeline = Arc::new(CompiledTimeline {
        id: "circuit_breaker_test",
        entry: "infinite_loop",
        beats: vec![Beat {
            id: "infinite_loop",
            kind: BeatKind::Loop {
                body: vec![Beat {
                    id: "hop_impact",
                    kind: BeatKind::Impact,
                    hook: Some("cb/one_damage_per_hop"),
                    selector: Some("cb/single_target"),
                    presentation: None,
                    payload: None,
                }],
                exit_when: "cb/never",
            },
            hook: None,
            selector: None,
            presentation: None,
            payload: None,
        }],
        edges: vec![],
    });

    let mut runner = BeatRunner::new(Arc::clone(&timeline), cast_id, caster_id, target_id);
    let mut pending = VecDeque::new();

    // max_steps=1000 >> MAX_HOPS=256: proves the circuit-breaker fires, not max_steps.
    let outcome = runner.run_to_completion(
        app.world_mut(),
        &regs,
        SkillCtxMode::Execute,
        &mut pending,
        1000,
    );

    // ── Circuit-breaker outcome ──────────────────────────────────────────────────

    assert_eq!(
        outcome,
        StepOutcome::Halted,
        "run_to_completion must return Halted when exit_when never fires"
    );

    // ── Bounded pending stream ────────────────────────────────────────────────────
    //
    // Semantics: the body fires for hop_index 0..=255 (MAX_HOPS iterations).
    // At hop_index=256 the circuit-breaker trips BEFORE the body executes, so no
    // 257th intent is enqueued. Exactly MAX_HOPS DealDamage intents accumulate.

    let dmg_count = pending
        .iter()
        .filter(|i| matches!(i, Intent::DealDamage { .. }))
        .count();

    assert_eq!(
        dmg_count, MAX_HOPS,
        "pending must contain exactly MAX_HOPS={MAX_HOPS} DealDamage intents, got {dmg_count}"
    );

    assert!(
        pending.len() <= MAX_HOPS + 1,
        "total pending must be bounded: got {} (MAX_HOPS+1={})",
        pending.len(),
        MAX_HOPS + 1
    );

    // ── World effect via intent_applier ───────────────────────────────────────────
    // Drain pending through the real applier to verify no panic on bounded stream.

    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .extend(pending);
    app.update();

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    let events: Vec<CombatEvent> = cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect();

    let dmg_events = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .count();

    assert_eq!(
        dmg_events, MAX_HOPS,
        "intent_applier must emit exactly MAX_HOPS OnDamageDealt events, got {dmg_events}"
    );
}
