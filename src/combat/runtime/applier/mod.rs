mod effects;

use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;

use crate::combat::{
    runtime::{
        intent::{CastId, Intent},
    },
    buffs::DrBag,
    events::{CombatEvent, CombatEventKind},
    log::{ActionLog, LogEntry},
    status_effect::StatusBag,
    team::Team,
    toughness::Toughness,
    types::{DamageTag, UnitId},
    unit::{Ko, SlotIndex, Unit},
    kit::UnitSkills,
};

use effects::{
    apply_add_energy, apply_advance_turn, apply_blueprint_signal, apply_break_toughness,
    apply_buff, apply_damage_modifier, apply_deal_damage, apply_delay_turn,
    apply_grant_free_skill, apply_revive, apply_set_blueprint_state, apply_status,
};

/// Pending `Intent` queue drained each frame by `intent_applier`.
///
/// Skill hook functions push to this queue via `SkillCtx::enqueue`; the applier
/// system is the single consumer. Direct mutations from outside `intent_applier`
/// violate the P001 single-source-of-truth invariant.
#[derive(Resource, Default)]
pub struct IntentQueue(pub VecDeque<Intent>);

/// Per-cast execution metadata needed by the intent applier when timeline-backed
/// actions are resolved outside the legacy `step_app` path.
#[derive(Resource, Default)]
pub struct IntentExecutionMeta {
    pub follow_up_depths: HashMap<CastId, u8>,
}

/// Exclusive system that drains `IntentQueue` and routes each variant to the
/// appropriate combat subsystem.
///
/// # Routing policy (S01)
/// - `DealDamage` → wired end-to-end to the existing damage formula (canary).
/// - All other variants → `log::warn!` + no-op (migration arrives in S05+).
///
/// # Exclusivity
/// Takes `&mut World` to allow simultaneous read of source unit data and mutable
/// write of target unit HP without Bevy query aliasing restrictions.
pub fn intent_applier(world: &mut World) {
    let intents: Vec<Intent> = world
        .get_resource_mut::<IntentQueue>()
        .map(|mut q| q.0.drain(..).collect())
        .unwrap_or_default();

    if intents.is_empty() {
        return;
    }

    #[cfg(debug_assertions)]
    let _combat_apply_intents_span = bevy::log::info_span!(
        target: "combat.apply",
        "combat.apply.intent_queue",
        intent_count = intents.len(),
    )
    .entered();

    for intent in intents {
        match intent {
            Intent::DealDamage {
                source,
                target,
                amount,
                tag,
                cast_id,
            } => {
                apply_deal_damage(world, source, target, amount, tag, cast_id);
            }
            Intent::BreakToughness {
                source,
                target,
                amount,
                tag,
                cast_id,
            } => {
                apply_break_toughness(world, source, target, amount, tag, cast_id);
            }
            Intent::ApplyStatus {
                source,
                target,
                kind,
                duration_turns,
                cast_id,
            } => {
                apply_status(
                    world,
                    source,
                    target,
                    kind,
                    duration_turns,
                    cast_id,
                    true,
                    true,
                );
            }
            Intent::ApplyBuff {
                target,
                kind,
                duration_turns,
                cast_id,
            } => {
                apply_buff(world, target, kind, duration_turns, cast_id);
            }
            Intent::ApplyDamageModifier {
                target,
                layer,
                multiplier_pct,
                cast_id,
            } => {
                apply_damage_modifier(world, target, layer, multiplier_pct, cast_id);
            }
            Intent::AdvanceTurn {
                target,
                amount_pct,
                cast_id,
            } => {
                apply_advance_turn(world, target, amount_pct, cast_id);
            }
            Intent::DelayTurn {
                target,
                amount_pct,
                cast_id,
            } => {
                apply_delay_turn(world, target, amount_pct, cast_id);
            }
            Intent::Revive {
                source,
                target,
                pct,
                cast_id,
            } => {
                apply_revive(world, source, target, pct, cast_id);
            }
            Intent::GrantFreeSkill {
                source,
                count,
                cast_id,
            } => {
                apply_grant_free_skill(world, source, count, cast_id);
            }
            Intent::AddEnergy {
                target,
                amount,
                cast_id,
            } => {
                apply_add_energy(world, target, amount, cast_id);
            }
            Intent::BlueprintSignal {
                source,
                owner,
                name,
                payload,
                cast_id,
            } => {
                apply_blueprint_signal(world, source, owner, name, payload, cast_id);
            }
            Intent::SetBlueprintState {
                actor,
                key,
                value,
                cast_id,
            } => {
                apply_set_blueprint_state(world, actor, key, value, cast_id);
            }
        }
    }
}

pub(crate) struct UnitSnapshot {
    pub entity: Entity,
    pub id: UnitId,
    pub unit: Unit,
    pub team: Team,
    pub weaknesses: Vec<DamageTag>,
    pub status: Option<StatusBag>,
    pub dr: Option<DrBag>,
}

pub(crate) fn emit_event(
    world: &mut World,
    kind: CombatEventKind,
    source: UnitId,
    target: UnitId,
    cast_id: CastId,
) {
    let follow_up_depth = world
        .get_resource::<IntentExecutionMeta>()
        .and_then(|meta| meta.follow_up_depths.get(&cast_id).copied())
        .unwrap_or(0);

    if let Some(mut log) = world.get_resource_mut::<ActionLog>() {
        match &kind {
            CombatEventKind::OnDamageDealt { amount, kind, .. } => {
                log.push(LogEntry::BasicHit {
                    attacker: source,
                    target,
                    amount: *amount,
                    kind: *kind,
                });
            }
            CombatEventKind::OnBreak { damage_tag } => {
                log.push(LogEntry::Break {
                    target,
                    damage_tag: *damage_tag,
                });
            }
            CombatEventKind::UnitDied { .. } => {
                log.push(LogEntry::Ko { target });
            }
            CombatEventKind::OnRevive { hp_after } => {
                log.push(LogEntry::Revive {
                    target,
                    hp_after: *hp_after,
                });
            }
            CombatEventKind::OnActionFailed { reason } => {
                log.push(LogEntry::ActionFailed {
                    reason: reason.clone(),
                });
            }
            CombatEventKind::AdvanceTurn {
                target: log_target,
                amount_pct,
            } => {
                log.push(LogEntry::AdvanceTurn {
                    target: *log_target,
                    amount_pct: *amount_pct,
                });
            }
            CombatEventKind::DelayTurn {
                target: log_target,
                amount_pct,
            } => {
                log.push(LogEntry::DelayTurn {
                    target: *log_target,
                    amount_pct: *amount_pct,
                });
            }
            _ => {}
        }
    }

    world
        .resource_mut::<Messages<CombatEvent>>()
        .write(CombatEvent {
            kind,
            source,
            target,
            follow_up_depth,
            cast_id,
        });
}

pub(crate) fn find_unit_snapshot(world: &mut World, id: UnitId) -> Option<UnitSnapshot> {
    let mut q = world.query::<(
        Entity,
        &Unit,
        &Team,
        Option<&Toughness>,
        Option<&StatusBag>,
        Option<&DrBag>,
    )>();
    q.iter(world)
        .find_map(|(entity, unit, team, toughness, status, dr)| {
            (unit.id == id).then(|| UnitSnapshot {
                entity,
                id,
                unit: unit.clone(),
                team: *team,
                weaknesses: toughness
                    .map(|tg| tg.weaknesses.clone())
                    .unwrap_or_default(),
                status: status.cloned(),
                dr: dr.cloned(),
            })
        })
}

pub(crate) fn find_unit_entity(world: &mut World, id: UnitId) -> Option<(Entity, Unit)> {
    let mut q = world.query::<(Entity, &Unit)>();
    q.iter(world)
        .find_map(|(entity, unit)| (unit.id == id).then(|| (entity, unit.clone())))
}

pub(crate) fn allied_basic_skill_ids(
    world: &World,
    source: UnitId,
) -> Option<Vec<(UnitId, crate::combat::types::SkillId)>> {
    let mut q =
        world.try_query::<(&Unit, &Team, Option<&Ko>, Option<&SlotIndex>, &UnitSkills)>()?;
    let caster_team = q
        .iter(world)
        .find_map(|(unit, team, _, _, _)| (unit.id == source).then_some(*team))?;

    let mut allies: Vec<(u8, UnitId, crate::combat::types::SkillId)> = q
        .iter(world)
        .filter(|(unit, team, ko, _, _)| {
            **team == caster_team && ko.is_none() && unit.hp_current > 0
        })
        .map(|(unit, _, _, slot, skills)| {
            (
                slot.map(|s| s.0).unwrap_or(u8::MAX),
                unit.id,
                skills.basic.clone(),
            )
        })
        .collect();

    allies.sort_by_key(|(slot, _, _)| *slot);
    Some(
        allies
            .into_iter()
            .map(|(_, id, skill_id)| (id, skill_id))
            .collect(),
    )
}
