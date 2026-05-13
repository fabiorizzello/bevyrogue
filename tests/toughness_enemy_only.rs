use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    log::{ActionLog, LogEntry},
    sp::SpPool,
    state::CombatState,
    stun::Stunned,
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
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

fn build_app() -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let book = SkillBook(vec![SkillDef {
        id: SkillId("slam".into()),
        name: "Slam".into(),
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
            },
            Effect::ToughnessHit(10),
        ],
        ..Default::default()
    }]);
    let handle = assets.add(book);

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool {
            current: 99,
            max: 99,
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);
    app
}

fn attacker(
    id: u32,
    team: Team,
) -> (
    Unit,
    Team,
    UltimateCharge,
    bevyrogue::combat::kit::UnitSkills,
) {
    let basic = SkillId("slam".into());
    (
        Unit {
            id: UnitId(id),
            name: format!("Attacker{id}"),
            hp_max: 100,
            hp_current: 100,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        team,
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        bevyrogue::combat::kit::UnitSkills {
            basic: basic.clone(),
            skills: vec![basic.clone()],
            ultimate: basic,
            follow_up: None,
        },
    )
}

fn defender(
    id: u32,
    team: Team,
    hp: i32,
    toughness_max: i32,
    toughness_current: i32,
) -> (Unit, Team, Toughness) {
    (
        Unit {
            id: UnitId(id),
            name: format!("Defender{id}"),
            hp_max: hp,
            hp_current: hp,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        team,
        Toughness {
            max: toughness_max,
            current: toughness_current,
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

fn entity_for_unit(world: &mut World, id: UnitId) -> Entity {
    let mut query = world.query::<(Entity, &Unit)>();
    query
        .iter(world)
        .find(|(_, unit)| unit.id == id)
        .map(|(entity, _)| entity)
        .unwrap_or_else(|| panic!("missing entity for {:?}", id))
}

#[test]
fn enemy_attack_damages_ally_without_breaking_or_stunning() {
    let mut app = build_app();

    app.world_mut().spawn(attacker(1, Team::Enemy));
    app.world_mut().spawn(defender(2, Team::Ally, 100, 10, 10));

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("slam".into()),
        target: UnitId(2),
    });
    app.update();

    let events = drain_messages(&mut event_cursor, &app);
    let log = app.world().resource::<ActionLog>().events.clone();
    let ally_entity = entity_for_unit(app.world_mut(), UnitId(2));
    let ally_unit = app.world().get::<Unit>(ally_entity).unwrap();
    let ally_toughness = app.world().get::<Toughness>(ally_entity).unwrap();

    assert!(
        ally_unit.hp_current < 100,
        "ally HP should still change on hit"
    );
    assert_eq!(
        ally_toughness.current, 10,
        "ally toughness must not be consumed"
    );
    assert!(
        !ally_toughness.broken,
        "ally toughness must not flip broken"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnDamageDealt { .. })),
        "expected a damage event"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnBreak { .. })),
        "ally hit must not emit OnBreak"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionFailed { .. })),
        "ally hit must not fail"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnKO)),
        "ally should survive the hit"
    );
    assert!(
        log.iter().any(
            |entry| matches!(entry, LogEntry::BasicHit { target, .. } if *target == UnitId(2))
        ),
        "damage log should still be written"
    );
    assert!(
        app.world().get::<Stunned>(ally_entity).is_none(),
        "ally should not be stunned by hidden toughness"
    );
}

#[test]
fn ally_attack_still_breaks_enemy_toughness_when_weak() {
    let mut app = build_app();

    app.world_mut().spawn(attacker(1, Team::Ally));
    app.world_mut().spawn(defender(2, Team::Enemy, 100, 10, 10));

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("slam".into()),
        target: UnitId(2),
    });
    app.update();

    let events = drain_messages(&mut event_cursor, &app);
    let enemy_entity = entity_for_unit(app.world_mut(), UnitId(2));
    let enemy_toughness = app.world().get::<Toughness>(enemy_entity).unwrap();
    let enemy_stunned = app.world().get::<Stunned>(enemy_entity).is_some();

    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnBreak { .. })),
        "enemy hit should emit OnBreak"
    );
    assert!(enemy_toughness.broken, "enemy toughness should flip broken");
    assert!(enemy_stunned, "enemy break should apply stun");
}
