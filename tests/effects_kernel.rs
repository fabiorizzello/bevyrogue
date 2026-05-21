//! Aggregated harness for the effects_kernel domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts integration tests for kernel `Effect::*` primitives (Cleanse, Heal, Revive).

#[path = "effects_kernel/cleanse_effect.rs"]
mod cleanse_effect;
#[path = "effects_kernel/heal_effect.rs"]
mod heal_effect;
#[path = "effects_kernel/revive_semantics.rs"]
mod revive_semantics;
