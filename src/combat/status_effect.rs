use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::combat::types::DamageTag;

/// Canon status taxonomy v0 (M017 D004+D009).
/// Re-application follows refresh_max_dur: keep the longer of old/new duration.
/// Per-status semantics (damage ticks, speed delta, cancel probability, ult boost)
/// are implemented in S03–S05; this module carries only the lifecycle skeleton.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffectKind {
    Heated,
    Chilled,
    Paralyzed,
    Slowed,
    Blessed,
    /// Reserved §H.1 — vocabulary anchor for RON/log; no active effect in v0.
    #[allow(dead_code)]
    Burn,
    /// Reserved §H.1 — vocabulary anchor for RON/log; no active effect in v0.
    #[allow(dead_code)]
    Shock,
}

/// Single status instance. Not a Component; owned by `StatusBag`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusInstance {
    pub kind: StatusEffectKind,
    pub duration_remaining: u32,
}

/// Buff or Debuff classification for cleanse targeting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffKind {
    Buff,
    Debuff,
}

/// Returns `BuffKind::Buff` for `Blessed`; `BuffKind::Debuff` for all other variants.
pub fn classify_buff_kind(kind: &StatusEffectKind) -> BuffKind {
    match kind {
        StatusEffectKind::Blessed => BuffKind::Buff,
        _ => BuffKind::Debuff,
    }
}

/// Per-unit status storage. One `StatusInstance` per active `StatusEffectKind`.
/// Re-application uses refresh_max_dur: duration is `max(old, new)`.
///
/// # Apply policy
/// A re-apply that fails the accuracy roll does NOT call `apply` — the `roll_pct(threshold)`
/// gate at `pipeline.rs:725-729` runs *before* `apply`, so a resisted re-apply emits
/// `OnStatusResisted` and leaves duration untouched.
#[derive(Component, Default, Debug, Clone)]
pub struct StatusBag(Vec<StatusInstance>);

impl StatusBag {
    /// Upsert: if `kind` already present keep `max(old_dur, dur)`, else push new instance.
    pub fn apply(&mut self, kind: StatusEffectKind, dur: u32) {
        if let Some(inst) = self.0.iter_mut().find(|i| i.kind == kind) {
            inst.duration_remaining = inst.duration_remaining.max(dur);
        } else {
            self.0.push(StatusInstance { kind, duration_remaining: dur });
        }
    }

    /// Decrement every instance by 1. Returns kinds whose duration reached 0, removes them.
    pub fn tick_all(&mut self) -> Vec<StatusEffectKind> {
        let mut expired = Vec::new();
        for inst in self.0.iter_mut() {
            inst.duration_remaining = inst.duration_remaining.saturating_sub(1);
            if inst.duration_remaining == 0 {
                expired.push(inst.kind.clone());
            }
        }
        self.0.retain(|i| i.duration_remaining > 0);
        expired
    }

    /// Remove every Debuff-classified instance; returns kinds removed. Blessed survives.
    pub fn cleanse_debuffs(&mut self) -> Vec<StatusEffectKind> {
        let mut removed = Vec::new();
        self.0.retain(|inst| {
            if classify_buff_kind(&inst.kind) == BuffKind::Debuff {
                removed.push(inst.kind.clone());
                false
            } else {
                true
            }
        });
        removed
    }

    pub fn has(&self, kind: &StatusEffectKind) -> bool {
        self.0.iter().any(|i| &i.kind == kind)
    }

    pub fn get_dur(&self, kind: &StatusEffectKind) -> Option<u32> {
        self.0.iter().find(|i| &i.kind == kind).map(|i| i.duration_remaining)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &StatusInstance> {
        self.0.iter()
    }
}

/// Returns the damage amplifier percentage for a given status bag and damage tag.
/// 115 when Heated+Fire or Chilled+Ice (canon §H.1); 100 otherwise.
pub fn status_amp_pct(bag: &StatusBag, tag: DamageTag) -> i32 {
    if (bag.has(&StatusEffectKind::Heated) && tag == DamageTag::Fire)
        || (bag.has(&StatusEffectKind::Chilled) && tag == DamageTag::Ice)
    {
        115
    } else {
        100
    }
}

/// Backward-compat shim. Single-instance Component superseded by `StatusBag + StatusInstance`.
/// Remove after T02 migrates all call sites.
#[allow(deprecated)]
#[deprecated(note = "migrate to StatusBag (M017/S02/T02)")]
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEffect {
    pub kind: StatusEffectKind,
    pub duration_remaining: u32,
}

#[allow(deprecated)]
impl StatusEffect {
    pub fn new(kind: StatusEffectKind, duration: u32) -> Self {
        StatusEffect { kind, duration_remaining: duration }
    }
    pub fn refresh(&mut self, new_duration: u32) {
        self.duration_remaining = self.duration_remaining.max(new_duration);
    }
    pub fn tick(&mut self) -> bool {
        self.duration_remaining = self.duration_remaining.saturating_sub(1);
        self.duration_remaining == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_refresh_max_dur_keeps_longer() {
        let mut bag = StatusBag::default();
        bag.apply(StatusEffectKind::Heated, 2);
        bag.apply(StatusEffectKind::Heated, 1);
        assert_eq!(bag.get_dur(&StatusEffectKind::Heated), Some(2));
    }

    #[test]
    fn apply_refresh_max_dur_replaces_with_longer() {
        let mut bag = StatusBag::default();
        bag.apply(StatusEffectKind::Heated, 2);
        bag.apply(StatusEffectKind::Heated, 5);
        assert_eq!(bag.get_dur(&StatusEffectKind::Heated), Some(5));
    }

    #[test]
    fn multi_kind_coexistence() {
        let mut bag = StatusBag::default();
        bag.apply(StatusEffectKind::Heated, 2);
        bag.apply(StatusEffectKind::Chilled, 3);
        bag.apply(StatusEffectKind::Blessed, 1);
        assert_eq!(bag.get_dur(&StatusEffectKind::Heated), Some(2));
        assert_eq!(bag.get_dur(&StatusEffectKind::Chilled), Some(3));
        assert_eq!(bag.get_dur(&StatusEffectKind::Blessed), Some(1));
        assert_eq!(bag.iter().count(), 3);
    }

    #[test]
    fn classify_buff_kind_totality() {
        assert_eq!(classify_buff_kind(&StatusEffectKind::Heated),    BuffKind::Debuff);
        assert_eq!(classify_buff_kind(&StatusEffectKind::Chilled),   BuffKind::Debuff);
        assert_eq!(classify_buff_kind(&StatusEffectKind::Paralyzed), BuffKind::Debuff);
        assert_eq!(classify_buff_kind(&StatusEffectKind::Slowed),    BuffKind::Debuff);
        assert_eq!(classify_buff_kind(&StatusEffectKind::Blessed),   BuffKind::Buff);
        assert_eq!(classify_buff_kind(&StatusEffectKind::Burn),      BuffKind::Debuff);
        assert_eq!(classify_buff_kind(&StatusEffectKind::Shock),     BuffKind::Debuff);
    }

    #[test]
    fn cleanse_debuffs_removes_debuffs_leaves_blessed() {
        let mut bag = StatusBag::default();
        bag.apply(StatusEffectKind::Heated, 2);
        bag.apply(StatusEffectKind::Blessed, 3);
        bag.apply(StatusEffectKind::Paralyzed, 1);
        let removed = bag.cleanse_debuffs();
        assert_eq!(removed.len(), 2);
        assert!(removed.contains(&StatusEffectKind::Heated));
        assert!(removed.contains(&StatusEffectKind::Paralyzed));
        assert!(!bag.has(&StatusEffectKind::Heated));
        assert!(!bag.has(&StatusEffectKind::Paralyzed));
        assert!(bag.has(&StatusEffectKind::Blessed));
        assert_eq!(bag.get_dur(&StatusEffectKind::Blessed), Some(3));
    }

    #[test]
    fn tick_all_returns_expired_and_removes_them() {
        let mut bag = StatusBag::default();
        bag.apply(StatusEffectKind::Heated, 1);
        bag.apply(StatusEffectKind::Chilled, 2);
        let expired = bag.tick_all();
        assert_eq!(expired, vec![StatusEffectKind::Heated]);
        assert!(!bag.has(&StatusEffectKind::Heated));
        assert!(bag.has(&StatusEffectKind::Chilled));
        assert_eq!(bag.get_dur(&StatusEffectKind::Chilled), Some(1));
    }

    #[test]
    fn tick_all_multi_expire() {
        let mut bag = StatusBag::default();
        bag.apply(StatusEffectKind::Heated, 1);
        bag.apply(StatusEffectKind::Slowed, 1);
        bag.apply(StatusEffectKind::Blessed, 3);
        let expired = bag.tick_all();
        assert_eq!(expired.len(), 2);
        assert!(expired.contains(&StatusEffectKind::Heated));
        assert!(expired.contains(&StatusEffectKind::Slowed));
        assert!(bag.has(&StatusEffectKind::Blessed));
        assert_eq!(bag.iter().count(), 1);
    }

    #[test]
    fn status_amp_no_status_returns_100() {
        let bag = StatusBag::default();
        assert_eq!(status_amp_pct(&bag, DamageTag::Fire), 100);
    }

    #[test]
    fn status_amp_heated_fire_returns_115() {
        let mut bag = StatusBag::default();
        bag.apply(StatusEffectKind::Heated, 2);
        assert_eq!(status_amp_pct(&bag, DamageTag::Fire), 115);
    }

    #[test]
    fn status_amp_heated_ice_returns_100() {
        let mut bag = StatusBag::default();
        bag.apply(StatusEffectKind::Heated, 2);
        assert_eq!(status_amp_pct(&bag, DamageTag::Ice), 100);
    }

    #[test]
    fn status_amp_chilled_ice_returns_115() {
        let mut bag = StatusBag::default();
        bag.apply(StatusEffectKind::Chilled, 2);
        assert_eq!(status_amp_pct(&bag, DamageTag::Ice), 115);
    }

    #[test]
    fn is_empty_reflects_state() {
        let mut bag = StatusBag::default();
        assert!(bag.is_empty());
        bag.apply(StatusEffectKind::Burn, 1);
        assert!(!bag.is_empty());
        bag.tick_all();
        assert!(bag.is_empty());
    }
}
