//! Combat core. Headless-first; UI legge solo eventi/log.
//!
//! Vedi `docs/combat_current.md` per design intent corrente. CLAUDE.md per onboarding.
//!
//! Modules are grouped by responsibility — declaration order is informational
//! only (Rust does not enforce it), but the buckets below act as a hint for
//! agents and humans navigating the 30+ files in this crate root.

// ─── Framework API (M021) ────────────────────────────────────────────────────
// Generic kernel primitives: Intent, Registry<E>, SignalBus, Clock, CastRng.
// No Digimon-specific names. No bevy::winit / bevy::render / bevy_egui imports.

/// M021 framework primitives: Intent, ExtPoint+Registry, SignalBus, Clock, CastRng.
pub mod api;

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

/// `ActionValue` component + `ActionValueUpdated` message (gauge math).
pub mod av;
/// Shared skill-preview seam for UI/AI consumers.
pub mod preview;
/// Tempo Resistance: diminishing returns on repeated Delay effects + MIN_ACTION_THRESHOLD_AV floor.
pub mod resistance;
/// Apply runtime payloads: traduce `Action` (resolved) → mutazioni stato + eventi.
pub mod resolution;
/// Speed component + modificatori temporanei (slow/haste).
pub mod speed;
/// `TurnOrder` action-value queue + `TurnAdvanced` event.
pub mod turn_order;
/// Turn pipeline: `advance_turn_system`, `resolve_action_system`, `check_victory_system`.
pub mod turn_system;

// ─── Combat mechanics ────────────────────────────────────────────────────────
// Damage math, defensive gauges, resources, status/follow-up reactions.

/// Damage-reduction bag (`DrBag` + `sum_dr`); generic multiplicative DR primitive.
pub mod buffs;
/// Calcolo danno (attribute matchup, resistenze, elementi).
pub mod damage;
/// Per-unit Energy component (max 100) + RoundEnergyTracker (10 secondary / 30 external per turn).
pub mod energy;
/// Reazioni follow-up FIFO + depth guard.
pub mod follow_up;
/// Ordered modifier aggregation and one-shot incoming-damage ledger.
pub mod modifiers;
/// Per-unit flags reset each round (Break Seal, etc.).
pub mod round_flags;
/// SP pool condiviso (cap 5, gen Basic, +2 extra/round). Vedi D038.
pub mod sp;
/// Buff/debuff con durata; tick a turn end.
pub mod status_effect;
/// Component `Stunned { turns_left }`.
pub mod stun;
/// Toughness/break gauge (HSR-like).
pub mod toughness;
/// Ultimate charge meter + accumulation triggers.
pub mod ultimate;

// ─── Enemy & encounter ───────────────────────────────────────────────────────
// Spawn composition, enemy AI, counterplay catalog, per-digimon blueprints.

/// Per-Digimon blueprint routing from RON custom signals into generic kernel transitions.
pub mod blueprints;
/// Spawn composizione encounter (party + nemici) da `SelectionRequest`.
pub mod bootstrap;
/// Typed enemy counterplay declarations shared by unit data and future query surfaces.
pub mod counterplay;
/// AI nemica: routing decisioni → `ActionIntent`.
pub mod enemy_ai;

// ─── Observability ───────────────────────────────────────────────────────────
// Event bus, structured logs, JSONL dump, validation snapshots, UI signals.

/// `CombatEvent` / `CombatEventKind` — bus single-source-of-truth.
pub mod events;
/// Floating damage numbers (component spawnato a hit, decaduto da `decay_floating_damage`).
pub mod floating;
/// Logger JSONL su stdout dietro env `BEVYROGUE_JSONL`.
pub mod jsonl_logger;
/// `ActionLog` ring buffer + `LogEntry` enum.
pub mod log;
/// Validation snapshot per debugging / contract testing.
pub mod observability;

/// Bevy `Plugin` wrapper for the full combat runtime (M021).
pub mod plugin;

// ─── Re-exports ──────────────────────────────────────────────────────────────
// Stable shortcuts for the most-imported types.

pub use plugin::CombatPlugin;
#[allow(unused_imports)]
pub use preview::query_skill_preview;
#[allow(deprecated)]
pub use status_effect::{StatusBag, StatusEffectKind};
