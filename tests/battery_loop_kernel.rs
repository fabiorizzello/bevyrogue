use bevy::prelude::*;

use bevyrogue::combat::api::{SignalPayload, intent::CastId};
use bevyrogue::combat::blueprints;
use bevyrogue::combat::battery_loop::{
    BATTERY_ENERGY_GRANT, BatteryLoopBlockedReason, BatteryLoopChargeKind, BatteryLoopSignal,
    BatteryLoopState, BatteryLoopStep, BatteryLoopTransition, apply_battery_loop_transition,
    apply_battery_loop_transitions_system,
};
use bevyrogue::combat::blueprints::tentomon::{
    OWNER as TENTOMON_OWNER, SIG_BUILD_STATIC_CHARGE, SIG_CYCLE_RESET,
};
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::kernel::{
    CombatKernelHook, CombatKernelTransition, TacticalCycleStep, TacticalCycleTransition,
    register_combat_kernel_runtime,
};
use bevyrogue::combat::battery_loop::{BatteryLoopSnapshot, format_battery_loop_snapshot};
use bevyrogue::combat::types::UnitId;

fn app_with_battery_loop() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>()
        .init_resource::<BatteryLoopState>()
        .add_systems(Update, apply_battery_loop_transitions_system);
    app
}

fn tentomon_blueprint_transition(name: &str, amount: i64) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: TENTOMON_OWNER.to_string(),
        name: name.to_string(),
        payload: SignalPayload::Amount(amount),
    }
}

fn emit_kernel_transition(app: &mut App, transition: CombatKernelTransition) {
    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::OnKernelTransition { transition },
        source: UnitId(1),
        target: UnitId(1),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
    app.update();
}

#[test]
fn runtime_registration_applies_battery_loop_transition_once() {
    let mut app = App::new();
    app.add_message::<CombatEvent>();
    register_combat_kernel_runtime(&mut app);
    blueprints::add_runtime_plugins(&mut app);

    emit_kernel_transition(
        &mut app,
        tentomon_blueprint_transition(SIG_BUILD_STATIC_CHARGE, 1),
    );

    let state = app.world().resource::<BatteryLoopState>();
    assert_eq!(state.static_charge, 1);
    assert_eq!(
        state.last_transition,
        Some(BatteryLoopTransition::build_static_charge(1))
    );
}

#[test]
fn wrapped_cycle_hook_emits_tentomon_blueprint_cycle_reset() {
    let hook = bevyrogue::combat::battery_loop::BatteryLoopHook;
    let mut out = Vec::new();

    hook.on_transition(
        &CombatKernelTransition::TacticalCycle(TacticalCycleTransition {
            before: TacticalCycleStep::default(),
            after: TacticalCycleStep::default(),
            wrapped_phase: false,
            wrapped_cycle: true,
        }),
        &mut out,
    );

    assert_eq!(
        out,
        vec![CombatKernelTransition::Blueprint {
            owner: TENTOMON_OWNER.to_string(),
            name: SIG_CYCLE_RESET.to_string(),
            payload: SignalPayload::Amount(0),
        }]
    );
}

#[test]
fn foreign_blueprint_transitions_do_not_mutate_battery_loop_state() {
    let mut app = app_with_battery_loop();
    let before = app.world().resource::<BatteryLoopState>().clone();

    emit_kernel_transition(
        &mut app,
        CombatKernelTransition::Blueprint {
            owner: "other".to_string(),
            name: SIG_BUILD_STATIC_CHARGE.to_string(),
            payload: SignalPayload::Amount(1),
        },
    );

    let after = app.world().resource::<BatteryLoopState>();
    assert_eq!(*after, before);
}

#[test]
fn static_charge_starts_clean_and_grants_energy_on_the_third_hit() {
    let mut state = BatteryLoopState::default();

    assert_eq!(state.static_charge, 0);
    assert_eq!(state.circuit_charge, 0);
    assert!(!state.threshold_grant_eligible());

    let first = state.gain_static_charge(1);
    assert_eq!(state.static_charge, 1);
    assert_eq!(first, BatteryLoopTransition::build_static_charge(1));

    let second = state.gain_static_charge(1);
    assert_eq!(state.static_charge, 2);
    assert_eq!(second, BatteryLoopTransition::build_static_charge(1));

    let third = state.gain_static_charge(1);
    assert_eq!(state.static_charge, 3);
    assert!(!state.threshold_grant_eligible());
    assert_eq!(
        third,
        BatteryLoopTransition::grant_energy(BATTERY_ENERGY_GRANT)
    );

    let snapshot = BatteryLoopSnapshot::from(&state);
    assert_eq!(snapshot.static_charge, 3);
    assert_eq!(snapshot.static_charge_cap, 3);
    assert_eq!(snapshot.last_transition, Some(third));
    assert_eq!(snapshot.last_blocked_reason, None);
    assert!(format_battery_loop_snapshot(&snapshot).contains("grant(5)"));
}

#[test]
fn static_charge_caps_and_reports_a_stable_blocked_reason_on_the_fourth_hit() {
    let mut state = BatteryLoopState::default();

    state.gain_static_charge(1);
    state.gain_static_charge(1);
    state.gain_static_charge(1);

    let fourth = state.gain_static_charge(1);
    assert_eq!(state.static_charge, 3);
    assert_eq!(fourth.signal, BatteryLoopSignal::Rejected);
    assert_eq!(
        fourth,
        BatteryLoopTransition::rejected(
            BatteryLoopStep::BuildStaticCharge { amount: 1 },
            BatteryLoopBlockedReason::ChargeCapReached {
                charge: BatteryLoopChargeKind::Static,
            },
        )
    );
    assert_eq!(state.last_transition, Some(fourth));
    assert_eq!(
        state.last_blocked_reason,
        Some(BatteryLoopBlockedReason::ChargeCapReached {
            charge: BatteryLoopChargeKind::Static,
        })
    );

    let snapshot = BatteryLoopSnapshot::from(&state);
    assert_eq!(snapshot.last_transition, Some(fourth));
    assert_eq!(
        snapshot.last_blocked_reason,
        Some(BatteryLoopBlockedReason::ChargeCapReached {
            charge: BatteryLoopChargeKind::Static,
        })
    );
}

#[test]
fn circuit_spend_cannot_underflow_and_records_a_blocked_reason() {
    let mut state = BatteryLoopState::default();

    let rejected = state.spend_circuit_charge(1);
    assert_eq!(state.circuit_charge, 0);
    assert_eq!(
        rejected,
        BatteryLoopTransition::rejected(
            BatteryLoopStep::SpendCircuitCharge { amount: 1 },
            BatteryLoopBlockedReason::ChargeUnderflow {
                charge: BatteryLoopChargeKind::Circuit,
            },
        )
    );
    assert_eq!(
        state.last_blocked_reason,
        Some(BatteryLoopBlockedReason::ChargeUnderflow {
            charge: BatteryLoopChargeKind::Circuit,
        })
    );

    let accepted =
        apply_battery_loop_transition(&mut state, BatteryLoopTransition::build_circuit_charge(3));
    assert_eq!(accepted, BatteryLoopTransition::build_circuit_charge(3));
    assert_eq!(state.circuit_charge, 3);

    let spent = state.spend_circuit_charge(2);
    assert_eq!(spent, BatteryLoopTransition::spend_circuit_charge(2));
    assert_eq!(state.circuit_charge, 1);
}

#[test]
fn tactical_cycle_reset_is_visible_after_message_flush() {
    let mut app = app_with_battery_loop();

    {
        let mut state = app.world_mut().resource_mut::<BatteryLoopState>();
        state.gain_static_charge(1);
        state.gain_static_charge(1);
        state.gain_static_charge(1);
        assert_eq!(state.static_charge, 3);
        assert!(state.threshold_grant_emitted_this_cycle);
    }

    emit_kernel_transition(&mut app, tentomon_blueprint_transition(SIG_CYCLE_RESET, 0));

    let state = app.world().resource::<BatteryLoopState>();
    assert_eq!(state.static_charge, 0);
    assert!(!state.threshold_grant_emitted_this_cycle);
    assert_eq!(
        state.last_transition,
        Some(BatteryLoopTransition::cycle_reset())
    );
}
