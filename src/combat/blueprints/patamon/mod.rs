//! Patamon blueprint: custom-signal dispatch + identity (Holy Support) wiring.
//!
//! `PatamonPlugin` owns Patamon-specific kernel-runtime registrations
//! (Holy Support resource, applier system, hook) so adding or removing
//! the digimon is a single `add_plugins` line at the call site.

use bevy::prelude::*;

use crate::combat::kernel::CombatKernelRegistry;

pub mod identity;
pub mod signals;

pub use signals::{OWNER, dispatch};

pub struct PatamonPlugin;

impl Plugin for PatamonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<identity::HolySupportState>()
            .add_systems(Update, identity::apply_holy_support_transitions_system);

        app.world_mut()
            .resource_mut::<CombatKernelRegistry>()
            .register(identity::HolySupportHook);
    }
}
