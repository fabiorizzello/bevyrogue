//! Integration test — Gate 3 (S02): chain_bolt pattern as CompiledTimeline driven by BeatRunner.
//!
//! A Loop timeline body fires 3 hops. The selector returns targets pre-ordered by ascending
//! hp_pct (hard-coded for this test since `SelectorCtx` has no world access); the hook picks
//! the first target not yet in `cast_hit_set`, computes `base * 0.8^hop_index` (integer floor),
//! and enqueues `Intent::DealDamage`. The predicate exits when `cast_hit_set.len() >= 3`.
//!
//! Assertions:
//! - Exactly 3 `DealDamage` intents produced in pending, targeting [UnitId(12), UnitId(11), UnitId(10)].
//! - Damage ladder: 100 → 80 → 64 (80 % falloff, integer floor).
//! - No target hit twice (`cast_hit_set` NoRepeat invariant).
//! - 3 `OnDamageDealt` events emitted after `app.update()`.

use bevy::prelude::*;
use bevyrogue::combat::{
    api::{
        CastId, CastIdGen, Intent, IntentQueue,
        applier::intent_applier,
        registry::ExtRegistries,
        runner::{BeatRunner, StepOutcome},
        skill_ctx::{SkillCtx, SkillCtxMode},
        timeline::{Beat, BeatEdge, BeatEvent, BeatKind, CompiledTimeline, SelectorCtx},
    },
    events::{CombatEvent, CombatEventKind},
    team::Team,
    types::{Attribute, DamageTag, EvoStage, UnitId},
    unit::Unit,
};
use std::collections::VecDeque;
use std::sync::Arc;

// Target order for this test fixture: ascending hp_pct (60/100, 80/100, 100/100).
const CHAIN_ORDER: [UnitId; 3] = [UnitId(12), UnitId(11), UnitId(10)];

fn setup_app() -> App {
    let mut app = App::new();
    app.init_resource::<IntentQueue>()
        .init_resource::<CastIdGen>()
        .add_message::<CombatEvent>()
        .add_systems(Update, intent_applier);
    app
}

fn spawn_unit(app: &mut App, id: UnitId, team: Team, hp_current: i32, hp_max: i32) {
    app.world_mut().spawn((
        Unit {
            id,
            name: format!("unit_{}", id.0),
            hp_max,
            hp_current,
            attribute: Attribute::Data,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        team,
    ));
}

/// Selector: returns all targets in ascending hp_pct order (hard-coded for this fixture).
/// World-unaware — SelectorCtx has no world access; real selectors query world in the runner.
/// The hook picks the first entry not yet in `cast_hit_set`.
fn lowest_hp_pct_alive_norepeat(_: &SelectorCtx<'_>) -> Vec<UnitId> {
    CHAIN_ORDER.to_vec()
}

/// Hook: picks the first target from `evt.beat_targets` not yet hit, computes
/// `base * (0.8^hop_index)` (integer floor via repeated ×8/10), enqueues DealDamage.
fn chain_bolt_hop(ev: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    let target = ev
        .beat_targets
        .iter()
        .find(|t| !ctx.cast_hit_set.contains(*t))
        .copied();

    let Some(target) = target else {
        return; // no eligible target this hop
    };

    // base_damage * 0.8^hop_index, integer floor.
    let mut amount: i32 = 100;
    for _ in 0..ev.hop_index {
        amount = amount * 8 / 10;
    }

    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target,
        amount,
        tag: DamageTag::Physical,
        cast_id: ctx.cast_id,
    });
}

/// Predicate: exit after 3 hops or when cast_hit_set has covered all 3 enemies.
fn pool_exhausted_or_max_hops(evt: &BeatEvent, ctx: &SkillCtx<'_>) -> bool {
    evt.hop_index >= 3 || ctx.cast_hit_set.len() >= CHAIN_ORDER.len()
}

#[test]
fn chain_bolt_hits_3_targets_with_falloff() {
    let mut app = setup_app();

    let caster_id = UnitId(1);
    let e_low = UnitId(12); // hp_current=60/100, lowest pct → hit first
    let e_mid = UnitId(11); // hp_current=80/100                → hit second
    let e_hi = UnitId(10);  // hp_current=100/100               → hit third

    spawn_unit(&mut app, caster_id, Team::Ally, 500, 500);
    spawn_unit(&mut app, e_low, Team::Enemy, 60, 100);
    spawn_unit(&mut app, e_mid, Team::Enemy, 80, 100);
    spawn_unit(&mut app, e_hi, Team::Enemy, 100, 100);

    let cast_id = app.world_mut().resource_mut::<CastIdGen>().next();

    let mut regs = ExtRegistries::default();
    regs.selectors
        .register("chain/lowest_hp_pct_alive_norepeat", lowest_hp_pct_alive_norepeat);
    regs.predicates
        .register("chain/pool_exhausted_or_max_hops", pool_exhausted_or_max_hops);
    regs.hooks.register("chain/bolt_hop", chain_bolt_hop);

    let timeline = Arc::new(CompiledTimeline {
        id: "chain_bolt",
        entry: "loop",
        beats: vec![Beat {
            id: "loop",
            kind: BeatKind::Loop {
                body: vec![Beat {
                    id: "hop",
                    kind: BeatKind::Impact,
                    hook: Some("chain/bolt_hop"),
                    selector: Some("chain/lowest_hp_pct_alive_norepeat"),
                    presentation: None,
                payload: None,
                }],
                exit_when: "chain/pool_exhausted_or_max_hops",
            },
            hook: None,
            selector: None,
            presentation: None,
                payload: None,
        }],
        edges: vec![],
    });

    let mut runner = BeatRunner::new(Arc::clone(&timeline), cast_id, caster_id, e_hi);
    let mut pending = VecDeque::new();

    let outcome = runner.run_to_completion(
        app.world_mut(),
        &regs,
        SkillCtxMode::Execute,
        &mut pending,
        64,
    );
    assert_eq!(outcome, StepOutcome::Done, "timeline should finish normally");

    // ── Intent-level assertions (before intent_applier runs) ─────────────────────

    let dmg_intents: Vec<(UnitId, i32)> = pending
        .iter()
        .filter_map(|i| match i {
            Intent::DealDamage { target, amount, .. } => Some((*target, *amount)),
            _ => None,
        })
        .collect();

    assert_eq!(
        dmg_intents.len(),
        3,
        "expected exactly 3 DealDamage intents, got: {:?}",
        dmg_intents
    );

    // Targets hit in ascending hp_pct order.
    assert_eq!(dmg_intents[0].0, e_low, "hop 0 must target lowest-HP unit");
    assert_eq!(dmg_intents[1].0, e_mid, "hop 1 must target mid-HP unit");
    assert_eq!(dmg_intents[2].0, e_hi, "hop 2 must target hi-HP unit");

    // No target hit twice.
    let unique_targets: std::collections::HashSet<UnitId> =
        dmg_intents.iter().map(|(t, _)| *t).collect();
    assert_eq!(unique_targets.len(), 3, "each target must be hit exactly once (NoRepeat)");

    // Damage ladder: base=100, ×0.8 per hop (integer floor).
    assert_eq!(dmg_intents[0].1, 100, "hop 0 damage should be 100");
    assert_eq!(dmg_intents[1].1, 80, "hop 1 damage should be 80 (100×0.8)");
    assert_eq!(dmg_intents[2].1, 64, "hop 2 damage should be 64 (80×0.8)");

    // ── Event-level assertions (after intent_applier drains the queue) ──────────

    app.world_mut().resource_mut::<IntentQueue>().0.extend(pending);
    app.update();

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    let events: Vec<CombatEvent> = cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect();

    let dmg_events: Vec<&CombatEvent> = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .collect();

    assert_eq!(
        dmg_events.len(),
        3,
        "expected 3 OnDamageDealt events after app.update(), got: {:?}",
        dmg_events
    );
}

#[test]
fn chain_bolt_respects_bounded_hops_without_extra_iterations() {
    let mut app = setup_app();

    let caster_id = UnitId(1);
    let e_low = UnitId(12);
    let e_mid = UnitId(11);
    let e_hi = UnitId(10);

    spawn_unit(&mut app, caster_id, Team::Ally, 500, 500);
    spawn_unit(&mut app, e_low, Team::Enemy, 60, 100);
    spawn_unit(&mut app, e_mid, Team::Enemy, 80, 100);
    spawn_unit(&mut app, e_hi, Team::Enemy, 100, 100);

    let cast_id = app.world_mut().resource_mut::<CastIdGen>().next();

    let mut regs = ExtRegistries::default();
    regs.selectors
        .register("chain/lowest_hp_pct_alive_norepeat", lowest_hp_pct_alive_norepeat);
    regs.predicates.register("chain/stop_after_two", |evt, ctx| {
        evt.hop_index >= 2 || ctx.cast_hit_set.len() >= 2
    });
    regs.hooks.register("chain/bolt_hop", chain_bolt_hop);

    let timeline = Arc::new(CompiledTimeline {
        id: "chain_bolt_bounded",
        entry: "loop",
        beats: vec![Beat {
            id: "loop",
            kind: BeatKind::Loop {
                body: vec![Beat {
                    id: "hop",
                    kind: BeatKind::Impact,
                    hook: Some("chain/bolt_hop"),
                    selector: Some("chain/lowest_hp_pct_alive_norepeat"),
                    presentation: None,
                    payload: None,
                }],
                exit_when: "chain/stop_after_two",
            },
            hook: None,
            selector: None,
            presentation: None,
            payload: None,
        }],
        edges: vec![],
    });

    let mut runner = BeatRunner::new(Arc::clone(&timeline), cast_id, caster_id, e_hi);
    let mut pending = VecDeque::new();

    let outcome = runner.run_to_completion(
        app.world_mut(),
        &regs,
        SkillCtxMode::Execute,
        &mut pending,
        64,
    );
    assert_eq!(outcome, StepOutcome::Done, "bounded timeline should finish normally");

    let dmg_intents: Vec<(UnitId, i32)> = pending
        .iter()
        .filter_map(|i| match i {
            Intent::DealDamage { target, amount, .. } => Some((*target, *amount)),
            _ => None,
        })
        .collect();

    assert_eq!(dmg_intents.len(), 2, "bounded loop should stop after two hops");
    assert_eq!(dmg_intents[0].0, e_low);
    assert_eq!(dmg_intents[1].0, e_mid);
}

