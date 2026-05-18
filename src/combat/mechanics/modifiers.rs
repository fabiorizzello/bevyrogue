use std::collections::HashMap;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::combat::types::UnitId;

/// Canonical modifier layers, ordered from earliest to latest in the damage pipeline.
///
/// The shared layer stays Digimon-free: callers decide which source contributed the
/// modifier; this module only guarantees a deterministic fold order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ModifierLayer {
    Intrinsic,
    Status,
    Buff,
    Passive,
}

/// One deterministic multiplicative modifier entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifierTerm {
    pub layer: ModifierLayer,
    /// Integer percentage: 100 = neutral, 50 = half damage, 115 = +15%.
    pub multiplier_pct: i32,
    seq: u64,
}

impl ModifierTerm {
    pub fn new(layer: ModifierLayer, multiplier_pct: i32, seq: u64) -> Self {
        Self {
            layer,
            multiplier_pct,
            seq,
        }
    }
}

/// Result of folding a chain of modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedModifiers {
    pub final_amount: i32,
    pub applied_layers: Vec<ModifierLayer>,
}

/// Generic deterministic modifier chain.
///
/// The chain is sorted by canonical layer order, then insertion order inside each
/// layer, so reactive passives cannot reorder status/buff effects by accident.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModifierChain {
    terms: Vec<ModifierTerm>,
    next_seq: u64,
}

impl ModifierChain {
    pub fn push(&mut self, layer: ModifierLayer, multiplier_pct: i32) {
        self.push_term(ModifierTerm::new(layer, multiplier_pct, self.next_seq));
        self.next_seq = self.next_seq.saturating_add(1);
    }

    pub fn push_term(&mut self, term: ModifierTerm) {
        if term.multiplier_pct == 100 {
            return;
        }
        self.next_seq = self.next_seq.max(term.seq.saturating_add(1));
        self.terms.push(term);
    }

    pub fn extend(&mut self, other: ModifierChain) {
        self.next_seq = self.next_seq.max(other.next_seq);
        self.terms.extend(
            other
                .terms
                .into_iter()
                .filter(|term| term.multiplier_pct != 100),
        );
    }

    // Public API for inspecting chain size; not yet consumed by tests.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    pub fn terms(&self) -> &[ModifierTerm] {
        &self.terms
    }

    pub fn apply_to(&self, base_amount: i32) -> AppliedModifiers {
        let mut terms = self.terms.clone();
        terms.sort_by_key(|term| (term.layer, term.seq));

        let mut amount = base_amount;
        let mut applied_layers = Vec::with_capacity(terms.len());
        for term in terms {
            applied_layers.push(term.layer);
            amount = ((amount as f32) * (term.multiplier_pct as f32 / 100.0))
                .round()
                .max(0.0) as i32;
        }

        AppliedModifiers {
            final_amount: amount,
            applied_layers,
        }
    }
}

/// Per-target ledger for one-shot incoming-damage modifiers.
///
/// Passives or other systems arm a modifier here; the damage applier drains the
/// ledger when the hit is resolved, so the modifier is consumed exactly once.
#[derive(Resource, Debug, Default, Clone)]
pub struct DamageModifierLedger {
    next_seq: u64,
    armed: HashMap<UnitId, Vec<ModifierTerm>>,
}

impl DamageModifierLedger {
    /// Arm a new one-shot modifier for a target.
    pub fn arm(&mut self, target: UnitId, layer: ModifierLayer, multiplier_pct: i32) {
        if multiplier_pct == 100 {
            return;
        }
        let term = ModifierTerm::new(layer, multiplier_pct, self.next_seq);
        self.next_seq = self.next_seq.saturating_add(1);
        self.armed.entry(target).or_default().push(term);
    }

    /// Drain all armed modifiers for a target.
    pub fn drain(&mut self, target: UnitId) -> ModifierChain {
        let terms = self.armed.remove(&target).unwrap_or_default();
        let next_seq = terms
            .iter()
            .map(|term| term.seq)
            .max()
            .map(|seq| seq.saturating_add(1))
            .unwrap_or(0);
        ModifierChain { terms, next_seq }
    }

    // Public API to check if a target has pending modifiers; not yet consumed.
    #[allow(dead_code)]
    pub fn is_armed(&self, target: UnitId) -> bool {
        self.armed
            .get(&target)
            .is_some_and(|terms| !terms.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_layer_order_is_canonical() {
        let mut chain = ModifierChain::default();
        chain.push(ModifierLayer::Passive, 50);
        chain.push(ModifierLayer::Status, 115);
        chain.push(ModifierLayer::Buff, 90);

        let applied = chain.apply_to(100);
        assert_eq!(
            applied.applied_layers,
            vec![
                ModifierLayer::Status,
                ModifierLayer::Buff,
                ModifierLayer::Passive
            ]
        );
    }

    #[test]
    fn ledger_drains_once() {
        let mut ledger = DamageModifierLedger::default();
        let target = UnitId(7);
        ledger.arm(target, ModifierLayer::Passive, 50);
        assert!(ledger.is_armed(target));

        let drained = ledger.drain(target);
        assert!(!drained.terms.is_empty());
        assert!(!ledger.is_armed(target));
        assert!(ledger.drain(target).terms.is_empty());
    }
}
