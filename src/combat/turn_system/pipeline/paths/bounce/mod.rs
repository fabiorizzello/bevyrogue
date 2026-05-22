use bevy::prelude::*;

use crate::combat::energy::Energy;
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::{CombatBeatId, CombatKernelRegistry};
use crate::combat::log::{ActionLog, LogEntry};
use crate::combat::runtime::intent::CastId;
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatPhase, CombatState, InFlightAction, UltEffect};
use crate::combat::turn_order::TurnOrder;
use crate::combat::types::UnitId;
use crate::combat::ult_gauge::UltGaugeMetadata;
use crate::data::skills_ron::TargetShape;

use super::super::super::{ResolveActorsQuery, emit_combat_beat, emit_combat_event, set_phase};

mod finalize;
mod hops;

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
    let TargetShape::Bounce {
        hops,
        selector,
        repeat,
    } = inflight.action.target_shape
    else {
        return false;
    };
    // Phase 0: build entity→id map (read-only pass released before mut borrows).
    let actor_pairs: Vec<(Entity, UnitId)> = actors
        .iter()
        .map(|(entity, _, unit, ..)| (entity, unit.id))
        .collect();

    // Phase 1: attacker validation + resource consumption (hoisted, paid once pre-loop).
    {
        let Ok((_, _, _, _, att_ult_opt, _, _, att_ko, att_stun, _, _, _, _, _)) =
            actors.get_mut(attacker_entity)
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

    hops::run_hop_loop(
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
        target_id,
        hops,
        selector,
        repeat,
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
