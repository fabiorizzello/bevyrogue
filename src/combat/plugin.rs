use bevy::prelude::*;

use crate::combat::kernel::register_combat_kernel_runtime;
use crate::combat::modifiers::DamageModifierLedger;
use crate::combat::rng::{
    CombatRng, DEFAULT_COMBAT_RNG_SEED, combat_entropy_plugin_from_seed, seed_unit_rngs,
};
use crate::combat::runtime::applier::{IntentExecutionMeta, intent_applier};
use crate::combat::runtime::intent::CastIdGen;
use crate::combat::runtime::{
    BlueprintState, Clock, DanglingTimelineRefs, ExtRegistries, IntentQueue, PassiveListeners,
    SignalBus, SignalTaxonomy, TimelineLibrary, combat_event_to_signal_system,
    passive_dispatch_system, register_kernel_builtins, validate_timeline_refs,
};

/// Bevy plugin that registers the full combat runtime.
///
/// Mounts framework Resources (ExtRegistries, SignalBus, Clock, CombatRng,
/// IntentQueue, CastIdGen), registers Bevy-native seeded entropy for per-entity
/// RNG streams, calls the combat kernel registration, and wires the
/// `intent_applier` exclusive system. Add once in main/bin.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(combat_entropy_plugin_from_seed(DEFAULT_COMBAT_RNG_SEED));
        register_combat_kernel_runtime(app);

        app.init_resource::<ExtRegistries>()
            .init_resource::<SignalBus>()
            .init_resource::<SignalTaxonomy>()
            .init_resource::<CastIdGen>()
            .init_resource::<IntentQueue>()
            .init_resource::<IntentExecutionMeta>()
            .init_resource::<TimelineLibrary<String>>()
            .init_resource::<BlueprintState>()
            .init_resource::<DamageModifierLedger>()
            .init_resource::<PassiveListeners>()
            .init_resource::<crate::combat::turn_system::OutOfTurnBurst>()
            .init_resource::<crate::combat::turn_system::PendingBurstQueue>()
            .insert_resource(Clock::default())
            .insert_resource(CombatRng::from_seed(DEFAULT_COMBAT_RNG_SEED))
            .add_systems(
                Update,
                (
                    // Fork per-entity RNG streams before any applier rolls so
                    // the timeline path reads the per-unit stream, not the
                    // resource fallback.
                    seed_unit_rngs.before(intent_applier),
                    intent_applier,
                    combat_event_to_signal_system.after(intent_applier),
                    passive_dispatch_system.after(combat_event_to_signal_system),
                ),
            );

        {
            let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
            register_kernel_builtins(&mut regs);
        }

        crate::combat::blueprints::register_blueprints(app);

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
        let library = world.resource::<TimelineLibrary<String>>();
        let regs = world.resource::<ExtRegistries>();

        let mut all_errors = Vec::new();
        for timeline in &library.timelines {
            if let Err(errs) = validate_timeline_refs(timeline, regs) {
                all_errors.extend(errs);
            }
        }

        if !all_errors.is_empty() {
            panic!(
                "CombatPlugin::finish — {}",
                DanglingTimelineRefs(all_errors)
            );
        }
    }
}
