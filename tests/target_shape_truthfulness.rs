use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    events::CombatEventKind,
    log::{ActionLog, LogEntry},
    sp::SpPool,
    state::CombatState,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

fn load_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
}

fn build_app(book: SkillBook) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book);

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool {
            current: 99,
            max: 99,
        })
        .insert_resource(ActionLog::default())
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<bevyrogue::combat::events::CombatEvent>()
        .add_systems(Update, resolve_action_system);
    app
}

fn actor(
    id: u32,
    skill_ids: Vec<SkillId>,
) -> (
    Unit,
    bevyrogue::combat::team::Team,
    UltimateCharge,
    bevyrogue::combat::kit::UnitSkills,
) {
    let basic = skill_ids
        .first()
        .cloned()
        .unwrap_or_else(|| SkillId("baby_flame".into()));
    (
        Unit {
            id: UnitId(id),
            name: format!("Actor{id}"),
            hp_max: 100,
            hp_current: 100,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        bevyrogue::combat::team::Team::Ally,
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        bevyrogue::combat::kit::UnitSkills {
            basic,
            skills: skill_ids,
            ultimate: SkillId("baby_flame".into()),
            follow_up: None,
        },
    )
}

fn target(
    id: u32,
) -> (
    Unit,
    bevyrogue::combat::team::Team,
    bevyrogue::combat::toughness::Toughness,
) {
    (
        Unit {
            id: UnitId(id),
            name: format!("Target{id}"),
            hp_max: 100,
            hp_current: 100,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        bevyrogue::combat::team::Team::Enemy,
        bevyrogue::combat::toughness::Toughness {
            max: 40,
            current: 40,
            weaknesses: vec![DamageTag::Fire],
            broken: false,
            category: Default::default(),
        },
    )
}

fn message_cursor<T: Message>(app: &mut App) -> MessageCursor<T> {
    app.world_mut().resource_mut::<Messages<T>>().get_cursor()
}

fn drain_messages<T: Message + Clone>(cursor: &mut MessageCursor<T>, app: &App) -> Vec<T> {
    cursor
        .read(app.world().resource::<Messages<T>>())
        .cloned()
        .collect()
}

fn setup_app_with_canonical_book() -> App {
    let book = load_skill_book();
    build_app(book)
}

fn setup_app_with_inline_book(book: SkillBook) -> App {
    build_app(book)
}

fn log_entries(app: &App) -> Vec<LogEntry> {
    app.world()
        .resource::<ActionLog>()
        .events
        .iter()
        .cloned()
        .collect()
}

#[test]
fn row_skill_is_rejected_before_mutation_or_lifecycle() {
    let mut app = setup_app_with_canonical_book();
    app.world_mut()
        .spawn(actor(1, vec![SkillId("heat_viper".into())]));
    app.world_mut().spawn(target(2));

    let mut cursor = message_cursor::<bevyrogue::combat::events::CombatEvent>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("heat_viper".into()),
        target: UnitId(2),
    });
    app.update();

    let events = drain_messages(&mut cursor, &app);
    let log = log_entries(&app);
    let target_entity = app
        .world_mut()
        .query::<(Entity, &Unit)>()
        .iter(app.world())
        .find(|(_, unit)| unit.id == UnitId(2))
        .map(|(entity, _)| entity)
        .expect("missing target entity");
    let target_unit = app
        .world()
        .get::<Unit>(target_entity)
        .expect("missing target unit");
    let target_toughness = app
        .world()
        .get::<bevyrogue::combat::toughness::Toughness>(target_entity)
        .expect("missing target toughness");

    assert!(events.iter().any(|event| matches!(&event.kind, CombatEventKind::OnActionFailed { reason } if reason.contains("UnimplementedTargetShape"))), "expected row failure reason");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionDeclared { .. })),
        "row failure must happen before declaration"
    );
    assert!(
        !events.iter().any(|event| matches!(
            event.kind,
            CombatEventKind::OnActionPreApp
                | CombatEventKind::OnActionApplied
                | CombatEventKind::OnActionResolved
        )),
        "row failure must not emit lifecycle events"
    );
    assert_eq!(
        target_unit.hp_current, 100,
        "row rejection must not mutate HP"
    );
    assert_eq!(
        target_toughness.current, 40,
        "row rejection must not consume toughness"
    );
    assert_eq!(
        app.world().resource::<SpPool>().current,
        99,
        "row rejection must not spend SP"
    );
    assert!(log.iter().any(|entry| matches!(entry, LogEntry::ActionFailed { reason } if reason.contains("UnimplementedTargetShape"))), "row failure must be logged");
}

#[test]
fn all_enemies_skill_is_rejected_before_mutation_or_lifecycle() {
    let mut app = setup_app_with_inline_book(SkillBook(vec![SkillDef {
        id: SkillId("wide_blast".into()),
        name: "Wide Blast".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::AllEnemies,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape,
        },
        effects: vec![
            Effect::Damage {
                amount: 20,
                target: TargetShape::AllEnemies,
            per_hop: Default::default(),
            },
            Effect::ToughnessHit(10),
        ],
        ..Default::default()
    }]));
    app.world_mut()
        .spawn(actor(1, vec![SkillId("wide_blast".into())]));
    app.world_mut().spawn(target(2));

    let mut cursor = message_cursor::<bevyrogue::combat::events::CombatEvent>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("wide_blast".into()),
        target: UnitId(2),
    });
    app.update();

    let events = drain_messages(&mut cursor, &app);
    let log = log_entries(&app);
    let target_entity = app
        .world_mut()
        .query::<(Entity, &Unit)>()
        .iter(app.world())
        .find(|(_, unit)| unit.id == UnitId(2))
        .map(|(entity, _)| entity)
        .expect("missing target entity");
    let target_unit = app
        .world()
        .get::<Unit>(target_entity)
        .expect("missing target unit");
    let target_toughness = app
        .world()
        .get::<bevyrogue::combat::toughness::Toughness>(target_entity)
        .expect("missing target toughness");

    assert!(events.iter().any(|event| matches!(&event.kind, CombatEventKind::OnActionFailed { reason } if reason.contains("UnimplementedTargetShape"))), "expected AllEnemies failure reason");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionDeclared { .. })),
        "AllEnemies failure must happen before declaration"
    );
    assert!(
        !events.iter().any(|event| matches!(
            event.kind,
            CombatEventKind::OnActionPreApp
                | CombatEventKind::OnActionApplied
                | CombatEventKind::OnActionResolved
        )),
        "AllEnemies failure must not emit lifecycle events"
    );
    assert_eq!(
        target_unit.hp_current, 100,
        "AllEnemies rejection must not mutate HP"
    );
    assert_eq!(
        target_toughness.current, 40,
        "AllEnemies rejection must not consume toughness"
    );
    assert_eq!(
        app.world().resource::<SpPool>().current,
        99,
        "AllEnemies rejection must not spend SP"
    );
    assert!(log.iter().any(|entry| matches!(entry, LogEntry::ActionFailed { reason } if reason.contains("UnimplementedTargetShape"))), "AllEnemies failure must be logged");
}

#[test]
fn single_target_skill_still_executes_normally() {
    let mut app = setup_app_with_inline_book(SkillBook(vec![SkillDef {
        id: SkillId("single_strike".into()),
        name: "Single Strike".into(),
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
        effects: vec![
            Effect::Damage {
                amount: 20,
                target: TargetShape::Single,
            per_hop: Default::default(),
            },
            Effect::ToughnessHit(10),
        ],
        ..Default::default()
    }]));
    app.world_mut()
        .spawn(actor(1, vec![SkillId("single_strike".into())]));
    app.world_mut().spawn(target(2));

    let mut cursor = message_cursor::<bevyrogue::combat::events::CombatEvent>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("single_strike".into()),
        target: UnitId(2),
    });
    app.update();

    let events = drain_messages(&mut cursor, &app);
    let log = log_entries(&app);
    let target_entity = app
        .world_mut()
        .query::<(Entity, &Unit)>()
        .iter(app.world())
        .find(|(_, unit)| unit.id == UnitId(2))
        .map(|(entity, _)| entity)
        .expect("missing target entity");
    let target_unit = app
        .world()
        .get::<Unit>(target_entity)
        .expect("missing target unit");
    let target_toughness = app
        .world()
        .get::<bevyrogue::combat::toughness::Toughness>(target_entity)
        .expect("missing target toughness");

    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionDeclared { .. })),
        "single-target skill should declare normally"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionPreApp)),
        "single-target skill should reach pre-app"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionApplied)),
        "single-target skill should apply"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionResolved)),
        "single-target skill should resolve"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnDamageDealt { .. })),
        "single-target skill should deal damage"
    );
    assert!(
        log.iter()
            .any(|entry| matches!(entry, LogEntry::BasicHit { .. })),
        "single-target skill should write a damage log"
    );
    assert!(
        target_unit.hp_current < 100,
        "single-target skill should mutate HP"
    );
    assert!(
        target_toughness.current < 40,
        "single-target skill should mutate toughness"
    );
}
