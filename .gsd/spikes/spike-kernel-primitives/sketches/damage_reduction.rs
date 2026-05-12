// SKETCH — non-compiled, illustrative only.
// Proposed `src/combat/damage_reduction.rs` implementing canon §02-08 §H.3.
// - Intra-unit (same source): replace-max
// - Cross-unit (different sources): additive
// - Clamp: 0.5 (50%)

use bevy::prelude::*;
use std::collections::HashMap;

use crate::combat::types::UnitId;

/// Identifier of the blueprint owning a DR instance.
/// (Hardcoded as &'static str during M017; promote to enum or interned ID later.)
pub type BlueprintId = &'static str;

#[derive(Debug, Clone)]
pub struct DrInstance {
    pub source_blueprint: BlueprintId, // e.g. "gabumon", "patamon"
    pub source_unit: UnitId,           // the entity that emitted the buff
    pub target: UnitId,
    pub value: f32,                    // 0.0..=1.0
    pub remaining_turns: Option<u32>,  // None = Permanent (Aura)
    pub kind: BuffKind,                // typically BuffKind::DR or BuffKind::Aura
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffKind { Buff, Debuff, DR, Aura, Mark }

#[derive(Resource, Default, Debug)]
pub struct DrRegistry {
    by_target: HashMap<UnitId, Vec<DrInstance>>,
}

impl DrRegistry {
    pub fn add(&mut self, inst: DrInstance) {
        self.by_target.entry(inst.target).or_default().push(inst);
    }

    pub fn tick_durations(&mut self) {
        for vec in self.by_target.values_mut() {
            vec.retain_mut(|inst| match inst.remaining_turns {
                None => true, // Permanent (Aura) — never auto-decays
                Some(0) => false,
                Some(n) => { inst.remaining_turns = Some(n - 1); true }
            });
        }
    }

    /// Canon §H.3 algorithm.
    pub fn compute_dr_for_target(&self, target: UnitId) -> f32 {
        let Some(instances) = self.by_target.get(&target) else { return 0.0; };

        // 1. Group by source_blueprint, take max value per source (intra-unit replace-max).
        let mut per_source: HashMap<BlueprintId, f32> = HashMap::new();
        for inst in instances {
            let entry = per_source.entry(inst.source_blueprint).or_insert(0.0);
            *entry = entry.max(inst.value);
        }

        // 2. Sum across sources (cross-unit additive).
        let total: f32 = per_source.values().sum();

        // 3. Clamp 0.5.
        total.min(0.5)
    }

    /// Remove DR when the source unit dies (canon §H.5 #3 — Aura auto-clear).
    pub fn on_source_died(&mut self, source: UnitId) {
        for vec in self.by_target.values_mut() {
            vec.retain(|inst| inst.source_unit != source);
        }
    }
}

// --- Integration with damage.rs::calculate_damage ---
//
// New signature:
//
//   pub fn calculate_damage(
//       attacker: &Unit,
//       attack: &AttackContext,
//       defender: &Unit,
//       weaknesses: &[DamageTag],
//       dr_registry: &DrRegistry,   // <-- new
//   ) -> DamageBreakdown {
//       let tag_mod = ...;
//       let tri = triangle_modifiers(...);
//       let break_mod = if attack.is_break { 2.0 } else { 1.0 };
//       let dr = dr_registry.compute_dr_for_target(defender.id);
//       let final_damage = (attack.base_damage as f32
//           * tag_mod
//           * tri.dmg_modifier
//           * (1.0 - dr)              // <-- new layer
//           * break_mod
//       ).round() as i32;
//       DamageBreakdown {
//           final_damage,
//           tag_mod_pct,
//           triangle_mod_pct,
//           dr_pct: (dr * 100.0).round() as i32, // <-- new breakdown field
//       }
//   }

// --- M017 roster wiring (canon §H.4) ---
//
//   - Gabumon `fur_cloak`        → DrRegistry.add(DrInstance{ kind:DR,    value: 0.20, source_bp: "gabumon", ... })
//   - Gabumon `blue_cyclone` ult → DrRegistry.add(DrInstance{ kind:DR,    value: 0.30, source_bp: "gabumon", dur: Some(1) })
//   - Patamon  `holy_aegis`      → DrRegistry.add(DrInstance{ kind:Aura,  value: 0.10, source_bp: "patamon", dur: None }) — emitted for every ally on combat-start, cleared on Patamon UnitDied
//
// Intra-unit collapse test (canon §H.3 worked example):
//   fur_cloak (20%) + blue_cyclone (30%) — both source="gabumon" — collapses to max(20,30)=30%.
//
// Cross-unit additive test:
//   patamon holy_aegis (10%) on Gabumon + gabumon fur_cloak (20%) on Gabumon → 10 + 20 = 30%.
//
// Clamp test:
//   five 15% DRs from distinct sources → 75% → clamped to 50%.

// --- Test sketch (tests/dr_stacking.rs) ---
//
//   #[test] fn intra_unit_replace_max() { ... }
//   #[test] fn cross_unit_additive() { ... }
//   #[test] fn clamp_50pct() { ... }
//   #[test] fn aura_clears_on_source_died() { ... }
