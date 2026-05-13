//! Combat core. Headless-first; UI legge solo eventi/log.
//!
//! Vedi `docs/combat_current.md` per design intent corrente. CLAUDE.md per onboarding.

/// Pure action legality / affordance query vocabulary.
pub mod action_query;
#[allow(dead_code)]
/// Spawn composizione encounter (party + nemici) da `SelectionRequest`.
pub mod bootstrap;
/// Calcolo danno (attribute matchup, resistenze, elementi).
pub mod damage;
/// AI nemica: routing decisioni → `ActionIntent`.
pub mod enemy_ai;
/// Typed enemy counterplay declarations mirrored into runtime snapshots.
pub mod enemy_counterplay;
/// Per-unit Energy component (max 100) + RoundEnergyTracker (10 secondary / 30 external per turn).
pub mod energy;
/// `CombatEvent` / `CombatEventKind` — bus single-source-of-truth.
pub mod events;
/// Logger JSONL su stdout dietro env `BEVYROGUE_JSONL`.
pub mod jsonl_logger;
#[allow(unused_imports)]
pub use action_query::{
    build_snapshot_from_ecs, build_snapshot_from_ecs_with_sp, enabled_target_ids,
    first_enabled_target_id, query_charged_telegraph_affordance, query_enemy_trait_affordances,
};
/// Battle loop redesign: Static/Circuit charge and battery payoff state.
pub mod battery_loop;
/// Per-Digimon blueprint routing from RON custom signals into generic kernel transitions.
pub mod blueprints;
/// Floating damage numbers (component spawnato a hit, decaduto da `decay_floating_damage`).
pub mod floating;
/// Reazioni follow-up FIFO + depth guard.
pub mod follow_up;
/// Holy support loop (Grace / Martyr Light) shared kernel seam.
pub mod holy_support;
/// Revised combat kernel primitives: Tactical Cycle, Strain, Flow, Fatigue, tags, beat IDs, and hooks.
pub mod kernel;
/// `UnitSkills` (basic / skill / ultimate IDs per Unit).
pub mod kit;
/// `ActionLog` ring buffer + `LogEntry` enum.
pub mod log;
/// Validation snapshot per debugging / contract testing.
pub mod observability;
pub mod precision_mind_game;
/// Predator loop (Exploit / Prey Lock / Berserk) shared kernel seam.
pub mod predator_loop;
/// Apply effects: traduce `Action` (resolved) → mutazioni stato + eventi.
pub mod resolution;
/// Deterministic RNG resource (`CombatRng`) — centralises all combat randomness (R019).
pub mod rng;
/// SP pool condiviso (cap 5, gen Basic, +2 extra/round). Vedi D038.
pub mod sp;
/// Speed component + modificatori temporanei (slow/haste).
pub mod speed;
/// `CombatState`, `CombatPhase`, `InFlightAction`.
pub mod state;
/// Buff/debuff con durata; tick a turn end.
pub mod status_effect;
pub mod twin_core;
#[allow(deprecated)]
pub use status_effect::{StatusBag, StatusEffect, StatusEffectKind};
/// Per-unit flags reset each round (Break Seal, etc.).
pub mod round_flags;
/// Component `Stunned { turns_left }`.
pub mod stun;
/// `Team::{Ally, Enemy}` — appartenenza unità.
pub mod team;
pub use round_flags::RoundFlags;
/// Toughness/break gauge (HSR-like).
pub mod toughness;
pub use toughness::ToughnessCategory;
pub mod av;
/// Typed enemy counterplay declarations shared by unit data and future query surfaces.
pub mod counterplay;
/// Tempo Resistance: diminishing returns on repeated Delay effects + MIN_ACTION_THRESHOLD_AV floor.
pub mod resistance;
/// `TurnOrder` action-value queue + `TurnAdvanced` event.
pub mod turn_order;
/// Turn pipeline: `advance_turn_system`, `resolve_action_system`, `check_victory_system`.
pub mod turn_system;
/// `UnitId`, `SkillId`, `Attribute`, `DamageTag`.
pub mod types;
/// Ultimate charge meter + accumulation triggers.
pub mod ultimate;
/// `Unit` component (HP, attribute), markers `Ko`, `Commander`.
pub mod unit;
