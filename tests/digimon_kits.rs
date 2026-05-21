//! Aggregated harness for the digimon_kits domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for individual Digimon blueprint kits and their runtime behaviors.

#[path = "common/mod.rs"]
mod common;

#[path = "digimon_kits/agumon_baby_burner_primary.rs"]
mod agumon_baby_burner_primary;
#[path = "digimon_kits/agumon_baby_burner_reactive.rs"]
mod agumon_baby_burner_reactive;
#[path = "digimon_kits/agumon_energy_gauge.rs"]
mod agumon_energy_gauge;
#[path = "digimon_kits/battery_loop_kernel.rs"]
mod battery_loop_kernel;
#[path = "digimon_kits/bouncing_fire_off_baseline.rs"]
mod bouncing_fire_off_baseline;
#[path = "digimon_kits/dorumon_blueprint.rs"]
mod dorumon_blueprint;
#[path = "digimon_kits/dorumon_predator_runtime.rs"]
mod dorumon_predator_runtime;
#[path = "digimon_kits/holy_support_affordance.rs"]
mod holy_support_affordance;
#[path = "digimon_kits/holy_support_mechanics.rs"]
mod holy_support_mechanics;
#[path = "digimon_kits/holy_support_resolution.rs"]
mod holy_support_resolution;
#[path = "digimon_kits/holy_support_roster_contract.rs"]
mod holy_support_roster_contract;
#[path = "digimon_kits/passive_kitsune_grace.rs"]
mod passive_kitsune_grace;
#[path = "digimon_kits/patamon_blueprint_seam.rs"]
mod patamon_blueprint_seam;
#[path = "digimon_kits/patamon_revive.rs"]
mod patamon_revive;
#[path = "digimon_kits/predator_loop_kernel.rs"]
mod predator_loop_kernel;
#[path = "digimon_kits/renamon_precision_runtime.rs"]
mod renamon_precision_runtime;
#[path = "digimon_kits/tentomon_blueprint.rs"]
mod tentomon_blueprint;
#[path = "digimon_kits/twin_core.rs"]
mod twin_core;
