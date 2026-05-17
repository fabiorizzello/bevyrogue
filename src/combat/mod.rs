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

// ─── Core kernel & primitives ────────────────────────────────────────────────
// Shared vocabulary used by every other module.

/// Revised combat kernel primitives: Tactical Cycle, Strain, Flow, Fatigue, tags, beat IDs, and hooks.
pub mod kernel;
/// `UnitId`, `SkillId`, `Attribute`, `DamageTag`.
pub mod types;
/// `Team::{Ally, Enemy}` — appartenenza unità.
pub mod team;
/// `Unit` component (HP, attribute), markers `Ko`, `Commander`.
pub mod unit;
/// `UnitSkills` (basic / skill / ultimate IDs per Unit).
pub mod kit;
/// `CombatState`, `CombatPhase`, `InFlightAction`.
pub mod state;
/// Pure action legality / affordance query vocabulary.
pub mod action_query;
/// Deterministic RNG resource (`CombatRng`) — centralises all combat randomness (R019).
pub mod rng;

// ─── Turn pipeline ───────────────────────────────────────────────────────────
// AV gauge, order queue, intent resolution, speed / tempo modifiers.

/// `TurnOrder` action-value queue + `TurnAdvanced` event.
pub mod turn_order;
/// Turn pipeline: `advance_turn_system`, `resolve_action_system`, `check_victory_system`.
pub mod turn_system;
/// Apply runtime payloads: traduce `Action` (resolved) → mutazioni stato + eventi.
pub mod resolution;
/// `ActionValue` component + `ActionValueUpdated` message (gauge math).
pub mod av;
/// Speed component + modificatori temporanei (slow/haste).
pub mod speed;
/// Tempo Resistance: diminishing returns on repeated Delay effects + MIN_ACTION_THRESHOLD_AV floor.
pub mod resistance;
/// Shared skill-preview seam for UI/AI consumers.
pub mod preview;

// ─── Combat mechanics ────────────────────────────────────────────────────────
// Damage math, defensive gauges, resources, status/follow-up reactions.

/// Calcolo danno (attribute matchup, resistenze, elementi).
pub mod damage;
/// Toughness/break gauge (HSR-like).
pub mod toughness;
/// Component `Stunned { turns_left }`.
pub mod stun;
/// Buff/debuff con durata; tick a turn end.
pub mod status_effect;
/// Damage-reduction bag (`DrBag` + `sum_dr`); generic multiplicative DR primitive.
pub mod buffs;
/// Ordered modifier aggregation and one-shot incoming-damage ledger.
pub mod modifiers;
/// SP pool condiviso (cap 5, gen Basic, +2 extra/round). Vedi D038.
pub mod sp;
/// Ultimate charge meter + accumulation triggers.
pub mod ultimate;
/// Per-unit Energy component (max 100) + RoundEnergyTracker (10 secondary / 30 external per turn).
pub mod energy;
/// Reazioni follow-up FIFO + depth guard.
pub mod follow_up;
/// Per-unit flags reset each round (Break Seal, etc.).
pub mod round_flags;

// ─── Enemy & encounter ───────────────────────────────────────────────────────
// Spawn composition, enemy AI, counterplay catalog, per-digimon blueprints.

/// Spawn composizione encounter (party + nemici) da `SelectionRequest`.
pub mod bootstrap;
/// AI nemica: routing decisioni → `ActionIntent`.
pub mod enemy_ai;
/// Typed enemy counterplay declarations mirrored into runtime snapshots.
pub mod enemy_counterplay;
/// Typed enemy counterplay declarations shared by unit data and future query surfaces.
pub mod counterplay;
/// Per-Digimon blueprint routing from RON custom signals into generic kernel transitions.
pub mod blueprints;

/// Battle loop redesign: Static/Circuit charge and battery payoff state.
pub mod battery_loop;
/// Precision mind game (Renamon): exploit weakness / Kitsune Grace stacks.
pub mod precision_mind_game;

// ─── Observability ───────────────────────────────────────────────────────────
// Event bus, structured logs, JSONL dump, validation snapshots, UI signals.

/// `CombatEvent` / `CombatEventKind` — bus single-source-of-truth.
pub mod events;
/// `ActionLog` ring buffer + `LogEntry` enum.
pub mod log;
/// Validation snapshot per debugging / contract testing.
pub mod observability;
/// Logger JSONL su stdout dietro env `BEVYROGUE_JSONL`.
pub mod jsonl_logger;
/// Floating damage numbers (component spawnato a hit, decaduto da `decay_floating_damage`).
pub mod floating;

/// Bevy `Plugin` wrapper for the full combat runtime (M021).
pub mod plugin;

// ─── Re-exports ──────────────────────────────────────────────────────────────
// Stable shortcuts for the most-imported types.

pub use plugin::CombatPlugin;
#[allow(unused_imports)]
pub use preview::query_skill_preview;
#[allow(deprecated)]
pub use status_effect::{StatusBag, StatusEffectKind};
