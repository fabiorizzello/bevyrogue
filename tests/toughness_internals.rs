//! Unit-level contracts for the pure helpers in `combat::toughness` that the
//! full-pipeline integration tests (`tests/toughness_categories.rs`) cover
//! only indirectly:
//!
//! * `apply_hit` partial-then-break + post-break idempotency (integration only
//!   asserts a single full break in `standard_breaks_in_one_full_hit`),
//! * `apply_hit` non-weak no-break path (no integration counterpart),
//! * `exposes_toughness_affordance` / `can_apply_toughness_damage` helper rules
//!   (no integration counterpart — these guard UI/AI seams, not the break
//!   pipeline),
//! * `classify` precedence ladder (no integration counterpart — pure function
//!   exercised by the resolution pipeline but never directly asserted).
//!
//! The 3 cases already covered by `tests/toughness_categories.rs` (armored
//! two-hit, shielded no-break, break-seal block-then-lift) are intentionally
//! NOT duplicated here.
use bevyrogue::combat::{
    team::Team,
    toughness::{
        DamageKind, Toughness, ToughnessCategory, can_apply_toughness_damage, classify,
        exposes_toughness_affordance,
    },
    types::DamageTag,
};

#[test]
fn apply_hit_weak_drains_and_breaks() {
    let mut t = Toughness::new(50, vec![DamageTag::Ice]);
    // Partial weak hit: does not cross 0
    assert!(!t.apply_hit(DamageTag::Ice, 30, false));
    assert!(!t.broken);
    // Break hit: crosses 0 with a weak element
    assert!(t.apply_hit(DamageTag::Ice, 30, false));
    assert!(t.broken);
    // Idempotent: subsequent hits return false
    assert!(!t.apply_hit(DamageTag::Ice, 100, false));
}

#[test]
fn apply_hit_non_weak_no_break() {
    let mut t = Toughness::new(50, vec![DamageTag::Ice]);
    // Fire is not a weakness — draining to 0 does not set broken
    assert!(!t.apply_hit(DamageTag::Fire, 100, false));
    assert!(!t.broken);
}

#[test]
fn exposes_affordance_only_for_enemies_with_positive_bars() {
    let enemy = Toughness::new(10, vec![]);
    let ally = Toughness::new(10, vec![]);
    assert!(exposes_toughness_affordance(Team::Enemy, Some(&enemy)));
    assert!(!exposes_toughness_affordance(Team::Ally, Some(&ally)));
    assert!(!exposes_toughness_affordance(
        Team::Enemy,
        Some(&Toughness::with_category(
            0,
            vec![],
            ToughnessCategory::Standard
        ))
    ));
}

#[test]
fn can_apply_matches_affordance_rule() {
    let enemy = Toughness::new(10, vec![]);
    assert!(can_apply_toughness_damage(Team::Enemy, Some(&enemy)));
    assert!(!can_apply_toughness_damage(Team::Ally, Some(&enemy)));
    assert!(!can_apply_toughness_damage(Team::Enemy, None));
}

#[test]
fn classify_prefers_break_over_weak() {
    let result = classify(DamageTag::Ice, &[DamageTag::Ice], &[], true);
    assert_eq!(result, DamageKind::Break);
}

#[test]
fn classify_weak_over_resist() {
    // Ice is both a weakness and a resist — Weak wins
    let result = classify(DamageTag::Ice, &[DamageTag::Ice], &[DamageTag::Ice], false);
    assert_eq!(result, DamageKind::Weak);
}

#[test]
fn classify_normal_default() {
    let result = classify(DamageTag::Fire, &[DamageTag::Ice], &[], false);
    assert_eq!(result, DamageKind::Normal);
}

#[test]
fn classify_resist() {
    let result = classify(DamageTag::Fire, &[], &[DamageTag::Fire], false);
    assert_eq!(result, DamageKind::Resist);
}
