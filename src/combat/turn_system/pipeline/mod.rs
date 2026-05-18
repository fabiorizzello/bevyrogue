//! M010 action pipeline (WIP). Multi-phase action lifecycle:
//! Declaration → PreApp → App → Resolution.
//!
//! See `.gsd/M010-HANDOFF.md` for integration status. The functions here
//! are the scaffolding; wire-up into the Bevy schedule is incomplete.

mod application;
mod declaration;
mod timeline_exec;

pub(crate) use application::step_app;
pub(crate) use declaration::step_declaration;
pub(crate) use timeline_exec::run_timeline_backed_action;
