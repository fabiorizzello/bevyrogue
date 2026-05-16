use std::collections::HashMap;
use std::convert::TryFrom;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::combat::api::intent::CastId;
use crate::combat::api::SignalPayload;
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::{CombatKernelState, CombatKernelTransition};
use crate::combat::types::UnitId;
use super::signals::{
    OWNER, SIGNAL_APPLY_PREY_LOCK, SIGNAL_BUILD_EXPLOIT, SIGNAL_CONSUME_PREY_LOCK_PAYOFF,
    SIGNAL_ENTER_BERSERK, SIGNAL_TICK,
};

pub const DEFAULT_EXPLOIT_CAP: u8 = 3;
pub const DEFAULT_PREY_LOCK_DURATION: u8 = 2;
pub const DEFAULT_BERSERK_STRAIN_THRESHOLD: u16 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredatorLoopCapKind {
    Exploit,
    PreyLock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredatorLoopBlockedReason {
    CapReached { cap: PredatorLoopCapKind },
    MissingExploit,
    MissingPreyLock,
    ExpiredPreyLock,
    InvalidTarget,
    BerserkBlockedByStrain { current: u16, threshold: u16 },
    UnsupportedRequest,
    MalformedData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredatorLoopStep {
    BuildExploit { target: UnitId, amount: u16 },
    ApplyPreyLock { target: UnitId },
    ConsumePreyLockPayoff { target: UnitId },
    EnterBerserk,
    Tick,
    Expire { target: UnitId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredatorLoopSignal {
    BuildExploit,
    ApplyPreyLock,
    ConsumePreyLockPayoff,
    EnterBerserk,
    Tick,
    Expire,
    Rejected,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredatorLoopTransition {
    pub signal: PredatorLoopSignal,
    pub target: Option<UnitId>,
    pub amount: u16,
    pub attempted: Option<PredatorLoopStep>,
    pub reason: Option<PredatorLoopBlockedReason>,
}

impl PredatorLoopTransition {
    pub const fn build_exploit(target: UnitId, amount: u16) -> Self {
        Self {
            signal: PredatorLoopSignal::BuildExploit,
            target: Some(target),
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn apply_prey_lock(target: UnitId, duration: u16) -> Self {
        Self {
            signal: PredatorLoopSignal::ApplyPreyLock,
            target: Some(target),
            amount: duration,
            attempted: None,
            reason: None,
        }
    }

    pub const fn consume_prey_lock_payoff(target: UnitId, amount: u16) -> Self {
        Self {
            signal: PredatorLoopSignal::ConsumePreyLockPayoff,
            target: Some(target),
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn enter_berserk(strain_current: u16) -> Self {
        Self {
            signal: PredatorLoopSignal::EnterBerserk,
            target: None,
            amount: strain_current,
            attempted: None,
            reason: None,
        }
    }

    pub const fn tick() -> Self {
        Self {
            signal: PredatorLoopSignal::Tick,
            target: None,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn expire(target: UnitId) -> Self {
        Self {
            signal: PredatorLoopSignal::Expire,
            target: Some(target),
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn rejected(attempted: PredatorLoopStep, reason: PredatorLoopBlockedReason) -> Self {
        Self {
            signal: PredatorLoopSignal::Rejected,
            target: match attempted {
                PredatorLoopStep::BuildExploit { target, .. }
                | PredatorLoopStep::ApplyPreyLock { target }
                | PredatorLoopStep::ConsumePreyLockPayoff { target }
                | PredatorLoopStep::Expire { target } => Some(target),
                PredatorLoopStep::EnterBerserk | PredatorLoopStep::Tick => None,
            },
            amount: 0,
            attempted: Some(attempted),
            reason: Some(reason),
        }
    }

    pub const fn ignored(attempted: PredatorLoopStep) -> Self {
        Self {
            signal: PredatorLoopSignal::Ignored,
            target: match attempted {
                PredatorLoopStep::BuildExploit { target, .. }
                | PredatorLoopStep::ApplyPreyLock { target }
                | PredatorLoopStep::ConsumePreyLockPayoff { target }
                | PredatorLoopStep::Expire { target } => Some(target),
                PredatorLoopStep::EnterBerserk | PredatorLoopStep::Tick => None,
            },
            amount: 0,
            attempted: Some(attempted),
            reason: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredatorLoopDesignTag {
    Exploit,
    PreyLock,
    Berserk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredatorLockState {
    pub turns_left: u8,
    pub consumed: bool,
}

impl PredatorLockState {
    pub const fn active(turns_left: u8) -> Self {
        Self {
            turns_left,
            consumed: false,
        }
    }

    pub const fn consumed() -> Self {
        Self {
            turns_left: 0,
            consumed: true,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.consumed || self.turns_left == 0 {
            return false;
        }

        self.turns_left -= 1;
        self.turns_left == 0
    }

    pub fn is_active(&self) -> bool {
        !self.consumed && self.turns_left > 0
    }

    pub fn is_expired(&self) -> bool {
        !self.consumed && self.turns_left == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredatorTargetState {
    pub exploit_stacks: u8,
    pub prey_lock: Option<PredatorLockState>,
}

impl PredatorTargetState {
    pub const fn new() -> Self {
        Self {
            exploit_stacks: 0,
            prey_lock: None,
        }
    }
}

impl Default for PredatorTargetState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
pub struct PredatorLoopState {
    pub exploit_cap: u8,
    pub prey_lock_duration: u8,
    pub berserk_strain_threshold: u16,
    pub targets: HashMap<UnitId, PredatorTargetState>,
    pub last_transition: Option<PredatorLoopTransition>,
    pub last_blocked_reason: Option<PredatorLoopBlockedReason>,
}

impl Default for PredatorLoopState {
    fn default() -> Self {
        Self {
            exploit_cap: DEFAULT_EXPLOIT_CAP,
            prey_lock_duration: DEFAULT_PREY_LOCK_DURATION,
            berserk_strain_threshold: DEFAULT_BERSERK_STRAIN_THRESHOLD,
            targets: HashMap::new(),
            last_transition: None,
            last_blocked_reason: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredatorTargetSnapshot {
    pub unit_id: UnitId,
    pub exploit_stacks: u8,
    pub prey_lock: Option<PredatorLockState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredatorLoopSnapshot {
    pub exploit_cap: u8,
    pub prey_lock_duration: u8,
    pub berserk_strain_threshold: u16,
    pub targets: Vec<PredatorTargetSnapshot>,
    pub last_transition: Option<PredatorLoopTransition>,
    pub last_blocked_reason: Option<PredatorLoopBlockedReason>,
}

impl From<&PredatorLoopState> for PredatorLoopSnapshot {
    fn from(state: &PredatorLoopState) -> Self {
        let mut targets = state
            .targets
            .iter()
            .map(|(unit_id, target)| PredatorTargetSnapshot {
                unit_id: *unit_id,
                exploit_stacks: target.exploit_stacks,
                prey_lock: target.prey_lock,
            })
            .collect::<Vec<_>>();
        targets.sort_by(|a, b| a.unit_id.0.cmp(&b.unit_id.0));

        Self {
            exploit_cap: state.exploit_cap,
            prey_lock_duration: state.prey_lock_duration,
            berserk_strain_threshold: state.berserk_strain_threshold,
            targets,
            last_transition: state.last_transition,
            last_blocked_reason: state.last_blocked_reason,
        }
    }
}

impl PredatorLoopState {
    pub fn snapshot(&self) -> PredatorLoopSnapshot {
        PredatorLoopSnapshot::from(self)
    }

    pub fn track_target(&mut self, unit_id: UnitId) -> &mut PredatorTargetState {
        self.targets.entry(unit_id).or_default()
    }

    pub fn build_exploit(&mut self, unit_id: UnitId, amount: u16) -> PredatorLoopTransition {
        apply_predator_loop_transition(self, PredatorLoopTransition::build_exploit(unit_id, amount))
    }

    pub fn apply_prey_lock(&mut self, unit_id: UnitId) -> PredatorLoopTransition {
        let duration = self.prey_lock_duration as u16;
        apply_predator_loop_transition(
            self,
            PredatorLoopTransition::apply_prey_lock(unit_id, duration),
        )
    }

    pub fn consume_prey_lock_payoff(&mut self, unit_id: UnitId) -> PredatorLoopTransition {
        apply_predator_loop_transition(
            self,
            PredatorLoopTransition::consume_prey_lock_payoff(unit_id, 1),
        )
    }

    pub fn enter_berserk(&mut self, strain_current: u16) -> PredatorLoopTransition {
        apply_predator_loop_transition(self, PredatorLoopTransition::enter_berserk(strain_current))
    }

    pub fn tick_all_prey_locks(&mut self) -> PredatorLoopTransition {
        apply_predator_loop_transition(self, PredatorLoopTransition::tick())
    }

    pub fn expire_prey_lock(&mut self, unit_id: UnitId) -> PredatorLoopTransition {
        apply_predator_loop_transition(self, PredatorLoopTransition::expire(unit_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredatorLoopRequestKind {
    BuildExploit { unit_id: UnitId, amount: u16 },
    ApplyPreyLock { unit_id: UnitId },
    ConsumePreyLockPayoff { unit_id: UnitId },
    EnterBerserk,
    Tick,
    Expire { unit_id: UnitId },
}

pub fn apply_predator_loop_transition(
    state: &mut PredatorLoopState,
    transition: PredatorLoopTransition,
) -> PredatorLoopTransition {
    let applied = match transition.signal {
        PredatorLoopSignal::BuildExploit => {
            apply_build_exploit(state, transition.target, transition.amount)
        }
        PredatorLoopSignal::ApplyPreyLock => apply_prey_lock(state, transition.target),
        PredatorLoopSignal::ConsumePreyLockPayoff => {
            apply_prey_lock_payoff(state, transition.target)
        }
        PredatorLoopSignal::EnterBerserk => apply_enter_berserk(state, transition.amount),
        PredatorLoopSignal::Tick => {
            tick_all_prey_locks(state);
            transition
        }
        PredatorLoopSignal::Expire => apply_expire_prey_lock(state, transition.target),
        PredatorLoopSignal::Rejected | PredatorLoopSignal::Ignored => transition,
    };

    record_applied_transition(state, applied)
}

fn decode_predator_loop_transition(
    transition: &CombatKernelTransition,
    target: UnitId,
    strain_current: u16,
) -> Option<PredatorLoopTransition> {
    let CombatKernelTransition::Blueprint { owner, name, payload } = transition else {
        return None;
    };

    if owner != OWNER {
        return None;
    }

    let SignalPayload::Amount(amount) = payload else {
        return None;
    };

    let amount = u16::try_from(*amount).ok()?;

    match name.as_str() {
        SIGNAL_BUILD_EXPLOIT => Some(PredatorLoopTransition::build_exploit(target, amount)),
        SIGNAL_APPLY_PREY_LOCK => Some(PredatorLoopTransition::apply_prey_lock(target, amount)),
        SIGNAL_CONSUME_PREY_LOCK_PAYOFF => Some(
            PredatorLoopTransition::consume_prey_lock_payoff(target, amount),
        ),
        SIGNAL_ENTER_BERSERK => Some(PredatorLoopTransition::enter_berserk(strain_current)),
        SIGNAL_TICK => Some(PredatorLoopTransition::tick()),
        _ => None,
    }
}

pub fn apply_predator_loop_transitions_system(
    mut messages: ParamSet<(MessageReader<CombatEvent>, MessageWriter<CombatEvent>)>,
    kernel: Res<CombatKernelState>,
    mut state: ResMut<PredatorLoopState>,
) {
    let events = messages
        .p0()
        .read()
        .filter_map(|event| {
            let CombatEventKind::OnKernelTransition { transition } = &event.kind else {
                return None;
            };

            let predator_transition = decode_predator_loop_transition(
                transition,
                event.target,
                kernel.strain.current,
            )?;

            Some((
                predator_transition,
                event.source,
                event.target,
                event.follow_up_depth,
            ))
        })
        .collect::<Vec<_>>();

    for (predator_transition, source, target, follow_up_depth) in events {
        let applied = match predator_transition.signal {
            PredatorLoopSignal::EnterBerserk => apply_predator_loop_transition(
                &mut state,
                PredatorLoopTransition::enter_berserk(kernel.strain.current),
            ),
            _ => apply_predator_loop_transition(&mut state, predator_transition),
        };

        messages.p1().write(CombatEvent {
            kind: CombatEventKind::PredatorLoopResolved {
                transition: applied,
            },
            source,
            target,
            follow_up_depth,
            cast_id: CastId::ROOT,
        });

        debug!("PredatorLoopResolved {:?}", applied);
    }
}

pub struct PredatorLoopHook;

fn apply_build_exploit(
    state: &mut PredatorLoopState,
    target: Option<UnitId>,
    amount: u16,
) -> PredatorLoopTransition {
    let Some(target) = target else {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::BuildExploit {
                target: UnitId(0),
                amount: amount as u16,
            },
            PredatorLoopBlockedReason::MalformedData,
        );
    };

    if amount == 0 {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::BuildExploit {
                target,
                amount: amount as u16,
            },
            PredatorLoopBlockedReason::MalformedData,
        );
    }

    let Some(target_state) = state.targets.get_mut(&target) else {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::BuildExploit {
                target,
                amount: amount as u16,
            },
            PredatorLoopBlockedReason::InvalidTarget,
        );
    };

    if target_state.exploit_stacks >= state.exploit_cap
        || amount > state.exploit_cap as u16
        || target_state.exploit_stacks as u16 + amount > state.exploit_cap as u16
    {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::BuildExploit {
                target,
                amount: amount as u16,
            },
            PredatorLoopBlockedReason::CapReached {
                cap: PredatorLoopCapKind::Exploit,
            },
        );
    }

    target_state.exploit_stacks = target_state.exploit_stacks.saturating_add(amount as u8);
    PredatorLoopTransition::build_exploit(target, amount)
}

fn apply_prey_lock(
    state: &mut PredatorLoopState,
    target: Option<UnitId>,
) -> PredatorLoopTransition {
    let Some(target) = target else {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::ApplyPreyLock { target: UnitId(0) },
            PredatorLoopBlockedReason::MalformedData,
        );
    };

    let Some(target_state) = state.targets.get_mut(&target) else {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::ApplyPreyLock { target },
            PredatorLoopBlockedReason::InvalidTarget,
        );
    };

    if target_state.exploit_stacks == 0 {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::ApplyPreyLock { target },
            PredatorLoopBlockedReason::MissingExploit,
        );
    }

    if target_state.prey_lock.is_some_and(|lock| lock.is_active()) {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::ApplyPreyLock { target },
            PredatorLoopBlockedReason::CapReached {
                cap: PredatorLoopCapKind::PreyLock,
            },
        );
    }

    target_state.prey_lock = Some(PredatorLockState::active(state.prey_lock_duration));
    PredatorLoopTransition::apply_prey_lock(target, state.prey_lock_duration as u16)
}

fn apply_prey_lock_payoff(
    state: &mut PredatorLoopState,
    target: Option<UnitId>,
) -> PredatorLoopTransition {
    let Some(target) = target else {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::ConsumePreyLockPayoff { target: UnitId(0) },
            PredatorLoopBlockedReason::MalformedData,
        );
    };

    let Some(target_state) = state.targets.get_mut(&target) else {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::ConsumePreyLockPayoff { target },
            PredatorLoopBlockedReason::InvalidTarget,
        );
    };

    let Some(lock) = target_state.prey_lock.as_mut() else {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::ConsumePreyLockPayoff { target },
            PredatorLoopBlockedReason::MissingPreyLock,
        );
    };

    if lock.consumed || lock.turns_left == 0 {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::ConsumePreyLockPayoff { target },
            PredatorLoopBlockedReason::ExpiredPreyLock,
        );
    }

    lock.consumed = true;
    lock.turns_left = 0;
    target_state.exploit_stacks = target_state.exploit_stacks.saturating_sub(1);
    PredatorLoopTransition::consume_prey_lock_payoff(target, 1)
}

fn apply_enter_berserk(
    state: &mut PredatorLoopState,
    strain_current: u16,
) -> PredatorLoopTransition {
    if strain_current >= state.berserk_strain_threshold {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::EnterBerserk,
            PredatorLoopBlockedReason::BerserkBlockedByStrain {
                current: strain_current,
                threshold: state.berserk_strain_threshold,
            },
        );
    }

    PredatorLoopTransition::enter_berserk(strain_current)
}

fn tick_all_prey_locks(state: &mut PredatorLoopState) {
    for target_state in state.targets.values_mut() {
        if let Some(lock) = target_state.prey_lock.as_mut() {
            let _expired = lock.tick();
        }
    }
}

fn apply_expire_prey_lock(
    state: &mut PredatorLoopState,
    target: Option<UnitId>,
) -> PredatorLoopTransition {
    let Some(target) = target else {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::Expire { target: UnitId(0) },
            PredatorLoopBlockedReason::MalformedData,
        );
    };

    let Some(target_state) = state.targets.get_mut(&target) else {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::Expire { target },
            PredatorLoopBlockedReason::InvalidTarget,
        );
    };

    let Some(lock) = target_state.prey_lock else {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::Expire { target },
            PredatorLoopBlockedReason::MissingPreyLock,
        );
    };

    if !lock.is_expired() && !lock.consumed {
        return PredatorLoopTransition::rejected(
            PredatorLoopStep::Expire { target },
            PredatorLoopBlockedReason::UnsupportedRequest,
        );
    }

    target_state.prey_lock = None;
    PredatorLoopTransition::expire(target)
}

fn record_applied_transition(
    state: &mut PredatorLoopState,
    transition: PredatorLoopTransition,
) -> PredatorLoopTransition {
    if matches!(transition.signal, PredatorLoopSignal::Rejected) {
        state.last_blocked_reason = transition.reason;
    } else {
        state.last_blocked_reason = None;
    }

    state.last_transition = Some(transition);
    debug!(
        "PredatorLoopState exploit_cap={} prey_lock_duration={} berserk_threshold={} targets={} last={:?} blocked={:?}",
        state.exploit_cap,
        state.prey_lock_duration,
        state.berserk_strain_threshold,
        state.targets.len(),
        state.last_transition,
        state.last_blocked_reason,
    );
    transition
}
