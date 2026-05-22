use bevy::log;
use bevy::prelude::*;

use crate::combat::{
    energy::Energy,
    events::CombatEventKind,
    runtime::intent::CastId,
    types::UnitId,
    unit::{Ko, Unit},
};

use super::super::{emit_event, find_unit_entity, find_unit_snapshot};

pub(in crate::combat::runtime::applier) fn apply_delay_turn(
    world: &mut World,
    target: UnitId,
    amount_pct: u32,
    cast_id: CastId,
) {
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

pub(in crate::combat::runtime::applier) fn apply_advance_turn(
    world: &mut World,
    target: UnitId,
    amount_pct: u32,
    cast_id: CastId,
) {
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

pub(in crate::combat::runtime::applier) fn apply_add_energy(
    world: &mut World,
    target: UnitId,
    amount: i32,
    cast_id: CastId,
) {
    if amount <= 0 {
        return;
    }

    let Some((entity, _unit)) = find_unit_entity(world, target) else {
        log::warn!("intent_applier AddEnergy: target {:?} not found", target);
        return;
    };

    let gained = if let Some(mut energy) = world.get_mut::<Energy>(entity) {
        energy.gain_capped(amount)
    } else {
        log::warn!(
            "intent_applier AddEnergy: target {:?} missing Energy component",
            target
        );
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

pub(in crate::combat::runtime::applier) fn apply_revive(
    world: &mut World,
    source: UnitId,
    target: UnitId,
    pct: i32,
    cast_id: CastId,
) {
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
