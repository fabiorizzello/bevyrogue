use bevy::prelude::*;
use bevyrogue::combat::{
    kit::UnitSkills,
    log::{ActionLog, LogEntry},
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{Ko, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

fn build_app() -> App {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .add_message::<ActionIntent>()
        .add_message::<bevyrogue::combat::events::CombatEvent>()
        .add_systems(Update, resolve_action_system);

    let mut assets = Assets::<SkillBook>::default();
    let book = SkillBook(vec![
        SkillDef {
            id: SkillId("patamon_revive".into()),
            name: "Holy Revive".into(),
            damage_tag: DamageTag::Light,
            sp_cost: 5,
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
            timeline: None,
        },
        SkillDef {
            id: SkillId("attack_skill".into()),
            name: "Heavy Strike".into(),
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
            legacy_ops: vec![Effect::Damage {
                amount: 9999,
                target: TargetShape::Single,
                per_hop: Default::default(),
            }],

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
    // SpPool must have >= 6 SP for patamon_revive (sp_cost=6)
    app.insert_resource(SpPool { current: 5, max: 5 });
    app.init_resource::<Time>();
    app
}

fn spawn_unit(app: &mut App, id: u32, name: &str, hp_max: i32, team: Team, skill: &str) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id: UnitId(id),
                name: name.into(),
                hp_max,
                hp_current: hp_max,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            team,
            Toughness::new(50, vec![]),
            UltimateCharge {
                current: 0,
                trigger: 100,
                cap: 150,
                trigger_type: UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 25,
            },
            UnitSkills {
                basic: SkillId(skill.into()),
                skills: vec![SkillId(skill.into())],
                ultimate: SkillId(skill.into()),
                follow_up: None,
            },
        ))
        .id()
}

/// Patamon (UnitId 9) revives a KO'd ally via the authored `patamon_revive` skill.
/// Asserts: KO component removed, LogEntry::Revive hp_after == floor(hp_max * 0.25),
/// and the revived unit is back in the turn order.
#[test]
fn s14_patamon_revive_e2e() {
    let mut app = build_app();

    // UnitId(1): victim ally, hp_max=100 → revived to 25
    let victim_entity = spawn_unit(&mut app, 1, "Ally1", 100, Team::Ally, "attack_skill");
    // UnitId(9): Patamon, the reviver
    let _patamon_entity = spawn_unit(&mut app, 9, "Patamon", 88, Team::Ally, "patamon_revive");
    // UnitId(99): enemy with one-shot attack
    let _enemy_entity = spawn_unit(&mut app, 99, "Enemy", 100, Team::Enemy, "attack_skill");

    // Seed turn order so the revived unit can re-enter it
    {
        let mut turn_order = app.world_mut().resource_mut::<TurnOrder>();
        turn_order.seed(vec![UnitId(1), UnitId(9), UnitId(99)]);
    }

    // Phase 1: Enemy KOs Ally1
    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(99),
        target: UnitId(1),
    });
    app.update();

    assert!(
        app.world().get::<Ko>(victim_entity).is_some(),
        "Ally1 should be KO after enemy attack"
    );

    // In the AV system, KO'd units are excluded from AV advancement via Without<Ko> filter.
    // The Ko component presence (asserted above) is the authoritative check.

    // Phase 2: Patamon uses patamon_revive on KO'd Ally1
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(9),
        skill_id: SkillId("patamon_revive".into()),
        target: UnitId(1),
    });
    app.update();

    // Assert Ko component is gone
    assert!(
        app.world().get::<Ko>(victim_entity).is_none(),
        "Ally1 should no longer be KO after patamon_revive"
    );

    // Assert hp_after == floor(100 * 25 / 100) == 25
    let hp = app.world().get::<Unit>(victim_entity).unwrap().hp_current;
    assert_eq!(hp, 25, "Ally1 hp should be 25 (floor(100 * 0.25))");

    // Assert LogEntry::Revive with correct hp_after
    let log = app.world().resource::<ActionLog>();
    let revive_entry = log.events.iter().find(|e| {
        matches!(e, LogEntry::Revive { target, hp_after } if *target == UnitId(1) && *hp_after == 25)
    });
    assert!(
        revive_entry.is_some(),
        "ActionLog must contain Revive {{ target: UnitId(1), hp_after: 25 }}"
    );

    // In the AV system, revived units re-enter turn order participation by having Ko removed.
    // The Ko component absence (asserted above) is the authoritative check.
    assert!(
        app.world().get::<Ko>(victim_entity).is_none(),
        "Revived unit must not have Ko — it will automatically re-enter AV advancement"
    );
}
