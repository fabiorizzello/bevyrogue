//! Aggregated harness for the timeline domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for compiled timeline, boundary contracts, pipeline dispatch, and runtime.

#[path = "common/mod.rs"]
mod common;

#[path = "timeline/boundary_contract.rs"]
mod boundary_contract;
#[path = "timeline/compiled_timeline_active_canon.rs"]
mod compiled_timeline_active_canon;
#[path = "timeline/compiled_timeline_boot_validation.rs"]
mod compiled_timeline_boot_validation;
#[path = "timeline/compiled_timeline_builtin_validation.rs"]
mod compiled_timeline_builtin_validation;
#[path = "timeline/compiled_timeline_runtime_dispatch.rs"]
mod compiled_timeline_runtime_dispatch;
#[path = "timeline/compiled_timeline_runtime_skills.rs"]
mod compiled_timeline_runtime_skills;
#[path = "timeline/perhop_guard.rs"]
mod perhop_guard;
#[path = "timeline/pipeline_dispatch.rs"]
mod pipeline_dispatch;
#[path = "timeline/runtime_runner_internals.rs"]
mod runtime_runner_internals;
#[path = "timeline/timeline_chain_bolt_port.rs"]
mod timeline_chain_bolt_port;
#[path = "timeline/timeline_circuit_breaker.rs"]
mod timeline_circuit_breaker;
#[path = "timeline/timeline_mode_parity.rs"]
mod timeline_mode_parity;
#[path = "timeline/timeline_onturnstart_kills.rs"]
mod timeline_onturnstart_kills;
#[path = "timeline/timeline_cue_barrier_pipeline.rs"]
mod timeline_cue_barrier_pipeline;
#[path = "timeline/timeline_two_clock_parity.rs"]
mod timeline_two_clock_parity;
#[path = "timeline/timeline_validate_loop_internals.rs"]
mod timeline_validate_loop_internals;
