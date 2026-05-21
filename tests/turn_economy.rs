//! Aggregated harness for the turn_economy domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for turn system, SP economy, energy, ultimate meter, and streaks.

#[path = "common/mod.rs"]
mod common;

#[path = "turn_economy/combat_resolution_streak.rs"]
mod combat_resolution_streak;
#[path = "turn_economy/energy_internals.rs"]
mod energy_internals;
#[path = "turn_economy/resource_caps.rs"]
mod resource_caps;
#[path = "turn_economy/sp_economy.rs"]
mod sp_economy;
#[path = "turn_economy/sp_mechanics_internals.rs"]
mod sp_mechanics_internals;
#[path = "turn_economy/turn_advance_split.rs"]
mod turn_advance_split;
#[path = "turn_economy/turn_system_av.rs"]
mod turn_system_av;
#[path = "turn_economy/turn_system_internals.rs"]
mod turn_system_internals;
#[path = "turn_economy/ultimate_charge_unit.rs"]
mod ultimate_charge_unit;
#[path = "turn_economy/ultimate_meter.rs"]
mod ultimate_meter;
