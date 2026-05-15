//! Framework primitives for M021: Intent, Registry, SignalBus, Clock, CastRng.
//!
//! This module contains the generic kernel API surface. No Digimon-specific names
//! or logic appear here (K001 / P001). All mutations must go through `Intent`;
//! all extension points must use `ExtPoint + Registry<E>`.
//!
//! ## Module map
//! - `intent`   — `CastId` + closed `Intent` enum (~18 variants).
//! - `registry` — `ExtPoint` trait + `Registry<E>` + `ExtRegistries` Resource (7 axes).
//! - `signal`   — `SignalBus` Resource scaffold (full impl in S04).
//! - `clock`    — `Clock { HeadlessAuto, Windowed }` enum.
//! - `rng`      — `CastRng` SplitMix64 deterministic per-cast RNG.
//!
//! ## Import constraints
//! No `use bevy::winit`, `use bevy::render`, or `use bevy_egui` in this module
//! tree. All types here must be usable in headless builds.

pub mod applier;
pub mod clock;
pub mod intent;
pub mod registry;
pub mod rng;
pub mod runner;
pub mod signal;
pub mod skill_ctx;
pub mod timeline;

// Stable re-exports for the most-imported types.
pub use applier::IntentQueue;
pub use clock::Clock;
pub use intent::{CastId, CastIdGen, Intent};
pub use registry::{
    AiUtilityExt, CueExt, ExtPoint, ExtRegistries, FormulaExt, HookExt, PredicateExt, Registry,
    SelectorExt, TickExt,
};
pub use rng::CastRng;
pub use runner::{BeatRunner, LoopFrame, StepOutcome};
pub use signal::SignalBus;
pub use skill_ctx::{SkillCtx, SkillCtxMode};
pub use timeline::{
    Beat, BeatEdge, BeatEvent, BeatId, BeatKind, CompiledTimeline, CueCtx, Presentation,
    SelectorCtx, TimelineLibrary, ValidationError, validate_timeline_refs,
};
