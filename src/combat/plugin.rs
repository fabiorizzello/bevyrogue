use bevy::prelude::*;

use crate::combat::{
    api::{
        applier::{intent_applier, IntentQueue},
        blueprint_state::BlueprintState,
        clock::Clock,
        event_bridge::combat_event_to_signal_system,
        intent::CastIdGen,
        passive_runner::{passive_dispatch_system, PassiveListeners},
        registry::ExtRegistries,
        signal::{SignalBus, SignalTaxonomy},
        timeline::{validate_timeline_refs, TimelineLibrary},
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
            .init_resource::<SignalTaxonomy>()
            .init_resource::<CastIdGen>()
            .init_resource::<IntentQueue>()
            .init_resource::<TimelineLibrary>()
            .init_resource::<BlueprintState>()
            .init_resource::<PassiveListeners>()
            .insert_resource(Clock::default())
            .insert_resource(CombatRng::from_seed(0xDEAD_BEEF))
            .add_systems(
                Update,
                (
                    intent_applier,
                    combat_event_to_signal_system.after(intent_applier),
                    passive_dispatch_system.after(combat_event_to_signal_system),
                ),
            );

        // Register kernel-side signals in SignalTaxonomy so ult-driven passive
        // activations pass the debug_assert! gate in intent_applier.
        app.world_mut()
            .resource_mut::<SignalTaxonomy>()
            .register("kernel", "ult_used");
    }

    /// Validates all registered `CompiledTimeline`s against `ExtRegistries`.
    ///
    /// Panics at boot if any dangling hook/selector/predicate reference is found.
    /// Wire concrete timelines into `TimelineLibrary` before `App::finish()` (S05+).
    fn finish(&self, app: &mut App) {
        let world = app.world();
        let library = world.resource::<TimelineLibrary>();
        let regs = world.resource::<ExtRegistries>();

        let mut all_errors = Vec::new();
        for timeline in &library.timelines {
            if let Err(errs) = validate_timeline_refs(timeline, regs) {
                all_errors.extend(errs);
            }
        }

        if !all_errors.is_empty() {
            let msg: Vec<String> = all_errors
                .iter()
                .map(|e| format!("[{}] missing '{}' at {}", e.axis, e.missing_id, e.site))
                .collect();
            panic!(
                "CombatPlugin::finish — dangling timeline references:\n{}",
                msg.join("\n")
            );
        }
    }
}
