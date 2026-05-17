use bevy::prelude::*;
use bevyrogue::combat::api::intent::CastId;
use bevyrogue::combat::{
    api::SignalPayload,
    events::{CombatEvent, CombatEventKind},
    floating::FloatingDamage,
    blueprints::patamon::HolySupportState,
    blueprints::twin_core::{TwinCoreState, TwinCoreTransition},
    kernel::{
        CombatKernelTransition, HolySupportTransition, PrecisionWindowKind,
        register_combat_kernel_runtime,
    },
    log::{ActionLog, LogEntry},
    observability::{
        ValidationStatusSnapshot, capture_validation_snapshot, format_validation_snapshot,
    },
    precision_mind_game::PrecisionMindGameState,
    blueprints::dorumon::PredatorLoopState,
    sp::SpPool,
    state::{CombatPhase, CombatState},
    status_effect::{StatusBag, StatusEffectKind},
    stun::Stunned,
    team::Team,
    toughness::{DamageKind, Toughness},
    turn_order::TurnOrder,
    types::{Attribute, DamageTag, EvoStage, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{Ko, Unit},
};

fn unit(id: u32, hp_current: i32, hp_max: i32, attribute: Attribute) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

#[test]
fn snapshot_contract_covers_promised_fields_and_shape() {
    let mut world = World::new();

    world.insert_resource(CombatState {
        phase: CombatPhase::Victory,
        winner: Some(Team::Ally),
    });
    world.insert_resource(SpPool { current: 5, max: 5 });
    world.insert_resource(ActionLog {
        events: [
            LogEntry::BasicHit {
                attacker: UnitId(1),
                target: UnitId(4),
                amount: 18,
                kind: DamageKind::Weak,
            },
            LogEntry::Break {
                target: UnitId(4),
                damage_tag: DamageTag::Fire,
            },
            LogEntry::Ko { target: UnitId(4) },
        ]
        .into_iter()
        .collect(),
    });
    let mut order = TurnOrder::default();
    order.seed([UnitId(1), UnitId(4), UnitId(2)]);
    world.insert_resource(order);
    world.insert_resource(TwinCoreState {
        active_thermal_spark_targets: vec![UnitId(4)],
        cross_resonance: 2,
        fire_spend_markers: 1,
        ice_spend_markers: 1,
        twin_burst_used_this_cycle: true,
        shatter_used_this_cycle: true,
        last_signal: Some(TwinCoreTransition::twin_burst(1)),
    });

    world.spawn((
        unit(1, 100, 100, Attribute::Vaccine),
        Team::Ally,
        Toughness::new(30, vec![DamageTag::Ice]),
        UltimateCharge {
            current: 75,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
    ));
    world.spawn((
        unit(2, 55, 80, Attribute::Data),
        Team::Ally,
        Toughness::new(20, vec![DamageTag::Dark]),
        UltimateCharge {
            current: 100,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        Stunned { turns_left: 2 },
    ));
    world.spawn((
        unit(4, 0, 90, Attribute::Virus),
        Team::Enemy,
        Toughness {
            max: 50,
            current: -5,
            weaknesses: vec![DamageTag::Fire],
            broken: true,
            category: Default::default(),
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        Ko,
        Stunned { turns_left: 1 },
    ));

    world.spawn(FloatingDamage {
        target: UnitId(4),
        amount: 18,
        kind: DamageKind::Weak,
        spawn_time: 0.0,
    });
    world.spawn(FloatingDamage {
        target: UnitId(2),
        amount: 5,
        kind: DamageKind::Normal,
        spawn_time: 0.1,
    });

    let snapshot = capture_validation_snapshot(&mut world).expect("snapshot should build");
    let formatted = format_validation_snapshot(&snapshot);

    assert_eq!(
        formatted,
        "phase=Victory winner=Ally sp=5/5 twin_core=cr=2 spark_targets=[4] fire=1 ice=1 burst_guard=true shatter_guard=true last=twin-burst(1) holy_support=none predator_loop=none precision=none battery_loop=none turn_preview=[1,2] action_log_tail=[hit(attacker=1,target=4,amount=18,kind=Weak)|break(target=4,element=Fire)|ko(target=4)] floating_live=2 units=[id=1,team=Ally,hp=100/100,tough=N/A,ult=75/100/150,ko=false,stun=0,statuses=[];id=2,team=Ally,hp=55/80,tough=N/A,ult=100/100/150,ko=false,stun=2,statuses=[];id=4,team=Enemy,hp=0/90,tough=-5/50,weaknesses=[Fire],broken=true,ult=0/100/150,ko=true,stun=1,statuses=[]]"
    );
}

#[test]
fn snapshot_defaults_empty_optional_surfaces() {
    let mut world = World::new();
    world.insert_resource(CombatState::default());
    world.insert_resource(SpPool::default());
    world.insert_resource(ActionLog::default());
    let mut order = TurnOrder::default();
    order.seed([UnitId(7)]);
    world.insert_resource(order);
    world.insert_resource(TwinCoreState::default());
    world.spawn((
        unit(7, 42, 70, Attribute::Free),
        Team::Ally,
        Toughness::new(12, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
    ));

    let snapshot = capture_validation_snapshot(&mut world).expect("snapshot should build");
    let formatted = format_validation_snapshot(&snapshot);

    assert_eq!(
        formatted,
        "phase=WaitingAction winner=none sp=3/5 twin_core=cr=0 spark_targets=[] fire=0 ice=0 burst_guard=false shatter_guard=false last=none holy_support=none predator_loop=none precision=none battery_loop=none turn_preview=[7] action_log_tail=[] floating_live=0 units=[id=7,team=Ally,hp=42/70,tough=N/A,ult=0/100/150,ko=false,stun=0,statuses=[]]"
    );
}

#[test]
fn runtime_registration_applies_all_kernel_transition_domains() {
    let mut app = App::new();
    app.add_message::<CombatEvent>();
    register_combat_kernel_runtime(&mut app);

    for transition in [
        CombatKernelTransition::Blueprint {
            owner: "twin_core".into(),
            name: "build_cross_resonance".into(),
            payload: SignalPayload::Amount(1),
        },
        CombatKernelTransition::Blueprint {
            owner: "patamon".into(),
            name: "build_holy_support_grace".into(),
            payload: SignalPayload::Amount(1),
        },
        CombatKernelTransition::Blueprint {
            owner: "renamon".into(),
            name: "open_momentum_window".into(),
            payload: SignalPayload::Empty,
        },
    ] {
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnKernelTransition { transition },
            source: UnitId(1),
            target: UnitId(2),
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });
    }

    app.update();

    let twin = app.world().resource::<TwinCoreState>();
    assert_eq!(twin.cross_resonance, 1);
    assert_eq!(
        twin.last_signal,
        Some(TwinCoreTransition::build_cross_resonance(1))
    );

    let holy = app.world().resource::<HolySupportState>();
    assert_eq!(holy.grace, 1);
    assert_eq!(
        holy.last_signal,
        Some(HolySupportTransition::build_grace(1))
    );

    let precision = app.world().resource::<PrecisionMindGameState>();
    assert!(precision.is_window_open());
    assert_eq!(
        precision.current_window,
        Some(PrecisionWindowKind::Momentum)
    );
}

#[test]
fn runtime_registration_populates_snapshot_kernel_resources() {
    let mut app = App::new();
    register_combat_kernel_runtime(&mut app);

    app.world_mut().insert_resource(CombatState::default());
    app.world_mut().insert_resource(SpPool::default());
    app.world_mut().insert_resource(ActionLog::default());
    let mut order = TurnOrder::default();
    order.seed([UnitId(21)]);
    app.world_mut().insert_resource(order);
    app.world_mut().spawn((
        unit(21, 70, 70, Attribute::Free),
        Team::Ally,
        Toughness::new(12, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
    ));

    assert!(app.world().contains_resource::<TwinCoreState>());
    assert!(app.world().contains_resource::<HolySupportState>());
    assert!(app.world().contains_resource::<PredatorLoopState>());
    assert!(app.world().contains_resource::<PrecisionMindGameState>());

    let snapshot =
        capture_validation_snapshot(app.world_mut()).expect("runtime snapshot should build");
    let formatted = format_validation_snapshot(&snapshot);
    assert!(formatted.contains("twin_core=cr=0"), "{formatted}");
    assert!(formatted.contains("holy_support=grace=0/3"), "{formatted}");
    assert!(
        formatted.contains("predator_loop=exploit_cap=3"),
        "{formatted}"
    );
    assert!(formatted.contains("precision=phase=Dormant"), "{formatted}");
}

#[test]
fn snapshot_hides_ally_missing_toughness_and_zero_max_enemy_bars() {
    let mut world = World::new();
    world.insert_resource(CombatState::default());
    world.insert_resource(SpPool::default());
    world.insert_resource(ActionLog::default());
    let mut order = TurnOrder::default();
    order.seed([UnitId(11), UnitId(12), UnitId(13)]);
    world.insert_resource(order);
    world.insert_resource(TwinCoreState::default());

    world.spawn((
        unit(11, 20, 20, Attribute::Vaccine),
        Team::Ally,
        UltimateCharge {
            current: 10,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
    ));
    world.spawn((
        unit(12, 30, 30, Attribute::Virus),
        Team::Enemy,
        Toughness::new(0, vec![DamageTag::Fire]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
    ));
    world.spawn((
        unit(13, 40, 40, Attribute::Data),
        Team::Enemy,
        Toughness::new(15, vec![DamageTag::Ice]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
    ));

    let snapshot = capture_validation_snapshot(&mut world).expect("snapshot should build");
    let formatted = format_validation_snapshot(&snapshot);

    assert_eq!(
        formatted,
        "phase=WaitingAction winner=none sp=3/5 twin_core=cr=0 spark_targets=[] fire=0 ice=0 burst_guard=false shatter_guard=false last=none holy_support=none predator_loop=none precision=none battery_loop=none turn_preview=[11,12,13] action_log_tail=[] floating_live=0 units=[id=11,team=Ally,hp=20/20,tough=N/A,ult=10/100/150,ko=false,stun=0,statuses=[];id=12,team=Enemy,hp=30/30,tough=N/A,ult=0/100/150,ko=false,stun=0,statuses=[];id=13,team=Enemy,hp=40/40,tough=15/15,weaknesses=[Ice],broken=false,ult=0/100/150,ko=false,stun=0,statuses=[]]"
    );
}

#[test]
fn per_unit_statuses_populated_deterministically() {
    let mut world = World::new();
    world.insert_resource(CombatState::default());
    world.insert_resource(SpPool::default());
    world.insert_resource(ActionLog::default());
    let mut order = TurnOrder::default();
    order.seed([UnitId(99)]);
    world.insert_resource(order);
    world.insert_resource(TwinCoreState::default());

    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Slowed, 3);
    bag.apply(StatusEffectKind::Heated, 2);
    bag.apply(StatusEffectKind::Blessed, 1);
    bag.apply(StatusEffectKind::Paralyzed, 4);
    bag.apply(StatusEffectKind::Chilled, 5);

    world.spawn((
        unit(99, 50, 50, Attribute::Vaccine),
        Team::Ally,
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        bag,
    ));

    let snapshot = capture_validation_snapshot(&mut world).expect("snapshot should build");

    let unit_snap = snapshot
        .units
        .iter()
        .find(|u| u.id == UnitId(99))
        .expect("unit 99 present");
    let expected_statuses = vec![
        ValidationStatusSnapshot { kind: StatusEffectKind::Heated, duration_remaining: 2 },
        ValidationStatusSnapshot { kind: StatusEffectKind::Chilled, duration_remaining: 5 },
        ValidationStatusSnapshot { kind: StatusEffectKind::Paralyzed, duration_remaining: 4 },
        ValidationStatusSnapshot { kind: StatusEffectKind::Slowed, duration_remaining: 3 },
        ValidationStatusSnapshot { kind: StatusEffectKind::Blessed, duration_remaining: 1 },
    ];
    assert_eq!(unit_snap.statuses, expected_statuses);

    let formatted = format_validation_snapshot(&snapshot);
    assert!(
        formatted.contains("statuses=[Heated(2),Chilled(5),Paralyzed(4),Slowed(3),Blessed(1)]"),
        "formatted did not contain expected statuses token: {formatted}"
    );
}
