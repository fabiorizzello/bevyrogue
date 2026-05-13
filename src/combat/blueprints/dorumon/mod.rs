//! Dorumon blueprint: custom-signal dispatch + identity (Predator Loop) wiring.
//!
//! `DorumonPlugin` owns Dorumon-specific kernel-runtime registrations
//! (Predator Loop resource, applier system, hook) so adding or removing
//! the digimon is a single `add_plugins` line at the call site.

use bevy::prelude::*;

use crate::combat::kernel::CombatKernelRegistry;

pub mod hooks;
pub mod identity;
pub mod signals;

pub use signals::{OWNER, dispatch};

pub struct DorumonPlugin;

impl Plugin for DorumonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<identity::PredatorLoopState>()
            .add_systems(Update, identity::apply_predator_loop_transitions_system);

        app.world_mut()
            .resource_mut::<CombatKernelRegistry>()
            .register(identity::PredatorLoopHook);
    }
}
