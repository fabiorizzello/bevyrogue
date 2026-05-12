// THROWAWAY SKETCH — NON-COMPILED, ILLUSTRATIVE ONLY.
// Spike SP2 — Option B central registry plumbing.
// Real implementation lands in M017 Slice A under src/combat/blueprint_registry.rs.

use std::collections::HashMap;

// References to the trait sketch (sibling file).
use super::blueprint_trait::{Blueprint, BlueprintCtx, BlueprintId, BlueprintSnapshot};

/// Central blueprint registry. Single Resource.
///
/// Owns `Box<dyn Blueprint>` for every registered blueprint, plus a reverse
/// index from `owner` strings (the `SkillCustomSignal.owner` field) to
/// `BlueprintId`. The reverse index is just `BlueprintId(owner_str)` today
/// because we encode them identically, but the indirection lets a future
/// alias (e.g. "kitsune_grace" -> blueprint owned by Renamon module) work.
#[derive(bevy::prelude::Resource, Default)]
pub struct BlueprintRegistry {
    blueprints: HashMap<BlueprintId, Box<dyn Blueprint>>,
    /// Owner-string index for `dispatch_commit` routing.
    owner_index: HashMap<String, BlueprintId>,
}

impl BlueprintRegistry {
    pub fn register<B: Blueprint>(&mut self, blueprint: B) {
        let id = blueprint.id();
        self.owner_index.insert(id.0.to_string(), id.clone());
        self.blueprints.insert(id, Box::new(blueprint));
    }

    /// Commit-time signal dispatch. Used by the action pipeline
    /// `dispatch_blueprint_transitions` to translate RON-declared
    /// custom_signals into kernel transitions. Replaces the legacy
    /// static `BLUEPRINTS` array in `blueprints/mod.rs`.
    pub fn dispatch_commit(
        &self,
        signal: &super::SkillCustomSignal,
        action: &super::ResolvedAction,
    ) -> Result<Vec<super::CombatKernelTransition>, super::CustomSignalDispatchError> {
        let id = self
            .owner_index
            .get(signal.owner())
            .ok_or_else(|| super::CustomSignalDispatchError::UnknownOwner {
                owner: signal.owner().to_string(),
            })?;
        let blueprint = self.blueprints.get(id).expect("owner_index must match");
        blueprint.commit_signals(signal, action)
    }

    /// Event-driven listener dispatch. Called once per CombatEvent by the
    /// single registry-driven listener system `blueprint_event_listener_system`.
    /// Returns the aggregated Effect cascade across all blueprints whose
    /// `on_event` produces non-empty output for this event.
    ///
    /// Iteration order: stable by BlueprintId for determinism (sort_unstable
    /// on the keys at registration time, cache the ordered slice).
    pub fn dispatch_event(
        &self,
        event: &super::CombatEvent,
        ctx: &BlueprintCtx,
    ) -> Vec<super::Effect> {
        let mut out = Vec::new();
        // In real impl: iterate over a pre-sorted Vec<BlueprintId> for
        // deterministic ordering. HashMap iteration order is non-deterministic.
        for (_id, bp) in self.blueprints.iter() {
            out.extend(bp.on_event(event, ctx));
        }
        out
    }

    /// Observability. Replaces the 5 hardcoded snapshot fields in
    /// `ValidationSnapshot`. Each blueprint contributes one entry
    /// keyed by its `id()`. Listener-only blueprints contribute
    /// `BlueprintSnapshot::Empty` and can be filtered at serialization.
    pub fn snapshot_all(
        &self,
        world: &bevy::prelude::World,
    ) -> HashMap<BlueprintId, BlueprintSnapshot> {
        self.blueprints
            .iter()
            .map(|(id, bp)| (id.clone(), bp.snapshot(world)))
            .collect()
    }
}

/// Single registration site. Called once at app setup, after
/// `register_combat_kernel_runtime`.
///
/// Adds the 6 existing blueprints + the 2 round-3 listener-only blueprints.
/// Order is irrelevant for correctness (id-based deterministic dispatch);
/// listed Digimon-by-Digimon for readability.
pub fn register_default_blueprints(app: &mut bevy::prelude::App) {
    let mut registry = BlueprintRegistry::default();

    // Commit-time blueprints (existing 6).
    registry.register(super::TwinCoreFireBlueprint);   // agumon
    registry.register(super::TwinCoreIceBlueprint);    // gabumon
    registry.register(super::PredatorLoopBlueprint);   // dorumon
    registry.register(super::BatteryLoopBlueprint);    // tentomon
    registry.register(super::HolySupportBlueprint);    // patamon
    registry.register(super::PrecisionMindGameBlueprint); // renamon

    // Round-3 listener-only blueprints (new). See kitsune_grace.rs + holy_aegis.rs.
    registry.register(super::KitsuneGraceBlueprint);
    registry.register(super::HolyAegisBlueprint);

    app.insert_resource(registry);

    // Register the single listener system that drains CombatEvent
    // and routes through registry.dispatch_event.
    app.add_systems(bevy::prelude::Update, blueprint_event_listener_system);
}

/// Single-system listener bridge. Reads `MessageReader<CombatEvent>`,
/// builds a `BlueprintCtx`, calls `registry.dispatch_event` per event,
/// and forwards the returned Effects into the existing cascade pipeline.
///
/// Pseudocode — actual signature depends on resolution.rs Effect-cascade
/// entry point, which is M017 S03e work.
fn blueprint_event_listener_system(
    /* world: ..., events: MessageReader<CombatEvent>, registry: Res<BlueprintRegistry>, ... */
) {
    // for event in events.read() {
    //     let ctx = BlueprintCtx::from_world(world);
    //     let effects = registry.dispatch_event(event, &ctx);
    //     for effect in effects {
    //         enqueue_into_cascade(effect, event.source, event.target, event.follow_up_depth);
    //     }
    // }
    unimplemented!("kernel cascade entry — depends on M017 S03e Effect interpreter")
}
