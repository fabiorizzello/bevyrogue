use crate::combat::api::intent::{CastId, CastIdGen};
use crate::combat::av::{AV_PER_SPEED, ActionValue, ActionValueUpdated, MAX_AV};
use crate::combat::buffs::DrBag;
use crate::combat::enemy_ai;
use crate::combat::resistance::{self, TempoResistance};
use crate::combat::rng::CombatRng;
use crate::combat::{
    StatusBag,
    action_query::{ActionQueryKind, build_snapshot_from_ecs, query_intent_legality},
    enemy_counterplay::EnemyCounterplayKit,
    energy::{Energy, RoundEnergyTracker},
    events::{ActionIntentKind, CombatEvent, CombatEventKind},
    kernel::{CombatBeatId, CombatKernelRegistry, CombatKernelTransition},
    kit::UnitSkills,
    log::ActionLog,
    preview::{summarize_preview_damage, try_query_skill_preview},
    round_flags::RoundFlags,
    sp::SpPool,
    speed::Speed,
    speed::SpeedModifier,
    state::{CombatPhase, CombatState},
    status_effect::StatusEffectKind,
    stun::Stunned,
    team::Team,
    toughness::{DamageKind, Toughness},
    turn_order::{TurnAdvanced, TurnOrder},
    types::{DamageTag, SkillId, UnitId},
    ultimate::UltimateCharge,
    unit::{BasicStreak, Commander, Ko, SlotIndex, Unit},
};
use crate::data::{SkillBookHandle, skills_ron::SkillBook};
use bevy::prelude::*;

pub const TICK_AV_AMOUNT: i32 = 1000; // Arbitrary tick amount for AV accumulation. For deterministic turns
// this could be based on a fixed percentage or smallest speed denominator.
// Using 1000 for now to ensure multiple units can cross MAX_AV without too many sub-ticks.

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

#[derive(Resource, Debug, Default, Clone)]
pub struct EnemyTurnRequestQueue(pub Vec<UnitId>);

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
        Option<&'static mut StatusBag>,
        Option<&'static mut BasicStreak>,
        Option<&'static mut RoundFlags>,
        Option<&'static SlotIndex>,
        Option<&'static mut DrBag>,
    ),
>;

pub(super) fn set_phase(state: &mut CombatState, next: CombatPhase) {
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
    mut cast_id_gen: Option<ResMut<CastIdGen>>,
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
            Option<&mut StatusBag>,
            Option<&UnitSkills>,
            Option<&UltimateCharge>,
            Option<&Toughness>,
            Option<&Commander>,
            Option<&mut RoundFlags>,
            Option<&mut RoundEnergyTracker>,
            Option<&mut DrBag>,
        ),
        Without<Ko>,
    >,
    mut turn_order: ResMut<TurnOrder>,
    mut state: ResMut<CombatState>,
    mut turn_flow: ParamSet<(MessageReader<TurnAdvanced>, MessageWriter<TurnAdvanced>)>,
    mut event_writer: MessageWriter<CombatEvent>,
    mut av_event_writer: MessageWriter<ActionValueUpdated>,
    mut enemy_turn_requests: Option<ResMut<EnemyTurnRequestQueue>>,
    _combat_rng: Option<ResMut<CombatRng>>,
) {
    // === Part 1: Process incoming TurnAdvanced messages ===
    // Collect snapshots first so we can do enemy AI after mutable status tick
    struct Snap {
        entity: Entity,
        id: UnitId,
        team: Team,
        is_stunned: bool,
        is_paralyzed: bool,
    }
    let snapshots: Vec<Snap> = query
        .iter_mut()
        .map(
            |(
                entity,
                unit,
                team,
                _,
                _,
                _,
                stunned,
                status_bag,
                _,
                _,
                _,
                _,
                _,
                _,
                _,
            )| {
                Snap {
                    entity,
                    id: unit.id,
                    team: *team,
                    is_stunned: stunned.is_some(),
                    is_paralyzed: status_bag
                        .as_ref()
                        .map(|b| b.has(&StatusEffectKind::Paralyzed))
                        .unwrap_or(false),
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

        let shock_cancelled = false;
        {
            let Ok((
                _,
                mut unit,
                _,
                _,
                _,
                _,
                stunned_opt,
                mut status_opt,
                _,
                _,
                _,
                _,
                mut round_flags_opt,
                mut round_energy_tracker_opt,
                mut dr_bag_opt,
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

            // Heated DoT: 4 HP Fire damage, bypasses stun (canon §H.1).
            // Runs unconditionally before stun-skip so Heated+Stunned units still burn.
            if let Some(ref bag) = status_opt {
                if bag.has(&StatusEffectKind::Heated) && unit.hp_current > 0 {
                    unit.hp_current = (unit.hp_current - 4).max(0);
                    emit_combat_event(
                        &mut event_writer,
                        CombatEventKind::OnDamageDealt {
                            amount: 4,
                            kind: DamageKind::Normal,
                            damage_tag: DamageTag::Fire,
                            tag_mod_pct: 100,
                            triangle_mod_pct: 100,
                        },
                        active_id,
                        active_id,
                        0,
                        CastId::ROOT,
                    );
                    if unit.hp_current <= 0 {
                        emit_combat_event(
                            &mut event_writer,
                            // No StatusBag in scope at stun-damage site; payload left empty.
                            CombatEventKind::UnitDied {
                                status_remaining: vec![],
                                heated_remaining: 0,
                            },
                            active_id,
                            active_id,
                            0,
                            CastId::ROOT,
                        );
                    }
                }
            }

            if let Some(mut s) = stunned_opt {
                if s.tick() {
                    commands.entity(snap.entity).remove::<Stunned>();
                }
                drop(status_opt);
                drop(unit);
                continue;
            }

            // Paralyzed: always skip action dispatch (canon §H.1). Bag is ticked so
            // duration decrements; OnStatusTick + OnStatusExpired fire as normal.
            if snap.is_paralyzed {
                if let Some(ref mut bag) = status_opt {
                    for inst in bag.iter() {
                        let turns_left = inst.duration_remaining.saturating_sub(1);
                        emit_combat_event(
                            &mut event_writer,
                            CombatEventKind::OnStatusTick {
                                kind: inst.kind.clone(),
                                turns_left,
                            },
                            active_id,
                            active_id,
                            0,
                            CastId::ROOT,
                        );
                    }
                    let expired = bag.tick_all();
                    for kind in expired {
                        emit_combat_event(
                            &mut event_writer,
                            CombatEventKind::OnStatusExpired { kind },
                            active_id,
                            active_id,
                            0,
                            CastId::ROOT,
                        );
                    }
                }
                emit_combat_event(
                    &mut event_writer,
                    CombatEventKind::OnActionFailed {
                        reason: "paralyzed".to_string(),
                    },
                    active_id,
                    active_id,
                    0,
                    CastId::ROOT,
                );
                drop(status_opt);
                drop(unit);
                continue;
            }

            if let Some(mut bag) = status_opt {
                // Per-status semantics (DoT, speed delta, cancel probability, ult boost)
                // are implemented in S03–S05. This is the v0 lifecycle skeleton only.
                // Emit OnStatusTick for every still-active instance before ticking.
                for inst in bag.iter() {
                    // Totality check — all 7 variants covered; no-op in v0.
                    match &inst.kind {
                        StatusEffectKind::Heated
                        | StatusEffectKind::Chilled
                        | StatusEffectKind::Paralyzed
                        | StatusEffectKind::Slowed
                        | StatusEffectKind::Blessed
                        | StatusEffectKind::Burn
                        | StatusEffectKind::Shock => {}
                    }
                    // turns_left after this tick = current - 1 (clamped to 0).
                    let turns_left = inst.duration_remaining.saturating_sub(1);
                    emit_combat_event(
                        &mut event_writer,
                        CombatEventKind::OnStatusTick {
                            kind: inst.kind.clone(),
                            turns_left,
                        },
                        active_id,
                        active_id,
                        0,
                        CastId::ROOT,
                    );
                }
                let expired = bag.tick_all();
                for kind in expired {
                    emit_combat_event(
                        &mut event_writer,
                        CombatEventKind::OnStatusExpired { kind },
                        active_id,
                        active_id,
                        0,
                        CastId::ROOT,
                    );
                }
                // Do NOT remove the bag component — it persists empty and is re-used on next apply.
            }

            // Tick DrBag: decrement durations and drop expired DR instances.
            if let Some(ref mut dr) = dr_bag_opt {
                dr.tick_all();
            }
        } // mutable borrow released

        // Enemy turns are bridged out to the preview-aware world-backed resolver.
        if snap.team == Team::Enemy && !shock_cancelled && !snap.is_stunned && !snap.is_paralyzed {
            if let Some(requests) = enemy_turn_requests.as_mut() {
                requests.0.push(snap.id);
            }
        }
    }

    // === Part 2: AV advancement (only in WaitingForTurn phase) ===
    if state.phase != CombatPhase::WaitingForTurn {
        return;
    }

    let mut units_ready: Vec<(UnitId, Entity, i32)> = Vec::new();

    for (
        entity,
        unit,
        _,
        speed_opt,
        speed_mod_opt,
        av_opt,
        stunned,
        status_bag_opt,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    ) in query.iter_mut()
    {
        if stunned.is_some() {
            continue;
        }
        let (Some(speed), Some(speed_mod), Some(mut av)) = (speed_opt, speed_mod_opt, av_opt)
        else {
            continue;
        };
        let chilled_delta = status_bag_opt
            .as_deref()
            .map(|b| crate::combat::status_effect::chilled_speed_delta(b, speed.0))
            .unwrap_or(0);
        let av_gain = (speed.0 + speed_mod.0 + chilled_delta) * AV_PER_SPEED;
        let old_av = av.0;
        av.advance(av_gain);
        av_event_writer.write(ActionValueUpdated {
            unit_entity: entity,
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
            let Ok((_, _, _, _, _, Some(mut av), _, _, _, _, _, _, _, _, _)) =
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

pub fn resolve_enemy_turn_action_system(world: &mut World) {
    let requests = {
        let Some(mut queue) = world.get_resource_mut::<EnemyTurnRequestQueue>() else {
            return;
        };
        std::mem::take(&mut queue.0)
    };

    if requests.is_empty() {
        return;
    }

    #[derive(Clone)]
    struct Snapshot {
        id: UnitId,
        team: Team,
        is_commander: bool,
        toughness_current: i32,
        toughness_max: i32,
        hp_current: i32,
        hp_max: i32,
        skills: Option<UnitSkills>,
        ult_ready: bool,
        alive: bool,
    }

    let mut snapshots = Vec::new();
    let mut query = world.query::<(
        &Unit,
        &Team,
        Option<&Toughness>,
        Option<&UnitSkills>,
        Option<&UltimateCharge>,
        Option<&Ko>,
        Option<&Commander>,
    )>();
    for (unit, team, toughness, skills, ult, ko, commander) in query.iter(world) {
        snapshots.push(Snapshot {
            id: unit.id,
            team: *team,
            is_commander: commander.is_some(),
            toughness_current: toughness.map(|value| value.current).unwrap_or(0),
            toughness_max: toughness.map(|value| value.max).unwrap_or(1),
            hp_current: unit.hp_current,
            hp_max: unit.hp_max,
            skills: skills.cloned(),
            ult_ready: ult.map(|value| value.ready()).unwrap_or(false),
            alive: ko.is_none() && unit.hp_current > 0,
        });
    }

    for attacker_id in requests {
        let Some(attacker) = snapshots.iter().find(|snapshot| {
            snapshot.id == attacker_id && snapshot.team == Team::Enemy && snapshot.alive
        }) else {
            continue;
        };

        let fallback_skills;
        let skills_ref: &UnitSkills = if let Some(skills) = attacker.skills.as_ref() {
            skills
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
            .filter(|snapshot| {
                snapshot.team == Team::Ally && !snapshot.is_commander && snapshot.alive
            })
            .map(|snapshot| enemy_ai::TargetInfo {
                id: snapshot.id,
                toughness_current: snapshot.toughness_current,
                toughness_max: snapshot.toughness_max,
                hp_current: snapshot.hp_current,
                hp_max: snapshot.hp_max,
            })
            .collect();

        let ctx = enemy_ai::EnemyTurnContext {
            attacker_id,
            attacker_skills: skills_ref,
            attacker_ult_ready: attacker.ult_ready,
            targets: &ally_targets,
        };

        if let Some(intent) = enemy_ai::pick_enemy_action_with_preview(&ctx, |skill_id, target| {
            let pending =
                try_query_skill_preview(world, skill_id, CastId::ROOT, attacker_id, target)?;
            Some(summarize_preview_damage(&pending))
        }) {
            world.write_message(intent);
        }
    }
}

/// Processes `CombatEvent::AdvanceTurn` and `DelayTurn` messages, applying the
/// corresponding AV delta via the T01 pure-logic primitives.
pub fn apply_av_ops_system(
    mut events: MessageReader<crate::combat::events::CombatEvent>,
    mut units: Query<(
        &crate::combat::unit::Unit,
        &mut ActionValue,
        Option<&mut TempoResistance>,
    )>,
) {
    use crate::combat::events::CombatEventKind;
    for event in events.read() {
        match &event.kind {
            CombatEventKind::AdvanceTurn { target, amount_pct } => {
                for (unit, mut av, _) in &mut units {
                    if unit.id == *target {
                        resistance::apply_advance(&mut av, *amount_pct);
                        break;
                    }
                }
            }
            CombatEventKind::DelayTurn { target, amount_pct } => {
                for (unit, mut av, mut res) in &mut units {
                    if unit.id == *target {
                        resistance::apply_delay(&mut av, *amount_pct, res.as_deref_mut());
                        break;
                    }
                }
            }
            _ => {}
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
