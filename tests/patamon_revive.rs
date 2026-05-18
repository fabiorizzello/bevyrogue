use bevy::prelude::*;
use bevyrogue::combat::{
    runtime::{
        ExtRegistries, register_kernel_builtins,
        timeline::{Beat, BeatEdge, BeatKind, BeatPayload, TimelineLibrary},
    },
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
    skill_timeline::{SkillTimeline, compile_skill_book_timelines},
    skills_ron::{
        SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting, TargetLife,
        TargetShape, TargetSide,
    },
};

fn build_app() -> App {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .add_message::<ActionIntent>()
        .add_message::<bevyrogue::combat::events::CombatEvent>()
        .add_systems(Update, resolve_action_system);

    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);

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
            legacy_ops: vec![],
            timeline: Some(SkillTimeline {
                entry: "cast".into(),
                beats: vec![
                    Beat {
                        id: "cast".into(),
                        kind: BeatKind::Cast,
                        hook: None,
                        selector: None,
                        presentation: None,
                        payload: None,
                    },
                    Beat {
                        id: "revive".into(),
                        kind: BeatKind::Impact,
                        hook: Some("core/revive".into()),
                        selector: Some("core/primary".into()),
                        presentation: None,
                        payload: Some(BeatPayload::Revive {
                            pct: 25,
                            target: TargetShape::Single,
                        }),
                    },
                ],
                edges: vec![BeatEdge {
                    from: "cast".into(),
                    to: "revive".into(),
                    gate: Some("core/always".into()),
                }],
            }),
            ..Default::default()
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
            legacy_ops: vec![],
            timeline: Some(SkillTimeline {
                entry: "cast".into(),
                beats: vec![
                    Beat {
                        id: "cast".into(),
                        kind: BeatKind::Cast,
                        hook: None,
                        selector: None,
                        presentation: None,
                        payload: None,
                    },
                    Beat {
                        id: "impact".into(),
                        kind: BeatKind::Impact,
                        hook: Some("core/deal_damage".into()),
                        selector: Some("core/primary".into()),
                        presentation: None,
                        payload: Some(BeatPayload::DealDamage {
                            amount: 9999,
                            tag: DamageTag::Fire,
                            target: TargetShape::Single,
                        }),
                    },
                ],
                edges: vec![BeatEdge {
                    from: "cast".into(),
                    to: "impact".into(),
                    gate: Some("core/always".into()),
                }],
            }),
            ..Default::default()
        },
    ]);

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());
    let compiled = compile_skill_book_timelines(&book, &regs)
        .expect("revive test book must compile into timelines");

    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    app.insert_resource(TimelineLibrary::<String>::default());
    app.insert_resource(regs);
    app.insert_resource(ActionLog::default());
    app.insert_resource(SpPool { current: 5, max: 5 });
    app.init_resource::<Time>();

    app.world_mut()
        .resource_mut::<TimelineLibrary<String>>()
        .timelines = compiled;

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
fn patamon_revive_e2e() {
    let mut app = build_app();

    let victim_entity = spawn_unit(&mut app, 1, "Ally1", 100, Team::Ally, "attack_skill");
    let _patamon_entity = spawn_unit(&mut app, 9, "Patamon", 88, Team::Ally, "patamon_revive");
    let _enemy_entity = spawn_unit(&mut app, 99, "Enemy", 100, Team::Enemy, "attack_skill");

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(99),
        target: UnitId(1),
    });
    app.update();

    assert!(
        app.world().get::<Ko>(victim_entity).is_some(),
        "Ally1 should be KO after enemy attack"
    );

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(9),
        skill_id: SkillId("patamon_revive".into()),
        target: UnitId(1),
    });
    app.update();

    assert!(
        app.world().get::<Ko>(victim_entity).is_none(),
        "Ally1 should no longer be KO after patamon_revive"
    );

    let hp = app.world().get::<Unit>(victim_entity).unwrap().hp_current;
    assert_eq!(hp, 25, "Ally1 hp should be 25 (floor(100 * 0.25))");

    let log = app.world().resource::<ActionLog>();
    let revive_entry = log.events.iter().find(|e| {
        matches!(e, LogEntry::Revive { target, hp_after } if *target == UnitId(1) && *hp_after == 25)
    });
    assert!(
        revive_entry.is_some(),
        "ActionLog must contain Revive {{ target: UnitId(1), hp_after: 25 }}"
    );

    assert!(
        app.world().get::<Ko>(victim_entity).is_none(),
        "Revived unit must not have Ko — it will automatically re-enter AV advancement"
    );
}
