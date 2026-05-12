use crate::combat::av::{AV_PER_SPEED, ActionValue, ActionValueUpdated, MAX_AV};
use crate::combat::enemy_ai;
use crate::combat::resistance::{self, TempoResistance};
use crate::combat::rng::CombatRng;
use crate::combat::{
    StatusEffect,
    action_query::{ActionQueryKind, build_snapshot_from_ecs, query_intent_legality},
    enemy_counterplay::EnemyCounterplayKit,
    energy::{Energy, RoundEnergyTracker},
    events::{ActionIntentKind, CombatEvent, CombatEventKind},
    kernel::{CombatBeatId, CombatKernelRegistry, CombatKernelTransition},
    kit::UnitSkills,
    log::ActionLog,
    round_flags::RoundFlags,
    sp::SpPool,
    speed::Speed,
    speed::SpeedModifier,
    state::{CombatPhase, CombatState},
    status_effect::StatusEffectKind,
    stun::Stunned,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    types::{SkillId, UnitId},
    ultimate::UltimateCharge,
    unit::{BasicStreak, Commander, Ko, Unit},
};
use crate::data::{SkillBookHandle, skills_ron::SkillBook};
use bevy::prelude::*;

pub const TICK_AV_AMOUNT: i32 = 1000; // Arbitrary tick amount for AV accumulation. For deterministic turns
// this could be based on a fixed percentage or smallest speed denominator.
// Using 1000 for now to ensure multiple units can cross MAX_AV without too many sub-ticks.

#[allow(dead_code)]
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum ActionIntent {
    Basic {
        attacker: UnitId,
        target: UnitId,
    },
    Skill {
        attacker: UnitId,
        skill_id: SkillId,
        target: UnitId,
    },
    Ultimate {
        attacker: UnitId,
        target: UnitId,
    },
}

pub(crate) type ResolveActorsQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Team,
        &'static mut Unit,
        Option<&'static UnitSkills>,
        Option<&'static mut UltimateCharge>,
        Option<&'static mut Toughness>,
        Option<&'static EnemyCounterplayKit>,
        Option<&'static Ko>,
        Option<&'static Stunned>,
        Option<&'static Commander>,
        Option<&'static mut StatusEffect>,
        Option<&'static mut BasicStreak>,
        Option<&'static mut RoundFlags>,
    ),
>;

pub(super) fn set_phase(state: &mut CombatState, next: CombatPhase) {
    if state.phase != next {
        debug!("phase: {:?} -> {:?}", state.phase, next);
        state.phase = next;
    }
}

#[allow(dead_code)]
fn intent_label(intent: &ActionIntent) -> &'static str {
    match intent {
        ActionIntent::Basic { .. } => "Basic",
        ActionIntent::Skill { .. } => "Skill",
        ActionIntent::Ultimate { .. } => "Ultimate",
    }
}

pub(crate) fn emit_combat_event(
    event_writer: &mut MessageWriter<CombatEvent>,
    kind: CombatEventKind,
    source: UnitId,
    target: UnitId,
    follow_up_depth: u8,
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
    });
}

pub(crate) fn emit_kernel_transition(
    event_writer: &mut MessageWriter<CombatEvent>,
    registry: Option<&CombatKernelRegistry>,
    transition: CombatKernelTransition,
    source: UnitId,
    target: UnitId,
    follow_up_depth: u8,
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
) {
    emit_combat_event(
        event_writer,
        CombatEventKind::OnCombatBeat { beat },
        source,
        target,
        follow_up_depth,
    );
    emit_kernel_transition(
        event_writer,
        registry,
        CombatKernelTransition::Beat(beat),
        source,
        target,
        follow_up_depth,
    );
}

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
    mut energy_q: Query<(&mut Energy, Option<&mut RoundEnergyTracker>)>,
) {
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
            super::follow_up::FollowUpOriginKind::FollowUp,
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
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::Declared,
            inflight.action.source,
            inflight.action.target,
            0,
        );
        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionPreApp,
            inflight.action.source,
            inflight.action.target,
            0,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::PreApp,
            inflight.action.source,
            inflight.action.target,
            0,
        );

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
            &mut energy_q,
        );

        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionApplied,
            inflight.action.source,
            inflight.action.target,
            0,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::Applied,
            inflight.action.source,
            inflight.action.target,
            0,
        );
        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionResolved,
            inflight.action.source,
            inflight.action.target,
            0,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::Resolved,
            inflight.action.source,
            inflight.action.target,
            0,
        );
    }
}

pub fn advance_turn_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Unit,
            &Team,
            Option<&Speed>,
            Option<&SpeedModifier>,
            Option<&mut ActionValue>,
            Option<&mut Stunned>,
            Option<&mut StatusEffect>,
            Option<&UnitSkills>,
            Option<&UltimateCharge>,
            Option<&Toughness>,
            Option<&Commander>,
            Option<&mut RoundFlags>,
            Option<&mut RoundEnergyTracker>,
        ),
        Without<Ko>,
    >,
    mut turn_order: ResMut<TurnOrder>,
    mut state: ResMut<CombatState>,
    mut turn_flow: ParamSet<(MessageReader<TurnAdvanced>, MessageWriter<TurnAdvanced>)>,
    mut event_writer: MessageWriter<CombatEvent>,
    mut intents_out: MessageWriter<ActionIntent>,
    mut av_event_writer: MessageWriter<ActionValueUpdated>,
    mut combat_rng: Option<ResMut<CombatRng>>,
) {
    // === Part 1: Process incoming TurnAdvanced messages ===
    // Collect snapshots first so we can do enemy AI after mutable status tick
    struct Snap {
        entity: Entity,
        id: UnitId,
        team: Team,
        is_stunned: bool,
        hp_current: i32,
        hp_max: i32,
        toughness_current: i32,
        toughness_max: i32,
        is_commander: bool,
        skills: Option<UnitSkills>,
        ult_ready: bool,
    }
    let snapshots: Vec<Snap> = query
        .iter_mut()
        .map(
            |(entity, unit, team, _, _, _, stunned, _, skills, ult, toughness, commander, _, _)| {
                Snap {
                    entity,
                    id: unit.id,
                    team: *team,
                    is_stunned: stunned.is_some(),
                    hp_current: unit.hp_current,
                    hp_max: unit.hp_max,
                    toughness_current: toughness.map(|t| t.current).unwrap_or(0),
                    toughness_max: toughness.map(|t| t.max).unwrap_or(1),
                    is_commander: commander.is_some(),
                    skills: skills.cloned(),
                    ult_ready: ult.map(|u| u.ready()).unwrap_or(false),
                }
            },
        )
        .collect();

    let turn_events: Vec<TurnAdvanced> = turn_flow.p0().read().copied().collect();

    for TurnAdvanced {
        unit_id: active_id, ..
    } in turn_events
    {
        let Some(snap) = snapshots.iter().find(|s| s.id == active_id) else {
            continue;
        };

        let mut shock_cancelled = false;
        {
            let Ok((
                _,
                mut unit,
                _,
                _,
                _,
                _,
                stunned_opt,
                status_opt,
                _,
                _,
                _,
                _,
                mut round_flags_opt,
                mut round_energy_tracker_opt,
            )) = query.get_mut(snap.entity)
            else {
                continue;
            };
            if let Some(ref mut flags) = round_flags_opt {
                flags.break_sealed = false;
                flags.form_identity_used = false;
                flags.acted_last_turn = flags.acted_this_turn;
                flags.acted_this_turn = false;
                flags.hits_received_this_round = 0;
            }
            if let Some(ref mut tracker) = round_energy_tracker_opt {
                tracker.reset();
            }

            if let Some(mut s) = stunned_opt {
                if s.tick() {
                    commands.entity(snap.entity).remove::<Stunned>();
                }
                drop(status_opt);
                drop(unit);
                continue;
            }

            if let Some(mut se) = status_opt {
                let kind = se.kind.clone();
                // Per-status semantics (DoT, speed delta, cancel probability, ult boost)
                // are implemented in S03–S05. This is the v0 lifecycle skeleton only.
                match &kind {
                    StatusEffectKind::Heated
                    | StatusEffectKind::Chilled
                    | StatusEffectKind::Paralyzed
                    | StatusEffectKind::Slowed
                    | StatusEffectKind::Blessed
                    | StatusEffectKind::Burn
                    | StatusEffectKind::Shock => {}
                }
                let expired = se.tick();
                let turns_left = se.duration_remaining;
                emit_combat_event(
                    &mut event_writer,
                    CombatEventKind::OnStatusTick {
                        kind: kind.clone(),
                        turns_left,
                    },
                    active_id,
                    active_id,
                    0,
                );
                if expired {
                    commands.entity(snap.entity).remove::<StatusEffect>();
                    emit_combat_event(
                        &mut event_writer,
                        CombatEventKind::OnStatusExpired { kind: kind.clone() },
                        active_id,
                        active_id,
                        0,
                    );
                }
            }
        } // mutable borrow released

        // Enemy AI: emit ActionIntent for enemy units whose action wasn't cancelled
        if snap.team == Team::Enemy && !shock_cancelled && !snap.is_stunned {
            let fallback_skills;
            let skills_ref: &UnitSkills = if let Some(s) = snap.skills.as_ref() {
                s
            } else {
                fallback_skills = UnitSkills {
                    basic: SkillId(String::new()),
                    skills: Vec::new(),
                    ultimate: SkillId(String::new()),
                    follow_up: None,
                };
                &fallback_skills
            };
            let ally_targets: Vec<enemy_ai::TargetInfo> = snapshots
                .iter()
                .filter(|s| s.team == Team::Ally && !s.is_commander)
                .map(|s| enemy_ai::TargetInfo {
                    id: s.id,
                    toughness_current: s.toughness_current,
                    toughness_max: s.toughness_max,
                    hp_current: s.hp_current,
                    hp_max: s.hp_max,
                })
                .collect();
            let ctx = enemy_ai::EnemyTurnContext {
                attacker_id: snap.id,
                attacker_skills: skills_ref,
                attacker_ult_ready: snap.ult_ready,
                targets: &ally_targets,
            };
            if let Some(intent) = enemy_ai::pick_enemy_action(&ctx) {
                intents_out.write(intent);
            }
        }
    }

    // === Part 2: AV advancement (only in WaitingForTurn phase) ===
    if state.phase != CombatPhase::WaitingForTurn {
        return;
    }

    let mut units_ready: Vec<(UnitId, Entity, i32)> = Vec::new();

    for (entity, unit, _, speed_opt, speed_mod_opt, av_opt, stunned, _, _, _, _, _, _, _) in
        query.iter_mut()
    {
        if stunned.is_some() {
            continue;
        }
        let (Some(speed), Some(speed_mod), Some(mut av)) = (speed_opt, speed_mod_opt, av_opt)
        else {
            continue;
        };
        let av_gain = (speed.0 + speed_mod.0) * AV_PER_SPEED;
        let old_av = av.0;
        av.advance(av_gain);
        av_event_writer.write(ActionValueUpdated {
            unit_id: entity,
            old_value: old_av,
            new_value: av.0,
        });
        if av.is_ready() {
            units_ready.push((unit.id, entity, av.0));
        }
    }

    units_ready.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.0.cmp(&b.0.0)));

    if let Some((unit_id_ready, entity_ready, _)) = units_ready.first() {
        if turn_order.active_unit.is_none() {
            let Ok((_, _, _, _, _, Some(mut av), _, _, _, _, _, _, _, _)) =
                query.get_mut(*entity_ready)
            else {
                return;
            };
            let old_av_val = av.0;
            av.reset();
            turn_flow.p1().write(TurnAdvanced {
                unit_id: *unit_id_ready,
                av_at_turn: old_av_val,
                av_change: MAX_AV,
            });
            turn_order.active_unit = Some(*unit_id_ready);
            set_phase(&mut *state, CombatPhase::WaitingAction);
        }
    }
}

/// Processes `CombatEvent::TurnAdvance` messages and applies the corresponding AV delta
/// to the target unit, factoring in `TempoResistance` for negative (Delay) amounts.
pub fn apply_turn_advance_system(
    mut events: MessageReader<crate::combat::events::CombatEvent>,
    mut units: Query<(
        &crate::combat::unit::Unit,
        &mut ActionValue,
        Option<&mut TempoResistance>,
    )>,
) {
    use crate::combat::events::CombatEventKind;
    for event in events.read() {
        let CombatEventKind::TurnAdvance { target, amount_pct } = &event.kind else {
            continue;
        };
        for (unit, mut av, mut res) in &mut units {
            if unit.id == *target {
                resistance::apply_av_change(&mut av, res.as_deref_mut(), *amount_pct);
                break;
            }
        }
    }
}

pub fn check_victory_system(
    mut state: ResMut<CombatState>,
    roster: Query<(&Unit, &Team, Option<&Ko>)>,
) {
    if state.winner.is_some() {
        return;
    }

    let mut ally_alive = false;
    let mut enemy_alive = false;
    let mut saw_ally = false;
    let mut saw_enemy = false;
    for (unit, team, ko) in &roster {
        let alive = ko.is_none() && unit.hp_current > 0;
        match team {
            Team::Ally => {
                saw_ally = true;
                ally_alive |= alive;
            }
            Team::Enemy => {
                saw_enemy = true;
                enemy_alive |= alive;
            }
        }
    }

    if !saw_ally || !saw_enemy {
        return;
    }

    if !enemy_alive {
        set_phase(&mut state, CombatPhase::Victory);
        state.winner = Some(Team::Ally);
        info!("victory: team=Ally");
    } else if !ally_alive {
        set_phase(&mut state, CombatPhase::Defeat);
        state.winner = Some(Team::Enemy);
        info!("defeat: team=Enemy");
    }
}

mod pipeline;

pub(crate) use pipeline::{step_app, step_declaration};

#[cfg(test)]
mod tests;
