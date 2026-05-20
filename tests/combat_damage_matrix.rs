mod common;

use bevyrogue::combat::mechanics::damage::calculate_damage;
use bevyrogue::combat::types::{Attribute, DamageTag};
use common::damage_helpers::{atk, make_unit};

// ──────────────────────────────────────────────────────────────────────────
// Matrix: 3 tag-buckets × 3 triangle-buckets × 2 break states = 18 tests
//
// Tag buckets via DamageTag::Fire:
//   weak    → weaknesses = [Fire]; resists = []
//   neutral → weaknesses = [];    resists = []
//   resist  → weaknesses = [];    resists = [Fire]
//
// Triangle buckets:
//   win  → Vaccine attacker vs Virus defender  → dmg_modifier = 1.11
//   tie  → Data attacker vs Data defender      → dmg_modifier = 1.0
//   lose → Virus attacker vs Vaccine defender  → dmg_modifier = 0.87
//
// Formulas (base = 100):
//   weak/win/no_break:   100×1.25×1.11       = 138.75 → 139
//   weak/win/break:      100×1.25×1.11×2     = 277.5  → 278
//   weak/tie/no_break:   100×1.25×1.0        = 125
//   weak/tie/break:      100×1.25×1.0×2      = 250
//   weak/lose/no_break:  100×1.25×0.87       = 108.75 → 109
//   weak/lose/break:     100×1.25×0.87×2     = 217.5  → 218
//   neut/win/no_break:   100×1.0×1.11        = 111
//   neut/win/break:      100×1.0×1.11×2      = 222
//   neut/tie/no_break:   100
//   neut/tie/break:      200
//   neut/lose/no_break:  100×0.87            = 87
//   neut/lose/break:     100×0.87×2          = 174
//   res/win/no_break:    100×0.75×1.11       = 83.25  → 83
//   res/win/break:       100×0.75×1.11×2     = 166.5  → 167
//   res/tie/no_break:    75
//   res/tie/break:       150
//   res/lose/no_break:   100×0.75×0.87       = 65.25  → 65
//   res/lose/break:      100×0.75×0.87×2     = 130.5  → 131
// ──────────────────────────────────────────────────────────────────────────

// ── tag = weak ────────────────────────────────────────────────────────────

#[test]
fn matrix_weak_win_no_break() {
    let a = make_unit(Attribute::Vaccine, vec![]);
    let d = make_unit(Attribute::Virus, vec![]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, false),
            &d,
            &[DamageTag::Fire],
            None,
            1.0,
            None,
        )
        .final_damage,
        139
    );
}

#[test]
fn matrix_weak_win_break() {
    let a = make_unit(Attribute::Vaccine, vec![]);
    let d = make_unit(Attribute::Virus, vec![]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, true),
            &d,
            &[DamageTag::Fire],
            None,
            1.0,
            None
        )
        .final_damage,
        278
    );
}

#[test]
fn matrix_weak_tie_no_break() {
    let a = make_unit(Attribute::Data, vec![]);
    let d = make_unit(Attribute::Data, vec![]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, false),
            &d,
            &[DamageTag::Fire],
            None,
            1.0,
            None,
        )
        .final_damage,
        125
    );
}

#[test]
fn matrix_weak_tie_break() {
    let a = make_unit(Attribute::Data, vec![]);
    let d = make_unit(Attribute::Data, vec![]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, true),
            &d,
            &[DamageTag::Fire],
            None,
            1.0,
            None
        )
        .final_damage,
        250
    );
}

#[test]
fn matrix_weak_lose_no_break() {
    let a = make_unit(Attribute::Virus, vec![]);
    let d = make_unit(Attribute::Vaccine, vec![]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, false),
            &d,
            &[DamageTag::Fire],
            None,
            1.0,
            None,
        )
        .final_damage,
        109
    );
}

#[test]
fn matrix_weak_lose_break() {
    let a = make_unit(Attribute::Virus, vec![]);
    let d = make_unit(Attribute::Vaccine, vec![]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, true),
            &d,
            &[DamageTag::Fire],
            None,
            1.0,
            None
        )
        .final_damage,
        218
    );
}

// ── tag = neutral ─────────────────────────────────────────────────────────

#[test]
fn matrix_neutral_win_no_break() {
    let a = make_unit(Attribute::Vaccine, vec![]);
    let d = make_unit(Attribute::Virus, vec![]);
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
        111
    );
}

#[test]
fn matrix_neutral_win_break() {
    let a = make_unit(Attribute::Vaccine, vec![]);
    let d = make_unit(Attribute::Virus, vec![]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, true),
            &d,
            &[],
            None,
            1.0,
            None
        )
        .final_damage,
        222
    );
}

#[test]
fn matrix_neutral_tie_no_break() {
    let a = make_unit(Attribute::Data, vec![]);
    let d = make_unit(Attribute::Data, vec![]);
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
        100
    );
}

#[test]
fn matrix_neutral_tie_break() {
    let a = make_unit(Attribute::Data, vec![]);
    let d = make_unit(Attribute::Data, vec![]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, true),
            &d,
            &[],
            None,
            1.0,
            None
        )
        .final_damage,
        200
    );
}

#[test]
fn matrix_neutral_lose_no_break() {
    let a = make_unit(Attribute::Virus, vec![]);
    let d = make_unit(Attribute::Vaccine, vec![]);
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
        87
    );
}

#[test]
fn matrix_neutral_lose_break() {
    let a = make_unit(Attribute::Virus, vec![]);
    let d = make_unit(Attribute::Vaccine, vec![]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, true),
            &d,
            &[],
            None,
            1.0,
            None
        )
        .final_damage,
        174
    );
}

// ── tag = resist ──────────────────────────────────────────────────────────

#[test]
fn matrix_resist_win_no_break() {
    let a = make_unit(Attribute::Vaccine, vec![]);
    let d = make_unit(Attribute::Virus, vec![DamageTag::Fire]);
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
        83
    );
}

#[test]
fn matrix_resist_win_break() {
    let a = make_unit(Attribute::Vaccine, vec![]);
    let d = make_unit(Attribute::Virus, vec![DamageTag::Fire]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, true),
            &d,
            &[],
            None,
            1.0,
            None
        )
        .final_damage,
        167
    );
}

#[test]
fn matrix_resist_tie_no_break() {
    let a = make_unit(Attribute::Data, vec![]);
    let d = make_unit(Attribute::Data, vec![DamageTag::Fire]);
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
        75
    );
}

#[test]
fn matrix_resist_tie_break() {
    let a = make_unit(Attribute::Data, vec![]);
    let d = make_unit(Attribute::Data, vec![DamageTag::Fire]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, true),
            &d,
            &[],
            None,
            1.0,
            None
        )
        .final_damage,
        150
    );
}

#[test]
fn matrix_resist_lose_no_break() {
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

#[test]
fn matrix_resist_lose_break() {
    let a = make_unit(Attribute::Virus, vec![]);
    let d = make_unit(Attribute::Vaccine, vec![DamageTag::Fire]);
    assert_eq!(
        calculate_damage(
            &a,
            &atk(DamageTag::Fire, 100, true),
            &d,
            &[],
            None,
            1.0,
            None
        )
        .final_damage,
        131
    );
}
