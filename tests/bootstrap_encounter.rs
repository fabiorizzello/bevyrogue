//! Aggregated harness for the bootstrap_encounter domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for encounter bootstrap, spawn composition, and end-to-end combat setup.

#[path = "bootstrap_encounter/bootstrap_spawn_composition.rs"]
mod bootstrap_spawn_composition;
#[path = "bootstrap_encounter/combat_cli_shared_surface.rs"]
mod combat_cli_shared_surface;
#[path = "bootstrap_encounter/encounter_bootstrap_internals.rs"]
mod encounter_bootstrap_internals;
#[path = "bootstrap_encounter/encounter_bootstrap_windowed.rs"]
mod encounter_bootstrap_windowed;
#[path = "bootstrap_encounter/encounter_e2e.rs"]
mod encounter_e2e;
#[path = "bootstrap_encounter/party_validation.rs"]
mod party_validation;
#[path = "bootstrap_encounter/slot_index_tiebreak.rs"]
mod slot_index_tiebreak;
