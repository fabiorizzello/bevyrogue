use bevyrogue::combat::resolution::select_bounce_hop;
use bevyrogue::combat::{team::Team, types::UnitId};
use bevyrogue::data::skills_ron::{BounceSelector, RepeatPolicy};
use crate::common::resolution_helpers::snap_hp;
use std::collections::HashSet;

// ── select_bounce_hop dispatcher tests ──────────────────────────────────

#[test]
fn bounce_lowest_hp_pct_picks_lowest_pct() {
    // Three enemies: slot 0 @ 500‰, slot 1 @ 300‰, slot 2 @ 800‰
    // LowestHpPctAlive should pick slot 1 (300‰)
    let s = snap_hp(vec![
        (UnitId(10), Team::Enemy, 0, true, 500),
        (UnitId(11), Team::Enemy, 1, true, 300),
        (UnitId(12), Team::Enemy, 2, true, 800),
    ]);
    let already_hit = HashSet::new();
    let result = select_bounce_hop(
        BounceSelector::LowestHpPctAlive,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        None,
    );
    assert_eq!(result, Some(UnitId(11)));
}

#[test]
fn bounce_lowest_hp_pct_tiebreak_slot_asc() {
    // Three enemies all at 500‰; lowest slot_index should win
    let s = snap_hp(vec![
        (UnitId(12), Team::Enemy, 2, true, 500),
        (UnitId(10), Team::Enemy, 0, true, 500),
        (UnitId(11), Team::Enemy, 1, true, 500),
    ]);
    let already_hit = HashSet::new();
    let result = select_bounce_hop(
        BounceSelector::LowestHpPctAlive,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        None,
    );
    assert_eq!(result, Some(UnitId(10))); // slot 0 wins tie
}

#[test]
fn bounce_lowest_hp_pct_excludes_already_hit_no_repeat() {
    // slot 0 @ 100‰ (lowest), slot 1 @ 400‰ — slot 0 already hit → slot 1 wins
    let s = snap_hp(vec![
        (UnitId(10), Team::Enemy, 0, true, 100),
        (UnitId(11), Team::Enemy, 1, true, 400),
    ]);
    let mut already_hit = HashSet::new();
    already_hit.insert(UnitId(10));
    let result = select_bounce_hop(
        BounceSelector::LowestHpPctAlive,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        None,
    );
    assert_eq!(result, Some(UnitId(11)));
}

#[test]
fn bounce_lowest_hp_pct_allow_repeat_can_repick_same() {
    // Only one alive enemy; with NoRepeat it would return None (already in set),
    // but AllowRepeat allows re-selecting it.
    let s = snap_hp(vec![(UnitId(10), Team::Enemy, 0, true, 100)]);
    let mut already_hit = HashSet::new();
    already_hit.insert(UnitId(10));
    let result = select_bounce_hop(
        BounceSelector::LowestHpPctAlive,
        &s,
        &already_hit,
        RepeatPolicy::AllowRepeat,
        Team::Enemy,
        None,
    );
    assert_eq!(
        result,
        Some(UnitId(10)),
        "AllowRepeat should re-pick the only target"
    );

    // Confirm NoRepeat returns None in same scenario
    let result_no_repeat = select_bounce_hop(
        BounceSelector::LowestHpPctAlive,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        None,
    );
    assert_eq!(
        result_no_repeat, None,
        "NoRepeat should return None when only target already hit"
    );
}

#[test]
fn bounce_lowest_hp_pct_empty_pool_returns_none() {
    // No alive enemies at all
    let s = snap_hp(vec![(UnitId(10), Team::Enemy, 0, false, 0)]);
    let already_hit = HashSet::new();
    let result = select_bounce_hop(
        BounceSelector::LowestHpPctAlive,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        None,
    );
    assert_eq!(result, None);
}

#[test]
fn bounce_next_slot_picks_next_above_last() {
    // Last hit slot = 0; candidates: slot 0 (already hit → excluded), slot 1, slot 2
    // NextSlotAlive should pick slot 1 (first slot > 0)
    let s = snap_hp(vec![
        (UnitId(10), Team::Enemy, 0, true, 500),
        (UnitId(11), Team::Enemy, 1, true, 800),
        (UnitId(12), Team::Enemy, 2, true, 300),
    ]);
    let mut already_hit = HashSet::new();
    already_hit.insert(UnitId(10));
    let result = select_bounce_hop(
        BounceSelector::NextSlotAlive,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        Some(0),
    );
    assert_eq!(result, Some(UnitId(11)));
}

#[test]
fn bounce_next_slot_no_slot_above_last_returns_none() {
    // Last hit = slot 2 (highest); no slot > 2 exists → None
    let s = snap_hp(vec![
        (UnitId(10), Team::Enemy, 0, true, 500),
        (UnitId(11), Team::Enemy, 1, true, 500),
        (UnitId(12), Team::Enemy, 2, true, 500),
    ]);
    let mut already_hit = HashSet::new();
    already_hit.insert(UnitId(12));
    let result = select_bounce_hop(
        BounceSelector::NextSlotAlive,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        Some(2),
    );
    assert_eq!(result, None);
}

#[test]
fn bounce_next_slot_no_last_picks_lowest_slot() {
    // No last_slot → pick the alive enemy with the lowest slot_index
    let s = snap_hp(vec![
        (UnitId(12), Team::Enemy, 2, true, 300),
        (UnitId(10), Team::Enemy, 0, true, 800),
        (UnitId(11), Team::Enemy, 1, true, 500),
    ]);
    let already_hit = HashSet::new();
    let result = select_bounce_hop(
        BounceSelector::NextSlotAlive,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        None,
    );
    assert_eq!(result, Some(UnitId(10))); // slot 0
}

#[test]
fn bounce_adj_lowest_picks_adjacent_with_lowest_hp() {
    // Last hit slot = 1; adjacent = slots 0 and 2.
    // slot 0 @ 600‰, slot 2 @ 200‰ → slot 2 wins
    let s = snap_hp(vec![
        (UnitId(10), Team::Enemy, 0, true, 600),
        (UnitId(11), Team::Enemy, 1, true, 500), // last hit, excluded by already_hit
        (UnitId(12), Team::Enemy, 2, true, 200),
    ]);
    let mut already_hit = HashSet::new();
    already_hit.insert(UnitId(11));
    let result = select_bounce_hop(
        BounceSelector::AdjLowest,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        Some(1),
    );
    assert_eq!(result, Some(UnitId(12)));
}

#[test]
fn bounce_adj_lowest_tiebreak_slot_asc() {
    // Last hit slot = 1; both slot 0 and slot 2 at same HP% → slot 0 wins (lower index)
    let s = snap_hp(vec![
        (UnitId(10), Team::Enemy, 0, true, 400),
        (UnitId(11), Team::Enemy, 1, true, 800), // last hit, excluded
        (UnitId(12), Team::Enemy, 2, true, 400),
    ]);
    let mut already_hit = HashSet::new();
    already_hit.insert(UnitId(11));
    let result = select_bounce_hop(
        BounceSelector::AdjLowest,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        Some(1),
    );
    assert_eq!(result, Some(UnitId(10))); // slot 0 wins tie
}

#[test]
fn bounce_adj_lowest_no_adjacent_alive_returns_none() {
    // Last hit slot = 1, but slots 0 and 2 are dead → None
    let s = snap_hp(vec![
        (UnitId(10), Team::Enemy, 0, false, 0),
        (UnitId(11), Team::Enemy, 1, true, 500),
        (UnitId(12), Team::Enemy, 2, false, 0),
    ]);
    let mut already_hit = HashSet::new();
    already_hit.insert(UnitId(11));
    let result = select_bounce_hop(
        BounceSelector::AdjLowest,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        Some(1),
    );
    assert_eq!(result, None);
}

#[test]
fn bounce_ignores_ally_team() {
    // Ally team entries should never be returned regardless of HP
    let s = snap_hp(vec![
        (UnitId(1), Team::Ally, 0, true, 50), // ally, very low HP
        (UnitId(10), Team::Enemy, 0, true, 900),
    ]);
    let already_hit = HashSet::new();
    let result = select_bounce_hop(
        BounceSelector::LowestHpPctAlive,
        &s,
        &already_hit,
        RepeatPolicy::NoRepeat,
        Team::Enemy,
        None,
    );
    assert_eq!(result, Some(UnitId(10)), "ally must never be selected");
}

#[test]
fn bounce_allow_repeat_picks_same_target_twice() {
    // Two enemies; AllowRepeat + LowestHpPct: slot 1 @ 200‰ wins both picks even when it's
    // already in the hit set (simulated by calling the dispatcher twice with it inserted).
    let s = snap_hp(vec![
        (UnitId(10), Team::Enemy, 0, true, 700),
        (UnitId(11), Team::Enemy, 1, true, 200),
    ]);
    let mut already_hit = HashSet::new();

    // First pick
    let first = select_bounce_hop(
        BounceSelector::LowestHpPctAlive,
        &s,
        &already_hit,
        RepeatPolicy::AllowRepeat,
        Team::Enemy,
        None,
    );
    assert_eq!(first, Some(UnitId(11)));
    already_hit.insert(UnitId(11));

    // Second pick — AllowRepeat ignores already_hit, so slot 1 wins again
    let second = select_bounce_hop(
        BounceSelector::LowestHpPctAlive,
        &s,
        &already_hit,
        RepeatPolicy::AllowRepeat,
        Team::Enemy,
        None,
    );
    assert_eq!(
        second,
        Some(UnitId(11)),
        "AllowRepeat: same lowest-HP target can be picked again"
    );
}
