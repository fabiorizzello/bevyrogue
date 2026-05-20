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
    Burn,
    /// Reserved §H.1 — vocabulary anchor for RON/log; no active effect in v0.
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
            self.0.push(StatusInstance {
                kind,
                duration_remaining: dur,
            });
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

    /// Remove up to `count` debuff-classified instances ordered duration-DESC, insertion-idx-ASC
    /// tiebreak. `None` removes all debuffs. Buff-classified entries (e.g. Blessed) are never
    /// touched. Returns the removed kinds in selection order.
    pub fn cleanse_n(&mut self, count: Option<u8>) -> Vec<StatusEffectKind> {
        use std::cmp::Reverse;

        // Build (original_idx, kind, duration) for every debuff entry.
        let mut candidates: Vec<(usize, StatusEffectKind, u32)> = self
            .0
            .iter()
            .enumerate()
            .filter(|(_, inst)| classify_buff_kind(&inst.kind) == BuffKind::Debuff)
            .map(|(idx, inst)| (idx, inst.kind.clone(), inst.duration_remaining))
            .collect();

        // Stable sort: duration DESC, then original idx ASC as tiebreak.
        candidates.sort_by_key(|(idx, _, dur)| (Reverse(*dur), *idx));

        let limit = count.map(|c| c as usize).unwrap_or(usize::MAX);
        candidates.truncate(limit);

        let kinds: Vec<StatusEffectKind> =
            candidates.iter().map(|(_, kind, _)| kind.clone()).collect();

        // Collect sorted removal indices for O(log n) membership check.
        let mut remove_idxs: Vec<usize> = candidates.iter().map(|(idx, _, _)| *idx).collect();
        remove_idxs.sort_unstable();

        // Rebuild the bag, preserving non-removed entries in their original order.
        let old = std::mem::take(&mut self.0);
        self.0 = old
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| remove_idxs.binary_search(idx).is_err())
            .map(|(_, inst)| inst)
            .collect();

        kinds
    }

    /// Remove every Debuff-classified instance; returns kinds removed. Blessed survives.
    // Consumed by tests/status_cleanse_policy.rs and tests/status_blessed_cleanse_immune.rs.
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
        self.0
            .iter()
            .find(|i| &i.kind == kind)
            .map(|i| i.duration_remaining)
    }

    // Public API for emptiness check; not yet consumed by external tests.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &StatusInstance> {
        self.0.iter()
    }
}

/// Returns the speed delta for a Chilled unit: `-(base_speed / 5)` (integer division, rounds
/// toward zero), else 0. Derived read — never mutates SpeedModifier.
pub fn chilled_speed_delta(bag: &StatusBag, base_speed: i32) -> i32 {
    if bag.has(&StatusEffectKind::Chilled) {
        -(base_speed / 5)
    } else {
        0
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
