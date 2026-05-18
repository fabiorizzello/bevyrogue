use bevy::prelude::*;
use bevyrogue::combat::runtime::intent::CastId;
use bevyrogue::combat::types::{Attribute, EvoStage};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    state::CombatState,
    team::Team,
    toughness::DamageKind,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system},
    types::{DamageTag, UnitId},
    ultimate::{UltAccumulationTrigger, UltGainQueue, UltimateCharge, ult_accumulation_system},
    unit::{Commander, Ko, Unit},
};

fn make_unit(id: u32, name: &str) -> Unit {
    Unit {
        id: UnitId(id),
        name: name.into(),
        hp_max: 100,
        hp_current: 100,
        attribute: Attribute::Vaccine,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

#[derive(Resource, Default)]
struct CapturedIntents(Vec<ActionIntent>);

fn capture_system(mut reader: MessageReader<ActionIntent>, mut captured: ResMut<CapturedIntents>) {
    for intent in reader.read() {
        captured.0.push(intent.clone());
    }
}

#[test]
fn enemy_does_not_target_commander() {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .add_message::<TurnAdvanced>()
        .add_message::<bevyrogue::combat::av::ActionValueUpdated>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .init_resource::<CapturedIntents>()
        .add_systems(Update, (advance_turn_system, capture_system).chain());

    app.world_mut()
        .spawn((make_unit(0, "Taichi"), Team::Ally, Commander));
    app.world_mut().spawn((make_unit(1, "Agumon"), Team::Ally));
    app.world_mut()
        .spawn((make_unit(2, "Greymon"), Team::Enemy));

    {
        let mut order = app.world_mut().resource_mut::<TurnOrder>();
        order.seed([UnitId(2), UnitId(0), UnitId(1)]);
    }

    app.world_mut().write_message(TurnAdvanced::of(UnitId(2)));
    app.update();

    let captured = app.world().resource::<CapturedIntents>();
    for intent in &captured.0 {
        match intent {
            ActionIntent::Basic { attacker, target }
            | ActionIntent::Skill { attacker, target, .. }
            | ActionIntent::Ultimate { attacker, target } => {
                assert_eq!(*attacker, UnitId(2), "attacker should be the enemy");
                assert_ne!(
                    *target,
                    UnitId(0),
                    "enemy must not target the Commander (Taichi)"
                );
            }
        }
    }
}

#[test]
fn enemy_emits_no_intent_when_only_commander_is_alive() {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .add_message::<TurnAdvanced>()
        .add_message::<bevyrogue::combat::av::ActionValueUpdated>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .init_resource::<CapturedIntents>()
        .add_systems(Update, (advance_turn_system, capture_system).chain());

    app.world_mut()
        .spawn((make_unit(0, "Taichi"), Team::Ally, Commander));

    let a1 = app
        .world_mut()
        .spawn((make_unit(1, "Agumon"), Team::Ally))
        .id();
    app.world_mut().entity_mut(a1).insert(Ko);

    app.world_mut()
        .spawn((make_unit(2, "Greymon"), Team::Enemy));

    {
        let mut order = app.world_mut().resource_mut::<TurnOrder>();
        order.seed([UnitId(2), UnitId(0), UnitId(1)]);
    }

    app.world_mut().write_message(TurnAdvanced::of(UnitId(2)));
    app.update();

    let captured = app.world().resource::<CapturedIntents>();
    assert!(
        captured.0.is_empty(),
        "enemy should emit no intent when only the Commander is alive: got {:?}",
        captured.0
    );
}

#[test]
fn ally_offensive_event_charges_commander_ultimate() {
    let mut app = App::new();
    app.add_message::<CombatEvent>()
        .init_resource::<UltGainQueue>()
        .add_systems(Update, ult_accumulation_system);

    app.world_mut().spawn((
        make_unit(0, "Taichi"),
        Team::Ally,
        Commander,
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnOffensivePartyEvent,
            charge_per_event: 10,
        },
    ));
    app.world_mut().spawn((make_unit(1, "Agumon"), Team::Ally));

    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::OnDamageDealt {
            amount: 30,
            kind: DamageKind::Normal,
            tag_mod_pct: 100,
            triangle_mod_pct: 100,
            damage_tag: DamageTag::Fire,
        },
        source: UnitId(1),
        target: UnitId(99),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });

    app.update();

    let mut q = app.world_mut().query::<&UltimateCharge>();
    let charge = q
        .iter(app.world())
        .next()
        .expect("commander should have UltimateCharge");

    assert_eq!(
        charge.current, 10,
        "charge should increase by charge_per_event per offensive ally event"
    );
}

#[test]
fn enemy_offensive_event_does_not_charge_commander_ultimate() {
    let mut app = App::new();
    app.add_message::<CombatEvent>()
        .init_resource::<UltGainQueue>()
        .add_systems(Update, ult_accumulation_system);

    app.world_mut().spawn((
        make_unit(0, "Taichi"),
        Team::Ally,
        Commander,
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnOffensivePartyEvent,
            charge_per_event: 10,
        },
    ));
    app.world_mut()
        .spawn((make_unit(2, "Greymon"), Team::Enemy));

    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::OnDamageDealt {
            amount: 20,
            kind: DamageKind::Normal,
            tag_mod_pct: 100,
            triangle_mod_pct: 100,
            damage_tag: DamageTag::Fire,
        },
        source: UnitId(2),
        target: UnitId(1),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });

    app.update();

    let mut q = app.world_mut().query::<&UltimateCharge>();
    let charge = q
        .iter(app.world())
        .next()
        .expect("commander should have UltimateCharge");

    assert_eq!(
        charge.current, 0,
        "enemy offensive events must not charge commander ultimate"
    );
}
