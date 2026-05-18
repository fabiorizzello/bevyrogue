use bevy::prelude::*;

use crate::combat::energy::{Energy, RoundEnergyTracker};
use crate::combat::events::CombatEvent;
use crate::combat::kernel::CombatKernelRegistry;
use crate::combat::log::ActionLog;
use crate::combat::rng::{CombatEntropy, CombatRng};
use crate::combat::runtime::intent::CastId;
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatState, InFlightAction};
use crate::combat::turn_order::TurnOrder;
use crate::combat::unit::Unit;

use super::super::ResolveActorsQuery;
use super::paths;

#[allow(clippy::too_many_arguments)]
pub(crate) fn step_app(
    commands: &mut Commands,
    inflight: &InFlightAction,
    state: &mut ResMut<CombatState>,
    sp: &mut ResMut<SpPool>,
    log: &mut ResMut<ActionLog>,
    _turn_order: &mut ResMut<TurnOrder>,
    time: &Res<Time>,
    event_writer: &mut MessageWriter<CombatEvent>,
    registry: Option<&CombatKernelRegistry>,
    actors: &mut ResolveActorsQuery,
    rng: &mut Option<ResMut<CombatRng>>,
    entropy_q: &mut Query<&mut CombatEntropy, With<Unit>>,
    energy_q: &mut Query<(&mut Energy, Option<&mut RoundEnergyTracker>)>,
    cast_id: CastId,
) {
    if inflight.interrupted {
        return;
    }

    let attacker_id = inflight.action.source;
    let target_id = inflight.action.target;

    #[cfg(debug_assertions)]
    let _combat_apply_span = bevy::log::info_span!(
        target: "combat.apply",
        "combat.apply",
        source = ?attacker_id,
        defender = ?target_id,
        skill_id = ?inflight.action.skill_id,
        follow_up_depth = inflight.follow_up_depth,
        cast_id = ?cast_id,
    )
    .entered();

    let attacker_entity = actors.iter().find_map(|(entity, _, unit, ..)| {
        if unit.id == attacker_id {
            Some(entity)
        } else {
            None
        }
    });
    let target_entity = actors.iter().find_map(|(entity, _, unit, ..)| {
        if unit.id == target_id {
            Some(entity)
        } else {
            None
        }
    });
    let (Some(attacker_entity), Some(target_entity)) = (attacker_entity, target_entity) else {
        return;
    };

    if paths::multi_target::run(
        commands,
        inflight,
        state,
        sp,
        log,
        _turn_order,
        time,
        event_writer,
        registry,
        actors,
        energy_q,
        cast_id,
        attacker_entity,
        target_entity,
        attacker_id,
        target_id,
    ) {
        return;
    }

    if paths::bounce::run(
        commands,
        inflight,
        state,
        sp,
        log,
        _turn_order,
        time,
        event_writer,
        registry,
        actors,
        energy_q,
        cast_id,
        attacker_entity,
        target_entity,
        attacker_id,
        target_id,
    ) {
        return;
    }

    if paths::self_target::run(
        commands,
        inflight,
        state,
        sp,
        log,
        _turn_order,
        time,
        event_writer,
        registry,
        actors,
        energy_q,
        cast_id,
        attacker_entity,
        target_entity,
        attacker_id,
        target_id,
    ) {
        return;
    }

    paths::single_target::run(
        commands,
        inflight,
        state,
        sp,
        log,
        _turn_order,
        time,
        event_writer,
        registry,
        actors,
        rng,
        entropy_q,
        energy_q,
        cast_id,
        attacker_entity,
        target_entity,
        attacker_id,
        target_id,
    );
}
