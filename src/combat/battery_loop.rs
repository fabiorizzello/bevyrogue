use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::events::{CombatEvent, CombatEventKind};
use super::kernel::{
    BatteryLoopBlockedReason, BatteryLoopChargeKind, BatteryLoopSignal, BatteryLoopStep,
    BatteryLoopTransition, CombatKernelHook, CombatKernelHookDomain, CombatKernelTransition,
    TacticalCycleTransition,
};
use super::observability::BatteryLoopSnapshot;
use super::blueprints::tentomon::{
    OWNER as TENTOMON_OWNER, SIG_BUILD_CIRCUIT_CHARGE, SIG_BUILD_STATIC_CHARGE,
    SIG_CYCLE_RESET, SIG_SPEND_CIRCUIT_CHARGE,
};
use crate::combat::api::{intent::CastId, SignalPayload};

pub const STATIC_CHARGE_THRESHOLD: u8 = 3;
pub const CIRCUIT_CHARGE_CAP: u8 = 3;
pub const BATTERY_ENERGY_GRANT: u8 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLoopDesignTag {
    StaticCharge,
    CircuitCharge,
    ShockTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLoopRequestKind {
    BuildStaticCharge,
    BuildCircuitCharge,
    SelfEnergyGain,
    TransferEnergy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
pub struct BatteryLoopState {
    pub static_charge: u8,
    pub circuit_charge: u8,
    pub static_charge_cap: u8,
    pub circuit_charge_cap: u8,
    pub static_charge_threshold: u8,
    pub threshold_grant_emitted_this_cycle: bool,
    pub block_reaction_armed: bool,
    pub last_block_reaction_cast_id: Option<CastId>,
    pub last_transition: Option<BatteryLoopTransition>,
    pub last_blocked_reason: Option<BatteryLoopBlockedReason>,
}

impl Default for BatteryLoopState {
    fn default() -> Self {
        Self {
            static_charge: 0,
            circuit_charge: 0,
            static_charge_cap: STATIC_CHARGE_THRESHOLD,
            circuit_charge_cap: CIRCUIT_CHARGE_CAP,
            static_charge_threshold: STATIC_CHARGE_THRESHOLD,
            threshold_grant_emitted_this_cycle: false,
            block_reaction_armed: false,
            last_block_reaction_cast_id: None,
            last_transition: None,
            last_blocked_reason: None,
        }
    }
}

impl BatteryLoopState {
    pub fn threshold_grant_eligible(&self) -> bool {
        self.static_charge >= self.static_charge_threshold
            && !self.threshold_grant_emitted_this_cycle
    }

    pub fn block_reaction_ready(&self) -> bool {
        self.block_reaction_armed
    }

    pub fn arm_block_reaction(&mut self) -> BatteryLoopTransition {
        apply_battery_loop_transition(self, BatteryLoopTransition::block_ready())
    }

    pub fn proc_block_reaction(&mut self) -> BatteryLoopTransition {
        apply_battery_loop_transition(self, BatteryLoopTransition::block_proc())
    }

    pub fn gain_static_charge(&mut self, amount: u8) -> BatteryLoopTransition {
        apply_battery_loop_transition(self, BatteryLoopTransition::build_static_charge(amount))
    }

    pub fn gain_circuit_charge(&mut self, amount: u8) -> BatteryLoopTransition {
        apply_battery_loop_transition(self, BatteryLoopTransition::build_circuit_charge(amount))
    }

    pub fn spend_circuit_charge(&mut self, amount: u8) -> BatteryLoopTransition {
        apply_battery_loop_transition(self, BatteryLoopTransition::spend_circuit_charge(amount))
    }

    pub fn record_self_energy_gain(&mut self, amount: u8) -> BatteryLoopTransition {
        apply_battery_loop_transition(self, BatteryLoopTransition::self_energy_gain(amount))
    }

    pub fn record_transfer_success(&mut self, amount: u8) -> BatteryLoopTransition {
        apply_battery_loop_transition(self, BatteryLoopTransition::transfer_energy(amount))
    }

    pub fn record_transfer_blocked(
        &mut self,
        amount: u8,
        reason: BatteryLoopBlockedReason,
    ) -> BatteryLoopTransition {
        apply_battery_loop_transition(
            self,
            BatteryLoopTransition::rejected(BatteryLoopStep::TransferEnergy { amount }, reason),
        )
    }

    pub fn reset_cycle_guards(&mut self) -> BatteryLoopTransition {
        apply_battery_loop_transition(self, BatteryLoopTransition::cycle_reset())
    }
}

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

        let CombatKernelTransition::Blueprint { owner, name, payload } = transition else {
            continue;
        };

        if owner != TENTOMON_OWNER {
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

pub struct BatteryLoopHook;

impl CombatKernelHook for BatteryLoopHook {
    fn domain(&self) -> CombatKernelHookDomain {
        CombatKernelHookDomain::Shared
    }

    fn on_transition(
        &self,
        transition: &CombatKernelTransition,
        out: &mut Vec<CombatKernelTransition>,
    ) {
        if matches!(
            transition,
            CombatKernelTransition::TacticalCycle(TacticalCycleTransition {
                wrapped_cycle: true,
                ..
            })
        ) {
            out.push(CombatKernelTransition::Blueprint {
                owner: TENTOMON_OWNER.to_string(),
                name: SIG_CYCLE_RESET.to_string(),
                payload: SignalPayload::Amount(0),
            });
        }
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
