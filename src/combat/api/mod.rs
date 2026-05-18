//! Framework primitives for M021: Intent, Registry, SignalBus, Clock, CastRng.
//!
//! This module contains the generic kernel API surface. No Digimon-specific names
//! or logic appear here (K001 / P001). All mutations must go through `Intent`;
//! all extension points must use `ExtPoint + Registry<E>`.
//!
//! ## Module map
//! - `intent`   — `CastId` + closed `Intent` enum (~18 variants).
//! - `registry` — `ExtPoint` trait + `Registry<E>` + `ExtRegistries` Resource (8 axes).
//! - `signal`   — `SignalBus` Resource scaffold (full impl in S04).
//! - `event_filter` — typed runtime filters for passive subscriptions.
//! - `rng`      — `CastRng` SplitMix64 deterministic per-cast RNG.
//!
//! ## Import constraints
//! No `use bevy::winit`, `use bevy::render`, or `use bevy_egui` in this module
//! tree. All types here must be usable in headless builds.

pub mod applier;
pub mod blueprint_state;
pub mod builtins;
pub mod clock;
pub mod event_bridge;
pub mod event_filter;
pub mod intent;
pub mod passive_runner;
pub mod registry;
pub mod rng;
pub mod runner;
pub mod runner_common;
pub mod signal;
pub mod skill_ctx;
pub mod timeline;

// Stable public API facade for the most-imported types. The lib target sees no
// in-crate consumer for several of these, but `tests/` import them via
// `bevyrogue::combat::api::{...}`; keep the facade and silence the false unused.
#[allow(unused_imports)]
pub use applier::{IntentExecutionMeta, IntentQueue, intent_applier};
pub use blueprint_state::BlueprintState;
pub use builtins::register_kernel_builtins;
pub use clock::Clock;
pub use event_bridge::combat_event_to_signal_system;
pub use event_filter::EventFilter;
#[allow(unused_imports)]
pub use intent::{CastId, CastIdGen, Intent};
pub use passive_runner::{PassiveListeners, PassiveRunner, passive_dispatch_system};
#[allow(unused_imports)]
pub use registry::{ExtRegistries, Registry, ValidationSection};
#[allow(unused_imports)]
pub use runner::StepOutcome;
#[allow(unused_imports)]
pub use signal::{Signal, SignalBus, SignalPayload, SignalTaxonomy};
pub use skill_ctx::{SkillCtx, SkillCtxMode};
#[allow(unused_imports)]
pub use timeline::{
    Beat, BeatEdge, BeatEvent, BeatId, BeatKind, BeatPayload, CompiledTimeline, Presentation,
    SelectorCtx, TimelineLibrary, ValidationError, validate_timeline_refs,
};
