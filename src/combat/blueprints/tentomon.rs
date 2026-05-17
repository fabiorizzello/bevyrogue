use serde::{Deserialize, Serialize};

use crate::combat::bevy_types::*;

use crate::combat::api::{SignalPayload, intent::CastId};
use crate::combat::api::registry::{ValidationField, ValidationSection};
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::{
    CombatKernelHook, CombatKernelHookDomain, CombatKernelTransition, TacticalCycleTransition,
};
use crate::combat::modifiers::{DamageModifierLedger, ModifierLayer};
use crate::combat::rng::CombatRng;
use crate::combat::types::UnitId;
use crate::combat::unit::Unit;

use super::{CustomSignalDispatchError, amount_payload};

pub const OWNER: &str = "tentomon";
pub const SIG_BUILD_STATIC_CHARGE: &str = "build_static_charge";
pub const SIG_BUILD_CIRCUIT_CHARGE: &str = "build_circuit_charge";
pub const SIG_SPEND_CIRCUIT_CHARGE: &str = "spend_circuit_charge";
pub const SIG_CYCLE_RESET: &str = "cycle_reset";
const BLOCK_REACTION_CHANCE_PCT: i32 = 30;
const BLOCK_REACTION_MITIGATION_PCT: i32 = 50;

pub const STATIC_CHARGE_THRESHOLD: u8 = 3;
pub const CIRCUIT_CHARGE_CAP: u8 = 3;
pub const BATTERY_ENERGY_GRANT: u8 = 5;

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
                owner: OWNER.to_string(),
                name: SIG_CYCLE_RESET.to_string(),
                payload: SignalPayload::Amount(0),
            });
        }
    }
}

pub struct TentomonPlugin;

impl Plugin for TentomonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BatteryLoopState>().add_systems(
            Update,
            apply_battery_loop_transitions_system,
        );

        app.world_mut()
            .resource_mut::<crate::combat::kernel::CombatKernelRegistry>()
            .register(BatteryLoopHook);
    }
}

pub fn register_tentomon_ext(regs: &mut crate::combat::api::ExtRegistries) {
    regs.validation
        .register("battery/validation", battery_validation_section);
}

fn battery_validation_section(world: &World) -> Option<ValidationSection> {
    world.get_resource::<BatteryLoopState>().map(|state| {
        let snapshot = BatteryLoopSnapshot::from(state);
        ValidationSection::new(
            "battery",
            vec![
                ValidationField::new("static", format!("{}/{}", snapshot.static_charge, snapshot.static_charge_cap)),
                ValidationField::new("circuit", format!("{}/{}", snapshot.circuit_charge, snapshot.circuit_charge_cap)),
                ValidationField::new("block_ready", snapshot.block_reaction_armed.to_string()),
                ValidationField::new(
                    "last",
                    snapshot
                        .last_transition
                        .map(format_battery_loop_transition)
                        .unwrap_or_else(|| "none".to_string()),
                ),
            ],
        )
    })
}

fn blueprint_transition(name: &'static str, amount: i64) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: OWNER.to_string(),
        name: name.to_string(),
        payload: SignalPayload::Amount(amount),
    }
}

pub fn dispatch(
    signal: &crate::data::skills_ron::SkillCustomSignal,
    _action: &crate::combat::state::ResolvedAction,
) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> {
    if signal.owner() != OWNER {
        return Err(CustomSignalDispatchError::UnknownOwner {
            owner: signal.owner().to_owned(),
        });
    }

    match signal.signal() {
        SIG_BUILD_STATIC_CHARGE => {
            let amount = amount_payload(signal, OWNER, SIG_BUILD_STATIC_CHARGE)?;
            Ok(vec![blueprint_transition(
                SIG_BUILD_STATIC_CHARGE,
                amount as i64,
            )])
        }
        SIG_BUILD_CIRCUIT_CHARGE => {
            let amount = amount_payload(signal, OWNER, SIG_BUILD_CIRCUIT_CHARGE)?;
            Ok(vec![blueprint_transition(
                SIG_BUILD_CIRCUIT_CHARGE,
                amount as i64,
            )])
        }
        SIG_SPEND_CIRCUIT_CHARGE => {
            let amount = amount_payload(signal, OWNER, SIG_SPEND_CIRCUIT_CHARGE)?;
            Ok(vec![blueprint_transition(
                SIG_SPEND_CIRCUIT_CHARGE,
                amount as i64,
            )])
        }
        SIG_CYCLE_RESET => Ok(vec![blueprint_transition(SIG_CYCLE_RESET, 0)]),
        _ => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_owned(),
            signal: signal.signal().to_owned(),
        }),
    }
}

pub fn resolve_block_reaction_in_world(
    world: &mut World,
    target: UnitId,
    cast_id: CastId,
) -> Option<i32> {
    if !world
        .get_resource::<BatteryLoopState>()?
        .block_reaction_ready()
    {
        return None;
    }

    let rolled = world.resource_scope(|_w, mut rng: Mut<CombatRng>| {
        rng.roll_pct(BLOCK_REACTION_CHANCE_PCT)
    });
    if !rolled {
        return None;
    }

    world.resource_scope(|_w, mut state: Mut<BatteryLoopState>| {
        state.proc_block_reaction();
        state.last_block_reaction_cast_id = Some(cast_id);
    });

    if let Some(mut ledger) = world.get_resource_mut::<DamageModifierLedger>() {
        ledger.arm(target, ModifierLayer::Passive, BLOCK_REACTION_MITIGATION_PCT);
    }

    debug!(
        "Tentomon block reaction triggered for target={:?} cast_id={:?}",
        target, cast_id
    );

    Some(BLOCK_REACTION_MITIGATION_PCT)
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryLoopSnapshot {
    pub static_charge: u8,
    pub static_charge_cap: u8,
    pub circuit_charge: u8,
    pub circuit_charge_cap: u8,
    pub static_charge_threshold: u8,
    pub threshold_grant_emitted_this_cycle: bool,
    pub block_reaction_armed: bool,
    pub last_block_reaction_cast_id: Option<crate::combat::api::intent::CastId>,
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
                format!("rejected({};reason={reason:?})", format_battery_loop_step(attempted))
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
