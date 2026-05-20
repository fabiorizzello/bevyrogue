/// Integration test for multi-kind coexistence in StatusBag via the skill-apply pipeline.
/// Applies Heated, Chilled, and Blessed to the same target sequentially and asserts
/// all three instances survive with correct durations.
use bevy::prelude::*;
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
    kit::UnitSkills,
    team::Team,
    toughness::Toughness,
    turn_system::ActionIntent,
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::skills_ron::{
    Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting, TargetLife,
    TargetShape, TargetSide,
};

mod common;
use common::app::skill_resolve_app;

fn status_skill(id: &str, kind: StatusEffectKind, duration: u32) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
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
                amount: 1,
                target: TargetShape::Single,
                per_hop: Default::default(),
            },
            Effect::ToughnessHit(0),
            Effect::ApplyStatus { kind, duration },
        ],
        ..Default::default()
    }
}

fn setup_app() -> (App, Entity) {
    let mut app = skill_resolve_app(
        SkillBook(vec![
            status_skill("heated", StatusEffectKind::Heated, 3),
            status_skill("chilled", StatusEffectKind::Chilled, 2),
            status_skill("blessed", StatusEffectKind::Blessed, 4),
        ]),
        0,
    );

    app.world_mut().spawn((
        Unit {
            id: UnitId(1),
            name: "Attacker".into(),
            hp_max: 500,
            hp_current: 500,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        UnitSkills {
            basic: SkillId("heated".into()),
            skills: vec![SkillId("chilled".into()), SkillId("blessed".into())],
            ultimate: SkillId("heated".into()),
            follow_up: None,
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 10,
        },
        Toughness::new(100, vec![]),
        StatusBag::default(),
    ));

    let defender = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(2),
                name: "Defender".into(),
                hp_max: 10_000,
                hp_current: 10_000,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Toughness::new(1_000, vec![]),
            StatusBag::default(),
        ))
        .id();

    (app, defender)
}

#[test]
fn three_different_kinds_coexist_in_bag() {
    let (mut app, defender) = setup_app();

    // Apply Heated(3).
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("heated".into()),
        target: UnitId(2),
    });
    app.update();

    // Apply Chilled(2).
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("chilled".into()),
        target: UnitId(2),
    });
    app.update();

    // Apply Blessed(4).
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("blessed".into()),
        target: UnitId(2),
    });
    app.update();

    let bag = app.world().get::<StatusBag>(defender).unwrap();

    assert_eq!(
        bag.iter().count(),
        3,
        "all three distinct kinds must coexist"
    );
    assert!(bag.has(&StatusEffectKind::Heated), "Heated must be present");
    assert!(
        bag.has(&StatusEffectKind::Chilled),
        "Chilled must be present"
    );
    assert!(
        bag.has(&StatusEffectKind::Blessed),
        "Blessed must be present"
    );
    assert_eq!(bag.get_dur(&StatusEffectKind::Heated), Some(3));
    assert_eq!(bag.get_dur(&StatusEffectKind::Chilled), Some(2));
    assert_eq!(bag.get_dur(&StatusEffectKind::Blessed), Some(4));
}
