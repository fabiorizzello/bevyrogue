use std::collections::VecDeque;

use bevy::log;
use bevy::prelude::*;

use crate::combat::{
    api::{
        blueprint_state::BlueprintState,
        intent::{CastId, Intent},
        signal::{Signal, SignalBus, SignalPayload, SignalTaxonomy},
    },
    damage::{AttackContext, calculate_damage},
    events::{CombatEvent, CombatEventKind, CombatKernelTransition},
    toughness::{DamageKind, Toughness},
    types::{DamageTag, UnitId},
    unit::Unit,
};

/// Pending `Intent` queue drained each frame by `intent_applier`.
///
/// Skill hook functions push to this queue via `SkillCtx::enqueue`; the applier
/// system is the single consumer. Direct mutations from outside `intent_applier`
/// violate the P001 single-source-of-truth invariant.
#[derive(Resource, Default)]
pub struct IntentQueue(pub VecDeque<Intent>);

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
    weaknesses: Vec<DamageTag>,
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
        let mut q = world.query::<(Entity, &Unit, Option<&Toughness>)>();
        q.iter(world)
            .map(|(e, u, t)| UnitSnapshot {
                entity: e,
                id: u.id,
                unit: u.clone(),
                weaknesses: t
                    .map(|tg| tg.weaknesses.clone())
                    .unwrap_or_default(),
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

    let attack = AttackContext {
        damage_tag: tag,
        base_damage,
        is_break: false,
    };
    let bd = calculate_damage(&src.unit, &attack, &tgt.unit, &tgt.weaknesses, None, 1.0, None);

    let tgt_entity = tgt.entity;

    // Apply HP mutation.
    if let Some(mut u) = world.get_mut::<Unit>(tgt_entity) {
        u.hp_current -= bd.final_damage;
    }

    // Emit OnDamageDealt event.
    world
        .resource_mut::<Messages<CombatEvent>>()
        .write(CombatEvent {
            kind: CombatEventKind::OnDamageDealt {
                amount: bd.final_damage,
                kind: DamageKind::Normal,
                tag_mod_pct: bd.tag_mod_pct,
                triangle_mod_pct: bd.triangle_mod_pct,
                damage_tag: tag,
            },
            source,
            target,
            follow_up_depth: 0,
            cast_id,
        });
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
