use bevy::log;
use bevy::prelude::*;

use crate::combat::{
    runtime::intent::CastId,
    damage::triangle_modifiers,
    events::CombatEventKind,
    modifiers::{DamageModifierLedger, ModifierLayer},
    rng::CombatRng,
    status_effect::{StatusBag, StatusEffectKind},
    types::UnitId,
    unit::Unit,
};

use super::super::{emit_event, find_unit_snapshot};

pub(in crate::combat::runtime::applier) fn apply_status(
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
        let tri = triangle_modifiers(
            source_snapshot.unit.attribute,
            target_snapshot.unit.attribute,
        );
        let threshold = (tri.status_acc_modifier * 100.0) as i32;
        let passes = world.get_resource_mut::<CombatRng>().map_or_else(
            || CombatRng::default().roll_pct(threshold),
            |mut rng| rng.roll_pct(threshold),
        );
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
                && bag
                    .as_deref()
                    .map_or(true, |b| !b.has(&StatusEffectKind::Slowed));
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
        log::warn!(
            "intent_applier ApplyStatus: target entity {:?} not found",
            target
        );
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

pub(in crate::combat::runtime::applier) fn apply_buff(
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

pub(in crate::combat::runtime::applier) fn apply_damage_modifier(
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
