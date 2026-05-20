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
    av::ActionValueUpdated,
    blueprints::{add_runtime_plugins, register_all_blueprint_exts},
    events::CombatEvent,
    follow_up::{
        FollowUpIntent, FollowUpTrace, follow_up_listener_system, form_identity_listener_system,
        resolve_follow_up_action_system,
    },
    kernel::register_combat_kernel_runtime,
    log::ActionLog,
    modifiers::DamageModifierLedger,
    plugin::CombatPlugin,
    rng::CombatRng,
    runtime::{
        BlueprintState, CastIdGen, ExtRegistries, IntentQueue, PassiveListeners, SignalBus,
        SignalTaxonomy, applier::intent_applier, combat_event_to_signal_system,
        passive_dispatch_system, register_kernel_builtins,
        timeline::TimelineLibrary,
    },
    sp::SpPool,
    state::CombatState,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, resolve_action_system},
};
use bevyrogue::data::{SkillBookHandle, skill_timeline::compile_skill_book_timelines, skills_ron::SkillBook};

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

/// Convenience: shape-D skill-book runtime app — `resolve_action +
/// follow_up_listener + resolve_follow_up_action` chain with a compiled
/// `TimelineLibrary` populated from `book`. Used by tests/follow_up_chains.rs,
/// tests/follow_up_triggers.rs, and tests/pipeline_dispatch.rs.
///
/// `SpPool::current` is preset to `999` so callers don't need to top it up.
pub fn skill_book_runtime_app(book: SkillBook) -> App {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(TimelineLibrary::<String>::default())
        .init_resource::<ExtRegistries>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<FollowUpIntent>()
        .add_message::<FollowUpTrace>()
        .add_systems(
            Update,
            (
                resolve_action_system,
                follow_up_listener_system,
                resolve_follow_up_action_system,
            )
                .chain(),
        );

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        register_kernel_builtins(&mut regs);
        register_all_blueprint_exts(&mut regs);
        let compiled = compile_skill_book_timelines(&book, &regs)
            .expect("skill_book_runtime_app: book must compile");
        app.world_mut().resource_mut::<TimelineLibrary<String>>().timelines = compiled;
    }
    app.world_mut().resource_mut::<SpPool>().current = 999;
    app
}

/// Convenience: skill-resolution test app — single `resolve_action_system`
/// driving a `SkillBook` asset. Inserts `SpPool{current:100,max:100}`, default
/// `CombatState/TurnOrder/ActionLog/Time`, `CombatRng::from_seed(seed)`, and
/// registers `ActionIntent + CombatEvent` messages.
///
/// Used by tests that drive a single `resolve_action_system` per `app.update()`
/// (damage_breakdown_log, status_accuracy, status_multi_kind_coexist,
/// status_refresh_max_dur, …). Callers spawn their own unit entities afterwards.
pub fn skill_resolve_app(book: SkillBook, seed: u64) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book);
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool {
            current: 100,
            max: 100,
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(seed))
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);
    app
}

/// Convenience: turn/AV-system test app — `App::new() + MinimalPlugins +
/// CombatState + TurnOrder + (TurnAdvanced, ActionValueUpdated, ActionIntent,
/// CombatEvent) messages`. No systems registered — caller adds the system(s)
/// under test. Used by `tests/enemy_ai.rs`, `tests/tempo_resistance.rs`, and
/// `tests/turn_system_av.rs`.
pub fn turn_av_base_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionValueUpdated>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>();
    app
}

/// Convenience: shape-D' follow-up engine app — same systems as
/// [`skill_book_runtime_app`] plus `form_identity_listener_system`, and
/// WITHOUT `TimelineLibrary`/`ExtRegistries`/skill-book compilation. Used by
/// tests/form_identity.rs, which exercises the follow-up resolver without
/// running compiled timelines.
pub fn form_identity_runtime_app(book: SkillBook) -> App {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<FollowUpIntent>()
        .add_message::<FollowUpTrace>()
        .add_systems(
            Update,
            (
                resolve_action_system,
                follow_up_listener_system,
                form_identity_listener_system,
                resolve_follow_up_action_system,
            )
                .chain(),
        );

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book);
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    app.world_mut().resource_mut::<SpPool>().current = 999;
    app
}
