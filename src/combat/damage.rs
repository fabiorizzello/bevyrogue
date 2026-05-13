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
///   tag_mod        = 1.25 if tag is a weakness, 0.75 if tag is resisted, else 1.0
///   tri_mod        = triangle_modifiers(attacker.attribute, defender.attribute).dmg_modifier
///   status_amp_mod = status_amp_pct(defender_status, damage_tag) / 100  (1.15 or 1.0)
///   damage         = round(base × tag_mod × tri_mod × break_mod × status_amp_mod)
///
/// No clamp: the multiplicative model is naturally bounded by the discrete modifier set.
pub fn calculate_damage(
    attacker: &Unit,
    attack: &AttackContext,
    defender: &Unit,
    weaknesses: &[DamageTag],
    defender_status: Option<&StatusBag>,
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
        (attack.base_damage as f32 * tag_mod * tri.dmg_modifier * break_mod * amp_mod).round()
            as i32;
    DamageBreakdown {
        final_damage,
        tag_mod_pct: (tag_mod * 100.0).round() as i32,
        triangle_mod_pct: (tri.dmg_modifier * 100.0).round() as i32,
        status_amp_pct: amp_pct,
    }
}

#[cfg(test)]
#[path = "damage_tests.rs"]
mod damage_tests;
