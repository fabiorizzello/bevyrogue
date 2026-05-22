use crate::common::damage_helpers::{atk, make_unit};
use bevyrogue::combat::mechanics::damage::calculate_damage;
use bevyrogue::combat::types::{Attribute, DamageTag};

// ──────────────────────────────────────────────────────────────────────────
// Edge cases
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn base_damage_zero_yields_zero() {
    let a = make_unit(Attribute::Vaccine, vec![]);
    let d = make_unit(Attribute::Virus, vec![]);
    // Even with all modifiers active, base=0 always produces 0
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 0, false),
            &d,
            &[DamageTag::Fire],
            None,
            1.0,
            None
        )
        .final_damage,
        0
    );
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 0, true),
            &d,
            &[DamageTag::Fire],
            None,
            1.0,
            None
        )
        .final_damage,
        0
    );
}

#[test]
fn free_attacker_is_neutral_vs_all() {
    // Free attacker → dmg_modifier = 1.0 for all defender types
    let a = make_unit(Attribute::Free, vec![]);
    for def_attr in [
        Attribute::Data,
        Attribute::Vaccine,
        Attribute::Virus,
        Attribute::Free,
    ] {
        let d = make_unit(def_attr, vec![]);
        assert_eq!(
            calculate_damage(
                &a,
                &atk(DamageTag::Fire, 100, false),
                &d,
                &[],
                None,
                1.0,
                None
            )
            .final_damage,
            100,
            "expected neutral vs {def_attr:?}"
        );
    }
}

#[test]
fn physical_tag_always_neutral_tag_mod() {
    // Physical is not weak/resist by default → tag_mod = 1.0
    let a = make_unit(Attribute::Data, vec![]);
    let d = make_unit(Attribute::Data, vec![]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Physical, 100, false),
            &d,
            &[],
            None,
            1.0,
            None
        )
        .final_damage,
        100
    );
}

#[test]
fn resist_and_triangle_lose_stack_multiplicatively() {
    // resist(0.75) × lose(0.87) = 0.6525; base=100 → 65.25 → 65
    let a = make_unit(Attribute::Virus, vec![]);
    let d = make_unit(Attribute::Vaccine, vec![DamageTag::Fire]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, false),
            &d,
            &[],
            None,
            1.0,
            None
        )
        .final_damage,
        65
    );
}

// ──────────────────────────────────────────────────────────────────────────
// DR integration tests
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn dr_30pct_reduces_damage_multiplicatively() {
    // base=100, neutral, dr=0.30 → factor=0.70 → 70
    use bevyrogue::combat::buffs::DrBag;
    let a = make_unit(Attribute::Data, vec![]);
    let d = make_unit(Attribute::Data, vec![]);
    let mut bag = DrBag::default();
    bag.apply(0.30, 2);
    let bd = calculate_damage(
        &a,
        &atk(DamageTag::Fire, 100, false),
        &d,
        &[],
        None,
        1.0,
        Some(&bag),
    );
    assert_eq!(bd.final_damage, 70);
    assert_eq!(bd.dr_reduction_pct, 30);
}

#[test]
fn dr_100pct_floors_damage_to_zero() {
    // sum_dr=1.0 → factor=0.0 → 0
    use bevyrogue::combat::buffs::DrBag;
    let a = make_unit(Attribute::Data, vec![]);
    let d = make_unit(Attribute::Data, vec![]);
    let mut bag = DrBag::default();
    bag.apply(1.0, 1);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, false),
            &d,
            &[],
            None,
            1.0,
            Some(&bag)
        )
        .final_damage,
        0
    );
}

#[test]
fn dr_over_100pct_still_floors_to_zero() {
    // unclamped sum > 1.0 → factor=max(0,<0)=0 → damage=0
    use bevyrogue::combat::buffs::DrBag;
    let a = make_unit(Attribute::Data, vec![]);
    let d = make_unit(Attribute::Data, vec![]);
    let mut bag = DrBag::default();
    bag.apply(0.6, 1);
    bag.apply(0.6, 1);
    let bd = calculate_damage(
        &a,
        &atk(DamageTag::Fire, 100, false),
        &d,
        &[],
        None,
        1.0,
        Some(&bag),
    );
    assert_eq!(bd.final_damage, 0);
    // dr_reduction_pct is capped at 100 for display purposes
    assert_eq!(bd.dr_reduction_pct, 100);
}
