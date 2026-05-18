
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::types::DamageTag;

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
    fn shielded_never_breaks_from_toughness_hit() {
        let mut t = Toughness::with_category(50, vec![DamageTag::Ice], ToughnessCategory::Shielded);
        // Even a massive weak hit doesn't break a Shielded unit
        assert!(!t.apply_hit(DamageTag::Ice, 999, false));
        assert!(!t.broken);
        assert_eq!(t.current, 0);
    }

    #[test]
    fn armored_requires_double_damage_to_break() {
        // Armored halves incoming (round up), so 50 toughness needs ~100 raw to break
        let mut t = Toughness::with_category(50, vec![DamageTag::Ice], ToughnessCategory::Armored);
        // 60 raw → effective 30 → current goes 50 → 20 (no break)
        assert!(!t.apply_hit(DamageTag::Ice, 60, false));
        assert!(!t.broken);
        assert_eq!(t.current, 20);
        // 60 raw → effective 30 → current goes 20 → -10 (breaks)
        assert!(t.apply_hit(DamageTag::Ice, 60, false));
        assert!(t.broken);
    }

    #[test]
    fn apply_hit_is_noop_when_break_sealed() {
        let mut t = Toughness::new(50, vec![DamageTag::Ice]);
        let original_current = t.current;
        // Break-sealed: must return false AND leave current unchanged
        assert!(!t.apply_hit(DamageTag::Ice, 999, true));
        assert_eq!(t.current, original_current);
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
}
