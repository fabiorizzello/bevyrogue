use crate::combat::{
    status_effect::{StatusBag, status_amp_pct},
    types::{Attribute, DamageTag},
    unit::Unit,
};

pub struct AttackContext {
    pub damage_tag: DamageTag,
    pub base_damage: i32,
    pub is_break: bool,
}

pub struct TriangleMods {
    /// Multiplier applied to outgoing damage (attacker perspective).
    /// Attacker wins type cycle → 1.11; defender wins → 0.87; neutral → 1.0.
    /// Asymmetry is intentional (MEM022/D043): winning bonus ≠ losing penalty.
    pub dmg_modifier: f32,
    /// Multiplier applied to toughness damage (reserved; 1.0 for all matchups in v5.3).
    pub tough_modifier: f32,
    /// Multiplier applied to status application accuracy rolls.
    /// Attacker wins → 1.1; defender wins → 0.9; neutral → 1.0.
    pub status_acc_modifier: f32,
}

/// Returns triangle modifiers for `(attacker_attr, defender_attr)`.
///
/// Cycle: Vaccine > Virus > Data > Vaccine. Free is neutral to all.
/// `dmg_modifier` is a single multiplier on outgoing damage (not split into
/// damage-in / damage-out); the attacker-wins / defender-wins asymmetry is
/// deliberate per MEM022: +11% when winning, −13% when losing.
pub fn triangle_modifiers(attacker: Attribute, defender: Attribute) -> TriangleMods {
    let outcome = match (attacker, defender) {
        // Attacker wins the type cycle
        (Attribute::Vaccine, Attribute::Virus)
        | (Attribute::Virus, Attribute::Data)
        | (Attribute::Data, Attribute::Vaccine) => 1,
        // Defender wins the type cycle
        (Attribute::Virus, Attribute::Vaccine)
        | (Attribute::Data, Attribute::Virus)
        | (Attribute::Vaccine, Attribute::Data) => -1,
        // All neutral cases: same vs same, any Free matchup
        _ => 0,
    };
    match outcome {
        1 => TriangleMods {
            dmg_modifier: 1.11,
            tough_modifier: 1.0,
            status_acc_modifier: 1.1,
        },
        -1 => TriangleMods {
            dmg_modifier: 0.87,
            tough_modifier: 1.0,
            status_acc_modifier: 0.9,
        },
        _ => TriangleMods {
            dmg_modifier: 1.0,
            tough_modifier: 1.0,
            status_acc_modifier: 1.0,
        },
    }
}

/// Breakdown returned by `calculate_damage` exposing per-modifier percentages.
///
/// All `_pct` fields are integer percentages (125 = ×1.25, 87 = ×0.87).
pub struct DamageBreakdown {
    pub final_damage: i32,
    /// Tag modifier as integer percentage: 125 (weak), 75 (resist), 100 (neutral).
    pub tag_mod_pct: i32,
    /// Triangle modifier as integer percentage: 111 (attacker loses), 87 (attacker wins), 100 (neutral).
    pub triangle_mod_pct: i32,
    /// Status amplification as integer percentage: 115 (Heated+Fire or Chilled+Ice), 100 otherwise.
    pub status_amp_pct: i32,
}

/// Compute final damage from `attacker` landing `attack` on `defender`.
///
/// Formula (v5.3 multiplicative model):
///   tag_mod           = 1.25 if tag is a weakness, 0.75 if tag is resisted, else 1.0
///   tri_mod           = triangle_modifiers(attacker.attribute, defender.attribute).dmg_modifier
///   status_amp_mod    = status_amp_pct(defender_status, damage_tag) / 100  (1.15 or 1.0)
///   attacker_dmg_mult = 1.15 when attacker has Blessed, else 1.0
///   damage            = round(base × tag_mod × tri_mod × break_mod × status_amp_mod × attacker_dmg_mult)
///
/// No clamp: the multiplicative model is naturally bounded by the discrete modifier set.
pub fn calculate_damage(
    attacker: &Unit,
    attack: &AttackContext,
    defender: &Unit,
    weaknesses: &[DamageTag],
    defender_status: Option<&StatusBag>,
    attacker_dmg_mult: f32,
) -> DamageBreakdown {
    let tag_mod = if weaknesses.contains(&attack.damage_tag) {
        1.25_f32
    } else if defender.resists.contains(&attack.damage_tag) {
        0.75_f32
    } else {
        1.0_f32
    };
    let tri = triangle_modifiers(attacker.attribute, defender.attribute);
    let break_mod = if attack.is_break { 2.0_f32 } else { 1.0_f32 };
    let amp_pct = defender_status
        .map(|bag| status_amp_pct(bag, attack.damage_tag))
        .unwrap_or(100);
    let amp_mod = amp_pct as f32 / 100.0;
    let final_damage =
        (attack.base_damage as f32 * tag_mod * tri.dmg_modifier * break_mod * amp_mod * attacker_dmg_mult).round()
            as i32;
    DamageBreakdown {
        final_damage,
        tag_mod_pct: (tag_mod * 100.0).round() as i32,
        triangle_mod_pct: (tri.dmg_modifier * 100.0).round() as i32,
        status_amp_pct: amp_pct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{
        types::{Attribute, DamageTag, EvoStage, UnitId},
        unit::Unit,
    };

    fn make_unit(attr: Attribute, resists: Vec<DamageTag>) -> Unit {
        Unit {
            id: UnitId(0),
            name: String::new(),
            hp_max: 100,
            hp_current: 100,
            attribute: attr,
            resists,
            evo_stage: EvoStage::Adult,
        }
    }

    fn atk(tag: DamageTag, base: i32, is_break: bool) -> AttackContext {
        AttackContext {
            damage_tag: tag,
            base_damage: base,
            is_break,
        }
    }

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
            calculate_damage(&a, &atk(DamageTag::Fire, 100, true), &d, &[DamageTag::Fire], None, 1.0).final_damage,
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
            calculate_damage(&a, &atk(DamageTag::Fire, 100, true), &d, &[DamageTag::Fire], None, 1.0).final_damage,
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
            calculate_damage(&a, &atk(DamageTag::Fire, 100, true), &d, &[DamageTag::Fire], None, 1.0).final_damage,
            218
        );
    }

    // ── tag = neutral ─────────────────────────────────────────────────────────

    #[test]
    fn matrix_neutral_win_no_break() {
        let a = make_unit(Attribute::Vaccine, vec![]);
        let d = make_unit(Attribute::Virus, vec![]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, false), &d, &[], None, 1.0).final_damage,
            111
        );
    }

    #[test]
    fn matrix_neutral_win_break() {
        let a = make_unit(Attribute::Vaccine, vec![]);
        let d = make_unit(Attribute::Virus, vec![]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, true), &d, &[], None, 1.0).final_damage,
            222
        );
    }

    #[test]
    fn matrix_neutral_tie_no_break() {
        let a = make_unit(Attribute::Data, vec![]);
        let d = make_unit(Attribute::Data, vec![]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, false), &d, &[], None, 1.0).final_damage,
            100
        );
    }

    #[test]
    fn matrix_neutral_tie_break() {
        let a = make_unit(Attribute::Data, vec![]);
        let d = make_unit(Attribute::Data, vec![]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, true), &d, &[], None, 1.0).final_damage,
            200
        );
    }

    #[test]
    fn matrix_neutral_lose_no_break() {
        let a = make_unit(Attribute::Virus, vec![]);
        let d = make_unit(Attribute::Vaccine, vec![]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, false), &d, &[], None, 1.0).final_damage,
            87
        );
    }

    #[test]
    fn matrix_neutral_lose_break() {
        let a = make_unit(Attribute::Virus, vec![]);
        let d = make_unit(Attribute::Vaccine, vec![]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, true), &d, &[], None, 1.0).final_damage,
            174
        );
    }

    // ── tag = resist ──────────────────────────────────────────────────────────

    #[test]
    fn matrix_resist_win_no_break() {
        let a = make_unit(Attribute::Vaccine, vec![]);
        let d = make_unit(Attribute::Virus, vec![DamageTag::Fire]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, false), &d, &[], None, 1.0).final_damage,
            83
        );
    }

    #[test]
    fn matrix_resist_win_break() {
        let a = make_unit(Attribute::Vaccine, vec![]);
        let d = make_unit(Attribute::Virus, vec![DamageTag::Fire]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, true), &d, &[], None, 1.0).final_damage,
            167
        );
    }

    #[test]
    fn matrix_resist_tie_no_break() {
        let a = make_unit(Attribute::Data, vec![]);
        let d = make_unit(Attribute::Data, vec![DamageTag::Fire]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, false), &d, &[], None, 1.0).final_damage,
            75
        );
    }

    #[test]
    fn matrix_resist_tie_break() {
        let a = make_unit(Attribute::Data, vec![]);
        let d = make_unit(Attribute::Data, vec![DamageTag::Fire]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, true), &d, &[], None, 1.0).final_damage,
            150
        );
    }

    #[test]
    fn matrix_resist_lose_no_break() {
        let a = make_unit(Attribute::Virus, vec![]);
        let d = make_unit(Attribute::Vaccine, vec![DamageTag::Fire]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, false), &d, &[], None, 1.0).final_damage,
            65
        );
    }

    #[test]
    fn matrix_resist_lose_break() {
        let a = make_unit(Attribute::Virus, vec![]);
        let d = make_unit(Attribute::Vaccine, vec![DamageTag::Fire]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, true), &d, &[], None, 1.0).final_damage,
            131
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Edge cases
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn base_damage_zero_yields_zero() {
        let a = make_unit(Attribute::Vaccine, vec![]);
        let d = make_unit(Attribute::Virus, vec![]);
        // Even with all modifiers active, base=0 always produces 0
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 0, false), &d, &[DamageTag::Fire], None, 1.0).final_damage,
            0
        );
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 0, true), &d, &[DamageTag::Fire], None, 1.0).final_damage,
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
                calculate_damage(&a, &atk(DamageTag::Fire, 100, false), &d, &[], None, 1.0).final_damage,
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
            calculate_damage(&a, &atk(DamageTag::Physical, 100, false), &d, &[], None, 1.0).final_damage,
            100
        );
    }

    #[test]
    fn resist_and_triangle_lose_stack_multiplicatively() {
        // resist(0.75) × lose(0.87) = 0.6525; base=100 → 65.25 → 65
        let a = make_unit(Attribute::Virus, vec![]);
        let d = make_unit(Attribute::Vaccine, vec![DamageTag::Fire]);
        assert_eq!(
            calculate_damage(&a, &atk(DamageTag::Fire, 100, false), &d, &[], None, 1.0).final_damage,
            65
        );
    }
}
