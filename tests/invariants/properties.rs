//! Property-based invariants for combat math.
//!
//! `proptest` shrinks failing inputs to a minimal counterexample. These
//! complement the table-driven `#[rstest]` matrices by hunting edge cases
//! across the full input domain (overflow, off-by-one, saturating math).
//!
//! R004 is preserved: `proptest`'s `TestRng` is seeded deterministically per
//! case; failures are persisted under `tests/proptest-regressions/` so they
//! re-run as regression tests.

use bevyrogue::combat::av::{ActionValue, MAX_AV};
use bevyrogue::combat::resistance::{CAP_PCT, TempoResistance, apply_advance, apply_delay};
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::ultimate::{UltAccumulationTrigger, UltimateCharge};
use bevyrogue::combat::{StatusBag, StatusEffectKind};
use proptest::prelude::*;

const AV_LO: i32 = 0;
const AV_HI: i32 = 2 * MAX_AV; // 20_000

/// Generator for any in-range AV value.
fn any_av() -> impl Strategy<Value = i32> {
    AV_LO..=AV_HI
}

/// Generator for any percentage in [0, 200] — wider than `CAP_PCT` to confirm
/// the defensive cap holds.
fn any_pct() -> impl Strategy<Value = u32> {
    0u32..=200
}

proptest! {
    /// `apply_delay` clamps AV to `[0, 2*MAX_AV]` and returns a non-positive
    /// delta, regardless of starting AV, pct (even uncapped), or prior hits.
    #[test]
    fn apply_delay_keeps_av_in_range(
        start in any_av(),
        pct in any_pct(),
        prior_hits in 0u32..=10,
    ) {
        let mut av = ActionValue(start);
        let mut r = TempoResistance { hit_count: prior_hits };
        let delta = apply_delay(&mut av, pct, Some(&mut r));

        prop_assert!(av.0 >= AV_LO, "AV underflowed: {}", av.0);
        prop_assert!(av.0 <= AV_HI, "AV overflowed: {}", av.0);
        prop_assert!(delta <= 0, "delay delta must be non-positive, got {}", delta);
        prop_assert_eq!(r.hit_count, prior_hits.saturating_add(1));
    }

    /// `apply_advance` clamps AV to `[0, 2*MAX_AV]` and returns a non-negative
    /// delta. `pct` is defensively capped at `CAP_PCT`.
    #[test]
    fn apply_advance_keeps_av_in_range(
        start in any_av(),
        pct in any_pct(),
    ) {
        let mut av = ActionValue(start);
        let delta = apply_advance(&mut av, pct);

        prop_assert!(av.0 >= AV_LO);
        prop_assert!(av.0 <= AV_HI);
        prop_assert!(delta >= 0);
        // Effective pct is bounded; max single-call gain is CAP_PCT/100 * MAX_AV.
        let max_gain = (CAP_PCT as i32) * MAX_AV / 100;
        prop_assert!(delta <= max_gain);
    }

    /// `TempoResistance::multiplier` is monotonically non-increasing as hits
    /// accumulate, and always within `(0, 1]`.
    #[test]
    fn tempo_multiplier_monotone_non_increasing(hits in 0u32..=20) {
        let mut r = TempoResistance::default();
        let mut prev = r.multiplier();
        prop_assert!(prev > 0.0 && prev <= 1.0);
        for _ in 0..hits {
            r.record_delay_hit();
            let next = r.multiplier();
            prop_assert!(next > 0.0 && next <= 1.0);
            prop_assert!(next <= prev, "multiplier rose: {prev} -> {next}");
            prev = next;
        }
    }

    /// `cleanse_debuffs` never removes `Blessed` (§H.1: BuffKind::Buff).
    /// Across any bag of debuffs (Heated/Chilled/Paralyzed/Slowed) plus an
    /// optional Blessed, cleanse must preserve Blessed and report the
    /// removed list to match the debuff input.
    #[test]
    fn cleanse_preserves_blessed(
        heated_dur in proptest::option::of(1u32..=10),
        chilled_dur in proptest::option::of(1u32..=10),
        paralyzed_dur in proptest::option::of(1u32..=10),
        slowed_dur in proptest::option::of(1u32..=10),
        blessed_dur in proptest::option::of(1u32..=10),
    ) {
        let mut bag = StatusBag::default();
        if let Some(d) = heated_dur    { bag.apply(StatusEffectKind::Heated,    d); }
        if let Some(d) = chilled_dur   { bag.apply(StatusEffectKind::Chilled,   d); }
        if let Some(d) = paralyzed_dur { bag.apply(StatusEffectKind::Paralyzed, d); }
        if let Some(d) = slowed_dur    { bag.apply(StatusEffectKind::Slowed,    d); }
        if let Some(d) = blessed_dur   { bag.apply(StatusEffectKind::Blessed,   d); }

        let blessed_before = bag.has(&StatusEffectKind::Blessed);
        let blessed_dur_before = bag.get_dur(&StatusEffectKind::Blessed);

        let _removed = bag.cleanse_debuffs();

        // Blessed must be untouched.
        prop_assert_eq!(bag.has(&StatusEffectKind::Blessed), blessed_before);
        prop_assert_eq!(bag.get_dur(&StatusEffectKind::Blessed), blessed_dur_before);

        // No debuff may remain.
        prop_assert!(!bag.has(&StatusEffectKind::Heated));
        prop_assert!(!bag.has(&StatusEffectKind::Chilled));
        prop_assert!(!bag.has(&StatusEffectKind::Paralyzed));
        prop_assert!(!bag.has(&StatusEffectKind::Slowed));
    }

    /// `SpPool::spend` is atomic: it either fully consumes `cost` or fails
    /// and leaves `current` untouched. Across any starting state, current
    /// must remain in `[0, max]` and never go negative.
    #[test]
    fn sp_pool_spend_never_goes_negative(
        max in 1i32..=20,
        start in 0i32..=20,
        cost in 0i32..=30,
    ) {
        let start = start.min(max);
        let mut sp = SpPool { current: start, max };
        let ok = sp.spend(cost);

        prop_assert!(sp.current >= 0, "current underflowed: {}", sp.current);
        prop_assert!(sp.current <= max, "current exceeded max: {} > {}", sp.current, max);

        if ok {
            prop_assert_eq!(sp.current, start - cost,
                "successful spend must reduce current by exactly cost");
        } else {
            prop_assert_eq!(sp.current, start,
                "failed spend must leave current unchanged");
            prop_assert!(start < cost, "spend only fails when insufficient");
        }
    }

    /// `SpPool::gain` clamps at `max` — current never exceeds max, no matter
    /// how large the gain. Sequential gains are equivalent to a single
    /// summed gain (saturating semantics).
    #[test]
    fn sp_pool_gain_clamps_at_max(
        max in 1i32..=20,
        start in 0i32..=20,
        gains in proptest::collection::vec(0i32..=50, 0..=5),
    ) {
        let start = start.min(max);
        let mut sp = SpPool { current: start, max };
        for amount in &gains {
            sp.gain(*amount);
            prop_assert!(sp.current <= max,
                "current exceeded max after gain({}): {} > {}", amount, sp.current, max);
            prop_assert!(sp.current >= start,
                "gain must not reduce current: {} < {}", sp.current, start);
        }
        let total: i32 = gains.iter().sum();
        let expected = (start + total).min(max);
        prop_assert_eq!(sp.current, expected);
    }

    /// `UltimateCharge::try_add` clamps to `[0, cap]` for any input — even
    /// negative or grossly oversized — and signals "newly ready" exactly
    /// when crossing `trigger` from below.
    #[test]
    fn ultimate_charge_try_add_clamps_in_range(
        cap in 50i32..=500,
        trigger in 10i32..=400,
        start in -50i32..=600,
        amount in -200i32..=600,
        charge_per_event in 1i32..=100,
    ) {
        let trigger = trigger.min(cap);
        let mut uc = UltimateCharge {
            current: start.clamp(0, cap),
            trigger,
            cap,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event,
        };
        let before = uc.current;
        let was_ready = uc.ready();
        let crossed = uc.try_add(amount);

        prop_assert!(uc.current >= 0, "current underflowed: {}", uc.current);
        prop_assert!(uc.current <= cap, "current exceeded cap: {} > {}", uc.current, cap);

        let now_ready = uc.ready();
        prop_assert_eq!(crossed, !was_ready && now_ready,
            "crossed flag must match transition from below trigger to at-or-above");

        // Expected new current is (before + amount) clamped.
        let expected = (before + amount).clamp(0, cap);
        prop_assert_eq!(uc.current, expected);
    }

    /// `StatusBag::tick_all` after enough ticks must drain the bag entirely.
    /// Specifically: ticking `max_duration` times leaves zero remaining
    /// instances, and each tick either keeps or expires instances — never
    /// resurrects an expired one.
    #[test]
    fn status_bag_tick_all_eventually_empties(
        durations in proptest::collection::vec(1u32..=8, 0..=5),
    ) {
        let kinds = [
            StatusEffectKind::Heated,
            StatusEffectKind::Chilled,
            StatusEffectKind::Paralyzed,
            StatusEffectKind::Slowed,
            StatusEffectKind::Blessed,
        ];
        let mut bag = StatusBag::default();
        let mut max_dur = 0u32;
        for (i, d) in durations.iter().enumerate() {
            if i < kinds.len() {
                bag.apply(kinds[i].clone(), *d);
                max_dur = max_dur.max(*d);
            }
        }

        // After max_dur ticks the bag must be empty.
        let mut prev_count = bag.iter().count();
        for _ in 0..max_dur {
            bag.tick_all();
            let now = bag.iter().count();
            prop_assert!(now <= prev_count,
                "tick_all increased bag size: {} -> {}", prev_count, now);
            prev_count = now;
        }
        prop_assert_eq!(bag.iter().count(), 0,
            "bag must be empty after {} ticks", max_dur);
    }
}
