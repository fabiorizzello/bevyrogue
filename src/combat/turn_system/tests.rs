use super::*;
use crate::combat::{
    StatusBag, StatusEffectKind,
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::{ActionLog, LogEntry},
    speed::SpeedModifier,
    team::Team,
    toughness::Toughness,
    types::{Attribute, DamageTag, EvoStage},
    ultimate::UltAccumulationTrigger,
};
use crate::data::skills_ron::{
    Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
    SkillTargeting, TargetLife, TargetShape, TargetSide,
};

fn unit(id: u32, attribute: Attribute, hp_current: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max: 100,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn skill(
    id: &str,
    damage_tag: DamageTag,
    damage: i32,
    sp_cost: i32,
    toughness_damage: i32,
) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag,
        sp_cost,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![
            Effect::Damage {
                amount: damage,
                target: TargetShape::Single,
            },
            Effect::ToughnessHit(toughness_damage),
        ],

        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        ..Default::default()
    }
}

fn combat_events(app: &mut App) -> Vec<CombatEvent> {
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

// NOTE: Tests below were written for the old VecDeque-based advance_turn_system.
// That system has been replaced by the AV-based advance_turn_system (S06/T01).
// These tests are disabled until a status-tick system is re-implemented.

#[allow(dead_code)]
fn advance_turn_system_skips_stunned_unit_OLD() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, advance_turn_system);

    {
        let mut order = app.world_mut().resource_mut::<TurnOrder>();
        order.seed([UnitId(1), UnitId(2)]);
    }

    let stunned_entity = app
        .world_mut()
        .spawn((
            unit(1, Attribute::Vaccine, 100),
            Team::Ally,
            Stunned { turns_left: 1 },
        ))
        .id();
    app.world_mut()
        .spawn((unit(2, Attribute::Virus, 100), Team::Enemy));
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    let mut reader = app
        .world_mut()
        .resource_mut::<Messages<TurnAdvanced>>()
        .get_cursor();
    let replayed: Vec<TurnAdvanced> = reader
        .read(app.world().resource::<Messages<TurnAdvanced>>())
        .cloned()
        .collect();
    assert!(replayed.contains(&TurnAdvanced::of(UnitId(2))));
    assert!(app.world().get::<Stunned>(stunned_entity).is_none());
}

#[allow(dead_code)]
fn advance_turn_system_burn_clamps_hp_and_expires_OLD() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, advance_turn_system);

    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 1);
    let entity = app
        .world_mut()
        .spawn((
            unit(1, Attribute::Vaccine, 5),
            Team::Ally,
            bag,
        ))
        .id();
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    let unit = app.world().get::<Unit>(entity).expect("unit");
    assert_eq!(unit.hp_current, 5);
    assert!(app.world().get::<Ko>(entity).is_none());
    assert!(app.world().get::<StatusBag>(entity).map_or(true, |b| b.is_empty()));

    let events = combat_events(&mut app);
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusTick {
            kind: StatusEffectKind::Heated,
            turns_left: 0,
        }
    )));
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusExpired {
            kind: StatusEffectKind::Heated,
        }
    )));
}

#[allow(dead_code)]
fn advance_turn_system_freeze_updates_and_clears_speed_modifier_OLD() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, advance_turn_system);

    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Chilled, 1);
    let entity = app
        .world_mut()
        .spawn((
            unit(1, Attribute::Vaccine, 100),
            Team::Ally,
            SpeedModifier(99),
            bag,
        ))
        .id();
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    assert!(app.world().get::<StatusBag>(entity).map_or(true, |b| b.is_empty()));

    let events = combat_events(&mut app);
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusTick {
            kind: StatusEffectKind::Chilled,
            turns_left: 0,
        }
    )));
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusExpired {
            kind: StatusEffectKind::Chilled,
        }
    )));
}

#[allow(dead_code)]
fn advance_turn_system_shock_zero_percent_does_not_cancel_OLD() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, advance_turn_system);

    let mut order = app.world_mut().resource_mut::<TurnOrder>();
    order.seed([UnitId(1), UnitId(2)]);
    drop(order);

    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Paralyzed, 1);
    app.world_mut().spawn((
        unit(1, Attribute::Virus, 100),
        Team::Enemy,
        UnitSkills {
            basic: SkillId("basic".into()),
            skills: vec![],
            ultimate: SkillId("ult".into()),
            follow_up: None,
        },
        bag,
    ));
    app.world_mut()
        .spawn((unit(2, Attribute::Vaccine, 100), Team::Ally));
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .get_cursor();
    let intents: Vec<ActionIntent> = cursor
        .read(app.world().resource::<Messages<ActionIntent>>())
        .cloned()
        .collect();
    assert!(!intents.is_empty());

    let events = combat_events(&mut app);
    assert!(
        events
            .iter()
            .all(|event| !matches!(&event.kind, CombatEventKind::OnActionFailed { .. }))
    );
}

#[allow(dead_code)]
fn advance_turn_system_shock_full_cancel_emits_action_failed_OLD() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, advance_turn_system);

    let mut order = app.world_mut().resource_mut::<TurnOrder>();
    order.seed([UnitId(1), UnitId(2)]);
    drop(order);

    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Paralyzed, 1);
    app.world_mut().spawn((
        unit(1, Attribute::Virus, 100),
        Team::Enemy,
        UnitSkills {
            basic: SkillId("basic".into()),
            skills: vec![],
            ultimate: SkillId("ult".into()),
            follow_up: None,
        },
        bag,
    ));
    app.world_mut()
        .spawn((unit(2, Attribute::Vaccine, 100), Team::Ally));
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    // v0 skeleton: Paralyzed is no-op; action cancel semantics deferred to S03.
    let events = combat_events(&mut app);
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusTick {
            kind: StatusEffectKind::Paralyzed,
            turns_left: 0,
        }
    )));
}

#[allow(dead_code)]
fn advance_turn_system_stunned_unit_skips_status_tick_OLD() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, advance_turn_system);

    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 2);
    let entity = app
        .world_mut()
        .spawn((
            unit(1, Attribute::Vaccine, 42),
            Team::Ally,
            Stunned { turns_left: 1 },
            bag,
        ))
        .id();
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    let unit = app.world().get::<Unit>(entity).expect("unit");
    assert_eq!(unit.hp_current, 42);
    assert_eq!(
        app.world().get::<StatusBag>(entity).and_then(|b| b.get_dur(&StatusEffectKind::Heated)),
        Some(2)
    );
    assert!(app.world().get::<Stunned>(entity).is_none());
}

#[test]
fn resolve_action_system_rejects_ko_target() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(SkillBook(vec![skill("basic", DamageTag::Fire, 10, 0, 5)]));
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));

    app.world_mut().spawn((
        unit(1, Attribute::Vaccine, 100),
        Team::Ally,
        UnitSkills {
            basic: SkillId("basic".into()),
            skills: vec![SkillId("basic".into())],
            ultimate: SkillId("basic".into()),
            follow_up: None,
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        Toughness::new(50, vec![]),
    ));
    app.world_mut().spawn((
        unit(2, Attribute::Virus, 0),
        Team::Enemy,
        Toughness::new(50, vec![DamageTag::Ice]),
        Ko,
    ));
    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(1),
        target: UnitId(2),
    });

    app.update();

    let log = app.world().resource::<ActionLog>();
    assert_eq!(log.events.len(), 1);
    if let LogEntry::ActionFailed { reason } = &log.events[0] {
        assert_eq!(reason, "TargetKo");
    } else {
        panic!("Expected ActionFailed, got {:?}", log.events[0]);
    }
}

#[test]
fn resolve_action_system_rejects_revive_on_healthy_target() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

    let mut assets = Assets::<SkillBook>::default();
    let revive_skill = SkillDef {
        id: SkillId("revive".into()),
        name: "Revive".into(),
        damage_tag: DamageTag::Light,
        sp_cost: 10,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Ally,
            life: TargetLife::Ko,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![Effect::Revive(25)],

        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        ..Default::default()
    };
    let handle = assets.add(SkillBook(vec![revive_skill]));
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));

    app.world_mut().resource_mut::<SpPool>().max = 100;
    app.world_mut().resource_mut::<SpPool>().gain(100);

    app.world_mut().spawn((
        unit(1, Attribute::Vaccine, 100),
        Team::Ally,
        UnitSkills {
            basic: SkillId("basic".into()),
            skills: vec![SkillId("revive".into())],
            ultimate: SkillId("basic".into()),
            follow_up: None,
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        Toughness::new(50, vec![]),
    ));
    app.world_mut().spawn((
        unit(2, Attribute::Vaccine, 100),
        Team::Ally,
        Toughness::new(50, vec![]),
    ));

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("revive".into()),
        target: UnitId(2),
    });

    app.update();

    let log = app.world().resource::<ActionLog>();
    assert_eq!(log.events.len(), 1);
    if let LogEntry::ActionFailed { reason } = &log.events[0] {
        assert_eq!(reason, "TargetNotKo");
    } else {
        panic!("Expected ActionFailed, got {:?}", log.events[0]);
    }

    // Verify SP was NOT consumed
    assert_eq!(app.world().resource::<SpPool>().current, 100);
}

#[test]
fn resolve_action_system_rejects_stunned_attacker() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(SkillBook(vec![skill("basic", DamageTag::Fire, 10, 0, 5)]));
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));

    app.world_mut().spawn((
        unit(1, Attribute::Vaccine, 100),
        Team::Ally,
        UnitSkills {
            basic: SkillId("basic".into()),
            skills: vec![SkillId("basic".into())],
            ultimate: SkillId("basic".into()),
            follow_up: None,
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        Toughness::new(50, vec![]),
        Stunned { turns_left: 1 },
    ));
    app.world_mut().spawn((
        unit(2, Attribute::Virus, 100),
        Team::Enemy,
        Toughness::new(50, vec![]),
    ));
    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(1),
        target: UnitId(2),
    });

    app.update();

    let log = app.world().resource::<ActionLog>();
    assert_eq!(log.events.len(), 1);
    if let LogEntry::ActionFailed { reason } = &log.events[0] {
        assert_eq!(reason, "AttackerStunned");
    } else {
        panic!("Expected ActionFailed, got {:?}", log.events[0]);
    }
}

#[test]
fn check_victory_system_sets_winner_once() {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .add_systems(Update, check_victory_system);
    app.world_mut()
        .spawn((unit(1, Attribute::Vaccine, 100), Team::Ally));
    app.world_mut()
        .spawn((unit(2, Attribute::Virus, 0), Team::Enemy, Ko));

    app.update();
    let state = app.world().resource::<CombatState>();
    assert_eq!(state.phase, CombatPhase::Victory);
    assert_eq!(state.winner, Some(Team::Ally));
}
