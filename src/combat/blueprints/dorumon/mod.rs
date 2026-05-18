//! Dorumon blueprint: custom-signal dispatch + identity (Predator Loop) wiring.
//!
//! `DorumonPlugin` owns Dorumon-specific kernel-runtime registrations
//! (Predator Loop resource, applier system, hook) so adding or removing
//! the digimon is a single `add_plugins` line at the call site.

use crate::combat::bevy_types::*;

use crate::combat::api::registry::{ValidationField, ValidationSection};
use crate::combat::kernel::CombatKernelRegistry;
use crate::combat::blueprints::dorumon::identity::format_predator_loop_transition;
pub mod hooks;
pub mod identity;
pub mod signals;

// Public blueprint surface: lib-target sees no consumer, but `tests/` import these
// via `bevyrogue::combat::blueprints::dorumon::{...}`. Kept as the stable seam.
#[allow(unused_imports)]
pub use identity::{
    PredatorLoopBlockedReason, PredatorLoopCapKind, PredatorLoopSignal, PredatorLoopSnapshot,
    PredatorLoopState, PredatorLoopStep, PredatorLoopTransition,
    apply_predator_loop_transitions_system,
};
pub use signals::{OWNER, dispatch};

pub fn register_validation_ext(regs: &mut crate::combat::api::ExtRegistries) {
    regs.validation
        .register("predator/validation", predator_validation_section);
}

fn format_predator_targets(targets: &[identity::PredatorTargetSnapshot]) -> String {
    let joined = targets
        .iter()
        .map(|target| {
            format!(
                "{}:e{}:p{}",
                target.unit_id.0,
                target.exploit_stacks,
                target
                    .prey_lock
                    .map(|lock| format!(
                        "{}{}",
                        lock.turns_left,
                        if lock.consumed { "c" } else { "" }
                    ))
                    .unwrap_or_else(|| "none".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
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

/// Register only the Dorumon extension-point functions into an `ExtRegistries`.
// Called from register_all_blueprint_exts (consumed by tests).
#[allow(dead_code)]
pub fn register_dorumon_ext(_regs: &mut crate::combat::api::ExtRegistries) {
    // Dorumon is currently driven by Bevy systems, no fn-by-id extensions yet.
}

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
