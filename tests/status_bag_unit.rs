//! Unit-level coverage for `StatusBag` derived reads and `cleanse_n` ordering.
//!
//! Bag lifecycle covered elsewhere:
//!   * `apply` refresh_max_dur ─→ `status_refresh_max_dur.rs`
//!   * `cleanse_debuffs` semantics + `cleanse_n` blessed-immunity ─→ `status_blessed.rs`,
//!     `properties.rs::cleanse_preserves_blessed`
//!   * multi-kind coexistence ─→ `status_multi_kind_coexist.rs`
//!
//! What stays here: pure-function totality (`classify_buff_kind`), tick expiry,
//! `status_amp_pct`, `chilled_speed_delta`, `cleanse_n` ordering & boundary
//! cases (count=Some(0), count > available, idx-asc tiebreak).
//!
//! Relocated from `src/combat/mechanics/status_effect.rs` per R003.

use bevyrogue::combat::status_effect::{
    BuffKind, chilled_speed_delta, classify_buff_kind, status_amp_pct,
};
use bevyrogue::combat::types::DamageTag;
use bevyrogue::combat::{StatusBag, StatusEffectKind};
use rstest::rstest;

// ── classify_buff_kind totality ─────────────────────────────────────────────

#[rstest]
#[case(StatusEffectKind::Heated, BuffKind::Debuff)]
#[case(StatusEffectKind::Chilled, BuffKind::Debuff)]
#[case(StatusEffectKind::Paralyzed, BuffKind::Debuff)]
#[case(StatusEffectKind::Slowed, BuffKind::Debuff)]
#[case(StatusEffectKind::Blessed, BuffKind::Buff)]
#[case(StatusEffectKind::Burn, BuffKind::Debuff)]
#[case(StatusEffectKind::Shock, BuffKind::Debuff)]
fn classify_buff_kind_totality(#[case] kind: StatusEffectKind, #[case] expected: BuffKind) {
    assert_eq!(classify_buff_kind(&kind), expected);
}

// ── tick_all expiry ─────────────────────────────────────────────────────────

#[test]
fn tick_all_returns_expired_and_removes_them() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 1);
    bag.apply(StatusEffectKind::Chilled, 2);
    let expired = bag.tick_all();
    assert_eq!(expired, vec![StatusEffectKind::Heated]);
    assert!(!bag.has(&StatusEffectKind::Heated));
    assert!(bag.has(&StatusEffectKind::Chilled));
    assert_eq!(bag.get_dur(&StatusEffectKind::Chilled), Some(1));
}

#[test]
fn tick_all_multi_expire() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 1);
    bag.apply(StatusEffectKind::Slowed, 1);
    bag.apply(StatusEffectKind::Blessed, 3);
    let expired = bag.tick_all();
    assert_eq!(expired.len(), 2);
    assert!(expired.contains(&StatusEffectKind::Heated));
    assert!(expired.contains(&StatusEffectKind::Slowed));
    assert!(bag.has(&StatusEffectKind::Blessed));
    assert_eq!(bag.iter().count(), 1);
}

#[test]
fn is_empty_reflects_state() {
    let mut bag = StatusBag::default();
    assert!(bag.is_empty());
    bag.apply(StatusEffectKind::Burn, 1);
    assert!(!bag.is_empty());
    bag.tick_all();
    assert!(bag.is_empty());
}

// ── status_amp_pct ──────────────────────────────────────────────────────────

#[rstest]
#[case::no_status(None, DamageTag::Fire, 100)]
#[case::heated_fire(Some(StatusEffectKind::Heated), DamageTag::Fire, 115)]
#[case::heated_ice(Some(StatusEffectKind::Heated), DamageTag::Ice, 100)]
#[case::chilled_ice(Some(StatusEffectKind::Chilled), DamageTag::Ice, 115)]
fn status_amp_pct_matrix(
    #[case] applied: Option<StatusEffectKind>,
    #[case] tag: DamageTag,
    #[case] expected: i32,
) {
    let mut bag = StatusBag::default();
    if let Some(k) = applied {
        bag.apply(k, 2);
    }
    assert_eq!(status_amp_pct(&bag, tag), expected);
}

// ── chilled_speed_delta ─────────────────────────────────────────────────────

#[rstest]
#[case::no_status(false, 100, 0)]
#[case::chilled_base_100(true,  100, -20)]
#[case::chilled_base_80(true,   80, -16)]
fn chilled_speed_delta_matrix(
    #[case] chilled: bool,
    #[case] base_speed: i32,
    #[case] expected: i32,
) {
    let mut bag = StatusBag::default();
    if chilled {
        bag.apply(StatusEffectKind::Chilled, 2);
    }
    assert_eq!(chilled_speed_delta(&bag, base_speed), expected);
}

// ── cleanse_n ordering & boundaries ─────────────────────────────────────────

#[test]
fn cleanse_n_orders_by_duration_desc() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 1); // idx 0, dur 1
    bag.apply(StatusEffectKind::Slowed, 3); // idx 1, dur 3
    bag.apply(StatusEffectKind::Paralyzed, 2); // idx 2, dur 2
    let removed = bag.cleanse_n(Some(2));
    assert_eq!(
        removed,
        vec![StatusEffectKind::Slowed, StatusEffectKind::Paralyzed]
    );
    assert!(bag.has(&StatusEffectKind::Heated));
    assert!(!bag.has(&StatusEffectKind::Slowed));
    assert!(!bag.has(&StatusEffectKind::Paralyzed));
}

#[test]
fn cleanse_n_idx_asc_tiebreak_on_equal_duration() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 2); // idx 0
    bag.apply(StatusEffectKind::Slowed, 2); // idx 1
    bag.apply(StatusEffectKind::Paralyzed, 2); // idx 2
    let removed = bag.cleanse_n(Some(2));
    assert_eq!(
        removed,
        vec![StatusEffectKind::Heated, StatusEffectKind::Slowed]
    );
    assert!(!bag.has(&StatusEffectKind::Heated));
    assert!(!bag.has(&StatusEffectKind::Slowed));
    assert!(bag.has(&StatusEffectKind::Paralyzed));
}

#[test]
fn cleanse_n_none_removes_all_non_immune() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 1);
    bag.apply(StatusEffectKind::Chilled, 2);
    bag.apply(StatusEffectKind::Blessed, 5);
    let removed = bag.cleanse_n(None);
    assert_eq!(removed.len(), 2);
    assert!(removed.contains(&StatusEffectKind::Heated));
    assert!(removed.contains(&StatusEffectKind::Chilled));
    assert!(bag.has(&StatusEffectKind::Blessed));
}

#[test]
fn cleanse_n_some_zero_is_noop() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 3);
    bag.apply(StatusEffectKind::Slowed, 2);
    let removed = bag.cleanse_n(Some(0));
    assert!(removed.is_empty());
    assert!(bag.has(&StatusEffectKind::Heated));
    assert!(bag.has(&StatusEffectKind::Slowed));
}

#[test]
fn cleanse_n_count_exceeds_available_removes_all_without_panic() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 2);
    bag.apply(StatusEffectKind::Slowed, 1);
    let removed = bag.cleanse_n(Some(10));
    assert_eq!(removed.len(), 2);
    assert!(bag.is_empty());
}
