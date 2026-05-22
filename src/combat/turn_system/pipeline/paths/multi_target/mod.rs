use bevy::prelude::*;

use crate::combat::energy::Energy;
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::{CombatBeatId, CombatKernelRegistry};
use crate::combat::log::{ActionLog, LogEntry};
use crate::combat::resolution::{TargetEntry, TargetableSnapshot, resolve_targets};
use crate::combat::runtime::intent::CastId;
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatPhase, CombatState, InFlightAction, UltEffect};
use crate::combat::turn_order::TurnOrder;
use crate::combat::types::UnitId;
use crate::combat::ult_gauge::UltGaugeMetadata;
use crate::combat::unit::SlotIndex;
use crate::data::skills_ron::TargetShape;

use super::super::super::{ResolveActorsQuery, emit_combat_beat, emit_combat_event, set_phase};

mod finalize;
mod loops;

#[allow(clippy::too_many_arguments)]
pub(in crate::combat::turn_system::pipeline) fn run(
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
    energy_q: &mut Query<&mut Energy>,
    gauge_meta_q: &Query<&UltGaugeMetadata>,
    cast_id: CastId,
    attacker_entity: Entity,
    _target_entity: Entity,
    attacker_id: UnitId,
    target_id: UnitId,
) -> bool {
    if !matches!(
        inflight.action.target_shape,
        TargetShape::Blast | TargetShape::AllEnemies | TargetShape::AllAllies
    ) {
        return false;
    }
    // Phase 0: build entity→id map and snapshot (read-only pass, released before mut borrows)
    let actor_pairs: Vec<(Entity, UnitId)> = actors
        .iter()
        .map(|(entity, _, unit, ..)| (entity, unit.id))
        .collect();
    let snapshot = {
        let entries = actors
            .iter()
            .map(|(_, team, unit, _, _, _, _, ko, _, _, _, _, slot, _)| {
                let alive = ko.is_none() && unit.hp_current > 0;
                let hp_per_mille = if unit.hp_max > 0 {
                    ((unit.hp_current.max(0) as u64 * 1000) / unit.hp_max as u64) as u32
                } else {
                    0
                };
                TargetEntry {
                    id: unit.id,
                    team: *team,
                    slot_index: slot.map(|s: &SlotIndex| s.0).unwrap_or(0),
                    alive,
                    hp_per_mille,
                }
            })
            .collect();
        TargetableSnapshot { entries }
    };

    let target_ids = resolve_targets(
        &inflight.action.target_shape,
        inflight.action.target,
        &snapshot,
    );

    if target_ids.is_empty() {
        set_phase(state, CombatPhase::WaitingAction);
        return true;
    }

    // Phase 1: attacker validation + resource consumption (mut borrow released after block)
    {
        let Ok((
            _,
            _,
            _,
            _,
            att_ult_opt,
            _,
            _,
            att_ko,
            att_stun,
            _,
            _,
            _,
            _,
            _,
        )) = actors.get_mut(attacker_entity)
        else {
            return true;
        };

        if att_stun.is_some() {
            log.push(LogEntry::ActionFailed {
                reason: "Attacker is stunned".to_string(),
            });
            emit_combat_event(
                event_writer,
                CombatEventKind::OnActionFailed {
                    reason: "Attacker is stunned".to_string(),
                },
                attacker_id,
                target_id,
                inflight.follow_up_depth,
                cast_id,
            );
            set_phase(state, CombatPhase::WaitingAction);
            return true;
        }
        if att_ko.is_some() {
            log.push(LogEntry::ActionFailed {
                reason: "Attacker is KO".to_string(),
            });
            emit_combat_event(
                event_writer,
                CombatEventKind::OnActionFailed {
                    reason: "Attacker is KO".to_string(),
                },
                attacker_id,
                target_id,
                inflight.follow_up_depth,
                cast_id,
            );
            set_phase(state, CombatPhase::WaitingAction);
            return true;
        }

        let Some(att_ult) = att_ult_opt else {
            return true;
        };

        let effective_sp_cost = inflight.action.sp_cost;

        if effective_sp_cost > 0 && !sp.spend(effective_sp_cost) {
            emit_combat_event(
                event_writer,
                CombatEventKind::OnActionFailed {
                    reason: "SP shortfall".to_string(),
                },
                attacker_id,
                target_id,
                inflight.follow_up_depth,
                cast_id,
            );
            set_phase(state, CombatPhase::WaitingAction);
            return true;
        }

        if matches!(inflight.action.ult_effect, UltEffect::Reset) && !att_ult.ready() {
            emit_combat_event(
                event_writer,
                CombatEventKind::OnActionFailed {
                    reason: "SP shortfall".to_string(),
                },
                attacker_id,
                target_id,
                inflight.follow_up_depth,
                cast_id,
            );
            set_phase(state, CombatPhase::WaitingAction);
            return true;
        }
    } // attacker mut borrow released

    // Phase 2: per-target damage loop
    set_phase(state, CombatPhase::Resolving);
    emit_combat_beat(
        event_writer,
        registry,
        CombatBeatId::Impact,
        attacker_id,
        target_id,
        inflight.follow_up_depth,
        cast_id,
    );

    loops::run_target_loop(
        commands,
        inflight,
        log,
        time,
        event_writer,
        registry,
        actors,
        cast_id,
        attacker_entity,
        attacker_id,
        &target_ids,
        &actor_pairs,
    );

    finalize::finalize(
        inflight,
        state,
        sp,
        log,
        event_writer,
        registry,
        actors,
        energy_q,
        gauge_meta_q,
        cast_id,
        attacker_entity,
        attacker_id,
        target_id,
    )
}
