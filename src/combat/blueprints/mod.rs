//! Plugin-oriented routing for declarative skill custom signals.
//!
//! Each registered blueprint owns its own signal names and payload interpretation.
//! The shared layer only routes by owner and signal envelope shape.

pub mod agumon;
pub mod dorumon;
pub mod gabumon;
pub mod patamon;
pub mod renamon;
pub mod tentomon;
pub mod twin_core;

use std::fmt;

use crate::combat::{kernel::CombatKernelTransition, state::ResolvedAction};
use crate::data::skills_ron::{CustomSignalPayload, SkillCustomSignal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomSignalDispatchError {
    UnknownOwner {
        owner: String,
    },
    UnknownSignal {
        owner: String,
        signal: String,
    },
    MalformedPayload {
        owner: String,
        signal: String,
        detail: String,
    },
}

impl fmt::Display for CustomSignalDispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOwner { owner } => {
                write!(f, "unknown blueprint owner '{owner}'")
            }
            Self::UnknownSignal { owner, signal } => {
                write!(f, "unknown signal '{signal}' for owner '{owner}'")
            }
            Self::MalformedPayload {
                owner,
                signal,
                detail,
            } => {
                write!(f, "malformed payload for {owner}::{signal}: {detail}")
            }
        }
    }
}

impl std::error::Error for CustomSignalDispatchError {}

pub(crate) fn amount_payload(
    signal: &SkillCustomSignal,
    owner: &'static str,
    signal_name: &'static str,
) -> Result<u16, CustomSignalDispatchError> {
    match signal.payload() {
        CustomSignalPayload::Amount { amount } => {
            u16::try_from(amount).map_err(|_| CustomSignalDispatchError::MalformedPayload {
                owner: owner.to_string(),
                signal: signal_name.to_string(),
                detail: format!("amount {amount} does not fit in u16"),
            })
        }
        CustomSignalPayload::Empty => Err(CustomSignalDispatchError::MalformedPayload {
            owner: owner.to_string(),
            signal: signal_name.to_string(),
            detail: "expected Amount payload".to_string(),
        }),
    }
}

// kept for: empty-payload custom signals (Tentomon Battery Loop M026 /
// Renamon Kitsune Grace M027 reactive passives); sibling amount_payload
// is already wired — keep dispatcher pattern symmetric.
#[allow(dead_code)]
pub(crate) fn no_payload(
    signal: &SkillCustomSignal,
    owner: &'static str,
    signal_name: &'static str,
) -> Result<(), CustomSignalDispatchError> {
    match signal.payload() {
        CustomSignalPayload::Empty => Ok(()),
        CustomSignalPayload::Amount { .. } => Err(CustomSignalDispatchError::MalformedPayload {
            owner: owner.to_string(),
            signal: signal_name.to_string(),
            detail: "expected empty payload".to_string(),
        }),
    }
}

type DispatchFn = fn(
    &SkillCustomSignal,
    &ResolvedAction,
) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError>;

#[derive(Clone, Copy)]
struct BlueprintRegistration {
    owner: &'static str,
    dispatch: DispatchFn,
}

const BLUEPRINTS: &[BlueprintRegistration] = &[
    BlueprintRegistration {
        owner: agumon::OWNER,
        dispatch: agumon::dispatch,
    },
    BlueprintRegistration {
        owner: gabumon::OWNER,
        dispatch: gabumon::dispatch,
    },
    BlueprintRegistration {
        owner: patamon::OWNER,
        dispatch: patamon::dispatch,
    },
    BlueprintRegistration {
        owner: tentomon::OWNER,
        dispatch: tentomon::dispatch,
    },
    BlueprintRegistration {
        owner: dorumon::OWNER,
        dispatch: dorumon::dispatch,
    },
    BlueprintRegistration {
        owner: renamon::OWNER,
        dispatch: renamon::dispatch,
    },
];

/// Register all blueprint extension points (hooks, predicates, selectors) into `regs`.
///
/// Call this alongside `register_kernel_builtins` whenever you compile the full
/// `SkillBook` timeline set, so that blueprint-specific references in timelines
/// (like `"agumon/has_bouncing_fire"`) resolve correctly.
pub fn register_all_blueprint_exts(regs: &mut crate::combat::api::ExtRegistries) {
    agumon::register_agumon_ext(regs);
    gabumon::register_gabumon_ext(regs);
    patamon::register_patamon_ext(regs);
    renamon::register_renamon_ext(regs);
    tentomon::register_tentomon_ext(regs);
    dorumon::register_dorumon_ext(regs);
}

pub fn register_all_blueprint_validation_exts(regs: &mut crate::combat::api::ExtRegistries) {
    agumon::register_validation_ext(regs);
    gabumon::register_validation_ext(regs);
    twin_core::register_validation_ext(regs);
    patamon::register_validation_ext(regs);
    dorumon::register_validation_ext(regs);
    tentomon::register_tentomon_ext(regs);
    renamon::register_renamon_ext(regs);
}

pub fn register_canonical_passive_runners(app: &mut crate::combat::bevy_types::App) {
    agumon::register_passive_runtime(app);
    gabumon::register_passive_runtime(app);
    patamon::register_passive_runtime(app);
    renamon::register_passive_runtime(app);
}

pub fn add_runtime_plugins(app: &mut crate::combat::bevy_types::App) {
    app.add_plugins((
        twin_core::TwinCorePlugin,
        patamon::PatamonPlugin,
        dorumon::DorumonPlugin,
        tentomon::TentomonPlugin,
        renamon::RenamonPlugin,
    ));
}

pub fn dispatch_custom_signal(
    signal: &SkillCustomSignal,
    action: &ResolvedAction,
) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> {
    let Some(blueprint) = BLUEPRINTS
        .iter()
        .find(|blueprint| blueprint.owner == signal.owner())
    else {
        return Err(CustomSignalDispatchError::UnknownOwner {
            owner: signal.owner().to_string(),
        });
    };

    (blueprint.dispatch)(signal, action)
}

pub fn transitions_for_action(action: &ResolvedAction) -> Vec<CombatKernelTransition> {
    action
        .custom_signals
        .iter()
        .filter_map(|signal| dispatch_custom_signal(signal, action).ok())
        .flatten()
        .collect()
}

pub fn transitions_for_action_checked(
    action: &ResolvedAction,
) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> {
    let mut out = Vec::new();
    for signal in &action.custom_signals {
        out.extend(dispatch_custom_signal(signal, action)?);
    }
    Ok(out)
}
