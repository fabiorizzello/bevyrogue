use bevy::{ecs::message::MessageCursor, ecs::system::RunSystemOnce, prelude::*};
use bevyrogue::combat::{
    runtime::{
        BlueprintState, CastId, CastIdGen, EventFilter, ExtRegistries, Intent, IntentQueue,
        PassiveListeners, PassiveRunner, SignalBus, SignalPayload, SignalTaxonomy, SkillCtx,
        applier::intent_applier,
        combat_event_to_signal_system, passive_dispatch_system,
        timeline::{Beat, BeatEvent, BeatKind, CompiledTimeline},
    },
    events::{CombatEvent, CombatEventKind, CombatKernelTransition},
    team::Team,
    types::{Attribute, EvoStage, UnitId},
    unit::Unit,
};
use std::sync::Arc;

const RENAMON_ID: UnitId = UnitId(10);
const PATAMON_ID: UnitId = UnitId(11);
const ENEMY_ID: UnitId = UnitId(12);
const KITSUNE_TRIGGER_KEY: &str = "kitsune_grace/triggered";

fn setup_app() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>()
        .init_resource::<IntentQueue>()
        .init_resource::<CastIdGen>()
        .init_resource::<SignalBus>()
        .init_resource::<SignalTaxonomy>()
        .init_resource::<BlueprintState>()
        .init_resource::<ExtRegistries>()
        .init_resource::<PassiveListeners>()
        .add_systems(
            Update,
            (
                intent_applier,
                combat_event_to_signal_system.after(intent_applier),
                passive_dispatch_system.after(combat_event_to_signal_system),
            ),
        );
    app
}

fn spawn_unit(app: &mut App, id: UnitId, team: Team, hp_current: i32, hp_max: i32) {
    app.world_mut().spawn((
        Unit {
            id,
            name: format!("unit_{}", id.0),
            hp_max,
            hp_current,
            attribute: Attribute::Data,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        team,
    ));
}

fn kitsune_grace_trigger(evt: &BeatEvent, ctx: &SkillCtx<'_>) -> bool {
    let world = ctx.world;
    let Some(mut units) = world.try_query::<(&Unit, &Team)>() else {
        return false;
    };

    let Some((_, target_team)) = units
        .iter(world)
        .find(|(unit, _)| unit.id == ctx.primary_target)
    else {
        return false;
    };

    let Some((self_unit, self_team)) = units.iter(world).find(|(unit, _)| unit.id == ctx.caster)
    else {
        return false;
    };

    ctx.primary_target != ctx.caster
        && self_unit.hp_current > 0
        && self_team == &Team::Ally
        && target_team == &Team::Ally
        && evt.beat_id == "dormant"
        && ctx
            .world
            .resource::<BlueprintState>()
            .map
            .get(&(ctx.caster, KITSUNE_TRIGGER_KEY.to_string()))
            .copied()
            .unwrap_or_default()
            == 0
}

fn kitsune_grace_proc(evt: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    ctx.enqueue(Intent::SetBlueprintState {
        actor: ctx.caster,
        key: KITSUNE_TRIGGER_KEY.to_string(),
        value: 1,
        cast_id: evt.cast_id,
    });
    ctx.enqueue(Intent::BlueprintSignal {
        source: ctx.caster,
        owner: "renamon",
        name: "kitsune_grace",
        payload: SignalPayload::UnitTarget(ctx.primary_target),
        cast_id: evt.cast_id,
    });
}

fn build_kitsune_grace_timeline() -> Arc<CompiledTimeline> {
    Arc::new(CompiledTimeline {
        id: "kitsune_grace",
        entry: "dormant",
        beats: vec![
            Beat {
                id: "dormant",
                kind: BeatKind::Impact,
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "proc",
                kind: BeatKind::Impact,
                hook: Some("kitsune_grace/proc"),
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "resolve",
                kind: BeatKind::Impact,
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            },
        ],
        edges: vec![
            bevyrogue::combat::runtime::timeline::BeatEdge {
                from: "dormant",
                to: "proc",
                gate: Some("kitsune_grace/trigger"),
            },
            bevyrogue::combat::runtime::timeline::BeatEdge {
                from: "proc",
                to: "resolve",
                gate: None,
            },
            bevyrogue::combat::runtime::timeline::BeatEdge {
                from: "resolve",
                to: "dormant",
                gate: None,
            },
        ],
    })
}

fn register_kitsune_grace(app: &mut App) {
    let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
    regs.predicates
        .register("kitsune_grace/trigger", kitsune_grace_trigger);
    regs.hooks
        .register("kitsune_grace/proc", kitsune_grace_proc);

    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register("renamon", "kitsune_grace");

    app.world_mut()
        .resource_mut::<PassiveListeners>()
        .runners
        .push(PassiveRunner::new(
            build_kitsune_grace_timeline(),
            RENAMON_ID,
            vec![EventFilter::blueprint("kernel", "ult_used")],
        ));
}

fn read_events(app: &mut App) -> Vec<CombatEvent> {
    let mut cursor: MessageCursor<CombatEvent> = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn write_ult_used(app: &mut App, unit_id: UnitId) {
    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::UltimateUsed { unit_id },
        source: unit_id,
        target: unit_id,
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
}

#[test]
fn kitsune_grace_triggers_for_ally_ult_and_writes_blueprint_state() {
    let mut app = setup_app();
    spawn_unit(&mut app, RENAMON_ID, Team::Ally, 500, 500);
    spawn_unit(&mut app, PATAMON_ID, Team::Ally, 500, 500);
    spawn_unit(&mut app, ENEMY_ID, Team::Enemy, 500, 500);
    register_kitsune_grace(&mut app);

    write_ult_used(&mut app, PATAMON_ID);
    app.update();

    let drained: Vec<_> = app
        .world_mut()
        .resource_mut::<SignalBus>()
        .drain()
        .collect();
    assert!(
        drained.is_empty(),
        "SignalBus should be fully drained after update, got: {:?}",
        drained
    );

    let state = app.world().resource::<BlueprintState>();
    assert_eq!(
        state
            .map
            .get(&(RENAMON_ID, KITSUNE_TRIGGER_KEY.to_string())),
        Some(&1),
        "kitsune_grace sentinel should be written for ally ult trigger"
    );

    let events = read_events(&mut app);
    let kitsune_event = events.iter().find(|event| {
        matches!(
            &event.kind,
            CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::Blueprint { owner, name, .. }
            } if owner == "renamon" && name == "kitsune_grace"
        )
    });
    assert!(
        kitsune_event.is_some(),
        "Expected kitsune_grace Blueprint transition, got: {:?}",
        events
    );
}

#[test]
fn kitsune_grace_ignores_self_ult() {
    let mut app = setup_app();
    spawn_unit(&mut app, RENAMON_ID, Team::Ally, 500, 500);
    spawn_unit(&mut app, PATAMON_ID, Team::Ally, 500, 500);
    spawn_unit(&mut app, ENEMY_ID, Team::Enemy, 500, 500);
    register_kitsune_grace(&mut app);

    write_ult_used(&mut app, RENAMON_ID);
    app.update();

    let state = app.world().resource::<BlueprintState>();
    assert!(
        !state
            .map
            .contains_key(&(RENAMON_ID, KITSUNE_TRIGGER_KEY.to_string())),
        "self ult should not trigger kitsune_grace"
    );

    let events = read_events(&mut app);
    let kitsune_event = events.iter().find(|event| {
        matches!(
            &event.kind,
            CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::Blueprint { owner, name, .. }
            } if owner == "renamon" && name == "kitsune_grace"
        )
    });
    assert!(
        kitsune_event.is_none(),
        "self ult should not emit kitsune_grace Blueprint transition, got: {:?}",
        events
    );
}

#[test]
fn kitsune_grace_ignores_enemy_ult() {
    let mut app = setup_app();
    spawn_unit(&mut app, RENAMON_ID, Team::Ally, 500, 500);
    spawn_unit(&mut app, PATAMON_ID, Team::Ally, 500, 500);
    spawn_unit(&mut app, ENEMY_ID, Team::Enemy, 500, 500);
    register_kitsune_grace(&mut app);

    write_ult_used(&mut app, ENEMY_ID);
    app.update();

    let state = app.world().resource::<BlueprintState>();
    assert!(
        !state
            .map
            .contains_key(&(RENAMON_ID, KITSUNE_TRIGGER_KEY.to_string())),
        "enemy ult should not trigger kitsune_grace"
    );

    let events = read_events(&mut app);
    let kitsune_event = events.iter().find(|event| {
        matches!(
            &event.kind,
            CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::Blueprint { owner, name, .. }
            } if owner == "renamon" && name == "kitsune_grace"
        )
    });
    assert!(
        kitsune_event.is_none(),
        "enemy ult should not emit kitsune_grace Blueprint transition, got: {:?}",
        events
    );
}

#[test]
fn kitsune_grace_blueprint_event_round_trips_jsonl() {
    let mut app = setup_app();
    spawn_unit(&mut app, RENAMON_ID, Team::Ally, 500, 500);
    spawn_unit(&mut app, PATAMON_ID, Team::Ally, 500, 500);
    spawn_unit(&mut app, ENEMY_ID, Team::Enemy, 500, 500);
    register_kitsune_grace(&mut app);

    write_ult_used(&mut app, PATAMON_ID);
    app.update();

    let events = read_events(&mut app);
    let ev = events
        .into_iter()
        .find(|event| {
            matches!(
                &event.kind,
                CombatEventKind::OnKernelTransition {
                    transition: CombatKernelTransition::Blueprint { owner, name, .. }
                } if owner == "renamon" && name == "kitsune_grace"
            )
        })
        .expect("expected kitsune_grace blueprint event in positive case");

    let json = serde_json::to_string(&ev).expect("combat event should serialize to JSONL");
    let round_tripped: CombatEvent =
        serde_json::from_str(&json).expect("combat event should deserialize from JSONL");
    assert_eq!(
        ev, round_tripped,
        "CombatEvent::OnKernelTransition::Blueprint must round-trip through serde_json"
    );
}

#[test]
#[should_panic(expected = "unregistered signal")]
fn unregistered_blueprint_signal_panics_in_debug() {
    let mut app = setup_app();
    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .push_back(Intent::BlueprintSignal {
            source: RENAMON_ID,
            owner: "renamon",
            name: "missing_signal",
            payload: SignalPayload::Empty,
            cast_id: CastId::ROOT,
        });

    app.world_mut().run_system_once(intent_applier);
}
