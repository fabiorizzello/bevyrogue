//! Renamon blueprint: custom-signal dispatch + identity (MIND GAME) wiring.
//!
//! `RenamonPlugin` owns Renamon-specific kernel-runtime registrations
//! (MIND GAME resource, applier system, hook) so adding or removing
//! the digimon is a single `add_plugins` line at the call site.

use crate::combat::bevy_types::*;

use crate::combat::runtime::registry::{ValidationField, ValidationSection};
use crate::combat::{
    runtime::SignalPayload,
    kernel::{CombatKernelRegistry, CombatKernelTransition, CombatKernelHook, CombatKernelHookDomain},
    types::UnitId,
};
use crate::data::skills_ron::SkillCustomSignal;

use super::CustomSignalDispatchError;

pub mod apply;
pub mod identity;
pub mod passive;

// Public blueprint surface: kept stable so `tests/` imports via
// `bevyrogue::combat::blueprints::renamon::{...}` continue to resolve.
pub use apply::{
    apply_precision_mind_game_transition, apply_precision_mind_game_transitions_system,
};
pub use identity::{
    PrecisionCommitment, PrecisionMindGamePhase, PrecisionMindGameRejectReason,
    PrecisionMindGameSnapshot, PrecisionMindGameState, PrecisionMindGameStep,
    PrecisionMindGameTransition, PrecisionOutcome, PrecisionReveal, PrecisionWindowKind,
};
pub use passive::register_passive_runtime;

pub const OWNER: &str = "renamon";

const SIGNAL_OPEN_MOMENTUM_WINDOW: &str = "open_momentum_window";
const SIGNAL_COMMIT_PRECISION_PRESS: &str = "commit_precision_press";
const SIGNAL_REVEAL_BAIT: &str = "reveal_bait";
const SIGNAL_RESOLVE_PRECISION_SUCCESS: &str = "resolve_precision_success";

const PASSIVE_SIGNAL_NAME: &str = "kitsune_grace";
const PASSIVE_TRIGGER_KEY: &str = "renamon/kitsune_grace/triggered";
const PASSIVE_TIMELINE_ID: &str = "renamon_kitsune_grace_passive";
const PASSIVE_OWNER: UnitId = UnitId(7);

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

pub struct RenamonPlugin;

impl Plugin for RenamonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrecisionMindGameState>()
            .add_systems(Update, apply_precision_mind_game_transitions_system);

        app.world_mut()
            .resource_mut::<CombatKernelRegistry>()
            .register(PrecisionMindGameHook);
    }
}

fn blueprint_transition(name: &str) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: OWNER.to_owned(),
        name: name.to_owned(),
        payload: SignalPayload::Empty,
    }
}

pub fn register_renamon_ext(regs: &mut crate::combat::runtime::ExtRegistries) {
    regs.validation
        .register("mind_game/validation", precision_validation_section);
}

fn precision_validation_section(world: &World) -> Option<ValidationSection> {
    let state = world.get_resource::<PrecisionMindGameState>()?;
    Some(ValidationSection::new(
        "mind_game",
        vec![
            ValidationField::new("phase", format_precision_phase(state.phase)),
            ValidationField::new("window_index", state.window_index.to_string()),
            ValidationField::new(
                "window",
                state
                    .current_window
                    .map(|window| format_precision_window(Some(window)))
                    .unwrap_or("none"),
            ),
            ValidationField::new(
                "commitment",
                state
                    .commitment
                    .map(|commitment| format_precision_commitment(Some(commitment)))
                    .unwrap_or("none"),
            ),
            ValidationField::new(
                "reveal",
                state
                    .reveal
                    .map(|reveal| format_precision_reveal(Some(reveal)))
                    .unwrap_or("none"),
            ),
            ValidationField::new(
                "outcome",
                state
                    .outcome
                    .map(|outcome| format_precision_outcome(Some(outcome)))
                    .unwrap_or("none"),
            ),
            ValidationField::new(
                "last",
                state
                    .last_signal
                    .map(format_precision_transition)
                    .unwrap_or_else(|| "none".to_string()),
            ),
        ],
    ))
}

fn format_precision_transition(transition: PrecisionMindGameTransition) -> String {
    match transition {
        PrecisionMindGameTransition::OpenWindow { window } => {
            format!("open_window({})", format_precision_window(Some(window)))
        }
        PrecisionMindGameTransition::Commit { commitment } => {
            format!("commit({})", format_precision_commitment(Some(commitment)))
        }
        PrecisionMindGameTransition::Reveal { reveal } => {
            format!("reveal({})", format_precision_reveal(Some(reveal)))
        }
        PrecisionMindGameTransition::Resolve { outcome } => {
            format!("resolve({})", format_precision_outcome(Some(outcome)))
        }
        PrecisionMindGameTransition::Rejected { attempted, reason } => {
            format!("rejected({:?};reason={:?})", attempted, reason)
        }
        PrecisionMindGameTransition::Ignored { attempted } => {
            format!("ignored({:?})", attempted)
        }
    }
}

fn format_precision_phase(phase: PrecisionMindGamePhase) -> String {
    format!("{:?}", phase)
}

fn format_precision_window(window: Option<PrecisionWindowKind>) -> &'static str {
    match window {
        Some(PrecisionWindowKind::Momentum) => "Momentum",
        Some(PrecisionWindowKind::Counterplay) => "Counterplay",
        None => "none",
    }
}

fn format_precision_commitment(commitment: Option<PrecisionCommitment>) -> &'static str {
    match commitment {
        Some(PrecisionCommitment::Press) => "Press",
        Some(PrecisionCommitment::Hold) => "Hold",
        Some(PrecisionCommitment::Feint) => "Feint",
        None => "none",
    }
}

fn format_precision_reveal(reveal: Option<PrecisionReveal>) -> &'static str {
    match reveal {
        Some(PrecisionReveal::Guarded) => "Guarded",
        Some(PrecisionReveal::Baited) => "Baited",
        Some(PrecisionReveal::Trapped) => "Trapped",
        None => "none",
    }
}

fn format_precision_outcome(outcome: Option<PrecisionOutcome>) -> &'static str {
    match outcome {
        Some(PrecisionOutcome::Success) => "Success",
        Some(PrecisionOutcome::Countered) => "Countered",
        Some(PrecisionOutcome::Failed) => "Failed",
        None => "none",
    }
}

pub fn dispatch(
    signal: &SkillCustomSignal,
    _action: &crate::combat::state::ResolvedAction,
) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> {
    if signal.owner() != OWNER {
        return Err(CustomSignalDispatchError::UnknownOwner {
            owner: signal.owner().to_owned(),
        });
    }

    match signal.signal() {
        SIGNAL_OPEN_MOMENTUM_WINDOW => Ok(vec![blueprint_transition(SIGNAL_OPEN_MOMENTUM_WINDOW)]),
        SIGNAL_COMMIT_PRECISION_PRESS => {
            Ok(vec![blueprint_transition(SIGNAL_COMMIT_PRECISION_PRESS)])
        }
        SIGNAL_REVEAL_BAIT => Ok(vec![blueprint_transition(SIGNAL_REVEAL_BAIT)]),
        SIGNAL_RESOLVE_PRECISION_SUCCESS => {
            Ok(vec![blueprint_transition(SIGNAL_RESOLVE_PRECISION_SUCCESS)])
        }
        _ => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_owned(),
            signal: signal.signal().to_owned(),
        }),
    }
}
