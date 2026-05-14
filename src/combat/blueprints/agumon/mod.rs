//! Agumon blueprint: custom-signal dispatch + identity (Twin Core) wiring.
//!
//! `AgumonPlugin` owns Agumon-specific kernel-runtime registrations (Twin
//! Core resource, applier system, hook) so adding or removing the digimon is
//! a single `add_plugins` line at the call site. Twin Core is shared with
//! Gabumon (paired Fire/Ice identity); Agumon owns the registration as the
//! Fire half.

use bevy::prelude::*;

use crate::combat::kernel::CombatKernelRegistry;

pub mod identity;
pub mod signals;

pub use identity::{
    TAG_CHILLED, TAG_DEEP_CRACK, TAG_HEATED, TAG_MELTDOWN_CRACK, TAG_PRIMED, TAG_THERMAL_SPARK,
    TwinCoreDesignTag, TwinCoreHook, TwinCoreState,
    apply_twin_core_transitions_system, classify_twin_core_tag,
    twin_core_added_tag_transition, twin_core_design_tag, twin_core_design_tag_name,
};
pub use signals::{OWNER, dispatch};

pub struct AgumonPlugin;

impl Plugin for AgumonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<identity::TwinCoreState>()
            .add_systems(Update, identity::apply_twin_core_transitions_system);

        app.world_mut()
            .resource_mut::<CombatKernelRegistry>()
            .register(identity::TwinCoreHook);
    }
}
