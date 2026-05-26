//! Aggregated harness for the UI domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Headless-only (NO cfg gate): the cue seam is pure lib logic that must build
//! and pass without the `windowed` feature (R002/R005).

#[path = "ui/cue_registry.rs"]
mod cue_registry;
