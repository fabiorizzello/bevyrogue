//! Patamon blueprint: custom-signal dispatch + identity (Holy Support) wiring.
//!
//! `PatamonPlugin` owns Patamon-specific kernel-runtime registrations
//! (Holy Support resource, applier system, hook) so adding or removing
//! the digimon is a single `add_plugins` line at the call site.

use bevy::prelude::*;

use crate::combat::kernel::CombatKernelRegistry;

pub mod identity;
pub mod signals;

pub use identity::{
    GRACE_CAP, TAG_GRACE, TAG_MARTYR_LIGHT,
    HolySupportDesignTag, HolySupportHook, HolySupportRejectReason, HolySupportSnapshot,
    HolySupportState, HolySupportStep, HolySupportTransition,
    apply_holy_support_transitions_system, classify_holy_support_tag,
    holy_support_added_tag_transition, holy_support_design_tag, holy_support_design_tag_name,
};
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
