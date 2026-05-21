//! Aggregated harness for the tempo_toughness domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for tempo resistance, toughness mechanics, and attribute triangle.

#[path = "common/mod.rs"]
mod common;

#[path = "tempo_toughness/tempo_resistance.rs"]
mod tempo_resistance;
#[path = "tempo_toughness/tempo_resistance_internals.rs"]
mod tempo_resistance_internals;
#[path = "tempo_toughness/toughness_categories.rs"]
mod toughness_categories;
#[path = "tempo_toughness/toughness_enemy_only.rs"]
mod toughness_enemy_only;
#[path = "tempo_toughness/toughness_internals.rs"]
mod toughness_internals;
#[path = "tempo_toughness/triangle_matchup.rs"]
mod triangle_matchup;
