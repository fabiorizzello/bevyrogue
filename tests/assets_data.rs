//! Aggregated harness for the assets_data domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for RON data files, skill/unit definitions, and roster catalog.

#[path = "assets_data/add_new_digimon_isolation.rs"]
mod add_new_digimon_isolation;
#[path = "assets_data/data_skills_ron_bounce.rs"]
mod data_skills_ron_bounce;
#[path = "assets_data/data_skills_ron_roundtrip.rs"]
mod data_skills_ron_roundtrip;
#[path = "assets_data/data_skills_ron_validation.rs"]
mod data_skills_ron_validation;
#[path = "assets_data/data_units_ron_canonical.rs"]
mod data_units_ron_canonical;
#[path = "assets_data/data_units_ron_roundtrip.rs"]
mod data_units_ron_roundtrip;
#[path = "assets_data/roster_catalog.rs"]
mod roster_catalog;
