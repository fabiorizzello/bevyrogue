use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::kernel::{CombatKernelHook, CombatKernelHookDomain, CombatKernelTransition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionWindowKind {
    Momentum,
    Counterplay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionCommitment {
    Press,
    Hold,
    Feint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionReveal {
    Guarded,
    Baited,
    Trapped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionOutcome {
    Success,
    Countered,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionMindGamePhase {
    Dormant,
    WindowOpen,
    CommitmentLocked,
    CounterplayRevealed,
    Resolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionMindGameRejectReason {
    NoOpenWindow,
    WindowAlreadyOpen,
    DuplicateCommitment,
    MissingCommitment,
    DuplicateReveal,
    MissingReveal,
    AlreadyResolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionMindGameStep {
    OpenWindow { window: PrecisionWindowKind },
    Commit { commitment: PrecisionCommitment },
    Reveal { reveal: PrecisionReveal },
    Resolve { outcome: PrecisionOutcome },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionMindGameTransition {
    OpenWindow {
        window: PrecisionWindowKind,
    },
    Commit {
        commitment: PrecisionCommitment,
    },
    Reveal {
        reveal: PrecisionReveal,
    },
    Resolve {
        outcome: PrecisionOutcome,
    },
    Rejected {
        attempted: PrecisionMindGameStep,
        reason: PrecisionMindGameRejectReason,
    },
    Ignored {
        attempted: PrecisionMindGameStep,
    },
}

impl PrecisionMindGameTransition {
    pub const fn open_window(window: PrecisionWindowKind) -> Self {
        Self::OpenWindow { window }
    }

    pub const fn commit(commitment: PrecisionCommitment) -> Self {
        Self::Commit { commitment }
    }

    pub const fn reveal(reveal: PrecisionReveal) -> Self {
        Self::Reveal { reveal }
    }

    pub const fn resolve(outcome: PrecisionOutcome) -> Self {
        Self::Resolve { outcome }
    }

    pub const fn rejected(
        attempted: PrecisionMindGameStep,
        reason: PrecisionMindGameRejectReason,
    ) -> Self {
        Self::Rejected { attempted, reason }
    }

    pub const fn ignored(attempted: PrecisionMindGameStep) -> Self {
        Self::Ignored { attempted }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
pub struct PrecisionMindGameState {
    pub phase: PrecisionMindGamePhase,
    pub window_index: u32,
    pub current_window: Option<PrecisionWindowKind>,
    pub commitment: Option<PrecisionCommitment>,
    pub reveal: Option<PrecisionReveal>,
    pub outcome: Option<PrecisionOutcome>,
    pub last_signal: Option<PrecisionMindGameTransition>,
}

impl Default for PrecisionMindGameState {
    fn default() -> Self {
        Self {
            phase: PrecisionMindGamePhase::Dormant,
            window_index: 0,
            current_window: None,
            commitment: None,
            reveal: None,
            outcome: None,
            last_signal: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecisionMindGameSnapshot {
    pub phase: PrecisionMindGamePhase,
    pub window_index: u32,
    pub current_window: Option<PrecisionWindowKind>,
    pub commitment: Option<PrecisionCommitment>,
    pub reveal: Option<PrecisionReveal>,
    pub outcome: Option<PrecisionOutcome>,
    pub last_signal: Option<PrecisionMindGameTransition>,
}

impl From<&PrecisionMindGameState> for PrecisionMindGameSnapshot {
    fn from(state: &PrecisionMindGameState) -> Self {
        Self {
            phase: state.phase,
            window_index: state.window_index,
            current_window: state.current_window,
            commitment: state.commitment,
            reveal: state.reveal,
            outcome: state.outcome,
            last_signal: state.last_signal,
        }
    }
}

impl PrecisionMindGameState {
    pub fn is_window_open(&self) -> bool {
        self.phase == PrecisionMindGamePhase::WindowOpen
    }

    pub fn snapshot(&self) -> PrecisionMindGameSnapshot {
        PrecisionMindGameSnapshot::from(self)
    }
}

pub struct PrecisionMindGameHook;

impl CombatKernelHook for PrecisionMindGameHook {
    fn domain(&self) -> CombatKernelHookDomain {
        CombatKernelHookDomain::Shared
    }

    fn on_transition(
        &self,
        _transition: &CombatKernelTransition,
        _out: &mut Vec<CombatKernelTransition>,
    ) {
    }
}

pub fn apply_precision_mind_game_transition(
    state: &mut PrecisionMindGameState,
    transition: PrecisionMindGameTransition,
) {
    let before = state.clone();
    let mut accepted = false;

    match transition {
        PrecisionMindGameTransition::OpenWindow { window } => {
            if matches!(
                state.phase,
                PrecisionMindGamePhase::Dormant | PrecisionMindGamePhase::Resolved
            ) {
                state.phase = PrecisionMindGamePhase::WindowOpen;
                state.window_index = state.window_index.saturating_add(1);
                state.current_window = Some(window);
                state.commitment = None;
                state.reveal = None;
                state.outcome = None;
                accepted = true;
            } else {
                state.last_signal = Some(PrecisionMindGameTransition::rejected(
                    PrecisionMindGameStep::OpenWindow { window },
                    PrecisionMindGameRejectReason::WindowAlreadyOpen,
                ));
            }
        }
        PrecisionMindGameTransition::Commit { commitment } => {
            if state.phase == PrecisionMindGamePhase::WindowOpen && state.commitment.is_none() {
                state.phase = PrecisionMindGamePhase::CommitmentLocked;
                state.commitment = Some(commitment);
                accepted = true;
            } else {
                let reason = if state.current_window.is_none() {
                    PrecisionMindGameRejectReason::NoOpenWindow
                } else if state.commitment.is_some() {
                    PrecisionMindGameRejectReason::DuplicateCommitment
                } else {
                    PrecisionMindGameRejectReason::NoOpenWindow
                };
                state.last_signal = Some(PrecisionMindGameTransition::rejected(
                    PrecisionMindGameStep::Commit { commitment },
                    reason,
                ));
            }
        }
        PrecisionMindGameTransition::Reveal { reveal } => {
            if state.phase == PrecisionMindGamePhase::CommitmentLocked && state.reveal.is_none() {
                state.phase = PrecisionMindGamePhase::CounterplayRevealed;
                state.reveal = Some(reveal);
                accepted = true;
            } else {
                let reason = if state.commitment.is_none() {
                    PrecisionMindGameRejectReason::MissingCommitment
                } else if state.reveal.is_some() {
                    PrecisionMindGameRejectReason::DuplicateReveal
                } else {
                    PrecisionMindGameRejectReason::NoOpenWindow
                };
                state.last_signal = Some(PrecisionMindGameTransition::rejected(
                    PrecisionMindGameStep::Reveal { reveal },
                    reason,
                ));
            }
        }
        PrecisionMindGameTransition::Resolve { outcome } => {
            if state.phase == PrecisionMindGamePhase::CounterplayRevealed && state.outcome.is_none()
            {
                state.phase = PrecisionMindGamePhase::Resolved;
                state.outcome = Some(outcome);
                accepted = true;
            } else {
                let reason = if state.reveal.is_none() {
                    PrecisionMindGameRejectReason::MissingReveal
                } else if matches!(state.phase, PrecisionMindGamePhase::Resolved) {
                    PrecisionMindGameRejectReason::AlreadyResolved
                } else {
                    PrecisionMindGameRejectReason::MissingReveal
                };
                state.last_signal = Some(PrecisionMindGameTransition::rejected(
                    PrecisionMindGameStep::Resolve { outcome },
                    reason,
                ));
            }
        }
        PrecisionMindGameTransition::Rejected { attempted, reason } => {
            state.last_signal = Some(PrecisionMindGameTransition::Rejected { attempted, reason });
        }
        PrecisionMindGameTransition::Ignored { attempted } => {
            state.last_signal = Some(PrecisionMindGameTransition::Ignored { attempted });
        }
    }

    if accepted {
        state.last_signal = Some(transition);
    }

    debug!(
        "PrecisionMindGameState before={:?} after={:?} last={:?}",
        before, state, state.last_signal
    );
}
