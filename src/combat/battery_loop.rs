//! Tentomon-owned runtime shim.
//!
//! The concrete owner implementation lives under `combat::blueprints`; this
//! root module remains only as a stable import surface for existing callers.

pub use crate::combat::blueprints::tentomon::*;
