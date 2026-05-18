use bevy::prelude::*;

use crate::combat::api::{
    BlueprintState, Clock, ExtRegistries, IntentQueue, PassiveListeners, SignalBus,
    SignalTaxonomy, TimelineLibrary, combat_event_to_signal_system, passive_dispatch_system,
    register_kernel_builtins, validate_timeline_refs,
};
use crate::combat::api::applier::{IntentExecutionMeta, intent_applier};
use crate::combat::api::intent::CastIdGen;
use crate::combat::kernel::register_combat_kernel_runtime;
use crate::combat::modifiers::DamageModifierLedger;
use crate::combat::rng::CombatRng;

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
            .init_resource::<IntentExecutionMeta>()
            .init_resource::<TimelineLibrary<String>>()
            .init_resource::<BlueprintState>()
            .init_resource::<DamageModifierLedger>()
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
