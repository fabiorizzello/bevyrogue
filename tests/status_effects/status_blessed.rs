//! §H.1 Blessed buff — consolidated coverage.
//!
//! Replaces three single-purpose files:
//!   * `status_blessed_offensive.rs`     (×1.15 damage)
//!   * `status_blessed_ult_charge.rs`    (+1 charge / no leak on Reset)
//!   * `status_blessed_cleanse_immune.rs` (Blessed survives cleanse)
//!
//! Helpers live in `tests/common/{units,actions,apply}.rs`.

use bevyrogue::combat::{StatusBag, StatusEffectKind};
use rstest::rstest;

use crate::common::actions::{basic_resolved, ready_ult, ult_resolved};
use crate::common::apply::{ApplyOpts, LegacyOpsHarness, run_damage, run_ult_delta};

// ──────────────────────────────────────────────────────────────────────────────
// §H.1.a — Offensive ×1.15 multiplier
// ──────────────────────────────────────────────────────────────────────────────

fn bag_with(kind: StatusEffectKind, dur: u32) -> StatusBag {
    let mut bag = StatusBag::default();
    bag.apply(kind, dur);
    bag
}

/// Cases: (attacker status, expected damage, reason).
///
/// Baseline base=100, neutral attribute & tag, no resists → modifiers = 1.0;
/// Blessed alone activates ×1.15. Heated must NOT activate the offensive bonus.
#[rstest]
#[case::blessed(Some(bag_with(StatusEffectKind::Blessed, 2)), 115, "Blessed ×1.15")]
#[case::no_bag(None, 100, "no status bag → baseline")]
#[case::empty_bag(Some(StatusBag::default()), 100, "empty bag → no mult")]
#[case::heated_not_blessed(Some(bag_with(StatusEffectKind::Heated, 2)), 100, "Heated ≠ Blessed")]
fn offensive_multiplier(
    #[case] bag: Option<StatusBag>,
    #[case] expected: i32,
    #[case] reason: &str,
) {
    let dmg = run_damage(&basic_resolved(100), bag.as_ref());
    assert_eq!(dmg, expected, "{reason}: expected {expected}, got {dmg}");
}

// ──────────────────────────────────────────────────────────────────────────────
// §H.1.b — +1 ult-charge / no leak on Ult Reset
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn baseline_no_blessed_basic_action() {
    let delta = run_ult_delta(&basic_resolved(100), None);
    assert_eq!(
        delta, 25,
        "baseline delta must equal charge_per_event=25, got {delta}"
    );
}

#[test]
fn blessed_basic_action_gains_extra_charge() {
    let bag = bag_with(StatusEffectKind::Blessed, 2);
    let delta = run_ult_delta(&basic_resolved(100), Some(&bag));
    assert_eq!(
        delta, 26,
        "Blessed Basic must add 1 extra charge (25+1=26), got {delta}"
    );
}

#[test]
fn blessed_ult_cast_no_charge_leak() {
    let bag = bag_with(StatusEffectKind::Blessed, 2);

    // Start with a primed meter so UltEffect::Reset takes the Reset branch.
    let mut harness = LegacyOpsHarness::default().with_ult(ready_ult());
    harness.apply(
        &ult_resolved(100),
        ApplyOpts {
            attacker_status: Some(&bag),
            ..Default::default()
        },
    );

    assert_eq!(
        harness.ult.current, 0,
        "Ult Reset must zero meter; Blessed must not leak +1, got {}",
        harness.ult.current
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// §H.1.c — Cleanse-immunity (Blessed is BuffKind::Buff)
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn blessed_survives_cleanse_when_alone() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Blessed, 3);

    let removed = bag.cleanse_debuffs();

    assert!(
        removed.is_empty(),
        "cleanse must remove nothing when only Blessed is present"
    );
    assert!(bag.has(&StatusEffectKind::Blessed));
    assert_eq!(bag.get_dur(&StatusEffectKind::Blessed), Some(3));
}

#[test]
fn blessed_survives_cleanse_alongside_debuffs() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 2);
    bag.apply(StatusEffectKind::Paralyzed, 1);
    bag.apply(StatusEffectKind::Blessed, 5);

    let removed = bag.cleanse_debuffs();

    assert_eq!(removed.len(), 2, "only debuffs removed");
    assert!(!bag.has(&StatusEffectKind::Heated));
    assert!(!bag.has(&StatusEffectKind::Paralyzed));
    assert!(bag.has(&StatusEffectKind::Blessed));
    assert_eq!(bag.get_dur(&StatusEffectKind::Blessed), Some(5));
}
