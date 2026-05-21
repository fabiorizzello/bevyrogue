//! Aggregated harness for the follow_up domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for follow-up chain semantics, triggers, and trigger internals.

#[path = "common/mod.rs"]
mod common;

#[path = "follow_up/follow_up_chains.rs"]
mod follow_up_chains;
#[path = "follow_up/follow_up_triggers.rs"]
mod follow_up_triggers;
#[path = "follow_up/follow_up_triggers_internals.rs"]
mod follow_up_triggers_internals;
