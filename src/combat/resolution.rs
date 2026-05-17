use std::collections::HashSet;

use crate::combat::{
    StatusEffectKind,
    buffs::DrBag,
    damage::{AttackContext, DamageBreakdown, calculate_damage},
    events::CombatEventKind,
    kit::UnitSkills,
    sp::RoundSpTracker,
    state::{ResolvedAction, UltEffect},
    status_effect::StatusBag,
    team::Team,
    toughness::{DamageKind, Toughness, can_apply_toughness_damage, classify},
    turn_system::ActionIntent,
    types::{EvoStage, SkillId, UnitId},
    ultimate::UltimateCharge,
    unit::{BasicStreak, Unit},
};
use crate::data::skills_ron::{
    BounceSelector, DamageCurve, Effect, RepeatPolicy, SkillBook, TargetShape,
};

/// Emit one `OnSkillCast` per granted free-skill slot, using the provided ally basic skill ids.
/// Callers (e.g. `execute_action_intent`) collect the ally basics and call this; the function is
/// extracted here so resolution unit-tests can exercise it without a Bevy world.
pub fn grant_free_skill_events(count: usize, ally_basics: &[SkillId]) -> Vec<CombatEventKind> {
    ally_basics
        .iter()
        .take(count)
        .map(|skill_id| CombatEventKind::OnSkillCast {
            skill_id: skill_id.clone(),
        })
        .collect()
}

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

fn skill_base_damage(legacy_ops: &[Effect]) -> i32 {
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
fn ko_payload(bag: Option<&StatusBag>) -> (Vec<StatusEffectKind>, u32) {
    match bag {
        Some(b) => {
            let status_remaining = b.iter().map(|inst| inst.kind.clone()).collect();
            let heated_remaining = b.get_dur(&StatusEffectKind::Heated).unwrap_or(0);
            (status_remaining, heated_remaining)
        }
        None => (vec![], 0),
    }
}

/// Apply damage to a single defender without consuming attacker resources (SP, ult, streak).
/// Called in the per-target loop of Blast/AllEnemies fan-out; the caller hoists resource
/// consumption before the loop. Returns per-target events only: OnDamageDealt, OnBreak, UnitDied.
pub fn apply_damage_only(
    resolved: &ResolvedAction,
    attacker_unit: &Unit,
    defender_unit: &mut Unit,
    defender_team: Team,
    mut defender_tough: Option<&mut Toughness>,
    defender_is_commander: bool,
    defender_break_sealed: bool,
    defender_status: Option<&StatusBag>,
    attacker_statuses: Option<&StatusBag>,
    defender_dr: Option<&DrBag>,
) -> (ResolutionOutcome, Vec<CombatEventKind>) {
    if defender_is_commander {
        return (
            ResolutionOutcome::default(),
            vec![CombatEventKind::OnActionFailed {
                reason: "Target is a Commander".to_string(),
            }],
        );
    }
    if attacker_unit.is_ko() {
        return (
            ResolutionOutcome::default(),
            vec![CombatEventKind::OnActionFailed {
                reason: "Attacker is KO".to_string(),
            }],
        );
    }
    // KO'd adjacents are omitted by resolve_targets; guard here for stale-snapshot edge cases.
    if defender_unit.is_ko() {
        return (ResolutionOutcome::default(), vec![]);
    }

    let mut events = Vec::new();
    let mut outcome = ResolutionOutcome::default();

    if resolved.base_damage > 0 || resolved.toughness_damage > 0 {
        let toughness_weaknesses = defender_tough
            .as_deref()
            .map(|t| t.weaknesses.clone())
            .unwrap_or_default();
        let attack = AttackContext {
            damage_tag: resolved.damage_tag,
            base_damage: resolved.base_damage,
            is_break: false,
        };
        let attacker_dmg_mult = attacker_statuses
            .map(|bag| {
                if bag.has(&StatusEffectKind::Blessed) {
                    1.15_f32
                } else {
                    1.0_f32
                }
            })
            .unwrap_or(1.0_f32);
        let DamageBreakdown {
            final_damage: amount,
            tag_mod_pct,
            triangle_mod_pct,
            ..
        } = calculate_damage(
            attacker_unit,
            &attack,
            defender_unit,
            &toughness_weaknesses,
            defender_status,
            attacker_dmg_mult,
            defender_dr,
        );
        defender_unit.hp_current -= amount;
        let broke = if can_apply_toughness_damage(defender_team, defender_tough.as_deref()) {
            defender_tough
                .as_deref_mut()
                .map(|t| {
                    t.apply_hit(
                        resolved.damage_tag,
                        resolved.toughness_damage,
                        defender_break_sealed,
                    )
                })
                .unwrap_or(false)
        } else {
            false
        };
        let kind = classify(
            resolved.damage_tag,
            &toughness_weaknesses,
            &defender_unit.resists,
            broke,
        );
        let ko = defender_unit.hp_current <= 0;

        outcome.amount = amount;
        outcome.kind = kind;
        outcome.broke = broke;
        outcome.ko = ko;

        events.push(CombatEventKind::OnDamageDealt {
            amount,
            kind,
            tag_mod_pct,
            triangle_mod_pct,
            damage_tag: resolved.damage_tag,
        });
        if broke {
            events.push(CombatEventKind::OnBreak {
                damage_tag: resolved.damage_tag,
            });
        }
        if ko {
            let (status_remaining, heated_remaining) = ko_payload(defender_status);
            events.push(CombatEventKind::UnitDied {
                status_remaining,
                heated_remaining,
            });
        }
    }

    outcome.sp_ok = true;
    outcome.succeeded = true;
    (outcome, events)
}

/// Apply heal to a single target. KO targets are silently skipped (no event emitted).
/// Returns per-target events only: OnHealed. Caller hoists resource consumption.
pub fn apply_heal_only(
    resolved: &ResolvedAction,
    defender_unit: &mut Unit,
) -> (ResolutionOutcome, Vec<CombatEventKind>) {
    if defender_unit.is_ko() {
        return (
            ResolutionOutcome {
                sp_ok: true,
                ..ResolutionOutcome::default()
            },
            vec![],
        );
    }

    let hp_max = defender_unit.hp_max as i64;
    let hp_current = defender_unit.hp_current as i64;
    let pct = resolved.heal_pct as i64;
    // Floor division: (hp_max * pct) / 100; capped so hp_current does not exceed hp_max.
    let raw = (hp_max * pct) / 100;
    let healed = raw.min(hp_max - hp_current).max(0) as i32;
    defender_unit.hp_current += healed;
    let hp_after = defender_unit.hp_current;

    let mut outcome = ResolutionOutcome::default();
    outcome.amount = healed;
    outcome.sp_ok = true;
    outcome.succeeded = true;
    (
        outcome,
        vec![CombatEventKind::OnHealed {
            amount: healed,
            hp_after,
        }],
    )
}

/// Apply cleanse to a single target. KO targets are silently skipped (no event emitted).
/// Caller must ensure `action.cleanse_count` is `Some(_)` before calling this.
pub fn apply_cleanse_only(
    action: &ResolvedAction,
    bag: &mut StatusBag,
    defender_alive: bool,
) -> (ResolutionOutcome, Vec<CombatEventKind>) {
    if !defender_alive {
        return (
            ResolutionOutcome {
                sp_ok: true,
                ..ResolutionOutcome::default()
            },
            vec![],
        );
    }
    let inner_count = action
        .cleanse_count
        .expect("apply_cleanse_only called on action without cleanse_count");
    let kinds = bag.cleanse_n(inner_count);
    let outcome = ResolutionOutcome {
        sp_ok: true,
        succeeded: true,
        ..ResolutionOutcome::default()
    };
    (outcome, vec![CombatEventKind::OnCleansed { kinds }])
}

pub fn apply_legacy_ops(
    resolved: &ResolvedAction,
    attacker_unit: &Unit,
    defender_unit: &mut Unit,
    defender_team: Team,
    mut defender_tough: Option<&mut Toughness>,
    attacker_ult: &mut UltimateCharge,
    sp: &mut crate::combat::sp::SpPool,
    _sp_tracker: &mut RoundSpTracker,
    basic_streak: &mut BasicStreak,
    defender_is_commander: bool,
    defender_break_sealed: bool,
    defender_status: Option<&StatusBag>,
    attacker_statuses: Option<&StatusBag>,
    defender_dr: Option<&DrBag>,
) -> (ResolutionOutcome, Vec<CombatEventKind>) {
    let mut events = Vec::new();

    // 1. Validation (Does NOT consume SP on failure)
    if defender_is_commander {
        events.push(CombatEventKind::OnActionFailed {
            reason: "Target is a Commander".to_string(),
        });
        return (ResolutionOutcome::default(), events);
    }

    if attacker_unit.is_ko() {
        events.push(CombatEventKind::OnActionFailed {
            reason: "Attacker is KO".to_string(),
        });
        return (ResolutionOutcome::default(), events);
    }

    // Heal no-op on KO: sp_ok=true, no event, no SP consumed (resources not yet spent here).
    if resolved.heal_pct > 0 {
        if defender_unit.is_ko() {
            return (
                ResolutionOutcome {
                    sp_ok: true,
                    ..ResolutionOutcome::default()
                },
                events,
            );
        }
    } else if resolved.revive_pct > 0 {
        if !defender_unit.is_ko() {
            events.push(CombatEventKind::OnActionFailed {
                reason: "Target is not KO".to_string(),
            });
            return (ResolutionOutcome::default(), events);
        }
    } else if defender_unit.is_ko() {
        events.push(CombatEventKind::OnActionFailed {
            reason: "Target is KO".to_string(),
        });
        return (ResolutionOutcome::default(), events);
    }

    // 2. Resource Consumption
    // Child discount: -1 SP on next Skill after 2+ consecutive Basics
    let effective_sp_cost = if matches!(resolved.ult_effect, UltEffect::None)
        && resolved.sp_cost > 0
        && attacker_unit.evo_stage == EvoStage::Child
        && basic_streak.qualifies_for_discount()
    {
        basic_streak.reset();
        (resolved.sp_cost - 1).max(0)
    } else {
        resolved.sp_cost
    };

    if effective_sp_cost > 0 && !sp.spend(effective_sp_cost) {
        return (
            ResolutionOutcome {
                sp_ok: false,
                ..ResolutionOutcome::default()
            },
            Vec::new(),
        );
    }

    if matches!(resolved.ult_effect, UltEffect::Reset) && !attacker_ult.ready() {
        return (
            ResolutionOutcome {
                sp_ok: false,
                ..ResolutionOutcome::default()
            },
            Vec::new(),
        );
    }

    let mut outcome = ResolutionOutcome::default();

    if resolved.heal_pct > 0 {
        let (heal_outcome, heal_events) = apply_heal_only(resolved, defender_unit);
        outcome.amount = heal_outcome.amount;
        events.extend(heal_events);
    } else if resolved.revive_pct > 0 {
        defender_unit.revive(resolved.revive_pct);
        let hp_after = defender_unit.hp_current;
        events.push(CombatEventKind::OnRevive { hp_after });
        outcome.amount = hp_after;
    } else {
        // Short-circuit damage path for modifier-only skills (e.g. GrantEnergy with no hit).
        if resolved.base_damage > 0 || resolved.toughness_damage > 0 {
            let toughness_weaknesses = defender_tough
                .as_deref()
                .map(|t| t.weaknesses.clone())
                .unwrap_or_default();
            let attack = AttackContext {
                damage_tag: resolved.damage_tag,
                base_damage: resolved.base_damage,
                is_break: false,
            };
            let attacker_dmg_mult = attacker_statuses
                .map(|bag| {
                    if bag.has(&StatusEffectKind::Blessed) {
                        1.15_f32
                    } else {
                        1.0_f32
                    }
                })
                .unwrap_or(1.0_f32);
            let DamageBreakdown {
                final_damage: amount,
                tag_mod_pct,
                triangle_mod_pct,
                status_amp_pct: _status_amp_pct,
                ..
            } = calculate_damage(
                attacker_unit,
                &attack,
                defender_unit,
                &toughness_weaknesses,
                defender_status,
                attacker_dmg_mult,
                defender_dr,
            );
            defender_unit.hp_current -= amount;
            let broke = if can_apply_toughness_damage(defender_team, defender_tough.as_deref()) {
                defender_tough
                    .as_deref_mut()
                    .map(|t| {
                        t.apply_hit(
                            resolved.damage_tag,
                            resolved.toughness_damage,
                            defender_break_sealed,
                        )
                    })
                    .unwrap_or(false)
            } else {
                false
            };
            let kind = classify(
                resolved.damage_tag,
                &toughness_weaknesses,
                &defender_unit.resists,
                broke,
            );
            let ko = defender_unit.hp_current <= 0;

            outcome.amount = amount;
            outcome.kind = kind;
            outcome.broke = broke;
            outcome.ko = ko;

            events.push(CombatEventKind::OnDamageDealt {
                amount,
                kind,
                tag_mod_pct,
                triangle_mod_pct,
                damage_tag: resolved.damage_tag,
            });
            if broke {
                events.push(CombatEventKind::OnBreak {
                    damage_tag: resolved.damage_tag,
                });
            }
            if ko {
                let (status_remaining, heated_remaining) = ko_payload(defender_status);
                events.push(CombatEventKind::UnitDied {
                    status_remaining,
                    heated_remaining,
                });
            }
        }
    }

    match resolved.ult_effect {
        UltEffect::GainFromBasic => {
            sp.gain(1);
            let cpe = attacker_ult.charge_per_event;
            attacker_ult.try_add(cpe);
            basic_streak.increment();
        }
        UltEffect::None => {}
        UltEffect::Reset => {
            attacker_ult.current = 0;
        }
    }

    events.push(CombatEventKind::OnSkillCast {
        skill_id: resolved.skill_id.clone(),
    });

    if resolved.advance_pct != 0 {
        events.push(CombatEventKind::AdvanceTurn {
            target: resolved.target,
            amount_pct: resolved.advance_pct,
        });
    }

    if resolved.delay_pct != 0 {
        events.push(CombatEventKind::DelayTurn {
            target: resolved.target,
            amount_pct: resolved.delay_pct,
        });
    }

    if resolved.self_advance_pct != 0 {
        let capped = (resolved.self_advance_pct.max(0) as u32).min(50);
        if capped != 0 {
            events.push(CombatEventKind::AdvanceTurn {
                target: resolved.source,
                amount_pct: capped,
            });
        }
    }

    outcome.sp_ok = true;
    outcome.succeeded = true;

    // §H.1: Blessed grants +1 Ult charge per action, but not when the action is
    // an Ultimate cast (Reset branch) — skipping avoids self-feeding the firing Ult.
    if resolved.ult_effect != UltEffect::Reset {
        if let Some(bag) = attacker_statuses {
            if bag.has(&StatusEffectKind::Blessed) {
                attacker_ult.try_add(1);
            }
        }
    }

    (outcome, events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{
        sp::{RoundSpTracker, SpPool},
        team::Team,
        toughness::Toughness,
        turn_system::ActionIntent,
        types::{Attribute, DamageTag, EvoStage, UnitId},
        ultimate::UltAccumulationTrigger,
        unit::BasicStreak,
    };
    use crate::data::skills_ron::{
        Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    };

    fn grant_free_skill_def(id: &str, grant_count: usize) -> SkillDef {
        SkillDef {
            id: SkillId(id.into()),
            name: id.into(),
            damage_tag: DamageTag::Light,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![
                Effect::Damage {
                    amount: 30,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
                },
                Effect::ToughnessHit(15),
                Effect::GrantFreeSkill { count: grant_count },
            ],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            ..Default::default()
        }
    }

    fn unit(id: u32, attribute: Attribute, hp_current: i32) -> Unit {
        Unit {
            id: UnitId(id),
            name: format!("Unit{id}"),
            hp_max: 100,
            hp_current,
            attribute,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        }
    }

    fn child_unit(id: u32, attribute: Attribute, hp_current: i32) -> Unit {
        Unit {
            id: UnitId(id),
            name: format!("ChildUnit{id}"),
            hp_max: 100,
            hp_current,
            attribute,
            resists: vec![],
            evo_stage: EvoStage::Child,
        }
    }

    fn skill(
        id: &str,
        damage_tag: DamageTag,
        damage: i32,
        sp_cost: i32,
        toughness_damage: i32,
    ) -> SkillDef {
        SkillDef {
            id: SkillId(id.into()),
            name: id.into(),
            damage_tag,
            sp_cost,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![
                Effect::Damage {
                    amount: damage,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
                },
                Effect::ToughnessHit(toughness_damage),
            ],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            ..Default::default()
        }
    }

    fn revive_skill(id: &str, pct: i32, sp_cost: i32) -> SkillDef {
        SkillDef {
            id: SkillId(id.into()),
            name: id.into(),
            damage_tag: DamageTag::Light,
            sp_cost,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Ally,
                life: TargetLife::Ko,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![Effect::Revive(pct)],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            ..Default::default()
        }
    }

    fn resolved(intent: &ActionIntent, skill: SkillDef) -> ResolvedAction {
        let book = SkillBook(vec![skill.clone()]);
        let kit = UnitSkills {
            basic: skill.id.clone(),
            skills: vec![skill.id.clone()],
            ultimate: skill.id,
            follow_up: None,
        };
        resolve_action(intent, &kit, Some(&book)).expect("skill should resolve")
    }

    fn basic_intent() -> ActionIntent {
        ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(2),
        }
    }

    #[test]
    fn resolve_action_uses_targeting_shape_over_damage_effect_shape() {
        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("row".into()),
            target: UnitId(2),
        };
        let skill = SkillDef {
            id: SkillId("row".into()),
            name: "Row".into(),
            damage_tag: DamageTag::Fire,
            sp_cost: 3,
            targeting: SkillTargeting {
                shape: TargetShape::Row,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Deferred {
                reason: LegalityReasonCode::UnimplementedTargetShape,
            },
            legacy_ops: vec![Effect::Damage {
                amount: 12,
                target: TargetShape::Single,
                per_hop: Default::default(),
            }],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            ..Default::default()
        };

        let resolved = resolved(&intent, skill);

        assert_eq!(resolved.target_shape, TargetShape::Row);
    }

    #[test]
    fn resolve_action_uses_explicit_targeting_shape_for_revive_skills() {
        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("revive".into()),
            target: UnitId(2),
        };
        let skill = revive_skill("revive", 25, 6);

        let expected_shape = skill.targeting.shape;
        let resolved = resolved(&intent, skill);

        assert_eq!(resolved.target_shape, expected_shape);
    }

    #[test]
    fn resolve_apply_basic_adds_sp_and_not_on_skill_cast() {
        let attacker = unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let mut sp = SpPool { current: 3, max: 5 };
        let resolved = resolved(&basic_intent(), skill("basic", DamageTag::Fire, 10, 0, 5));

        let (outcome, events) = apply_legacy_ops(
            &resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut BasicStreak::default(),
            false,
            false,
            None,
            None,
            None,
        );

        assert!(outcome.sp_ok);
        assert_eq!(sp.current, 4);
        assert_eq!(ult.current, 25); // charge_per_event for this UltimateCharge
        assert!(defender.hp_current < 100);
        // Basic attacks now emit both OnDamageDealt and OnSkillCast (same as Skill/Ultimate).
        assert!(matches!(
            events.as_slice(),
            [
                CombatEventKind::OnDamageDealt { .. },
                CombatEventKind::OnSkillCast { .. }
            ]
        ));
    }

    #[test]
    fn resolve_apply_skill_spends_sp_and_emits_on_skill_cast() {
        let attacker = unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let mut sp = SpPool { current: 5, max: 5 };
        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("skill".into()),
            target: UnitId(2),
        };
        let resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 4, 5));

        let (outcome, events) = apply_legacy_ops(
            &resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut BasicStreak::default(),
            false,
            false,
            None,
            None,
            None,
        );

        assert!(outcome.sp_ok);
        assert_eq!(sp.current, 1);
        assert!(events.iter().any(|event| matches!(
            event,
            CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("skill".into())
        )));
    }

    #[test]
    fn resolve_apply_skill_fails_when_pool_too_low() {
        let attacker = unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let mut sp = SpPool { current: 1, max: 5 };
        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("skill".into()),
            target: UnitId(2),
        };
        let resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 4, 5));

        let (outcome, events) = apply_legacy_ops(
            &resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut BasicStreak::default(),
            false,
            false,
            None,
            None,
            None,
        );

        assert!(!outcome.sp_ok);
        assert_eq!(sp.current, 1);
        assert_eq!(defender.hp_current, 100);
        assert!(events.is_empty());
    }

    #[test]
    fn resolve_apply_break_sets_broke_flag_and_on_break_event() {
        let attacker = unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(10, vec![DamageTag::Fire]);
        let mut ult = UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let mut sp = SpPool { current: 3, max: 5 };
        let resolved = resolved(&basic_intent(), skill("basic", DamageTag::Fire, 10, 0, 10));

        let (outcome, events) = apply_legacy_ops(
            &resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut BasicStreak::default(),
            false,
            false,
            None,
            None,
            None,
        );

        assert!(outcome.broke);
        assert_eq!(outcome.kind, DamageKind::Break);
        assert!(tough.broken);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, CombatEventKind::OnBreak { damage_tag } if *damage_tag == DamageTag::Fire))
        );
    }

    #[test]
    fn resolve_apply_no_break_no_on_break_event() {
        let attacker = unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Fire]);
        let mut ult = UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let mut sp = SpPool { current: 3, max: 5 };
        let resolved = resolved(&basic_intent(), skill("basic", DamageTag::Fire, 10, 0, 5));

        let (outcome, events) = apply_legacy_ops(
            &resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut BasicStreak::default(),
            false,
            false,
            None,
            None,
            None,
        );

        assert!(!outcome.broke);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, CombatEventKind::OnBreak { .. }))
        );
    }

    #[test]
    fn resolve_apply_ko_flag_when_hp_drops_below_zero_and_emits_on_ko() {
        let attacker = unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 5);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let mut sp = SpPool { current: 3, max: 5 };
        let resolved = resolved(&basic_intent(), skill("basic", DamageTag::Fire, 10, 0, 5));

        let (outcome, events) = apply_legacy_ops(
            &resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut BasicStreak::default(),
            false,
            false,
            None,
            None,
            None,
        );

        assert!(outcome.ko);
        assert!(defender.hp_current <= 0);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, CombatEventKind::UnitDied { .. }))
        );
    }

    #[test]
    fn resolve_apply_no_ko_no_on_ko_event() {
        let attacker = unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let mut sp = SpPool { current: 3, max: 5 };
        let resolved = resolved(&basic_intent(), skill("basic", DamageTag::Fire, 10, 0, 5));

        let (outcome, events) = apply_legacy_ops(
            &resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut BasicStreak::default(),
            false,
            false,
            None,
            None,
            None,
        );

        assert!(!outcome.ko);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, CombatEventKind::UnitDied { .. }))
        );
    }

    #[test]
    fn resolve_apply_ultimate_resets_charge_and_emits_on_skill_cast() {
        let attacker = unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = UltimateCharge {
            current: 100,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let mut sp = SpPool { current: 3, max: 5 };
        let intent = ActionIntent::Ultimate {
            attacker: UnitId(1),
            target: UnitId(2),
        };
        let resolved = resolved(&intent, skill("ultimate", DamageTag::Fire, 30, 0, 20));

        let (outcome, events) = apply_legacy_ops(
            &resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut BasicStreak::default(),
            false,
            false,
            None,
            None,
            None,
        );

        assert!(outcome.sp_ok);
        assert_eq!(ult.current, 0);
        assert!(events.iter().any(|event| matches!(
            event,
            CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("ultimate".into())
        )));
    }

    #[test]
    fn test_apply_revive_success() {
        let attacker = unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 0); // KO
        let mut tough = Toughness::new(50, vec![DamageTag::Light]);
        let mut ult = UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let mut sp = SpPool { current: 5, max: 5 };
        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("revive".into()),
            target: UnitId(2),
        };
        let resolved = resolved(&intent, revive_skill("revive", 25, 4));

        let (outcome, events) = apply_legacy_ops(
            &resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut BasicStreak::default(),
            false,
            false,
            None,
            None,
            None,
        );

        assert!(outcome.sp_ok);
        assert_eq!(defender.hp_current, 25); // 25% of 100
        assert!(
            events
                .iter()
                .any(|e| matches!(e, CombatEventKind::OnRevive { hp_after: 25 }))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, CombatEventKind::OnSkillCast { .. }))
        );
    }

    #[test]
    fn grant_free_skill_resolve_sets_grant_count() {
        let intent = ActionIntent::Ultimate {
            attacker: UnitId(1),
            target: UnitId(2),
        };
        let skill = grant_free_skill_def("brave_tri_strike", 4);
        let book = SkillBook(vec![skill.clone()]);
        let kit = UnitSkills {
            basic: skill.id.clone(),
            skills: vec![skill.id.clone()],
            ultimate: skill.id,
            follow_up: None,
        };
        let resolved = resolve_action(&intent, &kit, Some(&book)).expect("should resolve");
        assert_eq!(resolved.grant_free_skill_count, 4);
    }

    #[test]
    fn grant_free_skill_events_emits_four_on_skill_cast() {
        let ally_basics: Vec<SkillId> = (1u32..=5).map(|i| SkillId(format!("basic_{i}"))).collect();
        let events = grant_free_skill_events(4, &ally_basics);
        assert_eq!(events.len(), 4, "expected exactly 4 OnSkillCast events");
        for (i, event) in events.iter().enumerate() {
            assert!(
                matches!(event, CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId(format!("basic_{}", i + 1))),
                "event {i} should be OnSkillCast for basic_{}",
                i + 1
            );
        }
    }

    #[test]
    fn grant_free_skill_events_caps_at_available_allies() {
        let ally_basics: Vec<SkillId> = vec![SkillId("basic_1".into()), SkillId("basic_2".into())];
        let events = grant_free_skill_events(4, &ally_basics);
        assert_eq!(events.len(), 2, "should not exceed available allies");
    }

    #[test]
    fn test_apply_revive_fails_on_active() {
        let attacker = unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 50); // Not KO
        let mut tough = Toughness::new(50, vec![DamageTag::Light]);
        let mut ult = UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let mut sp = SpPool { current: 5, max: 5 };
        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("revive".into()),
            target: UnitId(2),
        };
        let resolved = resolved(&intent, revive_skill("revive", 25, 4));

        let (_outcome, events) = apply_legacy_ops(
            &resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut BasicStreak::default(),
            false,
            false,
            None,
            None,
            None,
        );

        assert_eq!(defender.hp_current, 50); // No change
        assert!(
            events
                .iter()
                .any(|e| matches!(e, CombatEventKind::OnActionFailed { .. }))
        );
    }

    fn default_ult() -> UltimateCharge {
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        }
    }

    #[test]
    fn child_gets_minus1_sp_after_2_consecutive_basics() {
        let attacker = child_unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = default_ult();
        let mut sp = SpPool { current: 5, max: 5 };

        // Two basics build up streak
        let basic = basic_intent();
        let basic_resolved = resolved(&basic, skill("basic", DamageTag::Fire, 5, 0, 0));
        let mut streak = BasicStreak::default();
        apply_legacy_ops(
            &basic_resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut streak,
            false,
            false,
            None,
            None,
            None,
        );
        apply_legacy_ops(
            &basic_resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut streak,
            false,
            false,
            None,
            None,
            None,
        );
        assert_eq!(streak.count, 2);
        assert!(streak.qualifies_for_discount());

        // Skill with sp_cost 3 should cost only 2 due to Child discount
        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("skill".into()),
            target: UnitId(2),
        };
        let skill_resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 3, 0));
        sp.current = 3;
        let (outcome, _) = apply_legacy_ops(
            &skill_resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut streak,
            false,
            false,
            None,
            None,
            None,
        );

        assert!(outcome.sp_ok, "skill should succeed with discounted cost");
        assert_eq!(sp.current, 1, "paid 2 SP not 3 (discount applied)");
        assert_eq!(streak.count, 0, "streak reset after discount");
    }

    #[test]
    fn adult_gets_no_discount_after_consecutive_basics() {
        let attacker = unit(1, Attribute::Vaccine, 100); // Adult
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = default_ult();
        let mut sp = SpPool { current: 5, max: 5 };

        let mut streak = BasicStreak::default();
        // Adult can still track streak internally but never gets discount
        streak.increment();
        streak.increment();
        assert!(streak.qualifies_for_discount());

        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("skill".into()),
            target: UnitId(2),
        };
        let skill_resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 3, 0));
        sp.current = 3;
        let _ = apply_legacy_ops(
            &skill_resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut streak,
            false,
            false,
            None,
            None,
            None,
        );

        assert_eq!(sp.current, 0, "Adult paid full 3 SP, no discount");
        assert_eq!(
            streak.count, 2,
            "Adult streak not reset (no discount applied)"
        );
    }

    #[test]
    fn child_1_basic_not_enough_for_discount() {
        let attacker = child_unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = default_ult();
        let mut sp = SpPool { current: 5, max: 5 };

        let mut streak = BasicStreak::default();
        streak.increment(); // Only 1 basic
        assert!(!streak.qualifies_for_discount());

        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("skill".into()),
            target: UnitId(2),
        };
        let skill_resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 2, 0));
        let (outcome, _) = apply_legacy_ops(
            &skill_resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut streak,
            false,
            false,
            None,
            None,
            None,
        );

        assert!(outcome.sp_ok);
        assert_eq!(sp.current, 3, "paid full 2 SP, no discount for 1 basic");
        assert_eq!(streak.count, 1, "streak unchanged");
    }

    #[test]
    fn child_discount_resets_streak_needs_2_more_basics() {
        let attacker = child_unit(1, Attribute::Vaccine, 100);
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = default_ult();
        let mut sp = SpPool { current: 5, max: 5 };

        let mut streak = BasicStreak::default();
        streak.increment();
        streak.increment();

        // Use the discount
        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("skill".into()),
            target: UnitId(2),
        };
        let skill_resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 3, 0));
        apply_legacy_ops(
            &skill_resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut streak,
            false,
            false,
            None,
            None,
            None,
        );
        assert_eq!(streak.count, 0, "streak reset after discount use");

        // 1 more basic → still not enough
        streak.increment();
        assert!(
            !streak.qualifies_for_discount(),
            "needs 2 basics after reset"
        );

        // 2nd basic → qualifies again
        streak.increment();
        assert!(
            streak.qualifies_for_discount(),
            "2 basics after reset → qualifies again"
        );
    }

    #[test]
    fn adult_5_consecutive_basics_no_discount() {
        let attacker = unit(1, Attribute::Vaccine, 100); // Adult
        let mut defender = unit(2, Attribute::Virus, 100);
        let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
        let mut ult = default_ult();
        let mut sp = SpPool { current: 5, max: 5 };

        let mut streak = BasicStreak::default();
        for _ in 0..5 {
            streak.increment();
        }
        assert!(streak.qualifies_for_discount(), "streak counts up");

        let intent = ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("skill".into()),
            target: UnitId(2),
        };
        let skill_resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 3, 0));
        let (outcome, _) = apply_legacy_ops(
            &skill_resolved,
            &attacker,
            &mut defender,
            Team::Enemy,
            Some(&mut tough),
            &mut ult,
            &mut sp,
            &mut RoundSpTracker::default(),
            &mut streak,
            false,
            false,
            None,
            None,
            None,
        );

        assert!(outcome.sp_ok);
        assert_eq!(sp.current, 2, "Adult paid full 3 SP even with 5 basics");
        assert_eq!(streak.count, 5, "Adult streak unchanged");
    }

    // ── resolve_targets table-driven tests ──────────────────────────────────

    fn snap(entries: Vec<(UnitId, Team, u8, bool)>) -> TargetableSnapshot {
        TargetableSnapshot {
            entries: entries
                .into_iter()
                .map(|(id, team, slot_index, alive)| TargetEntry {
                    id,
                    team,
                    slot_index,
                    alive,
                    hp_per_mille: 1000, // full HP default for shape tests that don't use HP selector
                })
                .collect(),
        }
    }

    /// Build a snapshot with explicit HP percentages (per-mille: 0–1000).
    fn snap_hp(entries: Vec<(UnitId, Team, u8, bool, u32)>) -> TargetableSnapshot {
        TargetableSnapshot {
            entries: entries
                .into_iter()
                .map(|(id, team, slot_index, alive, hp_per_mille)| TargetEntry {
                    id,
                    team,
                    slot_index,
                    alive,
                    hp_per_mille,
                })
                .collect(),
        }
    }

    #[test]
    fn resolve_targets_single_returns_primary() {
        let s = snap(vec![
            (UnitId(1), Team::Ally, 0, true),
            (UnitId(2), Team::Enemy, 0, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::Single, UnitId(2), &s),
            vec![UnitId(2)]
        );
    }

    #[test]
    fn resolve_targets_blast_edge_slot_zero_returns_only_0_and_1() {
        // primary at slot 0 → slot -1 absent → only slots 0 and 1
        let s = snap(vec![
            (UnitId(10), Team::Enemy, 0, true),
            (UnitId(11), Team::Enemy, 1, true),
            (UnitId(12), Team::Enemy, 2, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::Blast, UnitId(10), &s),
            vec![UnitId(10), UnitId(11)],
        );
    }

    #[test]
    fn resolve_targets_blast_ko_adjacent_omitted() {
        // primary at slot 1, slot 0 KO'd → only [slot1, slot2]
        let s = snap(vec![
            (UnitId(10), Team::Enemy, 0, false),
            (UnitId(11), Team::Enemy, 1, true),
            (UnitId(12), Team::Enemy, 2, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::Blast, UnitId(11), &s),
            vec![UnitId(11), UnitId(12)],
        );
    }

    #[test]
    fn resolve_targets_blast_all_three_alive_sorted_asc() {
        // Inserted out of order → sorted by slot_index
        let s = snap(vec![
            (UnitId(12), Team::Enemy, 2, true),
            (UnitId(10), Team::Enemy, 0, true),
            (UnitId(11), Team::Enemy, 1, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::Blast, UnitId(11), &s),
            vec![UnitId(10), UnitId(11), UnitId(12)],
        );
    }

    #[test]
    fn resolve_targets_all_enemies_omits_dead() {
        let s = snap(vec![
            (UnitId(1), Team::Ally, 0, true),
            (UnitId(10), Team::Enemy, 0, true),
            (UnitId(11), Team::Enemy, 1, false),
            (UnitId(12), Team::Enemy, 2, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::AllEnemies, UnitId(10), &s),
            vec![UnitId(10), UnitId(12)],
        );
    }

    #[test]
    fn resolve_targets_all_enemies_sorted_slot_asc() {
        let s = snap(vec![
            (UnitId(12), Team::Enemy, 2, true),
            (UnitId(10), Team::Enemy, 0, true),
            (UnitId(11), Team::Enemy, 1, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::AllEnemies, UnitId(12), &s),
            vec![UnitId(10), UnitId(11), UnitId(12)],
        );
    }

    // ── select_bounce_hop dispatcher tests ──────────────────────────────────

    #[test]
    fn bounce_lowest_hp_pct_picks_lowest_pct() {
        // Three enemies: slot 0 @ 500‰, slot 1 @ 300‰, slot 2 @ 800‰
        // LowestHpPctAlive should pick slot 1 (300‰)
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 500),
            (UnitId(11), Team::Enemy, 1, true, 300),
            (UnitId(12), Team::Enemy, 2, true, 800),
        ]);
        let already_hit = HashSet::new();
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, Some(UnitId(11)));
    }

    #[test]
    fn bounce_lowest_hp_pct_tiebreak_slot_asc() {
        // Three enemies all at 500‰; lowest slot_index should win
        let s = snap_hp(vec![
            (UnitId(12), Team::Enemy, 2, true, 500),
            (UnitId(10), Team::Enemy, 0, true, 500),
            (UnitId(11), Team::Enemy, 1, true, 500),
        ]);
        let already_hit = HashSet::new();
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, Some(UnitId(10))); // slot 0 wins tie
    }

    #[test]
    fn bounce_lowest_hp_pct_excludes_already_hit_no_repeat() {
        // slot 0 @ 100‰ (lowest), slot 1 @ 400‰ — slot 0 already hit → slot 1 wins
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 100),
            (UnitId(11), Team::Enemy, 1, true, 400),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(10));
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, Some(UnitId(11)));
    }

    #[test]
    fn bounce_lowest_hp_pct_allow_repeat_can_repick_same() {
        // Only one alive enemy; with NoRepeat it would return None (already in set),
        // but AllowRepeat allows re-selecting it.
        let s = snap_hp(vec![(UnitId(10), Team::Enemy, 0, true, 100)]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(10));
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::AllowRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(
            result,
            Some(UnitId(10)),
            "AllowRepeat should re-pick the only target"
        );

        // Confirm NoRepeat returns None in same scenario
        let result_no_repeat = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(
            result_no_repeat, None,
            "NoRepeat should return None when only target already hit"
        );
    }

    #[test]
    fn bounce_lowest_hp_pct_empty_pool_returns_none() {
        // No alive enemies at all
        let s = snap_hp(vec![(UnitId(10), Team::Enemy, 0, false, 0)]);
        let already_hit = HashSet::new();
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn bounce_next_slot_picks_next_above_last() {
        // Last hit slot = 0; candidates: slot 0 (already hit → excluded), slot 1, slot 2
        // NextSlotAlive should pick slot 1 (first slot > 0)
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 500),
            (UnitId(11), Team::Enemy, 1, true, 800),
            (UnitId(12), Team::Enemy, 2, true, 300),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(10));
        let result = select_bounce_hop(
            BounceSelector::NextSlotAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            Some(0),
        );
        assert_eq!(result, Some(UnitId(11)));
    }

    #[test]
    fn bounce_next_slot_no_slot_above_last_returns_none() {
        // Last hit = slot 2 (highest); no slot > 2 exists → None
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 500),
            (UnitId(11), Team::Enemy, 1, true, 500),
            (UnitId(12), Team::Enemy, 2, true, 500),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(12));
        let result = select_bounce_hop(
            BounceSelector::NextSlotAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            Some(2),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn bounce_next_slot_no_last_picks_lowest_slot() {
        // No last_slot → pick the alive enemy with the lowest slot_index
        let s = snap_hp(vec![
            (UnitId(12), Team::Enemy, 2, true, 300),
            (UnitId(10), Team::Enemy, 0, true, 800),
            (UnitId(11), Team::Enemy, 1, true, 500),
        ]);
        let already_hit = HashSet::new();
        let result = select_bounce_hop(
            BounceSelector::NextSlotAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, Some(UnitId(10))); // slot 0
    }

    #[test]
    fn bounce_adj_lowest_picks_adjacent_with_lowest_hp() {
        // Last hit slot = 1; adjacent = slots 0 and 2.
        // slot 0 @ 600‰, slot 2 @ 200‰ → slot 2 wins
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 600),
            (UnitId(11), Team::Enemy, 1, true, 500), // last hit, excluded by already_hit
            (UnitId(12), Team::Enemy, 2, true, 200),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(11));
        let result = select_bounce_hop(
            BounceSelector::AdjLowest,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            Some(1),
        );
        assert_eq!(result, Some(UnitId(12)));
    }

    #[test]
    fn bounce_adj_lowest_tiebreak_slot_asc() {
        // Last hit slot = 1; both slot 0 and slot 2 at same HP% → slot 0 wins (lower index)
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 400),
            (UnitId(11), Team::Enemy, 1, true, 800), // last hit, excluded
            (UnitId(12), Team::Enemy, 2, true, 400),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(11));
        let result = select_bounce_hop(
            BounceSelector::AdjLowest,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            Some(1),
        );
        assert_eq!(result, Some(UnitId(10))); // slot 0 wins tie
    }

    #[test]
    fn bounce_adj_lowest_no_adjacent_alive_returns_none() {
        // Last hit slot = 1, but slots 0 and 2 are dead → None
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, false, 0),
            (UnitId(11), Team::Enemy, 1, true, 500),
            (UnitId(12), Team::Enemy, 2, false, 0),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(11));
        let result = select_bounce_hop(
            BounceSelector::AdjLowest,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            Some(1),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn bounce_ignores_ally_team() {
        // Ally team entries should never be returned regardless of HP
        let s = snap_hp(vec![
            (UnitId(1), Team::Ally, 0, true, 50), // ally, very low HP
            (UnitId(10), Team::Enemy, 0, true, 900),
        ]);
        let already_hit = HashSet::new();
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, Some(UnitId(10)), "ally must never be selected");
    }

    #[test]
    fn bounce_allow_repeat_picks_same_target_twice() {
        // Two enemies; AllowRepeat + LowestHpPct: slot 1 @ 200‰ wins both picks even when it's
        // already in the hit set (simulated by calling the dispatcher twice with it inserted).
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 700),
            (UnitId(11), Team::Enemy, 1, true, 200),
        ]);
        let mut already_hit = HashSet::new();

        // First pick
        let first = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::AllowRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(first, Some(UnitId(11)));
        already_hit.insert(UnitId(11));

        // Second pick — AllowRepeat ignores already_hit, so slot 1 wins again
        let second = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::AllowRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(
            second,
            Some(UnitId(11)),
            "AllowRepeat: same lowest-HP target can be picked again"
        );
    }
}
