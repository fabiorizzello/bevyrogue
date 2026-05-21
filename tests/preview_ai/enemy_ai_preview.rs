use bevy::prelude::*;
use bevyrogue::combat::{
    enemy_ai::{EnemyTurnContext, TargetInfo, pick_enemy_action_with_preview},
    events::CombatEvent,
    kit::UnitSkills,
    preview::PreviewDamageSummary,
    runtime::{
        CastIdGen,
        timeline::{Beat, BeatEdge, BeatKind, BeatPayload, CompiledTimeline, TimelineLibrary},
    },
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{
        ActionIntent, EnemyTurnRequestQueue, advance_turn_system, resolve_enemy_turn_action_system,
    },
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::skills_ron::TargetShape;

fn make_unit(id: u32, name: &str, hp: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: name.into(),
        hp_max: hp,
        hp_current: hp,
        attribute: Attribute::Vaccine,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn enemy_skills() -> UnitSkills {
    UnitSkills {
        basic: SkillId("enemy_basic".into()),
        skills: vec![SkillId("enemy_skill_fire".into())],
        ultimate: SkillId("enemy_ult_fire".into()),
        follow_up: None,
    }
}

fn timeline(id: &str, damage: i32) -> CompiledTimeline<String> {
    CompiledTimeline {
        id: id.into(),
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
                    amount: damage,
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
    }
}

fn build_app() -> App {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<EnemyTurnRequestQueue>()
        .init_resource::<TimelineLibrary<String>>()
        .init_resource::<Time>()
        .init_resource::<CastIdGen>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<bevyrogue::combat::av::ActionValueUpdated>()
        .add_systems(
            Update,
            (advance_turn_system, resolve_enemy_turn_action_system).chain(),
        );

    app.world_mut()
        .resource_mut::<TimelineLibrary<String>>()
        .timelines = vec![
        timeline("enemy_basic", 12),
        timeline("enemy_skill_fire", 48),
        timeline("enemy_ult_fire", 24),
    ];

    app
}

fn action_cursor(app: &mut App) -> bevy::ecs::message::MessageCursor<ActionIntent> {
    app.world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .get_cursor()
}

#[test]
fn preview_scoring_prefers_higher_damage_action() {
    let skills = UnitSkills {
        basic: SkillId("enemy_basic".into()),
        skills: vec![SkillId("enemy_skill_fire".into())],
        ultimate: SkillId("enemy_ult_fire".into()),
        follow_up: None,
    };
    let targets = vec![
        TargetInfo {
            id: UnitId(1),
            toughness_current: 50,
            toughness_max: 100,
            hp_current: 100,
            hp_max: 100,
        },
        TargetInfo {
            id: UnitId(2),
            toughness_current: 40,
            toughness_max: 100,
            hp_current: 100,
            hp_max: 100,
        },
    ];

    let ctx = EnemyTurnContext {
        attacker_id: UnitId(101),
        attacker_skills: &skills,
        attacker_ult_ready: true,
        targets: &targets,
    };

    let intent = pick_enemy_action_with_preview(&ctx, |skill_id, _target| {
        let damage = match skill_id.0.as_str() {
            "enemy_basic" => 12,
            "enemy_skill_fire" => 48,
            "enemy_ult_fire" => 24,
            other => panic!("unexpected skill id: {other}"),
        };
        Some(PreviewDamageSummary {
            total_damage: damage,
            deal_damage_intents: 1,
        })
    })
    .expect("preview scoring should produce an intent");

    assert!(
        matches!(
            &intent,
            ActionIntent::Skill {
                attacker: UnitId(101),
                skill_id,
                target: UnitId(1)
            } if skill_id.0 == "enemy_skill_fire"
        ),
        "expected preview-highest skill to win, got {:?}",
        intent
    );
}

#[test]
fn runtime_bridge_uses_preview_scoring_and_emits_one_stable_intent() {
    let mut app = build_app();

    app.world_mut().spawn((
        make_unit(1, "AllyA", 200),
        Team::Ally,
        Toughness::new(50, vec![]),
    ));
    app.world_mut().spawn((
        make_unit(2, "AllyB", 200),
        Team::Ally,
        Toughness::new(50, vec![]),
    ));
    app.world_mut().spawn((
        make_unit(101, "Enemy", 100),
        Team::Enemy,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 100,
            trigger: 100,
            cap: 100,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        enemy_skills(),
    ));

    let mut intent_cursor = action_cursor(&mut app);

    app.world_mut().write_message(TurnAdvanced::of(UnitId(101)));
    app.update();

    let intents = intent_cursor
        .read(app.world().resource::<Messages<ActionIntent>>())
        .cloned()
        .collect::<Vec<_>>();

    assert_eq!(
        intents.len(),
        1,
        "expected exactly one ActionIntent, got: {:?}",
        intents
    );
    assert!(
        matches!(
            &intents[0],
            ActionIntent::Skill {
                attacker: UnitId(101),
                skill_id,
                target: UnitId(1),
            } if skill_id.0 == "enemy_skill_fire"
        ),
        "expected preview-driven Skill(enemy_skill_fire -> UnitId(1)), got {:?}",
        intents[0]
    );
}
