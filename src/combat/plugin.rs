use bevy::prelude::*;

use crate::combat::{
    api::{
        applier::{intent_applier, IntentQueue},
        clock::Clock,
        intent::CastIdGen,
        registry::ExtRegistries,
        signal::SignalBus,
    },
    kernel::register_combat_kernel_runtime,
    rng::CombatRng,
};

/// Bevy plugin that registers the full combat runtime.
///
/// Mounts framework Resources (ExtRegistries, SignalBus, Clock, CombatRng,
/// IntentQueue, CastIdGen), calls the combat kernel registration, and wires
/// the `intent_applier` exclusive system. Add once in main/bin.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        register_combat_kernel_runtime(app);

        app.init_resource::<ExtRegistries>()
            .init_resource::<SignalBus>()
            .init_resource::<CastIdGen>()
            .init_resource::<IntentQueue>()
            .insert_resource(Clock::default())
            .insert_resource(CombatRng::from_seed(0xDEAD_BEEF))
            .add_systems(Update, intent_applier);
    }
}
