use bevy::log;
use bevy::prelude::*;

use crate::combat::{
    buffs::DrBag,
    damage::{AttackContext, calculate_damage},
    events::CombatEventKind,
    modifiers::{DamageModifierLedger, ModifierChain, ModifierLayer},
    round_flags::RoundFlags,
    runtime::{ExtRegistries, intent::CastId},
    status_effect::{StatusBag, StatusEffectKind},
    stun::Stunned,
    team::Team,
    toughness::{DamageKind, Toughness, can_apply_toughness_damage},
    types::{DamageTag, UnitId},
    unit::{Ko, Unit},
};

use super::super::{UnitSnapshot, emit_event, find_unit_snapshot};

pub(in crate::combat::runtime::applier) fn apply_deal_damage(
    world: &mut World,
    source: UnitId,
    target: UnitId,
    base_damage: i32,
    tag: DamageTag,
    cast_id: CastId,
) {
    // Snapshot all units to avoid aliased world borrows during calculation.
    let snapshot: Vec<UnitSnapshot> = {
        let mut q = world.query::<(
            Entity,
            &Unit,
            &Team,
            Option<&Toughness>,
            Option<&StatusBag>,
            Option<&DrBag>,
        )>();
        q.iter(world)
            .map(|(e, u, team, t, status, dr)| UnitSnapshot {
                entity: e,
                id: u.id,
                unit: u.clone(),
                team: *team,
                weaknesses: t.map(|tg| tg.weaknesses.clone()).unwrap_or_default(),
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

    // Run all registered pre-damage reactions (e.g. Tentomon block reaction).
    // Each may arm modifiers in the DamageModifierLedger and return a mitigation %.
    let pre_damage_mitigation: Option<i32> = {
        let fns: Vec<_> = world
            .get_resource::<ExtRegistries>()
            .map(|regs| regs.pre_damage_reactions.iter().map(|(_, f)| *f).collect())
            .unwrap_or_default();
        fns.iter().filter_map(|f| f(world, target, cast_id)).next()
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

    if let Some(mitigated_pct) =
        block_reaction_mitigated_pct.or_else(|| pre_damage_mitigation.map(|p| p as u8))
    {
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
        // Read the live StatusBag at KO time so reactive seams (e.g.
        // Baby Burner) see the Heated count that was on the target at death.
        let (status_remaining, heated_remaining) = world
            .get::<StatusBag>(tgt_entity)
            .map(|b| {
                (
                    b.iter().map(|inst| inst.kind.clone()).collect::<Vec<_>>(),
                    b.get_dur(&StatusEffectKind::Heated).unwrap_or(0),
                )
            })
            .unwrap_or_else(|| (vec![], 0));
        emit_event(
            world,
            CombatEventKind::UnitDied {
                status_remaining,
                heated_remaining,
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

pub(in crate::combat::runtime::applier) fn apply_break_toughness(
    world: &mut World,
    source: UnitId,
    target: UnitId,
    amount: i32,
    tag: DamageTag,
    cast_id: CastId,
) {
    let Some(source_snapshot) = find_unit_snapshot(world, source) else {
        log::warn!(
            "intent_applier BreakToughness: source {:?} not found",
            source
        );
        return;
    };
    let Some(target_snapshot) = find_unit_snapshot(world, target) else {
        log::warn!(
            "intent_applier BreakToughness: target {:?} not found",
            target
        );
        return;
    };

    let mut break_entity = None;
    let mut broke = false;
    {
        let mut q = world.query::<(
            Entity,
            &Unit,
            &Team,
            Option<&mut Toughness>,
            Option<&mut RoundFlags>,
        )>();
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
        emit_event(
            world,
            CombatEventKind::OnBreak { damage_tag: tag },
            source,
            target,
            cast_id,
        );
        let _ = source_snapshot;
        let _ = target_snapshot;
    }
}
