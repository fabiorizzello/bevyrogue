//! Aggregated harness for the blueprints_infra domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for blueprint signal dispatch and form identity.

#[path = "common/mod.rs"]
mod common;

#[path = "blueprints_infra/blueprint_signal_dispatcher.rs"]
mod blueprint_signal_dispatcher;
#[path = "blueprints_infra/digimon_signal_registry.rs"]
mod digimon_signal_registry;
#[path = "blueprints_infra/form_identity.rs"]
mod form_identity;
