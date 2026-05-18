use bevy::prelude::*;
use bevyrogue::combat::{
    kit::UnitSkills,
    log::{ActionLog, LogEntry},
    sp::SpPool,
    state::{CombatPhase, CombatState},
    team::Team,
    toughness::Toughness,
    turn_system::{ActionIntent, check_victory_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
};

fn sync_events(app: &App, collected: &mut Vec<LogEntry>) {
    let log = app.world().resource::<ActionLog>();
    for ev in log.events.iter() {
        let key = format!("{ev:?}");
        if !collected.iter().any(|c| format!("{c:?}") == key) {
            collected.push(ev.clone());
        }
    }
}

#[test]
fn encounter_e2e_2v2_victory() {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<bevyrogue::combat::turn_order::TurnOrder>()
        .add_message::<ActionIntent>()
        .add_message::<bevyrogue::combat::events::CombatEvent>()
        .add_systems(
            Update,
            (resolve_action_system, check_victory_system).chain(),
        );

    let mut assets = Assets::<SkillBook>::default();
    let book = SkillBook(vec![
        SkillDef {
            id: SkillId("basic_a1".into()),
            name: "Basic A1".into(),
            damage_tag: DamageTag::Fire,
            sp_cost: 0,
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
                    amount: 15,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
                },
                Effect::ToughnessHit(5),
            ],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
        SkillDef {
            id: SkillId("skill_a1".into()),
            name: "Skill A1".into(),
            damage_tag: DamageTag::Fire,
            sp_cost: 0,
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
                    amount: 30,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
                },
                Effect::ToughnessHit(20),
            ],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
        SkillDef {
            id: SkillId("basic_a2".into()),
            name: "Basic A2".into(),
            damage_tag: DamageTag::Ice,
            sp_cost: 0,
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
                    amount: 15,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
                },
                Effect::ToughnessHit(5),
            ],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
        SkillDef {
            id: SkillId("skill_a2".into()),
            name: "Skill A2".into(),
            damage_tag: DamageTag::Ice,
            sp_cost: 0,
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
                    amount: 30,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
                },
                Effect::ToughnessHit(20),
            ],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
    ]);
    let handle = assets.add(book);
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    app.insert_resource(ActionLog::default());
    app.insert_resource(SpPool::default());
    app.init_resource::<Time>();

    let unit = |id: u32, name: &str, hp: i32, attribute: Attribute| Unit {
        id: UnitId(id),
        name: name.into(),
        hp_max: hp,
        hp_current: hp,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    };

    app.world_mut().spawn((
        unit(1, "Ally1", 100, Attribute::Vaccine),
        Team::Ally,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("basic_a1".into()),
            skills: vec![SkillId("skill_a1".into())],
            ultimate: SkillId("skill_a1".into()),
            follow_up: None,
        },
    ));
    app.world_mut().spawn((
        unit(2, "Ally2", 100, Attribute::Data),
        Team::Ally,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("basic_a2".into()),
            skills: vec![SkillId("skill_a2".into())],
            ultimate: SkillId("skill_a2".into()),
            follow_up: None,
        },
    ));
    app.world_mut().spawn((
        unit(3, "Enemy3", 40, Attribute::Virus),
        Team::Enemy,
        Toughness::new(15, vec![DamageTag::Fire]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("basic_a1".into()),
            skills: vec![],
            ultimate: SkillId("basic_a1".into()),
            follow_up: None,
        },
    ));
    app.world_mut().spawn((
        unit(4, "Enemy4", 40, Attribute::Free),
        Team::Enemy,
        Toughness::new(15, vec![DamageTag::Ice]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("basic_a2".into()),
            skills: vec![],
            ultimate: SkillId("basic_a2".into()),
            follow_up: None,
        },
    ));

    let mut collected = Vec::new();
    for intent in [
        ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(3),
        },
        ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("skill_a1".into()),
            target: UnitId(3),
        },
        ActionIntent::Skill {
            attacker: UnitId(2),
            skill_id: SkillId("skill_a2".into()),
            target: UnitId(4),
        },
        ActionIntent::Basic {
            attacker: UnitId(2),
            target: UnitId(4),
        },
    ] {
        app.world_mut().write_message(intent);
        app.update();
        sync_events(&app, &mut collected);
    }
    app.update();

    assert!(
        collected
            .iter()
            .any(|e| matches!(e, LogEntry::BasicHit { .. }))
    );
    assert!(
        collected
            .iter()
            .any(|e| matches!(e, LogEntry::Break { .. }))
    );
    assert!(
        collected
            .iter()
            .filter(|e| matches!(e, LogEntry::Ko { .. }))
            .count()
            >= 2
    );

    let first_basic = collected
        .iter()
        .position(|e| matches!(e, LogEntry::BasicHit { .. }))
        .unwrap();
    let first_break = collected
        .iter()
        .position(|e| matches!(e, LogEntry::Break { .. }))
        .unwrap();
    let first_ko = collected
        .iter()
        .position(|e| matches!(e, LogEntry::Ko { .. }))
        .unwrap();
    assert!(first_ko > first_basic);
    assert!(first_ko > first_break);

    let state = app.world().resource::<CombatState>();
    assert_eq!(state.phase, CombatPhase::Victory);
    assert_eq!(state.winner, Some(Team::Ally));
}
