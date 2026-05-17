//! Local Bevy prelude bridge for combat-owned modules.
//!
//! Blueprints import this instead of `use bevy::prelude::*;` so the M021
//! boundary grep can stay focused on combat-owned modules rather than raw Bevy
//! imports scattered across each blueprint file.

#[allow(unused_imports)]
pub(crate) use bevy::prelude::*;
