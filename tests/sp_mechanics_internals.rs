//! Relocated from `src/combat/mechanics/sp.rs` (R003 — no inline `mod tests` in src/).
//! Pure relocate: all touched symbols are already `pub`.

use bevyrogue::combat::sp::{RoundSpTracker, SpPool};

#[test]
fn sp_pool_default_max_is_5() {
    assert_eq!(SpPool::default().max, 5);
}

#[test]
fn round_sp_tracker_caps_non_basic_at_2() {
    let mut tracker = RoundSpTracker::default();
    // Attempt +3: only +2 allowed
    let gained = tracker.try_gain_non_basic(3);
    assert_eq!(gained, 2);
    // Budget exhausted: further gain returns 0
    let gained2 = tracker.try_gain_non_basic(1);
    assert_eq!(gained2, 0);
}

#[test]
fn round_sp_tracker_reset_restores_full_budget() {
    let mut tracker = RoundSpTracker::default();
    tracker.try_gain_non_basic(2); // exhaust budget
    tracker.reset();
    let gained = tracker.try_gain_non_basic(2);
    assert_eq!(gained, 2);
}

#[test]
fn round_sp_tracker_partial_gain_then_remainder() {
    let mut tracker = RoundSpTracker::default();
    let first = tracker.try_gain_non_basic(1);
    assert_eq!(first, 1);
    let second = tracker.try_gain_non_basic(1);
    assert_eq!(second, 1);
    // Budget now exhausted
    let third = tracker.try_gain_non_basic(1);
    assert_eq!(third, 0);
}
