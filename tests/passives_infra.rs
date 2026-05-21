//! Aggregated harness for the passives_infra domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for passive event infrastructure and reactive canon.

#[path = "common/mod.rs"]
mod common;

#[path = "passives_infra/passive_canon_support.rs"]
mod passive_canon_support;
#[path = "passives_infra/passive_event_filters.rs"]
mod passive_event_filters;
#[path = "passives_infra/passive_reactive_canon.rs"]
mod passive_reactive_canon;
#[path = "passives_infra/passive_runner_internals.rs"]
mod passive_runner_internals;
