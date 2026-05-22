use crate::common::app::TestAppBuilder;
use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    buffs::DrBag,
    events::{CombatEvent, CombatEventKind},
    modifiers::ModifierLayer,
    runtime::{CastId, Intent, applier::IntentQueue},
    team::Team,
    types::{Attribute, DamageTag, EvoStage, UnitId},
    unit::Unit,
};

fn setup_app(seed: u64) -> App {
    TestAppBuilder::new()
        .with_intent_applier()
        .with_damage_ledger()
        .with_rng(seed)
        .build()
}

fn spawn_unit(app: &mut App, id: UnitId, team: Team, hp: i32, dr_pct: Option<f32>) {
    let mut entity = app.world_mut().spawn((
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

    if let Some(dr_pct) = dr_pct {
        let mut bag = DrBag::default();
        bag.apply(dr_pct, 2);
        entity.insert(bag);
    }
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

fn run_case(seed: u64, armed: bool) -> (i32, Vec<CombatEvent>) {
    let mut app = setup_app(seed);
    let attacker_id = UnitId(1);
    let defender_id = UnitId(2);

    spawn_unit(&mut app, attacker_id, Team::Ally, 500, None);
    spawn_unit(&mut app, defender_id, Team::Enemy, 500, Some(0.30));

    if armed {
        app.world_mut()
            .resource_mut::<IntentQueue>()
            .0
            .push_back(Intent::ApplyDamageModifier {
                target: defender_id,
                layer: ModifierLayer::Passive,
                multiplier_pct: 50,
                cast_id: CastId::ROOT,
            });
    }

    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .push_back(Intent::DealDamage {
            source: attacker_id,
            target: defender_id,
            amount: 100,
            tag: DamageTag::Physical,
            cast_id: CastId::ROOT,
        });

    app.update();

    let hp_after = {
        let mut q = app.world_mut().query::<(&Unit, &Team)>();
        q.iter(app.world())
            .find(|(unit, team)| unit.id == defender_id && **team == Team::Enemy)
            .map(|(unit, _)| unit.hp_current)
            .expect("defender not found")
    };

    (hp_after, drain_events(&mut app))
}

#[test]
fn block_reaction_applies_before_dr_and_replays_deterministically() {
    let seed = 0xCAFE_BABE;

    let (baseline_hp, baseline_events) = run_case(seed, false);
    assert_eq!(
        baseline_hp, 430,
        "expected 70 damage with 30% DR and no armed modifier"
    );
    assert!(baseline_events.iter().any(|ev| matches!(
        ev.kind,
        CombatEventKind::IncomingDamage {
            raw_amount: 100,
            damage_tag: DamageTag::Physical
        }
    )));
    assert!(
        baseline_events
            .iter()
            .any(|ev| matches!(ev.kind, CombatEventKind::OnDamageDealt { amount: 70, .. }))
    );
    assert!(
        baseline_events
            .iter()
            .all(|ev| !matches!(ev.kind, CombatEventKind::BlockReactionTriggered { .. }))
    );

    let (armed_hp, armed_events) = run_case(seed, true);
    assert_eq!(
        armed_hp, 465,
        "expected 35 damage when 50% block reaction applies before 30% DR"
    );

    let kinds: Vec<&'static str> = armed_events
        .iter()
        .map(|ev| match ev.kind {
            CombatEventKind::IncomingDamage { .. } => "IncomingDamage",
            CombatEventKind::OnDamageDealt { .. } => "OnDamageDealt",
            CombatEventKind::BlockReactionTriggered { .. } => "BlockReactionTriggered",
            CombatEventKind::OnEnemyKill => "OnEnemyKill",
            CombatEventKind::UnitDied { .. } => "UnitDied",
            _ => "Other",
        })
        .collect();
    assert!(
        kinds
            .windows(3)
            .any(|window| window == ["IncomingDamage", "OnDamageDealt", "BlockReactionTriggered"]),
        "expected IncomingDamage -> OnDamageDealt -> BlockReactionTriggered ordering, got {kinds:?}"
    );
    assert_eq!(
        armed_events
            .iter()
            .filter(|ev| matches!(ev.kind, CombatEventKind::BlockReactionTriggered { .. }))
            .count(),
        1,
        "BlockReactionTriggered must be emitted exactly once"
    );

    let (armed_hp_replay, armed_events_replay) = run_case(seed, true);
    assert_eq!(
        armed_hp_replay, armed_hp,
        "fixed seed should replay the same HP result"
    );
    assert_eq!(
        armed_events_replay, armed_events,
        "fixed seed should replay the same event stream"
    );
}
