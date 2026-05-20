
use bevy::prelude::Component;

use crate::combat::team::Team;
use crate::combat::types::DamageTag;

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
pub enum ToughnessCategory {
    #[default]
    Standard,
    /// Takes half toughness damage (rounded up) — requires ~2x hits to break.
    Armored,
    /// Cannot be broken by ToughnessHit; toughness bar clamps at 0 but broken never flips.
    Shielded,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Toughness {
    pub max: i32,
    pub current: i32,
    pub weaknesses: Vec<DamageTag>,
    pub broken: bool,
    pub category: ToughnessCategory,
}

impl Toughness {
    pub fn new(max: i32, weaknesses: Vec<DamageTag>) -> Self {
        Self {
            max,
            current: max,
            weaknesses,
            broken: false,
            category: ToughnessCategory::Standard,
        }
    }

    pub fn with_category(
        max: i32,
        weaknesses: Vec<DamageTag>,
        category: ToughnessCategory,
    ) -> Self {
        Self {
            max,
            current: max,
            weaknesses,
            broken: false,
            category,
        }
    }

    /// Returns true only on the transition that breaks toughness: current crosses ≤0 on a weak hit.
    ///
    /// `break_sealed`: if true, short-circuits to false without mutating — used by the Break Seal.
    /// Category rules:
    ///   - Shielded: never breaks from ToughnessHit; current clamps at 0 but `broken` stays false.
    ///   - Armored:  effective damage = (amount + 1) / 2 (round up), ~2x hits needed to break.
    ///   - Standard: existing semantics preserved.
    pub fn apply_hit(&mut self, damage_tag: DamageTag, amount: i32, break_sealed: bool) -> bool {
        if self.broken || break_sealed {
            return false;
        }
        match self.category {
            ToughnessCategory::Shielded => {
                self.current = self.current.saturating_sub(amount).max(0);
                false
            }
            ToughnessCategory::Armored => {
                let effective = (amount + 1) / 2;
                let was_positive = self.current > 0;
                self.current -= effective;
                if was_positive && self.current <= 0 && self.weaknesses.contains(&damage_tag) {
                    self.broken = true;
                    return true;
                }
                false
            }
            ToughnessCategory::Standard => {
                let was_positive = self.current > 0;
                self.current -= amount;
                if was_positive && self.current <= 0 && self.weaknesses.contains(&damage_tag) {
                    self.broken = true;
                    return true;
                }
                false
            }
        }
    }
}

/// True when the target should expose a visible break affordance.
///
/// For now, only enemies with a positive toughness bar do.
pub fn exposes_toughness_affordance(team: Team, toughness: Option<&Toughness>) -> bool {
    matches!(team, Team::Enemy) && toughness.is_some_and(|t| t.max > 0)
}

/// Condensed view of the toughness state that is safe to surface to players and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToughnessView {
    pub current: i32,
    pub max: i32,
    pub weaknesses: Vec<DamageTag>,
    pub broken: bool,
}

/// Returns the toughness view only when the target is allowed to expose a break bar.
pub fn visible_toughness(team: Team, toughness: Option<&Toughness>) -> Option<ToughnessView> {
    if exposes_toughness_affordance(team, toughness) {
        toughness.map(|t| ToughnessView {
            current: t.current,
            max: t.max,
            weaknesses: t.weaknesses.clone(),
            broken: t.broken,
        })
    } else {
        None
    }
}

/// True when the target can actually consume toughness damage and potentially break.
///
/// This currently matches the visible affordance rule.
pub fn can_apply_toughness_damage(team: Team, toughness: Option<&Toughness>) -> bool {
    exposes_toughness_affordance(team, toughness)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DamageKind {
    Normal,
    Weak,
    Resist,
    Break,
}

/// Pure classification: no Bevy types. Prefers Break > Weak > Resist > Normal.
pub fn classify(
    attack_tag: DamageTag,
    weaknesses: &[DamageTag],
    resists: &[DamageTag],
    is_break: bool,
) -> DamageKind {
    if is_break {
        return DamageKind::Break;
    }
    if weaknesses.contains(&attack_tag) {
        return DamageKind::Weak;
    }
    if resists.contains(&attack_tag) {
        return DamageKind::Resist;
    }
    DamageKind::Normal
}

