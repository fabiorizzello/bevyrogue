use crate::combat::bevy_types::*;

use crate::combat::{
    events::{CombatEvent, CombatEventKind},
    kernel::CombatKernelTransition,
};

use super::OWNER;
use super::identity::{
    PrecisionCommitment, PrecisionMindGamePhase, PrecisionMindGameRejectReason,
    PrecisionMindGameState, PrecisionMindGameStep, PrecisionMindGameTransition, PrecisionOutcome,
    PrecisionReveal, PrecisionWindowKind,
};
use super::{
    SIGNAL_COMMIT_PRECISION_PRESS, SIGNAL_OPEN_MOMENTUM_WINDOW, SIGNAL_RESOLVE_PRECISION_SUCCESS,
    SIGNAL_REVEAL_BAIT,
};

pub fn apply_precision_mind_game_transitions_system(
    mut events: MessageReader<CombatEvent>,
    mut state: ResMut<PrecisionMindGameState>,
) {
    for event in events.read() {
        if let CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint { owner, name, .. },
        } = &event.kind
        {
            if owner != OWNER {
                continue;
            }

            let step = match name.as_str() {
                SIGNAL_OPEN_MOMENTUM_WINDOW => Some(PrecisionMindGameTransition::open_window(
                    PrecisionWindowKind::Momentum,
                )),
                SIGNAL_COMMIT_PRECISION_PRESS => Some(PrecisionMindGameTransition::commit(
                    PrecisionCommitment::Press,
                )),
                SIGNAL_REVEAL_BAIT => {
                    Some(PrecisionMindGameTransition::reveal(PrecisionReveal::Baited))
                }
                SIGNAL_RESOLVE_PRECISION_SUCCESS => Some(PrecisionMindGameTransition::resolve(
                    PrecisionOutcome::Success,
                )),
                _ => None,
            };

            if let Some(transition) = step {
                apply_precision_mind_game_transition(&mut state, transition);
            }
        }
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
