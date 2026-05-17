//! Dorumon blueprint: custom-signal dispatch + identity (Predator Loop) wiring.
//!
//! `DorumonPlugin` owns Dorumon-specific kernel-runtime registrations
//! (Predator Loop resource, applier system, hook) so adding or removing
//! the digimon is a single `add_plugins` line at the call site.

use bevy::prelude::*;

use crate::combat::api::registry::{ValidationField, ValidationSection};
use crate::combat::kernel::CombatKernelRegistry;
use crate::combat::observability::{format_predator_loop_transition, format_predator_targets};

pub mod hooks;
pub mod identity;
pub mod signals;

pub use identity::{
    DEFAULT_BERSERK_STRAIN_THRESHOLD, DEFAULT_EXPLOIT_CAP, DEFAULT_PREY_LOCK_DURATION,
    PredatorLockState, PredatorLoopBlockedReason, PredatorLoopCapKind, PredatorLoopDesignTag,
    PredatorLoopHook, PredatorLoopRequestKind, PredatorLoopSignal, PredatorLoopSnapshot,
    PredatorLoopState, PredatorLoopStep, PredatorLoopTransition, PredatorTargetSnapshot,
    PredatorTargetState, apply_predator_loop_transition, apply_predator_loop_transitions_system,
};
pub use signals::{OWNER, dispatch};

pub fn register_validation_ext(regs: &mut crate::combat::api::ExtRegistries) {
    regs.validation
        .register("predator/validation", predator_validation_section);
}

fn predator_validation_section(world: &World) -> Option<ValidationSection> {
    let snapshot = world.get_resource::<identity::PredatorLoopState>()?.snapshot();
    Some(ValidationSection::new(
        "predator",
        vec![
            ValidationField::new("exploit_cap", snapshot.exploit_cap.to_string()),
            ValidationField::new(
                "prey_lock_duration",
                snapshot.prey_lock_duration.to_string(),
            ),
            ValidationField::new(
                "berserk_threshold",
                snapshot.berserk_strain_threshold.to_string(),
            ),
            ValidationField::new("targets", format_predator_targets(&snapshot.targets)),
            ValidationField::new(
                "last",
                snapshot
                    .last_transition
                    .map(format_predator_loop_transition)
                    .unwrap_or_else(|| "none".to_string()),
            ),
            ValidationField::new(
                "blocked",
                snapshot
                    .last_blocked_reason
                    .map(|reason| format!("{:?}", reason))
                    .unwrap_or_else(|| "none".to_string()),
            ),
        ],
    ))
}

pub struct DorumonPlugin;

impl Plugin for DorumonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<identity::PredatorLoopState>()
            .add_systems(Update, identity::apply_predator_loop_transitions_system);

        hooks::register_passive_runtime(app);

        app.world_mut()
            .resource_mut::<CombatKernelRegistry>()
            .register(identity::PredatorLoopHook);
    }
}
