use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::action_query::{
    ActionQueryKind, CombatQuerySnapshot, UnitQuerySnapshot, query_intent_legality,
};
use bevyrogue::combat::counterplay::EnemyCounterplayKit;
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::{CombatPhase, CombatState};
use bevyrogue::combat::stun::Stunned;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::turn_order::TurnOrder;
use bevyrogue::combat::turn_system::{ActionIntent, resolve_action_system};
use bevyrogue::combat::types::{Attribute, DamageTag, EvoStage, SkillId, UnitId};
use bevyrogue::combat::ultimate::{UltAccumulationTrigger, UltimateCharge};
use bevyrogue::combat::unit::{Commander, Ko, Unit};
use bevyrogue::data::SkillBookHandle;
use bevyrogue::data::skills_ron::{
    Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
    SkillTargeting, TargetHpRule, TargetLife, TargetShape, TargetSide,
};

#[derive(Clone)]
struct UnitFixture {
    id: UnitId,
    team: Team,
    hp_current: i32,
    hp_max: i32,
    is_ko: bool,
    is_stunned: bool,
    is_commander: bool,
    skills: Option<UnitSkills>,
    counterplay: Option<EnemyCounterplayKit>,
}

struct ParityCase {
    name: &'static str,
    actor_id: UnitId,
    target_id: UnitId,
    active_unit: Option<UnitId>,
    actor: UnitFixture,
    target: UnitFixture,
    extras: Vec<UnitFixture>,
    skill: SkillDef,
    intent: ActionIntent,
    expected_reason: LegalityReasonCode,
}

fn offensive_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: "Offensive Skill".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Damage {
            amount: 10,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn revive_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: "Revive Skill".into(),
        damage_tag: DamageTag::Light,
        sp_cost: 3,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Ally,
            life: TargetLife::Ko,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Revive(25)],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn unit_skills(skill_id: &str) -> UnitSkills {
    let skill_id = SkillId(skill_id.into());
    UnitSkills {
        basic: skill_id.clone(),
        skills: vec![skill_id.clone()],
        ultimate: skill_id,
        follow_up: None,
    }
}

fn unit_fixture(
    id: u32,
    team: Team,
    hp_current: i32,
    hp_max: i32,
    is_ko: bool,
    is_stunned: bool,
    is_commander: bool,
    skills: Option<UnitSkills>,
) -> UnitFixture {
    UnitFixture {
        id: UnitId(id),
        team,
        hp_current,
        hp_max,
        is_ko,
        is_stunned,
        is_commander,
        skills,
        counterplay: None,
    }
}

fn unit_component(fixture: &UnitFixture) -> Unit {
    Unit {
        id: fixture.id,
        name: format!("Unit{}", fixture.id.0),
        hp_max: fixture.hp_max,
        hp_current: fixture.hp_current,
        attribute: match fixture.team {
            Team::Ally => Attribute::Vaccine,
            Team::Enemy => Attribute::Virus,
        },
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn spawn_fixture(world: &mut World, fixture: &UnitFixture) -> Entity {
    let mut entity = world.spawn((unit_component(fixture), fixture.team));
    if let Some(skills) = &fixture.skills {
        entity.insert(skills.clone());
        entity.insert(UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        });
    }
    if fixture.is_ko {
        entity.insert(Ko);
    }
    if let Some(counterplay) = &fixture.counterplay {
        entity.insert(counterplay.clone());
    }

    if fixture.is_stunned {
        entity.insert(Stunned { turns_left: 1 });
    }
    if fixture.is_commander {
        entity.insert(Commander);
    }
    entity.id()
}

fn build_app(skill_book: SkillBook) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(skill_book);

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .insert_resource(ActionLog::default())
        .insert_resource(SpPool::default())
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

    app
}

fn build_snapshot(case: &ParityCase) -> CombatQuerySnapshot {
    let mut units = vec![
        snapshot_unit(
            &case.actor,
            case.actor_id == case.active_unit.unwrap_or(case.actor_id),
        ),
        snapshot_unit(
            &case.target,
            case.target_id == case.active_unit.unwrap_or(case.target_id),
        ),
    ];
    units.extend(
        case.extras
            .iter()
            .map(|fixture| snapshot_unit(fixture, case.active_unit == Some(fixture.id))),
    );

    let acting_unit = units
        .iter()
        .find(|unit| unit.id == case.actor_id)
        .cloned()
        .expect("actor must exist in snapshot");
    let target_unit = units.iter().find(|unit| unit.id == case.target_id).cloned();

    CombatQuerySnapshot {
        phase: CombatPhase::WaitingAction,
        acting_unit,
        target_unit,
        units,
    }
}

fn snapshot_unit(fixture: &UnitFixture, is_active: bool) -> UnitQuerySnapshot {
    UnitQuerySnapshot {
        id: fixture.id,
        team: fixture.team,
        is_active,
        is_ko: fixture.is_ko,
        is_stunned: fixture.is_stunned,
        is_commander: fixture.is_commander,
        hp_current: fixture.hp_current,
        hp_max: fixture.hp_max,
        sp: i32::MAX,
        ultimate_current: 0,
        ultimate_trigger: 100,
        ultimate_ready: false,
        energy: 0,
        skills: fixture.skills.clone(),
        toughness: None,
        ..Default::default()
    }
}

fn drain_events(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn run_case(case: ParityCase) {
    let mut app = build_app(SkillBook(vec![case.skill.clone()]));

    let _actor_entity = spawn_fixture(app.world_mut(), &case.actor);
    let target_entity = spawn_fixture(app.world_mut(), &case.target);
    for extra in &case.extras {
        spawn_fixture(app.world_mut(), extra);
    }

    if let Some(active_unit) = case.active_unit {
        app.world_mut().resource_mut::<TurnOrder>().active_unit = Some(active_unit);
    }

    let snapshot = build_snapshot(&case);
    let expected_reason = case.expected_reason.clone();
    let legality = query_intent_legality(
        &snapshot,
        app.world()
            .resource::<Assets<SkillBook>>()
            .get(&app.world().resource::<SkillBookHandle>().0)
            .expect("skill book asset"),
        case.actor_id,
        &ActionQueryKind::Skill(&case.skill.id),
        case.target_id,
    );
    assert_eq!(
        legality,
        Err(expected_reason.clone()),
        "pure query should reject {} with {:?}",
        case.name,
        expected_reason
    );

    let target_before = {
        let unit = app.world().get::<Unit>(target_entity).expect("target unit");
        (
            unit.hp_current,
            app.world().get::<Ko>(target_entity).is_some(),
        )
    };

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    app.world_mut().write_message(case.intent.clone());
    app.update();

    let events = drain_events(&mut cursor, &app);
    let failed: Vec<_> = events
        .iter()
        .filter(|event| matches!(event.kind, CombatEventKind::OnActionFailed { .. }))
        .collect();
    assert_eq!(
        failed.len(),
        1,
        "{} should emit exactly one failure event: {events:?}",
        case.name
    );
    assert!(
        matches!(
            &failed[0].kind,
            CombatEventKind::OnActionFailed { reason } if reason == &format!("{:?}", case.expected_reason)
        ),
        "{} should emit OnActionFailed with {:?}: {events:?}",
        case.name,
        case.expected_reason
    );

    assert!(
        !events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionDeclared { .. })),
        "{} should not declare an action on a rejected intent",
        case.name
    );
    assert!(
        !events.iter().any(|event| matches!(
            event.kind,
            CombatEventKind::OnActionPreApp
                | CombatEventKind::OnActionApplied
                | CombatEventKind::OnActionResolved
        )),
        "{} should not emit lifecycle events on a rejected intent",
        case.name
    );

    let target_after = {
        let unit = app.world().get::<Unit>(target_entity).expect("target unit");
        (
            unit.hp_current,
            app.world().get::<Ko>(target_entity).is_some(),
        )
    };
    assert_eq!(
        target_after, target_before,
        "{} should not mutate target state",
        case.name
    );
}

#[test]
fn revive_live_ally_matches_target_not_ko() {
    run_case(ParityCase {
        name: "revive live ally",
        actor_id: UnitId(1),
        target_id: UnitId(2),
        active_unit: Some(UnitId(1)),
        actor: unit_fixture(
            1,
            Team::Ally,
            100,
            100,
            false,
            false,
            false,
            Some(unit_skills("revive_skill")),
        ),
        target: unit_fixture(2, Team::Ally, 100, 100, false, false, false, None),
        extras: vec![],
        skill: revive_skill("revive_skill"),
        intent: ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("revive_skill".into()),
            target: UnitId(2),
        },
        expected_reason: LegalityReasonCode::TargetNotKo,
    });
}

#[test]
fn offensive_ko_enemy_matches_target_ko() {
    run_case(ParityCase {
        name: "offensive KO enemy",
        actor_id: UnitId(1),
        target_id: UnitId(2),
        active_unit: Some(UnitId(1)),
        actor: unit_fixture(
            1,
            Team::Ally,
            100,
            100,
            false,
            false,
            false,
            Some(unit_skills("offensive_skill")),
        ),
        target: unit_fixture(2, Team::Enemy, 0, 100, true, false, false, None),
        extras: vec![],
        skill: offensive_skill("offensive_skill"),
        intent: ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("offensive_skill".into()),
            target: UnitId(2),
        },
        expected_reason: LegalityReasonCode::TargetKo,
    });
}

#[test]
fn offensive_ally_matches_wrong_side() {
    run_case(ParityCase {
        name: "offensive ally",
        actor_id: UnitId(1),
        target_id: UnitId(2),
        active_unit: Some(UnitId(1)),
        actor: unit_fixture(
            1,
            Team::Ally,
            100,
            100,
            false,
            false,
            false,
            Some(unit_skills("offensive_skill")),
        ),
        target: unit_fixture(2, Team::Ally, 100, 100, false, false, false, None),
        extras: vec![],
        skill: offensive_skill("offensive_skill"),
        intent: ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("offensive_skill".into()),
            target: UnitId(2),
        },
        expected_reason: LegalityReasonCode::WrongSide,
    });
}

#[test]
fn commander_target_matches_target_is_commander() {
    run_case(ParityCase {
        name: "commander target",
        actor_id: UnitId(1),
        target_id: UnitId(2),
        active_unit: Some(UnitId(1)),
        actor: unit_fixture(
            1,
            Team::Ally,
            100,
            100,
            false,
            false,
            false,
            Some(unit_skills("offensive_skill")),
        ),
        target: unit_fixture(2, Team::Enemy, 100, 100, false, false, true, None),
        extras: vec![],
        skill: offensive_skill("offensive_skill"),
        intent: ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("offensive_skill".into()),
            target: UnitId(2),
        },
        expected_reason: LegalityReasonCode::TargetIsCommander,
    });
}

#[test]
fn ko_attacker_matches_attacker_ko() {
    run_case(ParityCase {
        name: "KO attacker",
        actor_id: UnitId(1),
        target_id: UnitId(2),
        active_unit: Some(UnitId(1)),
        actor: unit_fixture(
            1,
            Team::Ally,
            0,
            100,
            true,
            false,
            false,
            Some(unit_skills("offensive_skill")),
        ),
        target: unit_fixture(2, Team::Enemy, 100, 100, false, false, false, None),
        extras: vec![],
        skill: offensive_skill("offensive_skill"),
        intent: ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("offensive_skill".into()),
            target: UnitId(2),
        },
        expected_reason: LegalityReasonCode::AttackerKo,
    });
}

#[test]
fn stunned_attacker_matches_attacker_stunned() {
    run_case(ParityCase {
        name: "stunned attacker",
        actor_id: UnitId(1),
        target_id: UnitId(2),
        active_unit: Some(UnitId(1)),
        actor: unit_fixture(
            1,
            Team::Ally,
            100,
            100,
            false,
            true,
            false,
            Some(unit_skills("offensive_skill")),
        ),
        target: unit_fixture(2, Team::Enemy, 100, 100, false, false, false, None),
        extras: vec![],
        skill: offensive_skill("offensive_skill"),
        intent: ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("offensive_skill".into()),
            target: UnitId(2),
        },
        expected_reason: LegalityReasonCode::AttackerStunned,
    });
}

#[test]
fn non_active_actor_matches_not_active_unit() {
    run_case(ParityCase {
        name: "non-active actor",
        actor_id: UnitId(1),
        target_id: UnitId(3),
        active_unit: Some(UnitId(2)),
        actor: unit_fixture(
            1,
            Team::Ally,
            100,
            100,
            false,
            false,
            false,
            Some(unit_skills("offensive_skill")),
        ),
        target: unit_fixture(3, Team::Enemy, 100, 100, false, false, false, None),
        extras: vec![unit_fixture(
            2,
            Team::Enemy,
            100,
            100,
            false,
            false,
            false,
            None,
        )],
        skill: offensive_skill("offensive_skill"),
        intent: ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("offensive_skill".into()),
            target: UnitId(3),
        },
        expected_reason: LegalityReasonCode::NotActiveUnit,
    });
}
