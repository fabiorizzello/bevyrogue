//! Action pipeline: the multi-phase action lifecycle driven by
//! `resolve_action_system` (see `../resolve.rs`).
//!
//! - `declaration` — declaration phase (`step_declaration`).
//! - `application` — application phase dispatcher (`step_app`); routes to the
//!   per-target-shape handlers in `paths` (multi-target, bounce, self, single).
//! - `timeline_exec` — execution path for compiled-timeline skills,
//!   including persisted cue barriers (`run_timeline_backed_action`,
//!   `continue_suspended_timeline_system`).

mod application;
mod declaration;
mod paths;
mod timeline_exec;

pub(crate) use application::step_app;
pub(crate) use declaration::step_declaration;
pub use timeline_exec::{continue_suspended_timeline, continue_suspended_timeline_system};
pub(crate) use timeline_exec::run_timeline_backed_action;
