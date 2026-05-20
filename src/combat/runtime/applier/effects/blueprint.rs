use bevy::log;
use bevy::prelude::*;

use crate::combat::{
    events::{CombatEvent, CombatEventKind, CombatKernelTransition},
    runtime::{
        blueprint_state::BlueprintState,
        intent::CastId,
        signal::{Signal, SignalBus, SignalPayload, SignalTaxonomy},
    },
    types::UnitId,
};

use super::super::allied_basic_skill_ids;

pub(in crate::combat::runtime::applier) fn apply_grant_free_skill(
    world: &mut World,
    source: UnitId,
    count: usize,
    cast_id: CastId,
) {
    if count == 0 {
        return;
    }

    let Some(ally_basics) = allied_basic_skill_ids(world, source) else {
        log::warn!(
            "intent_applier GrantFreeSkill: source {:?} not found",
            source
        );
        return;
    };

    for (ally_id, skill_id) in ally_basics.into_iter().take(count) {
        world
            .resource_mut::<Messages<CombatEvent>>()
            .write(CombatEvent {
                kind: CombatEventKind::OnSkillCast { skill_id },
                source: ally_id,
                target: ally_id,
                follow_up_depth: 0,
                cast_id,
            });
    }
}

pub(in crate::combat::runtime::applier) fn apply_blueprint_signal(
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

    world
        .resource_mut::<Messages<CombatEvent>>()
        .write(CombatEvent {
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

pub(in crate::combat::runtime::applier) fn apply_set_blueprint_state(
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
