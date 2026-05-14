use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::blueprints;
use bevyrogue::combat::api::intent::CastId;
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::kernel::{
    register_combat_kernel_runtime, CombatKernelTransition, PredatorLoopTransition,
};
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::blueprints::dorumon::PredatorLoopState;
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::{CombatState, ResolvedAction, UltEffect};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::types::{Attribute, DamageTag, EvoStage, SkillId, UnitId};
use bevyrogue::combat::ultimate::{UltAccumulationTrigger, UltimateCharge};
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::skills_ron::{CustomSignalPayload, SkillCustomSignal, TargetShape};

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

fn ultimate_charge(current: i32) -> UltimateCharge {
    UltimateCharge {
        current,
        trigger: 80,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 20,
    }
}

fn dorumon_signal(signal: &str, payload: CustomSignalPayload) -> SkillCustomSignal {
    SkillCustomSignal::blueprint("dorumon", signal, payload)
}

fn dorumon_action() -> ResolvedAction {
    ResolvedAction {
        source: UnitId(7),
        target: UnitId(8),
        skill_id: SkillId("dorumon_predator_runtime_test".into()),
        damage_tag: DamageTag::Dark,
        base_damage: 0,
        toughness_damage: 0,
        revive_pct: 0,
        heal_pct: 0,
        sp_cost: 0,
        ult_effect: UltEffect::None,
        grant_free_skill_count: 0,
        status_to_apply: None,
        advance_pct: 0,
        delay_pct: 0,
        energy_grant: 0,
        self_advance_pct: 0,
        target_shape: TargetShape::Single,
        custom_signals: vec![
            dorumon_signal("build_exploit", CustomSignalPayload::Amount { amount: 2 }),
            dorumon_signal("apply_prey_lock", CustomSignalPayload::Empty),
        ],
        damage_curve: Default::default(),
        cleanse_count: None,
    }
}

fn cursor(app: &mut App) -> MessageCursor<CombatEvent> {
    app.world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor()
}

fn drain(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    let messages = app.world().resource::<Messages<CombatEvent>>();
    cursor.read(messages).cloned().collect()
}

fn app_with_dorumon_runtime() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>();
    register_combat_kernel_runtime(&mut app);
    app.init_resource::<CombatState>();
    app.init_resource::<SpPool>();
    app.init_resource::<ActionLog>();

    app.world_mut()
        .resource_mut::<PredatorLoopState>()
        .track_target(UnitId(8));

    app.world_mut().spawn((
        unit(7, 90, 90, Attribute::Vaccine),
        Team::Ally,
        ultimate_charge(0),
    ));
    app.world_mut().spawn((
        unit(8, 120, 120, Attribute::Virus),
        Team::Enemy,
        Toughness::new(30, vec![DamageTag::Light]),
        ultimate_charge(0),
    ));

    app
}

#[test]
fn dorumon_runtime_transitions_flow_through_canonical_predator_events() {
    let mut app = app_with_dorumon_runtime();

    let action = dorumon_action();
    let transitions = blueprints::transitions_for_action(&action);
    assert_eq!(
        transitions,
        vec![
            CombatKernelTransition::PredatorLoop(PredatorLoopTransition::build_exploit(
                action.target,
                2,
            )),
            CombatKernelTransition::PredatorLoop(PredatorLoopTransition::apply_prey_lock(
                action.target,
                0,
            )),
        ]
    );
    let expected_predator_results = vec![
        PredatorLoopTransition::build_exploit(action.target, 2),
        PredatorLoopTransition::apply_prey_lock(action.target, 2),
    ];

    for transition in transitions.iter().cloned() {
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnKernelTransition { transition },
            source: action.source,
            target: action.target,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });
    }

    let mut reader = cursor(&mut app);
    app.update();
    app.update();

    let events = drain(&mut reader, &app);
    assert_eq!(events.len(), expected_predator_results.len());
    assert!(events
        .iter()
        .all(|event| matches!(event.kind, CombatEventKind::PredatorLoopResolved { .. })));

    let predator_results: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.kind {
            CombatEventKind::PredatorLoopResolved { transition } => Some(*transition),
            _ => None,
        })
        .collect();

    assert_eq!(predator_results, expected_predator_results);

    let serialized = serde_json::to_string(&events).expect("serialize combat events");
    assert!(!serialized.contains("OnKernelTransition"));
    assert!(serialized.contains("PredatorLoopResolved"));
    assert!(serialized.contains("BuildExploit"));
    assert!(serialized.contains("ApplyPreyLock"));

    let state = app.world().resource::<PredatorLoopState>();
    let target = state.targets.get(&action.target).expect("tracked target");
    assert_eq!(target.exploit_stacks, 2);
    assert_eq!(
        target.prey_lock.map(|lock| lock.turns_left),
        Some(state.prey_lock_duration)
    );
    assert_eq!(
        state.last_transition,
        Some(PredatorLoopTransition::apply_prey_lock(
            action.target,
            state.prey_lock_duration as u16,
        ))
    );

    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot");
    let formatted = format_validation_snapshot(&snapshot);
    assert!(
        formatted.contains("predator_loop=exploit_cap=3"),
        "{formatted}"
    );
    assert!(formatted.contains("targets=[8:e2:p2]"), "{formatted}");
    assert!(
        formatted.contains("last=prey-lock(target=Some(UnitId(8));amount=2)"),
        "{formatted}"
    );
    assert!(formatted.contains("blocked=none"), "{formatted}");
}
