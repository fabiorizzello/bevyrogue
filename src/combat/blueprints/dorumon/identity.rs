use std::collections::HashMap;

use crate::combat::bevy_types::*;
use serde::{Deserialize, Serialize};

use crate::combat::types::UnitId;

// Re-export the apply/system/hook/format surface so external paths like
// `blueprints::dorumon::identity::format_predator_loop_transition` and
// `super::identity::{PredatorLoopHook, apply_predator_loop_transitions_system}`
// keep resolving after the split into `identity_apply.rs`.
pub(crate) use super::identity_apply::format_predator_loop_transition;
pub use super::identity_apply::{
    PredatorLoopHook, apply_predator_loop_transition, apply_predator_loop_transitions_system,
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

    // Constructor not yet consumed; kept for API symmetry with rejected().
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

    // Constructor for consumed state; kept for API completeness, not yet called.
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

    // Consumed by tests/predator_loop_kernel.rs.
    pub fn consume_prey_lock_payoff(&mut self, unit_id: UnitId) -> PredatorLoopTransition {
        apply_predator_loop_transition(
            self,
            PredatorLoopTransition::consume_prey_lock_payoff(unit_id, 1),
        )
    }

    // Consumed by tests/predator_loop_kernel.rs and tests/dorumon_blueprint.rs.
    pub fn enter_berserk(&mut self, strain_current: u16) -> PredatorLoopTransition {
        apply_predator_loop_transition(self, PredatorLoopTransition::enter_berserk(strain_current))
    }

    // Consumed by tests/predator_loop_kernel.rs.
    pub fn tick_all_prey_locks(&mut self) -> PredatorLoopTransition {
        apply_predator_loop_transition(self, PredatorLoopTransition::tick())
    }

    // Not yet consumed by tests; reserved for expiry handling.
    pub fn expire_prey_lock(&mut self, unit_id: UnitId) -> PredatorLoopTransition {
        apply_predator_loop_transition(self, PredatorLoopTransition::expire(unit_id))
    }
}
