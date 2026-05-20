use crate::combat::bevy_types::*;

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::CombatKernelTransition;
use crate::combat::runtime::SignalPayload;

use super::identity::{
    BatteryLoopBlockedReason, BatteryLoopChargeKind, BatteryLoopSignal, BatteryLoopSnapshot,
    BatteryLoopState, BatteryLoopStep, BatteryLoopTransition,
};
use super::{
    BATTERY_ENERGY_GRANT, OWNER, SIG_BUILD_CIRCUIT_CHARGE, SIG_BUILD_STATIC_CHARGE,
    SIG_CYCLE_RESET, SIG_SPEND_CIRCUIT_CHARGE,
};

pub fn apply_battery_loop_transition(
    state: &mut BatteryLoopState,
    transition: BatteryLoopTransition,
) -> BatteryLoopTransition {
    let applied = match transition.signal {
        BatteryLoopSignal::BuildStaticCharge => apply_static_charge(state, transition.amount),
        BatteryLoopSignal::BuildCircuitCharge => apply_circuit_charge(state, transition.amount),
        BatteryLoopSignal::SpendCircuitCharge => spend_circuit_charge(state, transition.amount),
        BatteryLoopSignal::BlockReady => {
            state.block_reaction_armed = true;
            transition
        }
        BatteryLoopSignal::BlockProc => {
            state.block_reaction_armed = false;
            transition
        }
        BatteryLoopSignal::GrantEnergy
        | BatteryLoopSignal::SelfEnergyGain
        | BatteryLoopSignal::TransferEnergy
        | BatteryLoopSignal::CycleReset => {
            if matches!(transition.signal, BatteryLoopSignal::CycleReset) {
                state.static_charge = 0;
                state.threshold_grant_emitted_this_cycle = false;
                state.block_reaction_armed = false;
                state.last_block_reaction_cast_id = None;
            }
            transition
        }
        BatteryLoopSignal::Rejected | BatteryLoopSignal::Ignored => transition,
    };

    record_applied_transition(state, applied)
}

pub fn apply_battery_loop_transitions_system(
    mut events: MessageReader<CombatEvent>,
    mut state: ResMut<BatteryLoopState>,
) {
    for event in events.read() {
        let CombatEventKind::OnKernelTransition { transition } = &event.kind else {
            continue;
        };

        let CombatKernelTransition::Blueprint {
            owner,
            name,
            payload,
        } = transition
        else {
            continue;
        };

        if owner != OWNER {
            continue;
        }

        let amount = match payload {
            SignalPayload::Amount(amount) => match u8::try_from(*amount) {
                Ok(amount) => amount,
                Err(_) => continue,
            },
            _ => continue,
        };

        let battery_transition = match name.as_str() {
            SIG_BUILD_STATIC_CHARGE => BatteryLoopTransition::build_static_charge(amount),
            SIG_BUILD_CIRCUIT_CHARGE => BatteryLoopTransition::build_circuit_charge(amount),
            SIG_SPEND_CIRCUIT_CHARGE => BatteryLoopTransition::spend_circuit_charge(amount),
            SIG_CYCLE_RESET => BatteryLoopTransition::cycle_reset(),
            _ => continue,
        };

        apply_battery_loop_transition(&mut state, battery_transition);
    }
}

fn apply_static_charge(state: &mut BatteryLoopState, amount: u8) -> BatteryLoopTransition {
    if amount == 0 {
        return BatteryLoopTransition::ignored(BatteryLoopStep::BuildStaticCharge { amount });
    }

    if state.static_charge >= state.static_charge_cap {
        return BatteryLoopTransition::rejected(
            BatteryLoopStep::BuildStaticCharge { amount },
            BatteryLoopBlockedReason::ChargeCapReached {
                charge: BatteryLoopChargeKind::Static,
            },
        );
    }

    state.static_charge = state
        .static_charge
        .saturating_add(amount)
        .min(state.static_charge_cap);

    if state.threshold_grant_eligible() {
        state.threshold_grant_emitted_this_cycle = true;
        state.block_reaction_armed = true;
        BatteryLoopTransition::grant_energy(BATTERY_ENERGY_GRANT)
    } else {
        BatteryLoopTransition::build_static_charge(amount)
    }
}

fn apply_circuit_charge(state: &mut BatteryLoopState, amount: u8) -> BatteryLoopTransition {
    if amount == 0 {
        return BatteryLoopTransition::ignored(BatteryLoopStep::BuildCircuitCharge { amount });
    }

    if state.circuit_charge >= state.circuit_charge_cap {
        return BatteryLoopTransition::rejected(
            BatteryLoopStep::BuildCircuitCharge { amount },
            BatteryLoopBlockedReason::ChargeCapReached {
                charge: BatteryLoopChargeKind::Circuit,
            },
        );
    }

    state.circuit_charge = state
        .circuit_charge
        .saturating_add(amount)
        .min(state.circuit_charge_cap);

    BatteryLoopTransition::build_circuit_charge(amount)
}

fn spend_circuit_charge(state: &mut BatteryLoopState, amount: u8) -> BatteryLoopTransition {
    if amount == 0 {
        return BatteryLoopTransition::ignored(BatteryLoopStep::SpendCircuitCharge { amount });
    }

    if amount > state.circuit_charge {
        return BatteryLoopTransition::rejected(
            BatteryLoopStep::SpendCircuitCharge { amount },
            BatteryLoopBlockedReason::ChargeUnderflow {
                charge: BatteryLoopChargeKind::Circuit,
            },
        );
    }

    state.circuit_charge -= amount;
    BatteryLoopTransition::spend_circuit_charge(amount)
}

fn record_applied_transition(
    state: &mut BatteryLoopState,
    transition: BatteryLoopTransition,
) -> BatteryLoopTransition {
    if matches!(transition.signal, BatteryLoopSignal::Rejected) {
        state.last_blocked_reason = transition.reason;
    } else {
        state.last_blocked_reason = None;
    }

    state.last_transition = Some(transition);
    debug!(
        "BatteryLoopState static={}/{} circuit={}/{} threshold={} grant_guard={} last={:?} blocked={:?}",
        state.static_charge,
        state.static_charge_cap,
        state.circuit_charge,
        state.circuit_charge_cap,
        state.static_charge_threshold,
        state.threshold_grant_emitted_this_cycle,
        state.last_transition,
        state.last_blocked_reason,
    );
    transition
}

pub fn format_battery_loop_snapshot(snapshot: &BatteryLoopSnapshot) -> String {
    format!(
        "static={}/{} circuit={}/{} threshold={} grant_guard={} block_ready={} last_block_cast={} last={} blocked={}",
        snapshot.static_charge,
        snapshot.static_charge_cap,
        snapshot.circuit_charge,
        snapshot.circuit_charge_cap,
        snapshot.static_charge_threshold,
        snapshot.threshold_grant_emitted_this_cycle,
        snapshot.block_reaction_armed,
        snapshot
            .last_block_reaction_cast_id
            .map(|cast_id| cast_id.0.get().to_string())
            .unwrap_or_else(|| "none".to_string()),
        snapshot
            .last_transition
            .map(format_battery_loop_transition)
            .unwrap_or_else(|| "none".to_string()),
        snapshot
            .last_blocked_reason
            .map(format_battery_loop_blocked_reason)
            .unwrap_or_else(|| "none".to_string()),
    )
}

pub(crate) fn format_battery_loop_transition(transition: BatteryLoopTransition) -> String {
    match transition.signal {
        BatteryLoopSignal::BuildStaticCharge => format!("build-static({})", transition.amount),
        BatteryLoopSignal::BuildCircuitCharge => format!("build-circuit({})", transition.amount),
        BatteryLoopSignal::SpendCircuitCharge => format!("spend-circuit({})", transition.amount),
        BatteryLoopSignal::BlockReady => "block-ready".to_string(),
        BatteryLoopSignal::BlockProc => "block-proc".to_string(),
        BatteryLoopSignal::GrantEnergy => format!("grant({})", transition.amount),
        BatteryLoopSignal::SelfEnergyGain => format!("self-gain({})", transition.amount),
        BatteryLoopSignal::TransferEnergy => format!("transfer({})", transition.amount),
        BatteryLoopSignal::CycleReset => "cycle-reset".to_string(),
        BatteryLoopSignal::Rejected => match (transition.attempted, transition.reason) {
            (Some(attempted), Some(reason)) => {
                format!(
                    "rejected({};reason={reason:?})",
                    format_battery_loop_step(attempted)
                )
            }
            (Some(attempted), None) => format!("rejected({})", format_battery_loop_step(attempted)),
            _ => "rejected".to_string(),
        },
        BatteryLoopSignal::Ignored => match transition.attempted {
            Some(attempted) => format!("ignored({})", format_battery_loop_step(attempted)),
            None => "ignored".to_string(),
        },
    }
}

fn format_battery_loop_step(step: BatteryLoopStep) -> String {
    match step {
        BatteryLoopStep::BuildStaticCharge { amount } => format!("build-static({amount})"),
        BatteryLoopStep::BuildCircuitCharge { amount } => format!("build-circuit({amount})"),
        BatteryLoopStep::SpendCircuitCharge { amount } => format!("spend-circuit({amount})"),
        BatteryLoopStep::BlockReady => "block-ready".to_string(),
        BatteryLoopStep::BlockProc => "block-proc".to_string(),
        BatteryLoopStep::GrantEnergy { amount } => format!("grant({amount})"),
        BatteryLoopStep::SelfEnergyGain { amount } => format!("self-gain({amount})"),
        BatteryLoopStep::TransferEnergy { amount } => format!("transfer({amount})"),
        BatteryLoopStep::CycleReset => "cycle-reset".to_string(),
    }
}

fn format_battery_loop_blocked_reason(reason: BatteryLoopBlockedReason) -> String {
    match reason {
        BatteryLoopBlockedReason::ChargeCapReached { charge } => format!("cap-reached({charge:?})"),
        BatteryLoopBlockedReason::ChargeUnderflow { charge } => format!("underflow({charge:?})"),
        BatteryLoopBlockedReason::MissingPreExistingShock => "missing-shock".to_string(),
        BatteryLoopBlockedReason::NoEligibleAlly => "no-eligible-ally".to_string(),
        BatteryLoopBlockedReason::UnsupportedRequest => "unsupported".to_string(),
        BatteryLoopBlockedReason::MalformedData => "malformed".to_string(),
    }
}
