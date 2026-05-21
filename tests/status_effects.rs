//! Aggregated harness for the status_effects domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for status mechanics, buffs, accuracy, and bag operations.

#[path = "common/mod.rs"]
mod common;

#[path = "status_effects/buffs_internals.rs"]
mod buffs_internals;
#[path = "status_effects/modifiers_internals.rs"]
mod modifiers_internals;
#[path = "status_effects/status_accuracy.rs"]
mod status_accuracy;
#[path = "status_effects/status_amp_pipeline.rs"]
mod status_amp_pipeline;
#[path = "status_effects/status_bag_unit.rs"]
mod status_bag_unit;
#[path = "status_effects/status_blessed.rs"]
mod status_blessed;
#[path = "status_effects/status_multi_kind_coexist.rs"]
mod status_multi_kind_coexist;
#[path = "status_effects/status_observability_canon.rs"]
mod status_observability_canon;
#[path = "status_effects/status_refresh_max_dur.rs"]
mod status_refresh_max_dur;
#[path = "status_effects/status_slowed_delay.rs"]
mod status_slowed_delay;
#[path = "status_effects/stun_internals.rs"]
mod stun_internals;
