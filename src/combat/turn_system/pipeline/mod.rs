//! Action pipeline: the multi-phase action lifecycle driven by
//! `resolve_action_system` (see `../resolve.rs`).
//!
//! - `declaration` — declaration phase (`step_declaration`).
//! - `application` — application phase dispatcher (`step_app`); routes to the
//!   per-target-shape handlers in `paths` (multi-target, bounce, self, single).
//! - `timeline_exec` — execution path for compiled-timeline skills
//!   (`run_timeline_backed_action`).

mod application;
mod declaration;
mod paths;
mod timeline_exec;

pub(crate) use application::step_app;
pub(crate) use declaration::step_declaration;
pub(crate) use timeline_exec::run_timeline_backed_action;
