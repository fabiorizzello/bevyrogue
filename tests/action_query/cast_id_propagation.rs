//! Tests that `CastId` is correctly assigned to `CombatEvent`s:
//! - Lifecycle events outside a cast (Declared, PreApp, Applied, Resolved) use `CastId::ROOT`.
//! - All events emitted within a single `step_app` invocation share one non-ROOT cast_id.
//! - Events from different casts receive distinct non-ROOT cast_ids.

use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::blueprints::register_all_blueprint_exts;
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    log::ActionLog,
    rng::CombatRng,
    runtime::{
        ExtRegistries, SignalBus, SignalTaxonomy,
        intent::{CastId, CastIdGen},
        register_kernel_builtins,
        timeline::TimelineLibrary,
    },
    sp::SpPool,
    state::CombatState,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle, skill_timeline::compile_skill_book_timelines, skills_ron::SkillBook,
};

fn load_skill_book() -> SkillBook {
    bevyrogue::data::aggregate_skill_book()
}

fn build_app() -> App {
    let book = load_skill_book();
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());

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
        .init_resource::<CastIdGen>()
        .insert_resource(CombatRng::from_seed(42))
        .insert_resource(TimelineLibrary::<String>::default())
        .init_resource::<SignalBus>()
        .init_resource::<ExtRegistries>()
        .init_resource::<SignalTaxonomy>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        register_kernel_builtins(&mut regs);
        register_all_blueprint_exts(&mut regs);
        let compiled = compile_skill_book_timelines(&book, &regs)
            .expect("canonical timeline book must compile");
        app.world_mut()
            .resource_mut::<TimelineLibrary<String>>()
            .timelines = compiled;
        app.world_mut()
            .resource_mut::<SignalTaxonomy>()
            .register("agumon", "apply_heated");
    }

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

fn spawn_attacker(app: &mut App, id: u32) {
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Attacker{id}"),
            hp_max: 200,
            hp_current: 200,
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
            basic: SkillId("baby_flame".into()),
            skills: vec![SkillId("baby_flame".into())],
            ultimate: SkillId("baby_flame".into()),
            follow_up: None,
        },
    ));
}

fn spawn_target(app: &mut App, id: u32) {
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Target{id}"),
            hp_max: 200,
            hp_current: 200,
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
    ));
}

/// (a) All events emitted during a cast share the same cast_id.
/// (b) That cast_id is not ROOT.
#[test]
fn cast_events_share_nonroot_cast_id() {
    let mut app = build_app();
    spawn_attacker(&mut app, 1);
    spawn_target(&mut app, 2);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);

    app.world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .write(ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(2),
        });
    app.update();

    let events = drain_events(&mut cursor, &app);
    assert!(!events.is_empty(), "expected combat events");

    // Collect cast_ids from events that are inside the cast (not lifecycle events)
    let cast_events: Vec<&CombatEvent> = events
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                CombatEventKind::OnDamageDealt { .. }
                    | CombatEventKind::OnHitTaken { .. }
                    | CombatEventKind::OnSkillCast { .. }
                    | CombatEventKind::UltGain { .. }
            )
        })
        .collect();

    assert!(!cast_events.is_empty(), "expected in-cast events");

    // (b) All in-cast events have a non-ROOT cast_id
    for ev in &cast_events {
        assert_ne!(
            ev.cast_id,
            CastId::ROOT,
            "in-cast event {:?} should have non-ROOT cast_id",
            ev.kind
        );
    }

    // (a) All in-cast events share the same cast_id
    let first_cast_id = cast_events[0].cast_id;
    for ev in &cast_events {
        assert_eq!(
            ev.cast_id, first_cast_id,
            "in-cast event {:?} has cast_id {:?}, expected {:?}",
            ev.kind, ev.cast_id, first_cast_id
        );
    }
}

/// (c) Pre-cast lifecycle events use CastId::ROOT; post-cast events carry the cast_id.
#[test]
fn lifecycle_events_pre_cast_use_root_cast_id() {
    let mut app = build_app();
    spawn_attacker(&mut app, 1);
    spawn_target(&mut app, 2);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);

    app.world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .write(ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(2),
        });
    app.update();

    let events = drain_events(&mut cursor, &app);

    let pre_cast: Vec<&CombatEvent> = events
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                CombatEventKind::OnActionDeclared { .. } | CombatEventKind::OnActionPreApp
            )
        })
        .collect();

    assert!(
        !pre_cast.is_empty(),
        "expected pre-cast lifecycle events (Declared, PreApp)"
    );

    for ev in pre_cast {
        assert_eq!(
            ev.cast_id,
            CastId::ROOT,
            "pre-cast event {:?} should have ROOT cast_id, got {:?}",
            ev.kind,
            ev.cast_id
        );
    }

    let post_cast: Vec<&CombatEvent> = events
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                CombatEventKind::OnActionApplied | CombatEventKind::OnActionResolved
            )
        })
        .collect();
    assert!(
        !post_cast.is_empty(),
        "expected post-cast lifecycle events (Applied, Resolved)"
    );
}

/// Two sequential casts receive distinct non-ROOT cast_ids.
#[test]
fn sequential_casts_have_distinct_cast_ids() {
    let mut app = build_app();
    spawn_attacker(&mut app, 1);
    spawn_target(&mut app, 2);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);

    // First cast
    app.world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .write(ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(2),
        });
    app.update();
    let events1 = drain_events(&mut cursor, &app);

    // Second cast
    app.world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .write(ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(2),
        });
    app.update();
    let events2 = drain_events(&mut cursor, &app);

    let cast_id_1 = events1
        .iter()
        .find(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .map(|e| e.cast_id)
        .expect("first cast should emit OnDamageDealt");

    let cast_id_2 = events2
        .iter()
        .find(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .map(|e| e.cast_id)
        .expect("second cast should emit OnDamageDealt");

    assert_ne!(cast_id_1, CastId::ROOT, "first cast id should be non-ROOT");
    assert_ne!(cast_id_2, CastId::ROOT, "second cast id should be non-ROOT");
    assert_ne!(
        cast_id_1, cast_id_2,
        "sequential casts should have distinct cast_ids"
    );
}
