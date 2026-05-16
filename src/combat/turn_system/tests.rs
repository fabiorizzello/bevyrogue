use super::*;
use crate::combat::{
    events::CombatEvent,
    kit::UnitSkills,
    log::{ActionLog, LogEntry},
    team::Team,
    toughness::Toughness,
    types::{Attribute, DamageTag, EvoStage},
    ultimate::UltAccumulationTrigger,
};
use crate::data::skills_ron::{
    Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
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
        legacy_ops: vec![
            Effect::Damage {
                amount: damage,
                target: TargetShape::Single,
                per_hop: Default::default(),
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
        legacy_ops: vec![Effect::Revive(25)],

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
