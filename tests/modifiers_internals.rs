//! Relocated from `src/combat/mechanics/modifiers.rs` (R003 — no inline `mod tests` in src/).
//! Pure relocate, with one adaptation: the inline tests reached into the private
//! `ModifierChain.terms` field; from outside the crate we use the `terms()` accessor.

use bevyrogue::combat::modifiers::{DamageModifierLedger, ModifierChain, ModifierLayer};
use bevyrogue::combat::types::UnitId;

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
    assert!(!drained.terms().is_empty());
    assert!(!ledger.is_armed(target));
    assert!(ledger.drain(target).terms().is_empty());
}
