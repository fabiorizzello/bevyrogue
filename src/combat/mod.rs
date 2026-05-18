//! Combat core. Headless-first; UI legge solo eventi/log.
//!
//! Vedi `docs/combat_current.md` per design intent corrente. CLAUDE.md per onboarding.
//!
//! Modules are grouped by responsibility — declaration order is informational
//! only (Rust does not enforce it), but the buckets below act as a hint for
//! agents and humans navigating the 30+ files in this crate root.

// ─── Runtime engine ─────────────────────────────────────────────────────────
// Intent routing, registry, signal bus, timeline FSM, skill execution.
// No Digimon-specific names. No bevy::winit / bevy::render / bevy_egui imports.

/// Runtime engine: Intent routing, ExtPoint+Registry, SignalBus, Clock, CastRng.
pub mod runtime;

pub(crate) mod bevy_types;

// ─── Core kernel & primitives ────────────────────────────────────────────────
// Shared vocabulary used by every other module.

/// Pure action legality / affordance query vocabulary.
pub mod action_query;
/// Revised combat kernel primitives: Tactical Cycle, Strain, Flow, Fatigue, tags, beat IDs, and hooks.
pub mod kernel;
/// `UnitSkills` (basic / skill / ultimate IDs per Unit).
pub mod kit;
/// Deterministic RNG resource (`CombatRng`) — centralises all combat randomness (R019).
pub mod rng;
/// `CombatState`, `CombatPhase`, `InFlightAction`.
pub mod state;
/// `Team::{Ally, Enemy}` — appartenenza unità.
pub mod team;
/// `UnitId`, `SkillId`, `Attribute`, `DamageTag`.
pub mod types;
/// `Unit` component (HP, attribute), markers `Ko`, `Commander`.
pub mod unit;

// ─── Turn pipeline ───────────────────────────────────────────────────────────
// AV gauge, order queue, intent resolution, speed / tempo modifiers.

/// Shared skill-preview seam for UI/AI consumers.
pub mod preview;
/// Apply runtime payloads: traduce `Action` (resolved) → mutazioni stato + eventi.
pub mod resolution;
/// Turn pipeline, AV gauge, turn order, speed, tempo resistance.
pub mod turn_system;
pub use turn_system::av;
pub use turn_system::resistance;
pub use turn_system::speed;
pub use turn_system::turn_order;

// ─── Combat mechanics ────────────────────────────────────────────────────────
// Damage math, defensive gauges, resources, status/follow-up reactions.

/// Submodule: damage math, defensive gauges, resources, status/follow-up reactions.
pub mod mechanics;
pub use mechanics::buffs;
pub use mechanics::damage;
pub use mechanics::energy;
pub use mechanics::follow_up;
pub use mechanics::modifiers;
pub use mechanics::round_flags;
pub use mechanics::sp;
pub use mechanics::status_effect;
pub use mechanics::stun;
pub use mechanics::toughness;
pub use mechanics::ultimate;

// ─── Enemy & encounter ───────────────────────────────────────────────────────
// Spawn composition, enemy AI, counterplay catalog, per-digimon blueprints.

/// Per-Digimon blueprint routing from RON custom signals into generic kernel transitions.
pub mod blueprints;
/// Encounter setup: spawn composition, enemy AI, counterplay catalog.
pub mod encounter;
pub use encounter::bootstrap;
pub use encounter::counterplay;
pub use encounter::enemy_ai;

// ─── Observability ───────────────────────────────────────────────────────────
// Event bus, structured logs, JSONL dump, validation snapshots, UI signals.

/// Observability: event bus, structured logs, JSONL dump, validation snapshots, UI signals.
pub mod observability;
pub use observability::events;
pub use observability::floating;
pub use observability::jsonl_logger;
pub use observability::log;

/// Bevy `Plugin` wrapper for the full combat runtime (M021).
pub mod plugin;

// ─── Re-exports ──────────────────────────────────────────────────────────────
// Stable shortcuts for the most-imported types.

pub use plugin::CombatPlugin;
pub use preview::query_skill_preview;
pub use status_effect::{StatusBag, StatusEffectKind};
