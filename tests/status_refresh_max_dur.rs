/// Integration test for §H.1 refresh_max_dur policy via the skill-apply pipeline.
/// Neutral matchup (Vaccine vs Vaccine) so accuracy never gates (threshold=100).
use bevy::prelude::*;
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
    events::CombatEvent,
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
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

fn heated_skill(id: &str, duration: u32) -> SkillDef {
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
            Effect::ApplyStatus {
                kind: StatusEffectKind::Heated,
                duration,
            },
        ],
        ..Default::default()
    }
}

fn setup_app() -> (App, Entity) {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(SkillBook(vec![
        heated_skill("h2", 2),
        heated_skill("h1", 1),
        heated_skill("h5", 5),
    ]));
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool {
            current: 100,
            max: 100,
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(0))
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

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
            basic: SkillId("h2".into()),
            skills: vec![SkillId("h1".into()), SkillId("h5".into())],
            ultimate: SkillId("h2".into()),
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
fn refresh_max_dur_keeps_longer_and_replaces_with_longer() {
    let (mut app, defender) = setup_app();

    // Apply Heated(dur=2).
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("h2".into()),
        target: UnitId(2),
    });
    app.update();

    let bag = app.world().get::<StatusBag>(defender).unwrap();
    assert_eq!(
        bag.get_dur(&StatusEffectKind::Heated),
        Some(2),
        "initial apply: dur must be 2"
    );
    assert_eq!(bag.iter().count(), 1, "single instance after first apply");

    // Re-apply Heated(dur=1): refresh_max_dur must keep max(2, 1) = 2.
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("h1".into()),
        target: UnitId(2),
    });
    app.update();

    let bag = app.world().get::<StatusBag>(defender).unwrap();
    assert_eq!(
        bag.get_dur(&StatusEffectKind::Heated),
        Some(2),
        "re-apply with shorter dur: refresh_max_dur must keep max(2,1)=2"
    );
    assert_eq!(
        bag.iter().count(),
        1,
        "re-apply must not duplicate the instance"
    );

    // Apply Heated(dur=5): refresh_max_dur must update to max(2, 5) = 5.
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("h5".into()),
        target: UnitId(2),
    });
    app.update();

    let bag = app.world().get::<StatusBag>(defender).unwrap();
    assert_eq!(
        bag.get_dur(&StatusEffectKind::Heated),
        Some(5),
        "re-apply with longer dur: refresh_max_dur must update to max(2,5)=5"
    );
    assert_eq!(
        bag.iter().count(),
        1,
        "still exactly one instance after three applies"
    );
}
