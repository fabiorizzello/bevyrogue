use std::collections::{HashMap, VecDeque};

use bevy::log;
use bevy::prelude::*;

use crate::combat::{
    api::{
        blueprint_state::BlueprintState,
        intent::{CastId, Intent},
        signal::{Signal, SignalBus, SignalPayload, SignalTaxonomy},
    },
    damage::{AttackContext, calculate_damage, triangle_modifiers},
    energy::{Energy, EnergyGainSource, RoundEnergyTracker},
    events::{CombatEvent, CombatEventKind, CombatKernelTransition},
    modifiers::{DamageModifierLedger, ModifierChain, ModifierLayer},
    round_flags::RoundFlags,
    buffs::DrBag,
    rng::CombatRng,
    status_effect::{StatusBag, StatusEffectKind, classify_buff_kind},
    stun::Stunned,
    team::Team,
    toughness::{DamageKind, Toughness, can_apply_toughness_damage, classify},
    types::{DamageTag, UnitId},
    unit::{Ko, SlotIndex, Unit},
    kit::UnitSkills,
    log::{ActionLog, LogEntry},
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
                apply_status(world, source, target, kind, duration_turns, cast_id, true, true);
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
            other => {
                log::warn!("intent_applier: unimplemented intent variant {:?}", other);
            }
        }
    }
}

struct UnitSnapshot {
    entity: Entity,
    id: UnitId,
    unit: Unit,
    team: Team,
    weaknesses: Vec<DamageTag>,
    status: Option<StatusBag>,
    dr: Option<DrBag>,
}

fn emit_event(
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

    world.resource_mut::<Messages<CombatEvent>>().write(CombatEvent {
        kind,
        source,
        target,
        follow_up_depth,
        cast_id,
    });
}

fn find_unit_snapshot(world: &mut World, id: UnitId) -> Option<UnitSnapshot> {
    let mut q = world.query::<(Entity, &Unit, &Team, Option<&Toughness>, Option<&StatusBag>, Option<&DrBag>)>();
    q.iter(world).find_map(|(entity, unit, team, toughness, status, dr)| {
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

fn find_unit_entity(world: &mut World, id: UnitId) -> Option<(Entity, Unit)> {
    let mut q = world.query::<(Entity, &Unit)>();
    q.iter(world)
        .find_map(|(entity, unit)| (unit.id == id).then(|| (entity, unit.clone())))
}

fn allied_basic_skill_ids(
    world: &World,
    source: UnitId,
) -> Option<Vec<(UnitId, crate::combat::types::SkillId)>> {
    let mut q = world.try_query::<(&Unit, &Team, Option<&Ko>, Option<&SlotIndex>, &UnitSkills)>()?;
    let caster_team = q
        .iter(world)
        .find_map(|(unit, team, _, _, _)| (unit.id == source).then_some(*team))?;

    let mut allies: Vec<(u8, UnitId, crate::combat::types::SkillId)> = q
        .iter(world)
        .filter(|(unit, team, ko, _, _)| **team == caster_team && ko.is_none() && unit.hp_current > 0)
        .map(|(unit, _, _, slot, skills)| {
            (
                slot.map(|s| s.0).unwrap_or(u8::MAX),
                unit.id,
                skills.basic.clone(),
            )
        })
        .collect();

    allies.sort_by_key(|(slot, _, _)| *slot);
    Some(allies.into_iter().map(|(_, id, skill_id)| (id, skill_id)).collect())
}

fn apply_deal_damage(
    world: &mut World,
    source: UnitId,
    target: UnitId,
    base_damage: i32,
    tag: DamageTag,
    cast_id: CastId,
) {
    // Snapshot all units to avoid aliased world borrows during calculation.
    let snapshot: Vec<UnitSnapshot> = {
        let mut q = world.query::<(Entity, &Unit, &Team, Option<&Toughness>, Option<&StatusBag>, Option<&DrBag>)>();
        q.iter(world)
            .map(|(e, u, team, t, status, dr)| UnitSnapshot {
                entity: e,
                id: u.id,
                unit: u.clone(),
                team: *team,
                weaknesses: t
                    .map(|tg| tg.weaknesses.clone())
                    .unwrap_or_default(),
                status: status.cloned(),
                dr: dr.cloned(),
            })
            .collect()
    };

    let Some(src) = snapshot.iter().find(|s| s.id == source) else {
        log::warn!("intent_applier DealDamage: source {:?} not found", source);
        return;
    };
    let Some(tgt) = snapshot.iter().find(|s| s.id == target) else {
        log::warn!("intent_applier DealDamage: target {:?} not found", target);
        return;
    };

    let reaction_chain = world
        .get_resource_mut::<DamageModifierLedger>()
        .map(|mut ledger| ledger.drain(target))
        .unwrap_or_default();

    let block_reaction_mitigated_pct = reaction_chain
        .terms()
        .iter()
        .find(|term| term.layer == ModifierLayer::Passive)
        .map(|term| (100 - term.multiplier_pct).clamp(0, 100) as u8);

    let mut modifier_chain = ModifierChain::default();
    if src
        .status
        .as_ref()
        .is_some_and(|bag| bag.has(&StatusEffectKind::Blessed))
    {
        modifier_chain.push(ModifierLayer::Status, 115);
    }
    modifier_chain.extend(reaction_chain.clone());

    emit_event(
        world,
        CombatEventKind::IncomingDamage {
            raw_amount: base_damage,
            damage_tag: tag,
        },
        source,
        target,
        cast_id,
    );

    let mut tentomon_block_triggered = false;
    if tgt.unit.name == "Tentomon"
        && world.contains_resource::<crate::combat::battery_loop::BatteryLoopState>()
        && world.contains_resource::<CombatRng>()
    {
        world.resource_scope(|world, mut state: Mut<crate::combat::battery_loop::BatteryLoopState>| {
            if state.block_reaction_ready() && state.last_block_reaction_cast_id != Some(cast_id) {
                state.last_block_reaction_cast_id = Some(cast_id);
                world.resource_scope(|_world, mut rng: Mut<CombatRng>| {
                    if rng.roll_pct(30) {
                        modifier_chain.push(ModifierLayer::Passive, 50);
                        tentomon_block_triggered = true;
                        let _ = state.proc_block_reaction();
                    }
                });
            }
        });
    }

    let modifier_trace = modifier_chain.apply_to(base_damage);

    let attack = AttackContext {
        damage_tag: tag,
        base_damage: modifier_trace.final_amount,
        is_break: false,
    };
    let bd = calculate_damage(
        &src.unit,
        &attack,
        &tgt.unit,
        &tgt.weaknesses,
        tgt.status.as_ref(),
        1.0,
        tgt.dr.as_ref(),
    );

    let tgt_entity = tgt.entity;

    // Apply HP mutation.
    let mut hp_after = None;
    if let Some(mut u) = world.get_mut::<Unit>(tgt_entity) {
        u.hp_current -= bd.final_damage;
        hp_after = Some(u.hp_current);
    }

    emit_event(
        world,
        CombatEventKind::OnDamageDealt {
            amount: bd.final_damage,
            kind: DamageKind::Normal,
            tag_mod_pct: bd.tag_mod_pct,
            triangle_mod_pct: bd.triangle_mod_pct,
            damage_tag: tag,
        },
        source,
        target,
        cast_id,
    );

    if let Some(mitigated_pct) = block_reaction_mitigated_pct.or_else(|| tentomon_block_triggered.then_some(50)) {
        emit_event(
            world,
            CombatEventKind::BlockReactionTriggered { mitigated_pct },
            source,
            target,
            cast_id,
        );
    }

    if hp_after.is_some_and(|hp| hp <= 0) {
        world.entity_mut(tgt_entity).insert(Ko);
        emit_event(
            world,
            CombatEventKind::UnitDied {
                status_remaining: vec![],
                heated_remaining: 0,
            },
            source,
            target,
            cast_id,
        );
        if src.team != tgt.team {
            emit_event(world, CombatEventKind::OnEnemyKill, source, target, cast_id);
        }
    }
}

fn apply_break_toughness(
    world: &mut World,
    source: UnitId,
    target: UnitId,
    amount: i32,
    tag: DamageTag,
    cast_id: CastId,
) {
    let Some(source_snapshot) = find_unit_snapshot(world, source) else {
        log::warn!("intent_applier BreakToughness: source {:?} not found", source);
        return;
    };
    let Some(target_snapshot) = find_unit_snapshot(world, target) else {
        log::warn!("intent_applier BreakToughness: target {:?} not found", target);
        return;
    };

    let mut break_entity = None;
    let mut broke = false;
    {
        let mut q = world.query::<(Entity, &Unit, &Team, Option<&mut Toughness>, Option<&mut RoundFlags>)>();
        for (entity, unit, team, mut toughness, mut flags) in q.iter_mut(world) {
            if unit.id != target_snapshot.id {
                continue;
            }
            if !can_apply_toughness_damage(*team, toughness.as_deref()) {
                return;
            }
            let break_sealed = flags.as_deref().map(|f| f.break_sealed).unwrap_or(false);
            broke = toughness
                .as_deref_mut()
                .map(|t| t.apply_hit(tag, amount, break_sealed))
                .unwrap_or(false);
            if broke {
                if let Some(ref mut f) = flags {
                    f.break_sealed = true;
                }
                break_entity = Some(entity);
            }
            break;
        }
    }

    if broke {
        if let Some(entity) = break_entity {
            world.entity_mut(entity).insert(Stunned { turns_left: 1 });
        }
        emit_event(world, CombatEventKind::OnBreak { damage_tag: tag }, source, target, cast_id);
        let _ = source_snapshot;
        let _ = target_snapshot;
    }
}

fn apply_status(
    world: &mut World,
    source: UnitId,
    target: UnitId,
    kind: StatusEffectKind,
    duration_turns: u32,
    cast_id: CastId,
    check_accuracy: bool,
    emit_slowed_delay: bool,
) {
    let Some(source_snapshot) = find_unit_snapshot(world, source) else {
        log::warn!("intent_applier ApplyStatus: source {:?} not found", source);
        return;
    };
    let Some(target_snapshot) = find_unit_snapshot(world, target) else {
        log::warn!("intent_applier ApplyStatus: target {:?} not found", target);
        return;
    };

    if target_snapshot.unit.hp_current <= 0 {
        return;
    }

    if check_accuracy {
        let tri = triangle_modifiers(source_snapshot.unit.attribute, target_snapshot.unit.attribute);
        let threshold = (tri.status_acc_modifier * 100.0) as i32;
        let passes = world
            .get_resource_mut::<CombatRng>()
            .map_or_else(|| CombatRng::default().roll_pct(threshold), |mut rng| rng.roll_pct(threshold));
        if !passes {
            emit_event(
                world,
                CombatEventKind::OnStatusResisted { kind },
                source,
                target,
                cast_id,
            );
            return;
        }
    }

    let mut target_entity = None;
    let mut is_first_slowed_apply = false;
    let mut fresh_bag = None;
    {
        let mut q = world.query::<(Entity, &Unit, Option<&mut StatusBag>)>();
        for (entity, unit, mut bag) in q.iter_mut(world) {
            if unit.id != target_snapshot.id {
                continue;
            }
            is_first_slowed_apply = emit_slowed_delay
                && matches!(kind, StatusEffectKind::Slowed)
                && bag.as_deref().map_or(true, |b| !b.has(&StatusEffectKind::Slowed));
            if let Some(ref mut bag) = bag {
                bag.apply(kind.clone(), duration_turns);
            } else {
                let mut bag = StatusBag::default();
                bag.apply(kind.clone(), duration_turns);
                fresh_bag = Some(bag);
            }
            target_entity = Some(entity);
            break;
        }
    }

    let Some(entity) = target_entity else {
        log::warn!("intent_applier ApplyStatus: target entity {:?} not found", target);
        return;
    };

    if let Some(bag) = fresh_bag {
        world.entity_mut(entity).insert(bag);
    }

    emit_event(
        world,
        CombatEventKind::OnStatusApplied { kind: kind.clone() },
        source,
        target,
        cast_id,
    );

    if is_first_slowed_apply {
        emit_event(
            world,
            CombatEventKind::DelayTurn {
                target,
                amount_pct: 30,
            },
            source,
            target,
            cast_id,
        );
    }
}

fn apply_buff(
    world: &mut World,
    target: UnitId,
    kind: StatusEffectKind,
    duration_turns: u32,
    cast_id: CastId,
) {
    apply_status(
        world,
        target,
        target,
        kind,
        duration_turns,
        cast_id,
        false,
        false,
    );
}

fn apply_damage_modifier(
    world: &mut World,
    target: UnitId,
    layer: ModifierLayer,
    multiplier_pct: i32,
    _cast_id: CastId,
) {
    if multiplier_pct == 100 {
        return;
    }

    world
        .resource_mut::<DamageModifierLedger>()
        .arm(target, layer, multiplier_pct);
}

fn apply_delay_turn(world: &mut World, target: UnitId, amount_pct: u32, cast_id: CastId) {
    if amount_pct == 0 {
        return;
    }
    emit_event(
        world,
        CombatEventKind::DelayTurn { target, amount_pct },
        target,
        target,
        cast_id,
    );
}

fn apply_advance_turn(world: &mut World, target: UnitId, amount_pct: u32, cast_id: CastId) {
    if amount_pct == 0 {
        return;
    }
    emit_event(
        world,
        CombatEventKind::AdvanceTurn { target, amount_pct },
        target,
        target,
        cast_id,
    );
}

fn apply_add_energy(world: &mut World, target: UnitId, amount: i32, cast_id: CastId) {
    if amount <= 0 {
        return;
    }

    let Some((entity, _unit)) = find_unit_entity(world, target) else {
        log::warn!("intent_applier AddEnergy: target {:?} not found", target);
        return;
    };

    let granted_by_round_cap = if let Some(mut tracker) = world.get_mut::<RoundEnergyTracker>(entity) {
        tracker.try_gain(EnergyGainSource::SecondaryAction, amount)
    } else {
        amount
    };

    let gained = if let Some(mut energy) = world.get_mut::<Energy>(entity) {
        energy.gain_capped(granted_by_round_cap)
    } else {
        log::warn!("intent_applier AddEnergy: target {:?} missing Energy component", target);
        return;
    };

    if gained > 0 {
        emit_event(
            world,
            CombatEventKind::EnergyGained {
                unit_id: target,
                amount: gained,
            },
            target,
            target,
            cast_id,
        );
    }
}

fn apply_revive(world: &mut World, source: UnitId, target: UnitId, pct: i32, cast_id: CastId) {
    let Some(snapshot) = find_unit_snapshot(world, target) else {
        log::warn!("intent_applier Revive: target {:?} not found", target);
        return;
    };

    if !snapshot.unit.is_ko() {
        emit_event(
            world,
            CombatEventKind::OnActionFailed {
                reason: "Target is not KO".to_string(),
            },
            source,
            target,
            cast_id,
        );
        return;
    }

    if let Some(mut unit) = world.get_mut::<Unit>(snapshot.entity) {
        unit.revive(pct);
    }
    world.entity_mut(snapshot.entity).remove::<Ko>();

    let hp_after = world
        .get::<Unit>(snapshot.entity)
        .map(|unit| unit.hp_current)
        .unwrap_or_default();

    emit_event(
        world,
        CombatEventKind::OnRevive { hp_after },
        source,
        target,
        cast_id,
    );
}

fn apply_grant_free_skill(world: &mut World, source: UnitId, count: usize, cast_id: CastId) {
    if count == 0 {
        return;
    }

    let Some(ally_basics) = allied_basic_skill_ids(world, source) else {
        log::warn!("intent_applier GrantFreeSkill: source {:?} not found", source);
        return;
    };

    for (ally_id, skill_id) in ally_basics.into_iter().take(count) {
        world.resource_mut::<Messages<CombatEvent>>().write(CombatEvent {
            kind: CombatEventKind::OnSkillCast { skill_id },
            source: ally_id,
            target: ally_id,
            follow_up_depth: 0,
            cast_id,
        });
    }
}

fn apply_blueprint_signal(
    world: &mut World,
    source: UnitId,
    owner: &'static str,
    name: &'static str,
    payload: SignalPayload,
    cast_id: CastId,
) {
    let taxonomy = world.resource::<SignalTaxonomy>();
    if !taxonomy.contains(owner, name) {
        debug_assert!(false, "unregistered signal: {}/{}", owner, name);
        log::warn!(
            "intent_applier BlueprintSignal: unregistered signal {}/{}",
            owner,
            name
        );
        return;
    }

    world.resource_mut::<SignalBus>().push(Signal::Blueprint {
        owner: owner.to_string(),
        name: name.to_string(),
        payload: payload.clone(),
        cast_id,
    });

    world.resource_mut::<Messages<CombatEvent>>().write(CombatEvent {
        kind: CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint {
                owner: owner.to_string(),
                name: name.to_string(),
                payload,
            },
        },
        source,
        target: source,
        follow_up_depth: 0,
        cast_id,
    });
}

fn apply_set_blueprint_state(
    world: &mut World,
    actor: UnitId,
    key: String,
    value: i64,
    _cast_id: CastId,
) {
    world
        .resource_mut::<BlueprintState>()
        .map
        .insert((actor, key), value);
}
