use super::*;
use crate::combat::runtime::intent::{CastId, CastIdGen};
use crate::combat::rng::CombatRng;
use crate::combat::{
    action_query::{ActionQueryKind, build_snapshot_from_ecs, query_intent_legality},
    energy::{Energy, RoundEnergyTracker},
    events::{ActionIntentKind, CombatEvent, CombatEventKind},
    kernel::{CombatBeatId, CombatKernelRegistry},
    log::ActionLog,
    sp::SpPool,
    state::{CombatPhase, CombatState},
    turn_order::TurnOrder,
};
use crate::data::{SkillBookHandle, skills_ron::SkillBook};
use bevy::prelude::*;

pub fn resolve_action_system(
    mut commands: Commands,
    mut intents: MessageReader<ActionIntent>,
    mut state: ResMut<CombatState>,
    mut sp: ResMut<SpPool>,
    mut log: ResMut<ActionLog>,
    mut turn_order: ResMut<TurnOrder>,
    time: Res<Time>,
    skill_books: Res<Assets<SkillBook>>,
    skill_book_handle: Option<Res<SkillBookHandle>>,
    mut event_writer: MessageWriter<CombatEvent>,
    registry: Option<Res<CombatKernelRegistry>>,
    mut actors: ResolveActorsQuery,
    mut combat_rng: Option<ResMut<CombatRng>>,
    mut entropy_q: Query<
        &mut crate::combat::rng::CombatEntropy,
        With<crate::combat::unit::Unit>,
    >,
    mut energy_q: Query<(&mut Energy, Option<&mut RoundEnergyTracker>)>,
    mut cast_id_gen: Option<ResMut<CastIdGen>>,
) {
    if state.phase == CombatPhase::Resolving {
        let dropped = intents.read().count();
        if dropped > 0 {
            debug!(
                target: "combat.timeline_barrier",
                dropped,
                "ignoring action intents while combat phase is Resolving"
            );
        }
        return;
    }

    if let Some(intent) = intents.read().next() {
        let (actor_id, target_id, query_kind) = match intent {
            ActionIntent::Basic { attacker, target } => {
                (*attacker, *target, ActionQueryKind::Basic)
            }
            ActionIntent::Skill {
                attacker,
                skill_id,
                target,
            } => (*attacker, *target, ActionQueryKind::Skill(skill_id)),
            ActionIntent::Ultimate { attacker, target } => {
                (*attacker, *target, ActionQueryKind::Ultimate)
            }
        };

        #[cfg(debug_assertions)]
        let _combat_resolution_span = bevy::log::info_span!(
            target: "combat.resolution",
            "combat.resolution",
            actor = ?actor_id,
            defender = ?target_id,
            intent = ?query_kind,
        )
        .entered();

        // Early Legality Validation
        if let Some(skill_book) = skill_book_handle
            .as_ref()
            .and_then(|h| skill_books.get(&h.0))
        {
            let actors_readonly = actors.as_readonly();
            let energy_readonly = energy_q.as_readonly();
            let units_data: Vec<_> = actors_readonly
                .iter()
                .map(
                    |(
                        entity,
                        team,
                        unit,
                        skills,
                        ult,
                        toughness,
                        counterplay,
                        ko,
                        stunned,
                        commander,
                        _,
                        _,
                        _,
                        _,
                        _,
                    )| {
                        let energy_data = energy_readonly.get(entity).ok();
                        let energy = energy_data.map(|(e, _)| e);
                        let tracker = energy_data.and_then(|(_, t)| t);
                        (
                            unit.id,
                            *team,
                            unit,
                            skills,
                            ult,
                            toughness,
                            counterplay,
                            ko.is_some(),
                            stunned.is_some(),
                            commander.is_some(),
                            energy,
                            tracker,
                        )
                    },
                )
                .collect();

            let snapshot =
                build_snapshot_from_ecs(&state, &turn_order, &sp, actor_id, target_id, units_data);

            if let Err(reason) =
                query_intent_legality(&snapshot, skill_book, actor_id, &query_kind, target_id)
            {
                let reason_str = format!("{:?}", reason);
                log.events
                    .push_back(crate::combat::log::LogEntry::ActionFailed {
                        reason: reason_str.clone(),
                    });
                event_writer.write(CombatEvent {
                    kind: CombatEventKind::OnActionFailed { reason: reason_str },
                    source: actor_id,
                    target: target_id,
                    follow_up_depth: 0,
                    cast_id: CastId::ROOT,
                });
                return;
            }
        }

        let intent_kind = match intent {
            ActionIntent::Basic { .. } => ActionIntentKind::Basic,
            ActionIntent::Skill { .. } => ActionIntentKind::Skill,
            ActionIntent::Ultimate { .. } => ActionIntentKind::Ultimate,
        };

        let Some(inflight) = pipeline::step_declaration(
            &mut commands,
            &intent,
            0,
            &mut state,
            crate::combat::follow_up::FollowUpOriginKind::FollowUp,
            &skill_books,
            skill_book_handle.as_ref(),
            &mut log,
            &mut event_writer,
            &mut actors,
        ) else {
            return;
        };

        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionDeclared { intent_kind },
            inflight.action.source,
            inflight.action.target,
            0,
            CastId::ROOT,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::Declared,
            inflight.action.source,
            inflight.action.target,
            0,
            CastId::ROOT,
        );
        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionPreApp,
            inflight.action.source,
            inflight.action.target,
            0,
            CastId::ROOT,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::PreApp,
            inflight.action.source,
            inflight.action.target,
            0,
            CastId::ROOT,
        );

        let action_cast_id = cast_id_gen
            .as_deref_mut()
            .map(|g| g.next())
            .unwrap_or(CastId::ROOT);

        let use_timeline = skill_book_handle
            .as_ref()
            .and_then(|h| skill_books.get(&h.0))
            .and_then(|book| {
                book.0
                    .iter()
                    .find(|skill| skill.id == inflight.action.skill_id)
            })
            .and_then(|skill| skill.timeline.as_ref())
            .is_some();

        if use_timeline {
            commands.queue(move |world: &mut bevy::prelude::World| {
                pipeline::run_timeline_backed_action(world, inflight, action_cast_id);
            });
        } else {
            pipeline::step_app(
                &mut commands,
                &inflight,
                &mut state,
                &mut sp,
                &mut log,
                &mut turn_order,
                &time,
                &mut event_writer,
                registry.as_deref(),
                &mut actors,
                &mut combat_rng,
                &mut entropy_q,
                &mut energy_q,
                action_cast_id,
            );

            emit_combat_event(
                &mut event_writer,
                CombatEventKind::OnActionApplied,
                inflight.action.source,
                inflight.action.target,
                0,
                CastId::ROOT,
            );
            emit_combat_beat(
                &mut event_writer,
                registry.as_deref(),
                CombatBeatId::Applied,
                inflight.action.source,
                inflight.action.target,
                0,
                CastId::ROOT,
            );
            emit_combat_event(
                &mut event_writer,
                CombatEventKind::OnActionResolved,
                inflight.action.source,
                inflight.action.target,
                0,
                CastId::ROOT,
            );
            emit_combat_beat(
                &mut event_writer,
                registry.as_deref(),
                CombatBeatId::Resolved,
                inflight.action.source,
                inflight.action.target,
                0,
                CastId::ROOT,
            );
        }
    }
}

