//! Aggregated harness for the runtime_events_obs domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for runtime internals, event bridge/filter, observability, and signal bus.

#[path = "runtime_events_obs/cast_rng_internals.rs"]
mod cast_rng_internals;
#[path = "runtime_events_obs/combat_state_internals.rs"]
mod combat_state_internals;
#[path = "runtime_events_obs/deterministic_rng_contract.rs"]
mod deterministic_rng_contract;
#[path = "runtime_events_obs/event_bridge_internals.rs"]
mod event_bridge_internals;
#[path = "runtime_events_obs/event_filter_internals.rs"]
mod event_filter_internals;
#[path = "runtime_events_obs/event_stream.rs"]
mod event_stream;
#[path = "runtime_events_obs/kernel_internals.rs"]
mod kernel_internals;
#[path = "runtime_events_obs/observability_log_internals.rs"]
mod observability_log_internals;
#[path = "runtime_events_obs/registry_internals.rs"]
mod registry_internals;
#[path = "runtime_events_obs/runtime_builtins_internals.rs"]
mod runtime_builtins_internals;
#[path = "runtime_events_obs/signal_bus_internals.rs"]
mod signal_bus_internals;
#[path = "runtime_events_obs/unit_died_payload.rs"]
mod unit_died_payload;
#[path = "runtime_events_obs/validation_snapshot.rs"]
mod validation_snapshot;
