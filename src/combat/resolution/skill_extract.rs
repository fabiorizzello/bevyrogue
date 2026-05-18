use crate::combat::{
    StatusEffectKind,
    kit::UnitSkills,
    state::{ResolvedAction, UltEffect},
    turn_system::ActionIntent,
};
use crate::data::skills_ron::{DamageCurve, Effect, SkillBook};

pub fn resolve_action(
    intent: &ActionIntent,
    kit: &UnitSkills,
    book: Option<&SkillBook>,
) -> Option<ResolvedAction> {
    let skill_id = match intent {
        ActionIntent::Basic { .. } => &kit.basic,
        ActionIntent::Skill { skill_id, .. } => skill_id,
        ActionIntent::Ultimate { .. } => &kit.ultimate,
    };
    let skill = book?.0.iter().find(|skill| &skill.id == skill_id)?;

    let (source, target, ult_effect) = match intent {
        ActionIntent::Basic { attacker, target } => (*attacker, *target, UltEffect::GainFromBasic),
        ActionIntent::Skill {
            attacker, target, ..
        } => (*attacker, *target, UltEffect::None),
        ActionIntent::Ultimate { attacker, target } => (*attacker, *target, UltEffect::Reset),
    };

    Some(ResolvedAction {
        source,
        target,
        skill_id: skill.id.clone(),
        damage_tag: skill.damage_tag,
        base_damage: skill_base_damage(&skill.legacy_ops),
        toughness_damage: skill_toughness_hit(&skill.legacy_ops),
        revive_pct: skill_revive_pct(&skill.legacy_ops),
        heal_pct: skill_heal_pct(&skill.legacy_ops),
        sp_cost: skill.sp_cost,
        ult_effect,
        grant_free_skill_count: skill_grant_free_count(&skill.legacy_ops),
        status_to_apply: skill_apply_status(&skill.legacy_ops),
        advance_pct: skill_advance(&skill.legacy_ops),
        delay_pct: skill_delay(&skill.legacy_ops),
        energy_grant: skill_grant_energy(&skill.legacy_ops),
        self_advance_pct: skill_self_advance(&skill.legacy_ops),
        target_shape: skill.targeting.shape,
        custom_signals: skill.custom_signals.clone(),
        damage_curve: skill_damage_curve(&skill.legacy_ops),
        cleanse_count: skill_cleanse_count(&skill.legacy_ops),
    })
}

pub(super) fn skill_base_damage(legacy_ops: &[Effect]) -> i32 {
    legacy_ops
        .iter()
        .find_map(|effect| match effect {
            Effect::Damage { amount, .. } => Some(*amount),
            _ => None,
        })
        .unwrap_or(0)
}

/// Extract the `DamageCurve` from the first `Effect::Damage` in `legacy_ops`.
/// Returns `DamageCurve::Constant` when no damage effect is found.
pub fn skill_damage_curve(legacy_ops: &[Effect]) -> DamageCurve {
    legacy_ops
        .iter()
        .find_map(|effect| match effect {
            Effect::Damage { per_hop, .. } => Some(per_hop.clone()),
            _ => None,
        })
        .unwrap_or(DamageCurve::Constant)
}

/// Compute the damage for hop `k` (0-based) given `base_damage` and a `DamageCurve`.
///
/// - `Constant`: always returns `base_damage`.
/// - `Falloff { pct }`: hop k = `base_damage * pct^k / 100^k`, i.e. applies `pct/100` repeatedly
///   starting from hop 0. Floored at 1 when `base_damage > 0`.
/// - `PerHop(v)`: returns `v[k]`. Clamps index to last element if `k >= v.len()`; returns 0 if empty.
pub fn compute_hop_damage(base_damage: i32, curve: &DamageCurve, hop: usize) -> i32 {
    match curve {
        DamageCurve::Constant => base_damage,
        DamageCurve::Falloff { pct } => {
            // Apply multiplicative falloff: multiply by pct/100 for each hop after 0.
            let mut dmg = base_damage as f64;
            for _ in 0..hop {
                dmg = dmg * (*pct as f64) / 100.0;
            }
            // Floor at 1 if original base_damage was > 0.
            let result = dmg.floor() as i32;
            if base_damage > 0 {
                result.max(1)
            } else {
                result
            }
        }
        DamageCurve::PerHop(v) => {
            // Clamp index to last element to stay total.
            let idx = hop.min(v.len().saturating_sub(1));
            v.get(idx).copied().unwrap_or(0)
        }
    }
}

fn skill_toughness_hit(legacy_ops: &[Effect]) -> i32 {
    legacy_ops
        .iter()
        .find_map(|effect| match effect {
            Effect::ToughnessHit(amount) => Some(*amount),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_revive_pct(legacy_ops: &[Effect]) -> i32 {
    legacy_ops
        .iter()
        .find_map(|effect| match effect {
            Effect::Revive(pct) => Some(*pct),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_heal_pct(legacy_ops: &[Effect]) -> u32 {
    legacy_ops
        .iter()
        .find_map(|effect| match effect {
            Effect::Heal {
                amount_pct_max_hp, ..
            } => Some(*amount_pct_max_hp),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_grant_free_count(legacy_ops: &[Effect]) -> usize {
    legacy_ops
        .iter()
        .find_map(|effect| match effect {
            Effect::GrantFreeSkill { count } => Some(*count),
            _ => None,
        })
        .unwrap_or(0)
}

/// First ApplyStatus effect in the skill's effect list; first match wins.
fn skill_apply_status(legacy_ops: &[Effect]) -> Option<(StatusEffectKind, u32)> {
    legacy_ops.iter().find_map(|effect| match effect {
        Effect::ApplyStatus { kind, duration } => Some((kind.clone(), *duration)),
        _ => None,
    })
}

fn skill_advance(legacy_ops: &[Effect]) -> u32 {
    legacy_ops
        .iter()
        .find_map(|effect| match effect {
            Effect::AdvanceTurn(amount) => Some((*amount).min(50)),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_delay(legacy_ops: &[Effect]) -> u32 {
    legacy_ops
        .iter()
        .find_map(|effect| match effect {
            Effect::DelayTurn(amount) => Some((*amount).min(50)),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_grant_energy(legacy_ops: &[Effect]) -> i32 {
    legacy_ops
        .iter()
        .find_map(|effect| match effect {
            Effect::GrantEnergy(amount) => Some(*amount),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_self_advance(legacy_ops: &[Effect]) -> i32 {
    legacy_ops
        .iter()
        .find_map(|effect| match effect {
            Effect::SelfAdvance(amount) => Some(*amount),
            _ => None,
        })
        .unwrap_or(0)
}

/// Returns `Some(count)` when the skill carries `Effect::Cleanse`, else `None`.
/// The inner `Option<u8>` is the count field from the effect (None = remove all).
fn skill_cleanse_count(legacy_ops: &[Effect]) -> Option<Option<u8>> {
    legacy_ops.iter().find_map(|effect| match effect {
        Effect::Cleanse { count, .. } => Some(*count),
        _ => None,
    })
}
