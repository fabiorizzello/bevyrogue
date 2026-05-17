use bevy::prelude::*;
use bevyrogue::combat::{
    api::{Intent, IntentQueue, applier::intent_applier, intent::CastId},
    battery_loop::BatteryLoopState,
    blueprints::dorumon::PredatorLoopState,
    events::{CombatEvent, CombatEventKind},
    kernel::register_combat_kernel_runtime,
    modifiers::DamageModifierLedger,
    rng::CombatRng,
    team::Team,
    types::{Attribute, DamageTag, EvoStage, UnitId},
    unit::Unit,
};

fn setup_app(seed: u64) -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>()
        .init_resource::<IntentQueue>()
        .init_resource::<DamageModifierLedger>()
        .insert_resource(CombatRng::from_seed(seed))
        .add_systems(Update, intent_applier);
    register_combat_kernel_runtime(&mut app);
    app
}

fn spawn_unit(app: &mut App, id: UnitId, name: &str, team: Team, hp: i32) {
    app.world_mut().spawn((
        Unit {
            id,
            name: name.to_string(),
            hp_max: hp,
            hp_current: hp,
            attribute: Attribute::Data,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        team,
    ));
}

fn read_events(app: &mut App) -> Vec<CombatEvent> {
    use bevy::ecs::message::MessageCursor;
    let mut cursor: MessageCursor<CombatEvent> = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn block_hit_seed() -> u64 {
    for seed in 0u64..10_000 {
        let (baseline_hp, _) = run_tentomon_case(seed, false);
        let (armed_hp, _) = run_tentomon_case(seed, true);
        if armed_hp > baseline_hp {
            return seed;
        }
    }
    panic!("expected at least one seed to produce a block reaction");
}

fn miss_seed(threshold: i32) -> u64 {
    (0u64..10_000)
        .find(|&seed| !CombatRng::from_seed(seed).roll_pct(threshold))
        .expect("expected a seed that misses")
}

#[test]
fn dorumon_enemy_kill_listener_reuses_predator_loop_state() {
    let mut app = setup_app(42);
    let dorumon = UnitId(5);
    let target = UnitId(8);

    spawn_unit(&mut app, dorumon, "Dorumon", Team::Ally, 90);
    spawn_unit(&mut app, target, "Dummy", Team::Enemy, 40);

    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::OnEnemyKill,
        source: dorumon,
        target,
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });

    app.update();

    let state = app.world().resource::<PredatorLoopState>();
    let tracked = state
        .targets
        .get(&target)
        .expect("target should be tracked");
    assert!(tracked.exploit_stacks >= 1);
    assert!(tracked.prey_lock.is_some());
    assert_eq!(state.last_blocked_reason, None);
}

fn run_tentomon_case(seed: u64, armed: bool) -> (i32, Vec<CombatEvent>) {
    let mut app = setup_app(seed);
    let tentomon = UnitId(4);
    let attacker = UnitId(9);

    spawn_unit(&mut app, attacker, "Attacker", Team::Enemy, 120);
    spawn_unit(&mut app, tentomon, "Tentomon", Team::Ally, 120);

    if armed {
        app.world_mut()
            .resource_mut::<BatteryLoopState>()
            .block_reaction_armed = true;
    }

    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .push_back(Intent::DealDamage {
            source: attacker,
            target: tentomon,
            amount: 100,
            tag: DamageTag::Physical,
            cast_id: CastId::ROOT,
        });

    app.update();

    let hp_after = {
        let mut q = app.world_mut().query::<&Unit>();
        q.iter(app.world())
            .find(|unit| unit.id == tentomon)
            .map(|unit| unit.hp_current)
            .expect("Tentomon should exist")
    };

    (hp_after, read_events(&mut app))
}

#[test]
fn tentomon_block_reaction_is_deterministic_when_armed() {
    let seed = block_hit_seed();

    let (baseline_hp, baseline_events) = run_tentomon_case(seed, false);
    assert!(
        baseline_events
            .iter()
            .any(|ev| matches!(ev.kind, CombatEventKind::IncomingDamage { .. }))
    );
    assert!(
        baseline_events
            .iter()
            .all(|ev| !matches!(ev.kind, CombatEventKind::BlockReactionTriggered { .. }))
    );

    let (armed_hp, armed_events) = run_tentomon_case(seed, true);
    let (armed_hp_replay, armed_events_replay) = run_tentomon_case(seed, true);

    assert_eq!(armed_hp, armed_hp_replay);
    assert_eq!(armed_events, armed_events_replay);
    assert!(
        armed_hp > baseline_hp,
        "armed Tentomon should take less damage"
    );
    assert!(
        armed_events
            .iter()
            .any(|ev| matches!(ev.kind, CombatEventKind::BlockReactionTriggered { .. }))
    );
}

#[test]
fn tentomon_block_reaction_stays_off_when_guard_conditions_fail() {
    let seed = miss_seed(30);

    let (baseline_hp, baseline_events) = run_tentomon_case(seed, false);
    let (miss_hp, miss_events) = run_tentomon_case(seed, true);

    assert_eq!(
        baseline_hp, miss_hp,
        "missed block should match baseline damage"
    );
    assert_eq!(
        baseline_events
            .iter()
            .filter(|ev| matches!(ev.kind, CombatEventKind::BlockReactionTriggered { .. }))
            .count(),
        0
    );
    assert_eq!(
        miss_events
            .iter()
            .filter(|ev| matches!(ev.kind, CombatEventKind::BlockReactionTriggered { .. }))
            .count(),
        0
    );
}
