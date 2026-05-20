//! Relocated from `src/combat/mechanics/energy.rs` (R003 — no inline `mod tests` in src/).
//! Unit tests for `Energy` clamp and `RoundEnergyTracker` per-source caps.

use bevyrogue::combat::mechanics::energy::{Energy, EnergyGainSource, RoundEnergyTracker};

#[test]
fn secondary_cap_at_10() {
    let mut tracker = RoundEnergyTracker::default();
    let gained = tracker.try_gain(EnergyGainSource::SecondaryAction, 15);
    assert_eq!(gained, 10);
    assert_eq!(tracker.secondary_gained, 10);
    let gained2 = tracker.try_gain(EnergyGainSource::SecondaryAction, 5);
    assert_eq!(gained2, 0);
}

#[test]
fn external_cap_at_30() {
    let mut tracker = RoundEnergyTracker::default();
    let gained = tracker.try_gain(EnergyGainSource::External, 50);
    assert_eq!(gained, 30);
    assert_eq!(tracker.external_gained, 30);
    let gained2 = tracker.try_gain(EnergyGainSource::External, 10);
    assert_eq!(gained2, 0);
}

#[test]
fn caps_are_independent() {
    let mut tracker = RoundEnergyTracker::default();
    tracker.try_gain(EnergyGainSource::SecondaryAction, 10);
    tracker.try_gain(EnergyGainSource::External, 30);
    assert_eq!(tracker.secondary_gained, 10);
    assert_eq!(tracker.external_gained, 30);
}

#[test]
fn reset_restores_full_budget() {
    let mut tracker = RoundEnergyTracker::default();
    tracker.try_gain(EnergyGainSource::SecondaryAction, 10);
    tracker.try_gain(EnergyGainSource::External, 30);
    tracker.reset();
    assert_eq!(tracker.secondary_gained, 0);
    assert_eq!(tracker.external_gained, 0);
    let s = tracker.try_gain(EnergyGainSource::SecondaryAction, 5);
    let e = tracker.try_gain(EnergyGainSource::External, 20);
    assert_eq!(s, 5);
    assert_eq!(e, 20);
}

#[test]
fn energy_gain_clamps_at_max() {
    let mut e = Energy::default();
    e.gain(150);
    assert_eq!(e.current, e.max);
    assert_eq!(e.current, 100);
}
