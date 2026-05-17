//! Canary test: `Intent::DealDamage` routed end-to-end through `intent_applier`.
//!
//! Spawns two units, enqueues a `DealDamage` intent, ticks once, and asserts:
//! 1. Defender HP is reduced.
//! 2. `CombatEvent::OnDamageDealt` is emitted with correct source/target.
//!
//! cast_id propagation on the event is verified in T03 (additive non-breaking
//! field on `CombatEvent` arrives there).

use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    api::{
        CastId, Intent,
        applier::{IntentQueue, intent_applier},
    },
    events::{CombatEvent, CombatEventKind},
    team::Team,
    types::{Attribute, DamageTag, EvoStage, UnitId},
    unit::Unit,
};

fn setup_app() -> App {
    let mut app = App::new();
    app.init_resource::<IntentQueue>()
        .add_message::<CombatEvent>()
        .add_systems(Update, intent_applier);
    app
}

fn spawn_unit(app: &mut App, id: UnitId, team: Team, attribute: Attribute, hp: i32) {
    app.world_mut().spawn((
        Unit {
            id,
            name: format!("unit_{}", id.0),
            hp_max: hp,
            hp_current: hp,
            attribute,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        team,
    ));
}

fn drain_events(app: &mut App) -> Vec<CombatEvent> {
    let mut cursor: MessageCursor<CombatEvent> = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

#[test]
fn deal_damage_intent_reduces_hp_and_emits_event() {
    let mut app = setup_app();

    let attacker_id = UnitId(1);
    let defender_id = UnitId(2);

    spawn_unit(&mut app, attacker_id, Team::Ally, Attribute::Data, 500);
    // Same attribute → no triangle modifier; no weaknesses → no tag modifier.
    // base_damage=100 → final_damage=100 exactly.
    spawn_unit(&mut app, defender_id, Team::Enemy, Attribute::Data, 500);

    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .push_back(Intent::DealDamage {
            source: attacker_id,
            target: defender_id,
            amount: 100,
            tag: DamageTag::Fire,
            cast_id: CastId::ROOT,
        });

    app.update();

    // HP reduced
    let defender_hp = {
        let mut q = app.world_mut().query::<(&Unit, &Team)>();
        q.iter(app.world())
            .find(|(u, t)| u.id == defender_id && **t == Team::Enemy)
            .map(|(u, _)| u.hp_current)
            .expect("defender not found in world")
    };
    assert!(
        defender_hp < 500,
        "expected defender HP < 500 after DealDamage, got {}",
        defender_hp
    );
    assert_eq!(
        defender_hp, 400,
        "expected exactly 100 damage (neutral matchup, no weaknesses)"
    );

    // OnDamageDealt event emitted
    let events = drain_events(&mut app);
    let dmg_ev = events
        .iter()
        .find(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }));
    assert!(
        dmg_ev.is_some(),
        "expected OnDamageDealt event, got: {:?}",
        events
    );
    let ev = dmg_ev.unwrap();
    assert_eq!(ev.source, attacker_id, "wrong source on OnDamageDealt");
    assert_eq!(ev.target, defender_id, "wrong target on OnDamageDealt");
    if let CombatEventKind::OnDamageDealt { amount, .. } = ev.kind {
        assert_eq!(amount, 100, "expected amount=100, got {}", amount);
    }
}

#[test]
fn intent_queue_is_empty_after_applier_runs() {
    let mut app = setup_app();
    spawn_unit(&mut app, UnitId(1), Team::Ally, Attribute::Data, 100);
    spawn_unit(&mut app, UnitId(2), Team::Enemy, Attribute::Data, 100);

    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .push_back(Intent::DealDamage {
            source: UnitId(1),
            target: UnitId(2),
            amount: 10,
            tag: DamageTag::Physical,
            cast_id: CastId::ROOT,
        });

    app.update();

    let remaining = app.world().resource::<IntentQueue>().0.len();
    assert_eq!(
        remaining, 0,
        "IntentQueue should be empty after applier drains it"
    );
}
