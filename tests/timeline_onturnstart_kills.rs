//! Integration test — Gate 1 (S02): fixture OnTurnStart kills target via BeatRunner.
//!
//! A single-beat Impact timeline is driven by `BeatRunner::run_to_completion`.
//! The hook enqueues `Intent::DealDamage { amount: 9999 }`, the pending queue is
//! transferred to `IntentQueue`, then `app.update()` lets `intent_applier` drain it.
//! Asserts: enemy HP ≤ 0 and `OnDamageDealt` event carries the correct `cast_id`.

use bevy::prelude::*;
use bevyrogue::combat::{
    runtime::{
        CastId, CastIdGen, Intent, IntentQueue,
        applier::intent_applier,
        registry::ExtRegistries,
        runner::{BeatRunner, StepOutcome},
        skill_ctx::{SkillCtx, SkillCtxMode},
        timeline::{Beat, BeatEvent, BeatKind, CompiledTimeline},
    },
    events::{CombatEvent, CombatEventKind},
    team::Team,
    types::{Attribute, DamageTag, EvoStage, UnitId},
    unit::Unit,
};
use std::collections::VecDeque;
use std::sync::Arc;

fn setup_app() -> App {
    let mut app = App::new();
    app.init_resource::<IntentQueue>()
        .init_resource::<CastIdGen>()
        .add_message::<CombatEvent>()
        .add_systems(Update, intent_applier);
    app
}

fn spawn_unit(app: &mut App, id: UnitId, team: Team, hp: i32) {
    app.world_mut().spawn((
        Unit {
            id,
            name: format!("unit_{}", id.0),
            hp_max: hp,
            hp_current: hp,
            attribute: Attribute::Data,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        team,
    ));
}

/// Hook: deal 9999 damage to `ctx.primary_target`.
fn ko_target(_ev: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target: ctx.primary_target,
        amount: 9999,
        tag: DamageTag::Physical,
        cast_id: ctx.cast_id,
    });
}

#[test]
fn fixture_onturnstart_kills_target() {
    let mut app = setup_app();

    let caster_id = UnitId(1);
    let enemy_id = UnitId(2);

    spawn_unit(&mut app, caster_id, Team::Ally, 500);
    spawn_unit(&mut app, enemy_id, Team::Enemy, 1);

    let cast_id = app.world_mut().resource_mut::<CastIdGen>().next();

    let mut regs = ExtRegistries::default();
    regs.hooks.register("test/ko_target", ko_target);

    let timeline = Arc::new(CompiledTimeline {
        id: "test/ko_timeline",
        entry: "impact",
        beats: vec![Beat {
            id: "impact",
            kind: BeatKind::Impact,
            hook: Some("test/ko_target"),
            selector: None,
            presentation: None,
            payload: None,
        }],
        edges: vec![],
    });

    let mut runner = BeatRunner::new(Arc::clone(&timeline), cast_id, caster_id, enemy_id);
    let mut pending = VecDeque::new();

    let outcome = runner.run_to_completion(
        app.world_mut(),
        &regs,
        SkillCtxMode::Execute,
        &mut pending,
        64,
    );
    assert_eq!(
        outcome,
        StepOutcome::Done,
        "timeline should finish normally"
    );
    assert_eq!(pending.len(), 1, "exactly one DealDamage intent expected");

    // Transfer intents to the queue and let intent_applier run.
    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .extend(pending);
    app.update();

    // Enemy HP must be ≤ 0 (KO'd by 9999 damage).
    let enemy_hp = {
        let mut q = app.world_mut().query::<(&Unit, &Team)>();
        q.iter(app.world())
            .find(|(u, t)| u.id == enemy_id && **t == Team::Enemy)
            .map(|(u, _)| u.hp_current)
            .expect("enemy entity not found after update")
    };
    assert!(
        enemy_hp <= 0,
        "expected enemy HP ≤ 0 after DealDamage 9999, got {}",
        enemy_hp
    );

    // OnDamageDealt event must carry the matching cast_id.
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    let events: Vec<CombatEvent> = cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect();

    let dmg_ev = events.iter().find(|e| {
        matches!(e.kind, CombatEventKind::OnDamageDealt { .. })
            && e.source == caster_id
            && e.target == enemy_id
            && e.cast_id == cast_id
    });
    assert!(
        dmg_ev.is_some(),
        "expected OnDamageDealt with cast_id={:?}, source={:?}, target={:?}; events: {:?}",
        cast_id,
        caster_id,
        enemy_id,
        events,
    );
}
