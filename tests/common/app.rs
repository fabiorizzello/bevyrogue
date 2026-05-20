//! `TestAppBuilder` — chainable builder that consolidates the 26 `setup_app()`
//! variants scattered across `tests/`.
//!
//! Three shape classes are supported, plus opt-in layers:
//!
//! - **A — minimal intent_applier**: `IntentQueue + CastIdGen + CombatEvent +
//!   intent_applier`. Used by every timeline / intent test.
//! - **B — passive dispatch**: A + `SignalBus + SignalTaxonomy + BlueprintState
//!   + ExtRegistries + PassiveListeners` and the
//!   `combat_event_to_signal -> passive_dispatch` chain after `intent_applier`.
//! - **C — full kernel**: A + `DamageModifierLedger` + `CombatRng` +
//!   `register_combat_kernel_runtime` + blueprint runtime plugins. Use
//!   [`TestAppBuilder::with_kernel`] for this.
//!
//! Independent layers:
//!
//! - [`TestAppBuilder::with_rng`] — `CombatRng::from_seed`.
//! - [`TestAppBuilder::with_damage_ledger`] — `DamageModifierLedger`.
//! - [`TestAppBuilder::with_signal_bus`] — `SignalBus + SignalTaxonomy +
//!   BlueprintState`, no systems.
//! - [`TestAppBuilder::with_combat_plugin`] — `CombatPlugin`; cannot mix with
//!   `with_intent_applier` (the plugin already adds it).
//!
//! Tests that have unique scheduling needs (e.g. `enemy_ai.rs`,
//! `turn_system_av.rs`, `tempo_resistance.rs`) can call
//! [`TestAppBuilder::raw_app`] for an `App::new()` already carrying
//! `CombatEvent` and then add their own systems.

#![allow(dead_code)]

use bevy::prelude::*;
use bevyrogue::combat::{
    blueprints::add_runtime_plugins,
    events::CombatEvent,
    kernel::register_combat_kernel_runtime,
    modifiers::DamageModifierLedger,
    plugin::CombatPlugin,
    rng::CombatRng,
    runtime::{
        BlueprintState, CastIdGen, ExtRegistries, IntentQueue, PassiveListeners, SignalBus,
        SignalTaxonomy, applier::intent_applier, combat_event_to_signal_system,
        passive_dispatch_system,
    },
};

/// Chainable test-app builder. Default is empty `App::new()` carrying just the
/// `CombatEvent` message; opt into layers via the `with_*` methods.
pub struct TestAppBuilder {
    app: App,
}

impl Default for TestAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TestAppBuilder {
    /// Empty app with the `CombatEvent` message registered.
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_message::<CombatEvent>();
        Self { app }
    }

    /// Shape-A baseline: adds `IntentQueue + CastIdGen` and schedules
    /// `intent_applier` on `Update`.
    pub fn with_intent_applier(mut self) -> Self {
        self.app
            .init_resource::<IntentQueue>()
            .init_resource::<CastIdGen>()
            .add_systems(Update, intent_applier);
        self
    }

    /// Adds `DamageModifierLedger`. Independent layer — chain after
    /// `with_intent_applier` for shape A+ledger.
    pub fn with_damage_ledger(mut self) -> Self {
        self.app.init_resource::<DamageModifierLedger>();
        self
    }

    /// Seeded `CombatRng`. Required for kernel/blueprint-runtime tests.
    pub fn with_rng(mut self, seed: u64) -> Self {
        self.app.insert_resource(CombatRng::from_seed(seed));
        self
    }

    /// Adds `SignalBus + SignalTaxonomy + BlueprintState` resources WITHOUT
    /// any systems. Use this when the test drives signals manually.
    pub fn with_signal_bus(mut self) -> Self {
        self.app
            .init_resource::<SignalBus>()
            .init_resource::<SignalTaxonomy>()
            .init_resource::<BlueprintState>();
        self
    }

    /// Shape-B layer: adds `ExtRegistries + PassiveListeners` and schedules
    /// `combat_event_to_signal_system` and `passive_dispatch_system` after
    /// `intent_applier`. Implies [`Self::with_signal_bus`]; the caller must
    /// have called [`Self::with_intent_applier`] earlier.
    pub fn with_passive_dispatch(mut self) -> Self {
        self.app
            .init_resource::<SignalBus>()
            .init_resource::<SignalTaxonomy>()
            .init_resource::<BlueprintState>()
            .init_resource::<ExtRegistries>()
            .init_resource::<PassiveListeners>()
            .add_systems(
                Update,
                (
                    combat_event_to_signal_system.after(intent_applier),
                    passive_dispatch_system.after(combat_event_to_signal_system),
                ),
            );
        self
    }

    /// Shape-C layer: registers the combat kernel runtime and blueprint
    /// plugins. Caller is responsible for providing `with_intent_applier`,
    /// `with_damage_ledger`, and `with_rng` first.
    pub fn with_kernel(mut self) -> Self {
        register_combat_kernel_runtime(&mut self.app);
        add_runtime_plugins(&mut self.app);
        self
    }

    /// Full `CombatPlugin`. Mutually exclusive with `with_intent_applier`
    /// (the plugin already wires intent_applier internally).
    pub fn with_combat_plugin(mut self) -> Self {
        self.app.add_plugins(CombatPlugin);
        self
    }

    /// Escape hatch for tests that need direct access during construction
    /// (e.g. to add custom systems before `build`).
    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    /// Finalize the builder.
    pub fn build(self) -> App {
        self.app
    }
}

/// Convenience: shape-A minimal app (intent_applier + CombatEvent only).
pub fn minimal_intent_app() -> App {
    TestAppBuilder::new().with_intent_applier().build()
}

/// Convenience: shape-B passive-dispatch app.
pub fn passive_dispatch_app() -> App {
    TestAppBuilder::new()
        .with_intent_applier()
        .with_passive_dispatch()
        .build()
}

/// Convenience: shape-C full kernel app (seeded RNG, damage ledger, kernel,
/// blueprint plugins).
pub fn kernel_app(seed: u64) -> App {
    TestAppBuilder::new()
        .with_intent_applier()
        .with_damage_ledger()
        .with_rng(seed)
        .with_kernel()
        .build()
}
