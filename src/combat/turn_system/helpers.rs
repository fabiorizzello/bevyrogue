use bevy::prelude::*;

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::{CombatBeatId, CombatKernelRegistry, CombatKernelTransition};
use crate::combat::runtime::intent::CastId;
use crate::combat::state::{CombatPhase, CombatState};
use crate::combat::types::UnitId;

pub(crate) fn set_phase(state: &mut CombatState, next: CombatPhase) {
    if state.phase != next {
        debug!("phase: {:?} -> {:?}", state.phase, next);
        state.phase = next;
    }
}

pub(crate) fn emit_combat_event(
    event_writer: &mut MessageWriter<CombatEvent>,
    kind: CombatEventKind,
    source: UnitId,
    target: UnitId,
    follow_up_depth: u8,
    cast_id: CastId,
) {
    debug!(
        target: "combat.events",
        ?kind,
        source = ?source,
        target = ?target,
        follow_up_depth,
        "CombatEvent emitted"
    );
    event_writer.write(CombatEvent {
        kind,
        source,
        target,
        follow_up_depth,
        cast_id,
    });
}

pub(crate) fn emit_kernel_transition(
    event_writer: &mut MessageWriter<CombatEvent>,
    registry: Option<&CombatKernelRegistry>,
    transition: CombatKernelTransition,
    source: UnitId,
    target: UnitId,
    follow_up_depth: u8,
    cast_id: CastId,
) {
    let transitions = registry
        .map(|registry| registry.dispatch(transition.clone()))
        .unwrap_or_else(|| vec![transition]);

    for transition in transitions {
        emit_combat_event(
            event_writer,
            CombatEventKind::OnKernelTransition { transition },
            source,
            target,
            follow_up_depth,
            cast_id,
        );
    }
}

pub(crate) fn emit_combat_beat(
    event_writer: &mut MessageWriter<CombatEvent>,
    registry: Option<&CombatKernelRegistry>,
    beat: CombatBeatId,
    source: UnitId,
    target: UnitId,
    follow_up_depth: u8,
    cast_id: CastId,
) {
    emit_combat_event(
        event_writer,
        CombatEventKind::OnCombatBeat { beat },
        source,
        target,
        follow_up_depth,
        cast_id,
    );
    emit_kernel_transition(
        event_writer,
        registry,
        CombatKernelTransition::Beat(beat),
        source,
        target,
        follow_up_depth,
        cast_id,
    );
}
