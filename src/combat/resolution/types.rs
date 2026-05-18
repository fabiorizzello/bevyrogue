use std::collections::HashSet;

use crate::combat::{
    StatusEffectKind, status_effect::StatusBag, team::Team, toughness::DamageKind, types::UnitId,
};
use crate::data::skills_ron::{BounceSelector, RepeatPolicy, TargetShape};

/// Emit one `OnSkillCast` per granted free-skill slot, using the provided ally basic skill ids.
/// Callers (e.g. `execute_action_intent`) collect the ally basics and call this; the function is
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionOutcome {
    pub amount: i32,
    pub kind: DamageKind,
    pub broke: bool,
    pub ko: bool,
    pub sp_ok: bool,
    /// True only when the action actually executed (past all guard checks).
    /// False on early returns (KO'd target, commander guard, etc.).
    pub succeeded: bool,
}

impl Default for ResolutionOutcome {
    fn default() -> Self {
        Self {
            amount: 0,
            kind: DamageKind::Normal,
            broke: false,
            ko: false,
            sp_ok: true,
            succeeded: false,
        }
    }
}

/// Lightweight entry in a TargetableSnapshot.
#[derive(Debug, Clone)]
pub struct TargetEntry {
    pub id: UnitId,
    pub team: Team,
    pub slot_index: u8,
    pub alive: bool,
    /// Integer per-mille HP percentage: `(hp_current * 1000) / hp_max`.
    /// Used by `LowestHpPctAlive` and `AdjLowest` selectors.
    /// Set to 0 when hp_max == 0 (dead-from-creation edge case).
    pub hp_per_mille: u32,
}

/// Snapshot of all targetable units for pure target-shape resolution.
/// Built outside the ECS; passed to resolve_targets() and similar helpers.
#[derive(Debug, Clone, Default)]
pub struct TargetableSnapshot {
    pub entries: Vec<TargetEntry>,
}

/// Resolve the full target list for a given shape and primary target.
///
/// - Single / Row / SelfOnly → [primary] (Row/SelfOnly: single-target fallback, fan-out deferred)
/// - Blast → slot ±1 on primary's team, alive only, slot_index ascending
/// - AllEnemies → all alive units on primary's team, slot_index ascending
pub fn resolve_targets(
    shape: &TargetShape,
    primary: UnitId,
    snapshot: &TargetableSnapshot,
) -> Vec<UnitId> {
    let Some(primary_entry) = snapshot.entries.iter().find(|e| e.id == primary) else {
        return vec![];
    };

    match shape {
        TargetShape::Single | TargetShape::Row | TargetShape::SelfOnly => vec![primary],
        TargetShape::AllAllies => {
            let caster_team = primary_entry.team;
            let mut targets: Vec<&TargetEntry> = snapshot
                .entries
                .iter()
                .filter(|e| e.team == caster_team && e.alive)
                .collect();
            targets.sort_by_key(|e| e.slot_index);
            targets.iter().map(|e| e.id).collect()
        }
        TargetShape::Blast => {
            let team = primary_entry.team;
            let slot = primary_entry.slot_index;
            let mut targets: Vec<&TargetEntry> = snapshot
                .entries
                .iter()
                .filter(|e| e.team == team && e.alive && e.slot_index.abs_diff(slot) <= 1)
                .collect();
            targets.sort_by_key(|e| e.slot_index);
            targets.iter().map(|e| e.id).collect()
        }
        TargetShape::AllEnemies => {
            let target_team = primary_entry.team;
            let mut targets: Vec<&TargetEntry> = snapshot
                .entries
                .iter()
                .filter(|e| e.team == target_team && e.alive)
                .collect();
            targets.sort_by_key(|e| e.slot_index);
            targets.iter().map(|e| e.id).collect()
        }
        TargetShape::Bounce {
            hops,
            selector,
            repeat,
        } => {
            let enemy_team = primary_entry.team;
            let mut chain: Vec<UnitId> = Vec::with_capacity(*hops as usize);
            let mut already_hit: HashSet<UnitId> = HashSet::new();
            let mut last_slot: Option<u8> = None;

            for _ in 0..*hops {
                let next = select_bounce_hop(
                    *selector,
                    snapshot,
                    &already_hit,
                    *repeat,
                    enemy_team,
                    last_slot,
                );
                match next {
                    Some(id) => {
                        let entry = snapshot.entries.iter().find(|e| e.id == id).unwrap();
                        last_slot = Some(entry.slot_index);
                        already_hit.insert(id);
                        chain.push(id);
                    }
                    None => break,
                }
            }
            chain
        }
    }
}

// ── Bounce hop selector dispatcher ──────────────────────────────────────────

/// Dispatcher: select the next hop target for a Bounce chain.
///
/// - `selector`: which selection strategy to use.
/// - `snapshot`: current view of all targetable units.
/// - `already_hit`: IDs already hit in this chain.
/// - `policy`: if `NoRepeat`, candidates in `already_hit` are excluded.
///   If `AllowRepeat`, `already_hit` is ignored.
/// - `enemy_team`: only candidates on this team are considered.
/// - `last_target_slot`: slot index of the most-recently-hit unit; used by
///   `NextSlotAlive` and `AdjLowest`. Pass `None` if this is the first hop
///   (both selectors degrade gracefully: `NextSlotAlive` returns any alive
///   enemy, `AdjLowest` also returns any alive enemy with lowest HP%).
pub fn select_bounce_hop(
    selector: BounceSelector,
    snapshot: &TargetableSnapshot,
    already_hit: &HashSet<UnitId>,
    policy: RepeatPolicy,
    enemy_team: Team,
    last_target_slot: Option<u8>,
) -> Option<UnitId> {
    // Build the candidate pool, applying the repeat-policy filter.
    let candidates: Vec<&TargetEntry> = snapshot
        .entries
        .iter()
        .filter(|e| {
            e.team == enemy_team
                && e.alive
                && (policy == RepeatPolicy::AllowRepeat || !already_hit.contains(&e.id))
        })
        .collect();

    match selector {
        BounceSelector::LowestHpPctAlive => select_lowest_hp_pct_alive(&candidates),
        BounceSelector::NextSlotAlive => select_next_slot_alive(&candidates, last_target_slot),
        BounceSelector::AdjLowest => select_adj_lowest(&candidates, last_target_slot),
    }
}

/// `LowestHpPctAlive`: pick the candidate with the lowest HP percentage.
/// HP% is computed as integer per-mille `(hp_current * 1000) / hp_max` to
/// avoid float drift. Tie-break: lowest `slot_index` ascending.
fn select_lowest_hp_pct_alive(candidates: &[&TargetEntry]) -> Option<UnitId> {
    candidates
        .iter()
        .min_by_key(|e| (e.hp_per_mille, e.slot_index))
        .map(|e| e.id)
}

/// `NextSlotAlive`: pick the candidate with the lowest `slot_index` strictly
/// greater than `last_slot`. When `last_slot` is `None`, picks the candidate
/// with the lowest `slot_index` overall (first alive enemy in slot order).
fn select_next_slot_alive(candidates: &[&TargetEntry], last_slot: Option<u8>) -> Option<UnitId> {
    match last_slot {
        None => candidates.iter().min_by_key(|e| e.slot_index).map(|e| e.id),
        Some(last) => candidates
            .iter()
            .filter(|e| e.slot_index > last)
            .min_by_key(|e| e.slot_index)
            .map(|e| e.id),
    }
}

/// `AdjLowest`: pick the candidate adjacent to `last_slot` (|slot - last| <= 1)
/// with the lowest HP%. Tie-break: lowest `slot_index`. When `last_slot` is
/// `None`, falls back to `select_lowest_hp_pct_alive` across all candidates.
fn select_adj_lowest(candidates: &[&TargetEntry], last_slot: Option<u8>) -> Option<UnitId> {
    match last_slot {
        None => select_lowest_hp_pct_alive(candidates),
        Some(last) => {
            let adj: Vec<&TargetEntry> = candidates
                .iter()
                .filter(|e| e.slot_index.abs_diff(last) <= 1)
                .copied()
                .collect();
            select_lowest_hp_pct_alive(&adj)
        }
    }
}

pub fn target_shape_is_executable_now(shape: TargetShape) -> bool {
    matches!(
        shape,
        TargetShape::Single
            | TargetShape::Blast
            | TargetShape::AllEnemies
            | TargetShape::SelfOnly
            | TargetShape::AllAllies
            | TargetShape::Bounce { .. }
    )
}

pub fn target_shape_rejection_reason(shape: TargetShape) -> Option<String> {
    if target_shape_is_executable_now(shape) {
        None
    } else {
        Some(format!("UnimplementedTargetShape:{shape:?}"))
    }
}

/// Build `UnitDied` payload fields from an optional defender StatusBag snapshot.
pub(super) fn ko_payload(bag: Option<&StatusBag>) -> (Vec<StatusEffectKind>, u32) {
    match bag {
        Some(b) => {
            let status_remaining = b.iter().map(|inst| inst.kind.clone()).collect();
            let heated_remaining = b.get_dur(&StatusEffectKind::Heated).unwrap_or(0);
            (status_remaining, heated_remaining)
        }
        None => (vec![], 0),
    }
}
