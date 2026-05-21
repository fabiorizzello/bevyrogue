//! Gameplay-ability execution runtime: Intent routing, Registry extension points,
//! SignalBus, Clock, CastRng, and the timeline-backed skill resolution pipeline.
//!
//!
//! No Digimon-specific names or logic appear here (K001 / P001). All mutations
//! must go through `Intent`; all extension points use `ExtPoint + Registry<E>`.
//!
//! ## Module map
//! - `intent`       — `CastId` + closed `Intent` enum (~18 variants).
//! - `registry`     — `ExtPoint` trait + `Registry<E>` + `ExtRegistries` Resource (9 axes).
//! - `post_action`  — owner-neutral post-application reaction seam with KO context.
//! - `signal`       — `SignalBus` + `SignalTaxonomy` for blueprint-owned custom signals.
//! - `event_filter` — typed runtime filters for passive subscriptions.
//! - `rng`          — `CastRng` deterministic per-cast RNG backed by `bevy_prng`.
//! - `applier`      — exclusive `intent_applier` system that drains `IntentQueue`.
//! - `runner`       — timeline-backed skill execution (FSM stepping, beat evaluation).
//! - `timeline`     — compiled timeline schema and evaluation helpers.
//!
//! ## Import constraints
//! No `use bevy::winit`, `use bevy::render`, or `use bevy_egui` in this module
//! tree. All types here must be usable in headless builds.

pub mod applier;
pub mod blueprint_state;
pub mod builtins;
pub mod clock;
pub mod cue_barrier;
pub mod event_bridge;
pub mod event_filter;
pub mod intent;
pub mod passive_runner;
pub mod post_action;
pub mod registry;
pub mod rng;
pub mod runner;
pub mod runner_common;
pub mod signal;
pub mod skill_ctx;
pub mod timeline;

// Stable public API facade for the most-imported types. The lib target sees no
// in-crate consumer for several of these, but `tests/` import them via
// `bevyrogue::combat::runtime::{...}`; keep the facade and silence the false unused.
pub use applier::{IntentExecutionMeta, IntentQueue, intent_applier};
pub use blueprint_state::BlueprintState;
pub use builtins::register_kernel_builtins;
pub use clock::Clock;
pub use cue_barrier::{
    CUE_BARRIER_TIMEOUT_FRAMES, CueBarrierStatus, CueReleaseResult, SuspendedTimelineState,
    TimelineClock, request_timeline_cue_release,
};
pub use event_bridge::combat_event_to_signal_system;
pub use event_filter::EventFilter;
pub use intent::{CastId, CastIdGen, Intent};
pub use passive_runner::{PassiveListeners, PassiveRunner, passive_dispatch_system};
pub use post_action::{
    PostActionContext, PostActionQueue, PostActionUnitDied, PostActionUnitSnapshot,
    dispatch_post_action_reactions,
};
pub use registry::{ExtRegistries, Registry, ValidationSection};
pub use runner::StepOutcome;
pub use signal::{Signal, SignalBus, SignalPayload, SignalTaxonomy};
pub use skill_ctx::{SkillCtx, SkillCtxMode};
pub use timeline::{
    Beat, BeatEdge, BeatEvent, BeatId, BeatKind, BeatPayload, CompiledTimeline,
    DanglingTimelineRefs, Presentation, SelectorCtx, TimelineLibrary, ValidationError,
    validate_timeline_refs,
};
