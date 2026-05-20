use crate::combat::bevy_types::*;
use serde::{Deserialize, Serialize};

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::{
    CombatKernelHook, CombatKernelHookDomain, CombatKernelTransition, CombatTagChangeKind,
    CombatTagId, CombatTagState, CombatTagTransition, TacticalCycleTransition,
};
use crate::combat::runtime::SignalPayload;
use crate::combat::types::UnitId;

use super::{
    SIGNAL_BUILD_HOLY_SUPPORT_GRACE, SIGNAL_CONSUME_MARTYR_LIGHT, SIGNAL_CYCLE_RESET,
    SIGNAL_MARK_MARTYR_LIGHT, SIGNAL_SPEND_HOLY_SUPPORT_GRACE,
};

pub const GRACE_CAP: u8 = 3;

pub const TAG_GRACE: &str = "Grace";
pub const TAG_MARTYR_LIGHT: &str = "Martyr Light";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolySupportSignal {
    BuildGrace,
    SpendGrace,
    MarkMartyrLight,
    ConsumeMartyrLight,
    CycleReset,
    Rejected,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolySupportStep {
    BuildGrace { amount: u8 },
    SpendGrace { amount: u8 },
    MarkMartyrLight,
    ConsumeMartyrLight,
    CycleReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolySupportRejectReason {
    GraceUnderflow,
    MartyrAlreadyMarked,
    MartyrNotMarked,
    MartyrAlreadyConsumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolySupportTransition {
    pub signal: HolySupportSignal,
    pub amount: u8,
    pub attempted: Option<HolySupportStep>,
    pub reason: Option<HolySupportRejectReason>,
}

impl HolySupportTransition {
    pub const fn build_grace(amount: u8) -> Self {
        Self {
            signal: HolySupportSignal::BuildGrace,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn spend_grace(amount: u8) -> Self {
        Self {
            signal: HolySupportSignal::SpendGrace,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn mark_martyr_light() -> Self {
        Self {
            signal: HolySupportSignal::MarkMartyrLight,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn consume_martyr_light() -> Self {
        Self {
            signal: HolySupportSignal::ConsumeMartyrLight,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn cycle_reset() -> Self {
        Self {
            signal: HolySupportSignal::CycleReset,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn rejected(attempted: HolySupportStep, reason: HolySupportRejectReason) -> Self {
        Self {
            signal: HolySupportSignal::Rejected,
            amount: 0,
            attempted: Some(attempted),
            reason: Some(reason),
        }
    }

    // Constructor not consumed; kept for API symmetry with rejected().
    pub const fn ignored(attempted: HolySupportStep) -> Self {
        Self {
            signal: HolySupportSignal::Ignored,
            amount: 0,
            attempted: Some(attempted),
            reason: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolySupportDesignTag {
    Grace,
    MartyrLight,
}

// Called from holy_support_design_tag which is consumed by tests/holy_support_mechanics.rs.
pub fn holy_support_design_tag_name(tag: HolySupportDesignTag) -> &'static str {
    match tag {
        HolySupportDesignTag::Grace => TAG_GRACE,
        HolySupportDesignTag::MartyrLight => TAG_MARTYR_LIGHT,
    }
}

pub fn holy_support_design_tag(tag: HolySupportDesignTag) -> CombatTagId {
    CombatTagId(holy_support_design_tag_name(tag).to_string())
}

pub fn classify_holy_support_tag(tag: &CombatTagId) -> Option<HolySupportDesignTag> {
    match tag.0.as_str() {
        TAG_GRACE => Some(HolySupportDesignTag::Grace),
        TAG_MARTYR_LIGHT => Some(HolySupportDesignTag::MartyrLight),
        _ => None,
    }
}

fn blueprint_transition(name: &str, payload: SignalPayload) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: super::signals::OWNER.to_owned(),
        name: name.to_owned(),
        payload,
    }
}

// Public API re-exported from patamon/mod.rs; not yet consumed by tests.
pub fn holy_support_added_tag_transition(
    tag: HolySupportDesignTag,
    turns_left: u8,
) -> CombatKernelTransition {
    let id = holy_support_design_tag(tag);
    let before = CombatTagState::new(id.clone(), turns_left);
    let after = CombatTagState::new(id, turns_left);
    CombatKernelTransition::Tag(CombatTagTransition {
        before,
        after,
        kind: CombatTagChangeKind::Added,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
pub struct HolySupportState {
    pub grace: u8,
    pub martyr_light_marked_this_cycle: bool,
    pub martyr_light_consumed_this_cycle: bool,
    pub last_signal: Option<HolySupportTransition>,
}

impl Default for HolySupportState {
    fn default() -> Self {
        Self {
            grace: 0,
            martyr_light_marked_this_cycle: false,
            martyr_light_consumed_this_cycle: false,
            last_signal: None,
        }
    }
}

// Public snapshot type; not yet consumed by tests or binary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolySupportSnapshot {
    pub grace: u8,
    pub grace_cap: u8,
    pub martyr_light_marked_this_cycle: bool,
    pub martyr_light_consumed_this_cycle: bool,
    pub last_signal: Option<HolySupportTransition>,
}

impl From<&HolySupportState> for HolySupportSnapshot {
    fn from(state: &HolySupportState) -> Self {
        Self {
            grace: state.grace,
            grace_cap: GRACE_CAP,
            martyr_light_marked_this_cycle: state.martyr_light_marked_this_cycle,
            martyr_light_consumed_this_cycle: state.martyr_light_consumed_this_cycle,
            last_signal: state.last_signal,
        }
    }
}

impl HolySupportState {
    // snapshot() not yet consumed; kept as public API surface.
    pub fn snapshot(&self) -> HolySupportSnapshot {
        HolySupportSnapshot::from(self)
    }
}

pub struct HolySupportHook;

impl CombatKernelHook for HolySupportHook {
    fn domain(&self) -> CombatKernelHookDomain {
        CombatKernelHookDomain::Shared
    }

    fn on_transition(
        &self,
        transition: &CombatKernelTransition,
        out: &mut Vec<CombatKernelTransition>,
    ) {
        match transition {
            CombatKernelTransition::Tag(tag_transition)
                if tag_transition.kind == CombatTagChangeKind::Added =>
            {
                if let Some(tag) = classify_holy_support_tag(&tag_transition.after.id) {
                    match tag {
                        HolySupportDesignTag::Grace => out.push(blueprint_transition(
                            SIGNAL_BUILD_HOLY_SUPPORT_GRACE,
                            SignalPayload::Amount(1),
                        )),
                        HolySupportDesignTag::MartyrLight => out.push(blueprint_transition(
                            SIGNAL_MARK_MARTYR_LIGHT,
                            SignalPayload::Empty,
                        )),
                    }
                }
            }
            CombatKernelTransition::Tag(tag_transition)
                if tag_transition.kind == CombatTagChangeKind::Consumed
                    || tag_transition.kind == CombatTagChangeKind::Expired =>
            {
                if let Some(tag) = classify_holy_support_tag(&tag_transition.after.id) {
                    match tag {
                        HolySupportDesignTag::Grace => out.push(blueprint_transition(
                            SIGNAL_SPEND_HOLY_SUPPORT_GRACE,
                            SignalPayload::Amount(1),
                        )),
                        HolySupportDesignTag::MartyrLight => out.push(blueprint_transition(
                            SIGNAL_CONSUME_MARTYR_LIGHT,
                            SignalPayload::Empty,
                        )),
                    }
                }
            }
            CombatKernelTransition::TacticalCycle(TacticalCycleTransition {
                wrapped_cycle: true,
                ..
            }) => {
                out.push(blueprint_transition(
                    SIGNAL_CYCLE_RESET,
                    SignalPayload::Empty,
                ));
            }
            _ => {}
        }
    }
}

pub fn apply_holy_support_transitions_system(
    mut events: MessageReader<CombatEvent>,
    mut state: ResMut<HolySupportState>,
) {
    for event in events.read() {
        let CombatEventKind::OnKernelTransition { transition } = &event.kind else {
            continue;
        };

        match transition {
            CombatKernelTransition::Blueprint {
                owner,
                name,
                payload,
            } if owner == super::signals::OWNER => {
                let Some(holy_transition) = decode_holy_support_blueprint_transition(name, payload)
                else {
                    continue;
                };

                apply_holy_support_transition(&mut state, holy_transition, event.target);
            }
            _ => continue,
        }
    }
}

fn decode_holy_support_blueprint_transition(
    name: &str,
    payload: &SignalPayload,
) -> Option<HolySupportTransition> {
    match name {
        SIGNAL_BUILD_HOLY_SUPPORT_GRACE => match payload {
            SignalPayload::Amount(amount) => u8::try_from(*amount)
                .ok()
                .filter(|amount| *amount > 0)
                .map(HolySupportTransition::build_grace),
            _ => None,
        },
        SIGNAL_SPEND_HOLY_SUPPORT_GRACE => match payload {
            SignalPayload::Amount(amount) => u8::try_from(*amount)
                .ok()
                .filter(|amount| *amount > 0)
                .map(HolySupportTransition::spend_grace),
            _ => None,
        },
        SIGNAL_MARK_MARTYR_LIGHT => match payload {
            SignalPayload::Empty => Some(HolySupportTransition::mark_martyr_light()),
            _ => None,
        },
        SIGNAL_CONSUME_MARTYR_LIGHT => match payload {
            SignalPayload::Empty => Some(HolySupportTransition::consume_martyr_light()),
            _ => None,
        },
        SIGNAL_CYCLE_RESET => match payload {
            SignalPayload::Empty => Some(HolySupportTransition::cycle_reset()),
            _ => None,
        },
        _ => None,
    }
}

fn apply_holy_support_transition(
    state: &mut HolySupportState,
    transition: HolySupportTransition,
    target: UnitId,
) {
    match transition.signal {
        HolySupportSignal::BuildGrace => {
            state.grace = state.grace.saturating_add(transition.amount).min(GRACE_CAP);
            state.last_signal = Some(transition);
        }
        HolySupportSignal::SpendGrace => {
            if transition.amount > state.grace {
                state.last_signal = Some(HolySupportTransition::rejected(
                    HolySupportStep::SpendGrace {
                        amount: transition.amount,
                    },
                    HolySupportRejectReason::GraceUnderflow,
                ));
            } else {
                state.grace = state.grace.saturating_sub(transition.amount);
                state.last_signal = Some(transition);
            }
        }
        HolySupportSignal::MarkMartyrLight => {
            if state.martyr_light_marked_this_cycle {
                state.last_signal = Some(HolySupportTransition::rejected(
                    HolySupportStep::MarkMartyrLight,
                    HolySupportRejectReason::MartyrAlreadyMarked,
                ));
            } else {
                state.martyr_light_marked_this_cycle = true;
                state.martyr_light_consumed_this_cycle = false;
                state.last_signal = Some(transition);
            }
        }
        HolySupportSignal::ConsumeMartyrLight => {
            if !state.martyr_light_marked_this_cycle {
                state.last_signal = Some(HolySupportTransition::rejected(
                    HolySupportStep::ConsumeMartyrLight,
                    HolySupportRejectReason::MartyrNotMarked,
                ));
            } else if state.martyr_light_consumed_this_cycle {
                state.last_signal = Some(HolySupportTransition::rejected(
                    HolySupportStep::ConsumeMartyrLight,
                    HolySupportRejectReason::MartyrAlreadyConsumed,
                ));
            } else {
                state.martyr_light_consumed_this_cycle = true;
                state.last_signal = Some(transition);
            }
        }
        HolySupportSignal::CycleReset => {
            state.martyr_light_marked_this_cycle = false;
            state.martyr_light_consumed_this_cycle = false;
            state.last_signal = Some(transition);
        }
        HolySupportSignal::Rejected | HolySupportSignal::Ignored => {
            state.last_signal = Some(transition);
        }
    }

    debug!(
        "HolySupportState target={:?} grace={} marked={} consumed={} last={:?}",
        target,
        state.grace,
        state.martyr_light_marked_this_cycle,
        state.martyr_light_consumed_this_cycle,
        state.last_signal,
    );
}

pub(crate) fn format_holy_support_transition(transition: HolySupportTransition) -> String {
    let signal = match transition.signal {
        HolySupportSignal::BuildGrace => "build",
        HolySupportSignal::SpendGrace => "spend",
        HolySupportSignal::MarkMartyrLight => "mark-martyr",
        HolySupportSignal::ConsumeMartyrLight => "consume-martyr",
        HolySupportSignal::CycleReset => "cycle-reset",
        HolySupportSignal::Rejected => "rejected",
        HolySupportSignal::Ignored => "ignored",
    };

    match transition.signal {
        HolySupportSignal::BuildGrace | HolySupportSignal::SpendGrace => {
            format!("{signal}({})", transition.amount)
        }
        HolySupportSignal::Rejected | HolySupportSignal::Ignored => {
            match (transition.attempted, transition.reason) {
                (Some(attempted), Some(reason)) => {
                    format!(
                        "{signal}({};reason={reason:?})",
                        format_holy_support_step(attempted)
                    )
                }
                (Some(attempted), None) => {
                    format!("{signal}({})", format_holy_support_step(attempted))
                }
                _ => signal.to_string(),
            }
        }
        _ => signal.to_string(),
    }
}

fn format_holy_support_step(step: HolySupportStep) -> String {
    match step {
        HolySupportStep::BuildGrace { amount } => format!("build({amount})"),
        HolySupportStep::SpendGrace { amount } => format!("spend({amount})"),
        HolySupportStep::MarkMartyrLight => "mark-martyr".to_string(),
        HolySupportStep::ConsumeMartyrLight => "consume-martyr".to_string(),
        HolySupportStep::CycleReset => "cycle-reset".to_string(),
    }
}
