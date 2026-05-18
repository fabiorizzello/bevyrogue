use std::convert::TryFrom;

use crate::combat::bevy_types::*;

use super::signals::{
    OWNER, SIGNAL_APPLY_PREY_LOCK, SIGNAL_BUILD_EXPLOIT, SIGNAL_CONSUME_PREY_LOCK_PAYOFF,
    SIGNAL_ENTER_BERSERK, SIGNAL_TICK,
};
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::{CombatKernelState, CombatKernelTransition};
use crate::combat::runtime::SignalPayload;
use crate::combat::types::UnitId;

use super::identity::{
    PredatorLockState, PredatorLoopBlockedReason, PredatorLoopCapKind, PredatorLoopSignal,
    PredatorLoopState, PredatorLoopStep, PredatorLoopTransition,
};

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
    let CombatKernelTransition::Blueprint {
        owner,
        name,
        payload,
    } = transition
    else {
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
        SIGNAL_CONSUME_PREY_LOCK_PAYOFF => Some(PredatorLoopTransition::consume_prey_lock_payoff(
            target, amount,
        )),
        SIGNAL_ENTER_BERSERK => Some(PredatorLoopTransition::enter_berserk(strain_current)),
        SIGNAL_TICK => Some(PredatorLoopTransition::tick()),
        _ => None,
    }
}

pub fn apply_predator_loop_transitions_system(
    mut events: MessageReader<CombatEvent>,
    kernel: Res<CombatKernelState>,
    mut state: ResMut<PredatorLoopState>,
) {
    let events = events
        .read()
        .filter_map(|event| {
            let CombatEventKind::OnKernelTransition { transition } = &event.kind else {
                return None;
            };

            let predator_transition =
                decode_predator_loop_transition(transition, event.target, kernel.strain.current)?;

            Some(predator_transition)
        })
        .collect::<Vec<_>>();

    for predator_transition in events {
        let applied = match predator_transition.signal {
            PredatorLoopSignal::EnterBerserk => apply_predator_loop_transition(
                &mut state,
                PredatorLoopTransition::enter_berserk(kernel.strain.current),
            ),
            _ => apply_predator_loop_transition(&mut state, predator_transition),
        };

        debug!("PredatorLoop applied {:?}", applied);
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

pub(crate) fn format_predator_loop_transition(transition: PredatorLoopTransition) -> String {
    let signal = match transition.signal {
        PredatorLoopSignal::BuildExploit => "build-exploit",
        PredatorLoopSignal::ApplyPreyLock => "prey-lock",
        PredatorLoopSignal::ConsumePreyLockPayoff => "payoff",
        PredatorLoopSignal::EnterBerserk => "berserk",
        PredatorLoopSignal::Tick => "tick",
        PredatorLoopSignal::Expire => "expire",
        PredatorLoopSignal::Rejected => "rejected",
        PredatorLoopSignal::Ignored => "ignored",
    };
    match transition.signal {
        PredatorLoopSignal::BuildExploit
        | PredatorLoopSignal::ApplyPreyLock
        | PredatorLoopSignal::ConsumePreyLockPayoff
        | PredatorLoopSignal::EnterBerserk
        | PredatorLoopSignal::Expire => {
            format!(
                "{signal}(target={:?};amount={})",
                transition.target, transition.amount
            )
        }
        PredatorLoopSignal::Tick => signal.to_string(),
        PredatorLoopSignal::Rejected | PredatorLoopSignal::Ignored => {
            match (transition.attempted, transition.reason) {
                (Some(attempted), Some(reason)) => {
                    format!(
                        "{signal}({};reason={reason:?})",
                        format_predator_loop_step(attempted)
                    )
                }
                (Some(attempted), None) => {
                    format!("{signal}({})", format_predator_loop_step(attempted))
                }
                _ => signal.to_string(),
            }
        }
    }
}

fn format_predator_loop_step(step: PredatorLoopStep) -> String {
    match step {
        PredatorLoopStep::BuildExploit { target, amount } => {
            format!("build({}:{})", target.0, amount)
        }
        PredatorLoopStep::ApplyPreyLock { target } => format!("prey-lock({})", target.0),
        PredatorLoopStep::ConsumePreyLockPayoff { target } => format!("payoff({})", target.0),
        PredatorLoopStep::EnterBerserk => "berserk".to_string(),
        PredatorLoopStep::Tick => "tick".to_string(),
        PredatorLoopStep::Expire { target } => format!("expire({})", target.0),
    }
}
