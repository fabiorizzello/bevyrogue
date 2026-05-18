use crate::combat::bevy_types::*;
use serde::{Deserialize, Serialize};

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

    // Constructor not consumed; kept for API symmetry with rejected().
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

// Snapshot type; not yet consumed by integration tests.
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
    // Consumed by tests/renamon_precision_runtime.rs and tests/validation_snapshot.rs.
    pub fn is_window_open(&self) -> bool {
        self.phase == PrecisionMindGamePhase::WindowOpen
    }

    // Public snapshot method; not yet consumed by tests.
    pub fn snapshot(&self) -> PrecisionMindGameSnapshot {
        PrecisionMindGameSnapshot::from(self)
    }
}
