
use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::blueprints::agumon::baby_burner;
use bevyrogue::combat::runtime::{
    PostActionContext, PostActionQueue, PostActionUnitDied, PostActionUnitSnapshot, SignalPayload,
};
use bevyrogue::combat::{
    CombatPlugin, StatusBag, StatusEffectKind,
    events::{CombatEvent, CombatEventKind, CombatKernelTransition},
    kit::UnitSkills,
    log::ActionLog,
    sp::SpPool,
    state::CombatState,
    team::Team,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{SlotIndex, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
};

const AGUMON_ID: UnitId = UnitId(1);
const PRIMARY_ID: UnitId = UnitId(11);
const LEFT_ID: UnitId = UnitId(10);
const RIGHT_ID: UnitId = UnitId(12);
const FAR_ID: UnitId = UnitId(13);
const TEST_NON_BABY_BURNER_ULT: &str = "test_non_baby_burner_ult";

fn canonical_skill_book() -> SkillBook {
    let mut book = bevyrogue::data::aggregate_skill_book();
    book.0.push(SkillDef {
        id: SkillId(TEST_NON_BABY_BURNER_ULT.into()),
        name: "Test Non Baby Burner Ult".into(),
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
                amount: 50,
                target: TargetShape::Single,
                per_hop: Default::default(),
            },
            Effect::ToughnessHit(30),
        ],
        ..Default::default()
    });
    book
}

fn build_app() -> App {
    let book = canonical_skill_book();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book);

    let mut app = App::new();
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
        .add_plugins(CombatPlugin)
        .add_systems(Update, resolve_action_system);
    app
}

fn message_cursor<T: Message>(app: &mut App) -> MessageCursor<T> {
    app.world_mut().resource_mut::<Messages<T>>().get_cursor()
}

fn drain_events(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn spawn_agumon(app: &mut App, ultimate_skill: &str) {
    app.world_mut().spawn((
        Unit {
            id: AGUMON_ID,
            name: "Agumon".into(),
            hp_max: 120,
            hp_current: 120,
            attribute: Attribute::Free,
            resists: vec![],
            evo_stage: EvoStage::Child,
        },
        Team::Ally,
        SlotIndex(0),
        UltimateCharge {
            current: 100,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("baby_flame".into()),
            skills: vec![SkillId("baby_flame".into())],
            ultimate: SkillId(ultimate_skill.into()),
            follow_up: None,
        },
    ));
}

fn spawn_enemy(app: &mut App, id: UnitId, slot: u8, hp: i32, statuses: &[StatusEffectKind]) {
    let mut bag = StatusBag::default();
    for status in statuses {
        match status {
            StatusEffectKind::Heated => bag.apply(StatusEffectKind::Heated, 2),
            StatusEffectKind::Slowed => bag.apply(StatusEffectKind::Slowed, 1),
            other => panic!("unexpected fixture status: {other:?}"),
        }
    }

    app.world_mut().spawn((
        Unit {
            id,
            name: format!("Enemy{}", id.0),
            hp_max: hp,
            hp_current: hp,
            attribute: Attribute::Free,
            resists: vec![],
            evo_stage: EvoStage::Child,
        },
        Team::Enemy,
        SlotIndex(slot),
        bag,
    ));
}

fn fire_ultimate(app: &mut App, target: UnitId) {
    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: AGUMON_ID,
        target,
    });
}

fn unit_hp(app: &mut App, id: UnitId) -> i32 {
    let mut query = app.world_mut().query::<&Unit>();
    query
        .iter(app.world())
        .find(|unit| unit.id == id)
        .map(|unit| unit.hp_current)
        .unwrap_or_else(|| panic!("unit {id:?} missing"))
}

fn damage_targets(events: &[CombatEvent]) -> Vec<UnitId> {
    events
        .iter()
        .filter_map(|event| match event.kind {
            CombatEventKind::OnDamageDealt { .. } => Some(event.target),
            _ => None,
        })
        .collect()
}

fn detonate_transition_targets(events: &[CombatEvent]) -> Vec<UnitId> {
    events
        .iter()
        .filter_map(|event| match &event.kind {
            CombatEventKind::OnKernelTransition {
                transition:
                    CombatKernelTransition::Blueprint {
                        owner,
                        name,
                        payload: SignalPayload::UnitTarget(target),
                    },
            } if owner == "agumon" && name == "baby_burner_detonate" => Some(*target),
            _ => None,
        })
        .collect()
}

#[test]
fn baby_burner_reaction_maps_ko_context_to_adjacent_targets() {
    let ctx = PostActionContext::new(
        SkillId("agumon_ult".into()),
        AGUMON_ID,
        PRIMARY_ID,
        bevyrogue::combat::runtime::CastId::ROOT,
        0,
        Some(PostActionUnitDied::new(
            vec![StatusEffectKind::Heated, StatusEffectKind::Slowed],
            2,
        )),
        vec![
            PostActionUnitSnapshot::new(AGUMON_ID, Team::Ally, Some(0), 120, 120, true),
            PostActionUnitSnapshot::new(LEFT_ID, Team::Enemy, Some(0), 100, 100, true),
            PostActionUnitSnapshot::new(PRIMARY_ID, Team::Enemy, Some(1), 0, 40, false),
            PostActionUnitSnapshot::new(RIGHT_ID, Team::Enemy, Some(2), 100, 100, true),
            PostActionUnitSnapshot::new(FAR_ID, Team::Enemy, Some(3), 100, 100, true),
        ],
    );
    let mut out = PostActionQueue::default();

    baby_burner::enqueue_reactive_detonate(&ctx, &mut out);

    assert_eq!(
        out.intents.len(),
        4,
        "2 damage + 2 blueprint-signal intents"
    );
}

#[test]
fn lethal_heated_baby_burner_detonates_adjacent_alive_enemies_once() {
    let mut app = build_app();
    spawn_agumon(&mut app, "agumon_ult");
    spawn_enemy(&mut app, LEFT_ID, 0, 100, &[]);
    spawn_enemy(
        &mut app,
        PRIMARY_ID,
        1,
        40,
        &[StatusEffectKind::Heated, StatusEffectKind::Slowed],
    );
    spawn_enemy(&mut app, RIGHT_ID, 2, 100, &[]);
    spawn_enemy(&mut app, FAR_ID, 3, 100, &[]);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    fire_ultimate(&mut app, PRIMARY_ID);

    app.update();
    let first_tick = drain_events(&mut cursor, &app);
    assert!(
        first_tick.iter().any(|event| {
            matches!(
                &event.kind,
                CombatEventKind::UnitDied {
                    heated_remaining: 2,
                    ..
                }
            ) && event.target == PRIMARY_ID
        }),
        "primary KO must preserve Heated(2) in UnitDied payload: {first_tick:?}"
    );

    app.update();
    let second_tick = drain_events(&mut cursor, &app);
    let combined: Vec<CombatEvent> = first_tick
        .iter()
        .chain(second_tick.iter())
        .cloned()
        .collect();
    assert_eq!(
        damage_targets(&combined)
            .into_iter()
            .filter(|target| *target != PRIMARY_ID)
            .collect::<Vec<_>>(),
        vec![LEFT_ID, RIGHT_ID],
        "detonate damage must hit each adjacent alive enemy exactly once"
    );
    assert_eq!(
        detonate_transition_targets(&combined),
        vec![LEFT_ID, RIGHT_ID],
        "detonate blueprint transitions must exist exactly for real detonate targets"
    );
    assert_eq!(unit_hp(&mut app, LEFT_ID), 84);
    assert_eq!(unit_hp(&mut app, RIGHT_ID), 84);
    assert_eq!(
        unit_hp(&mut app, FAR_ID),
        100,
        "non-adjacent enemy untouched"
    );
    assert!(unit_hp(&mut app, PRIMARY_ID) <= 0, "primary remains KO'd");

    app.update();
    let third_tick = drain_events(&mut cursor, &app);
    assert!(
        damage_targets(&third_tick).is_empty(),
        "extra updates must not duplicate detonate damage: {third_tick:?}"
    );
    assert!(
        detonate_transition_targets(&third_tick).is_empty(),
        "extra updates must not duplicate detonate transitions: {third_tick:?}"
    );
}

#[test]
fn nonlethal_baby_burner_does_not_detonate() {
    let mut app = build_app();
    spawn_agumon(&mut app, "agumon_ult");
    spawn_enemy(&mut app, LEFT_ID, 0, 100, &[]);
    spawn_enemy(&mut app, PRIMARY_ID, 1, 70, &[StatusEffectKind::Heated]);
    spawn_enemy(&mut app, RIGHT_ID, 2, 100, &[]);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    fire_ultimate(&mut app, PRIMARY_ID);

    app.update();
    let first_tick = drain_events(&mut cursor, &app);
    app.update();
    let second_tick = drain_events(&mut cursor, &app);
    let combined: Vec<CombatEvent> = first_tick
        .iter()
        .chain(second_tick.iter())
        .cloned()
        .collect();

    assert!(
        damage_targets(&combined)
            .into_iter()
            .filter(|target| *target != PRIMARY_ID)
            .collect::<Vec<_>>()
            .is_empty(),
        "nonlethal Baby Burner must not queue detonate damage: {combined:?}"
    );
    assert!(
        detonate_transition_targets(&combined).is_empty(),
        "nonlethal Baby Burner must not emit detonate transitions: {combined:?}"
    );
    assert_eq!(unit_hp(&mut app, LEFT_ID), 100);
    assert_eq!(unit_hp(&mut app, RIGHT_ID), 100);
}

#[test]
fn lethal_non_baby_burner_does_not_detonate() {
    let mut app = build_app();
    spawn_agumon(&mut app, TEST_NON_BABY_BURNER_ULT);
    spawn_enemy(&mut app, LEFT_ID, 0, 100, &[]);
    spawn_enemy(&mut app, PRIMARY_ID, 1, 40, &[StatusEffectKind::Heated]);
    spawn_enemy(&mut app, RIGHT_ID, 2, 100, &[]);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    fire_ultimate(&mut app, PRIMARY_ID);

    app.update();
    let first_tick = drain_events(&mut cursor, &app);
    app.update();
    let second_tick = drain_events(&mut cursor, &app);
    let combined: Vec<CombatEvent> = first_tick
        .iter()
        .chain(second_tick.iter())
        .cloned()
        .collect();

    assert!(
        damage_targets(&combined)
            .into_iter()
            .filter(|target| *target != PRIMARY_ID)
            .collect::<Vec<_>>()
            .is_empty(),
        "non-Baby-Burner lethal cast must not queue detonate damage: {combined:?}"
    );
    assert!(
        detonate_transition_targets(&combined).is_empty(),
        "non-Baby-Burner lethal cast must not emit detonate transitions: {combined:?}"
    );
    assert_eq!(unit_hp(&mut app, LEFT_ID), 100);
    assert_eq!(unit_hp(&mut app, RIGHT_ID), 100);
}

#[test]
fn lethal_baby_burner_without_heated_payload_does_not_detonate() {
    let mut app = build_app();
    spawn_agumon(&mut app, "agumon_ult");
    spawn_enemy(&mut app, LEFT_ID, 0, 100, &[]);
    spawn_enemy(&mut app, PRIMARY_ID, 1, 40, &[StatusEffectKind::Slowed]);
    spawn_enemy(&mut app, RIGHT_ID, 2, 100, &[]);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    fire_ultimate(&mut app, PRIMARY_ID);

    app.update();
    let first_tick = drain_events(&mut cursor, &app);
    assert!(
        first_tick.iter().any(|event| {
            matches!(
                &event.kind,
                CombatEventKind::UnitDied {
                    heated_remaining: 0,
                    ..
                }
            ) && event.target == PRIMARY_ID
        }),
        "fixture must prove the lethal cast carried zero Heated payload: {first_tick:?}"
    );

    app.update();
    let second_tick = drain_events(&mut cursor, &app);
    let combined: Vec<CombatEvent> = first_tick
        .iter()
        .chain(second_tick.iter())
        .cloned()
        .collect();
    assert!(
        damage_targets(&combined)
            .into_iter()
            .filter(|target| *target != PRIMARY_ID)
            .collect::<Vec<_>>()
            .is_empty(),
        "zero-Heated payload must not queue detonate damage: {combined:?}"
    );
    assert!(
        detonate_transition_targets(&combined).is_empty(),
        "zero-Heated payload must not emit detonate transitions: {combined:?}"
    );
    assert_eq!(unit_hp(&mut app, LEFT_ID), 100);
    assert_eq!(unit_hp(&mut app, RIGHT_ID), 100);
}
