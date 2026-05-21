//! Aggregated harness for the invariants domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts property-based invariant tests for combat math and status mechanics.

#[path = "invariants/properties.rs"]
mod properties;
#[path = "invariants/status_paralyzed_skip.rs"]
mod status_paralyzed_skip;
