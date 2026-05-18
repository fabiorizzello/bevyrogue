use bevy::prelude::*;

use crate::combat::energy::{Energy, EnergyGainSource, RoundEnergyTracker};
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::floating::FloatingDamage;
use crate::combat::kernel::{CombatBeatId, CombatKernelRegistry};
use crate::combat::log::{ActionLog, LogEntry};
use crate::combat::resolution::{apply_cleanse_only, apply_legacy_ops};
use crate::combat::runtime::intent::CastId;
use crate::combat::sp::{RoundSpTracker, SpPool};
use crate::combat::state::{CombatPhase, CombatState, InFlightAction, UltEffect};
use crate::combat::stun::Stunned;
use crate::combat::turn_order::TurnOrder;
use crate::combat::types::UnitId;
use crate::combat::unit::{BasicStreak, Ko};

use super::super::super::{ResolveActorsQuery, emit_combat_beat, emit_combat_event, set_phase};
use super::dispatch_blueprint_transitions;

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
    energy_q: &mut Query<(&mut Energy, Option<&mut RoundEnergyTracker>)>,
    cast_id: CastId,
    attacker_entity: Entity,
    target_entity: Entity,
    attacker_id: UnitId,
    target_id: UnitId,
) -> bool {
    if !(attacker_entity == target_entity
        && inflight.action.base_damage == 0
        && inflight.action.toughness_damage == 0
        && inflight.action.revive_pct == 0)
    {
        return false;
    }
    let Ok((
        _,
        attacker_team,
        attacker_unit,
        _attacker_kit,
        attacker_ult,
        mut defender_tough,
        _attacker_counterplay,
        attacker_ko,
        attacker_stunned,
        attacker_commander,
        mut attacker_bag,
        mut attacker_streak,
        attacker_round_flags,
        _attacker_slot,
        _attacker_dr,
    )) = actors.get_mut(attacker_entity)
    else {
        return true;
    };

    if attacker_stunned.is_some() {
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
    if attacker_ko.is_some() {
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
    if attacker_commander.is_some() {
        log.push(LogEntry::ActionFailed {
            reason: "Target is a Commander".to_string(),
        });
        emit_combat_event(
            event_writer,
            CombatEventKind::OnActionFailed {
                reason: "Target is a Commander".to_string(),
            },
            attacker_id,
            target_id,
            inflight.follow_up_depth,
            cast_id,
        );
        set_phase(state, CombatPhase::WaitingAction);
        return true;
    }

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
    let Some(mut attacker_ult) = attacker_ult else {
        return true;
    };
    let mut local_streak = BasicStreak::default();
    let streak_ref: &mut BasicStreak = if let Some(ref mut s) = attacker_streak {
        &mut **s
    } else {
        &mut local_streak
    };
    let mut defender_unit = attacker_unit.clone();
    let defender_team = *attacker_team;
    let defender_break_sealed = attacker_round_flags
        .as_ref()
        .map(|flags| flags.break_sealed)
        .unwrap_or(false);
    let ult_before = attacker_ult.current;
    let mut sp_tracker = RoundSpTracker::default();
    let (outcome, core_events) = apply_legacy_ops(
        &inflight.action,
        &attacker_unit,
        &mut defender_unit,
        defender_team,
        defender_tough.as_deref_mut(),
        &mut attacker_ult,
        sp,
        &mut sp_tracker,
        streak_ref,
        attacker_commander.is_some(),
        defender_break_sealed,
        None,
        attacker_bag.as_deref(),
        None, // self-target: no defender DrBag
    );

    if !outcome.sp_ok {
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

    for kind in core_events {
        let hit_taken_amount = if let CombatEventKind::OnDamageDealt { amount, .. } = &kind {
            Some(*amount)
        } else {
            None
        };

        match &kind {
            CombatEventKind::OnDamageDealt {
                amount,
                kind: dkind,
                ..
            } => {
                log.push(LogEntry::BasicHit {
                    attacker: attacker_id,
                    target: target_id,
                    amount: *amount,
                    kind: *dkind,
                });
                commands.spawn(FloatingDamage {
                    target: target_id,
                    amount: *amount,
                    kind: *dkind,
                    spawn_time: time.elapsed_secs(),
                });
            }
            CombatEventKind::OnBreak { damage_tag } => {
                commands
                    .entity(target_entity)
                    .insert(Stunned { turns_left: 1 });
                log.push(LogEntry::Break {
                    target: target_id,
                    damage_tag: *damage_tag,
                });
            }
            CombatEventKind::UnitDied { .. } => {
                commands.entity(target_entity).insert(Ko);
                log.push(LogEntry::Ko { target: target_id });
            }
            CombatEventKind::OnRevive { hp_after } => {
                commands.entity(target_entity).remove::<Ko>();
                log.push(LogEntry::Revive {
                    target: target_id,
                    hp_after: *hp_after,
                });
            }
            CombatEventKind::OnActionFailed { reason } => {
                log.push(LogEntry::ActionFailed {
                    reason: reason.clone(),
                });
            }
            CombatEventKind::AdvanceTurn {
                target: t_id,
                amount_pct,
            } => {
                log.push(LogEntry::AdvanceTurn {
                    target: *t_id,
                    amount_pct: *amount_pct,
                });
            }
            CombatEventKind::DelayTurn {
                target: t_id,
                amount_pct,
            } => {
                log.push(LogEntry::DelayTurn {
                    target: *t_id,
                    amount_pct: *amount_pct,
                });
            }
            _ => {}
        }
        emit_combat_event(
            event_writer,
            kind,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
            cast_id,
        );

        if let Some(amount) = hit_taken_amount {
            emit_combat_event(
                event_writer,
                CombatEventKind::OnHitTaken { amount },
                attacker_id,
                target_id,
                inflight.follow_up_depth,
                cast_id,
            );
            emit_combat_beat(
                event_writer,
                registry,
                CombatBeatId::Damage,
                attacker_id,
                target_id,
                inflight.follow_up_depth,
                cast_id,
            );
        }
    }

    if matches!(inflight.action.ult_effect, UltEffect::GainFromBasic) {
        let delta = attacker_ult.current - ult_before;
        if delta > 0 {
            emit_combat_event(
                event_writer,
                CombatEventKind::UltGain {
                    unit_id: attacker_id,
                    amount: delta,
                },
                attacker_id,
                attacker_id,
                inflight.follow_up_depth,
                cast_id,
            );
        }
    }
    if matches!(inflight.action.ult_effect, UltEffect::Reset) {
        emit_combat_event(
            event_writer,
            CombatEventKind::UltimateUsed {
                unit_id: attacker_id,
            },
            attacker_id,
            attacker_id,
            inflight.follow_up_depth,
            cast_id,
        );
    }

    if outcome.succeeded && inflight.action.energy_grant > 0 {
        if let Ok((mut energy, mut tracker)) = energy_q.get_mut(attacker_entity) {
            let granted_by_round_cap = tracker
                .as_deref_mut()
                .map(|tracker| {
                    tracker.try_gain(
                        EnergyGainSource::SecondaryAction,
                        inflight.action.energy_grant,
                    )
                })
                .unwrap_or(inflight.action.energy_grant);
            let applied = energy.gain_capped(granted_by_round_cap);
            if applied > 0 {
                emit_combat_event(
                    event_writer,
                    CombatEventKind::EnergyGained {
                        unit_id: attacker_id,
                        amount: applied,
                    },
                    attacker_id,
                    attacker_id,
                    inflight.follow_up_depth,
                    cast_id,
                );
            }
        }
    }

    if outcome.succeeded {
        if inflight.action.cleanse_count.is_some() {
            let defender_alive = !outcome.ko;
            let cleanse_events = if let Some(ref mut bag) = attacker_bag {
                let (_co, evs) = apply_cleanse_only(&inflight.action, &mut **bag, defender_alive);
                evs
            } else if defender_alive {
                vec![CombatEventKind::OnCleansed { kinds: vec![] }]
            } else {
                vec![]
            };
            for kind in cleanse_events {
                emit_combat_event(
                    event_writer,
                    kind,
                    inflight.action.source,
                    target_id,
                    inflight.follow_up_depth,
                    cast_id,
                );
            }
        }
        dispatch_blueprint_transitions(inflight, log, event_writer, registry, cast_id);
    }

    set_phase(state, CombatPhase::WaitingAction);
    return true;
}
