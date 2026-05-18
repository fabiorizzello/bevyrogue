pub(super) mod bounce;
pub(super) mod multi_target;
pub(super) mod self_target;
pub(super) mod single_target;

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::CombatKernelRegistry;
use crate::combat::log::{ActionLog, LogEntry};
use crate::combat::runtime::intent::CastId;
use crate::combat::state::InFlightAction;
use bevy::prelude::*;

use super::super::{emit_combat_event, emit_kernel_transition};

pub(super) fn dispatch_blueprint_transitions(
    inflight: &InFlightAction,
    log: &mut ResMut<ActionLog>,
    event_writer: &mut MessageWriter<CombatEvent>,
    registry: Option<&CombatKernelRegistry>,
    cast_id: CastId,
) {
    match crate::combat::blueprints::transitions_for_action_checked(&inflight.action) {
        Ok(transitions) => {
            for transition in transitions {
                emit_kernel_transition(
                    event_writer,
                    registry,
                    transition,
                    inflight.action.source,
                    inflight.action.target,
                    inflight.follow_up_depth,
                    cast_id,
                );
            }
        }
        Err(error) => {
            let reason = error.to_string();
            log.push(LogEntry::ActionFailed {
                reason: reason.clone(),
            });
            emit_combat_event(
                event_writer,
                CombatEventKind::OnActionFailed { reason },
                inflight.action.source,
                inflight.action.target,
                inflight.follow_up_depth,
                cast_id,
            );
        }
    }
}
