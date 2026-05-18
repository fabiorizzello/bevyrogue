use serde::{Deserialize, Serialize};

use crate::combat::bevy_types::*;

use crate::combat::runtime::intent::CastId;

use super::{
    CIRCUIT_CHARGE_CAP, STATIC_CHARGE_THRESHOLD, apply::apply_battery_loop_transition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLoopChargeKind {
    Static,
    Circuit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLoopBlockedReason {
    ChargeCapReached { charge: BatteryLoopChargeKind },
    ChargeUnderflow { charge: BatteryLoopChargeKind },
    MissingPreExistingShock,
    NoEligibleAlly,
    UnsupportedRequest,
    MalformedData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLoopStep {
    BuildStaticCharge { amount: u8 },
    BuildCircuitCharge { amount: u8 },
    SpendCircuitCharge { amount: u8 },
    BlockReady,
    BlockProc,
    GrantEnergy { amount: u8 },
    SelfEnergyGain { amount: u8 },
    TransferEnergy { amount: u8 },
    CycleReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLoopSignal {
    BuildStaticCharge,
    BuildCircuitCharge,
    SpendCircuitCharge,
    BlockReady,
    BlockProc,
    GrantEnergy,
    SelfEnergyGain,
    TransferEnergy,
    CycleReset,
    Rejected,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryLoopTransition {
    pub signal: BatteryLoopSignal,
    pub amount: u8,
    pub attempted: Option<BatteryLoopStep>,
    pub reason: Option<BatteryLoopBlockedReason>,
}

impl BatteryLoopTransition {
    pub const fn build_static_charge(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::BuildStaticCharge,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn build_circuit_charge(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::BuildCircuitCharge,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn spend_circuit_charge(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::SpendCircuitCharge,
            amount,
            attempted: None,
            reason: None,
        }
    }

    // block_ready/self_energy_gain/transfer_energy not yet consumed; kept for API completeness.
    pub const fn block_ready() -> Self {
        Self {
            signal: BatteryLoopSignal::BlockReady,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn block_proc() -> Self {
        Self {
            signal: BatteryLoopSignal::BlockProc,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn grant_energy(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::GrantEnergy,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn self_energy_gain(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::SelfEnergyGain,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn transfer_energy(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::TransferEnergy,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn cycle_reset() -> Self {
        Self {
            signal: BatteryLoopSignal::CycleReset,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn rejected(attempted: BatteryLoopStep, reason: BatteryLoopBlockedReason) -> Self {
        Self {
            signal: BatteryLoopSignal::Rejected,
            amount: 0,
            attempted: Some(attempted),
            reason: Some(reason),
        }
    }

    pub const fn ignored(attempted: BatteryLoopStep) -> Self {
        Self {
            signal: BatteryLoopSignal::Ignored,
            amount: 0,
            attempted: Some(attempted),
            reason: None,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryLoopSnapshot {
    pub static_charge: u8,
    pub static_charge_cap: u8,
    pub circuit_charge: u8,
    pub circuit_charge_cap: u8,
    pub static_charge_threshold: u8,
    pub threshold_grant_emitted_this_cycle: bool,
    pub block_reaction_armed: bool,
    pub last_block_reaction_cast_id: Option<crate::combat::runtime::intent::CastId>,
    pub last_transition: Option<BatteryLoopTransition>,
    pub last_blocked_reason: Option<BatteryLoopBlockedReason>,
}

impl From<&BatteryLoopState> for BatteryLoopSnapshot {
    fn from(state: &BatteryLoopState) -> Self {
        Self {
            static_charge: state.static_charge,
            static_charge_cap: state.static_charge_cap,
            circuit_charge: state.circuit_charge,
            circuit_charge_cap: state.circuit_charge_cap,
            static_charge_threshold: state.static_charge_threshold,
            threshold_grant_emitted_this_cycle: state.threshold_grant_emitted_this_cycle,
            block_reaction_armed: state.block_reaction_armed,
            last_block_reaction_cast_id: state.last_block_reaction_cast_id,
            last_transition: state.last_transition,
            last_blocked_reason: state.last_blocked_reason,
        }
    }
}
