use bevy::prelude::*;

use crate::combat::damage::triangle_modifiers;
use crate::combat::energy::{Energy, EnergyGainSource, RoundEnergyTracker};
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::floating::FloatingDamage;
use crate::combat::kernel::{CombatBeatId, CombatKernelRegistry};
use crate::combat::log::{ActionLog, LogEntry};
use crate::combat::resolution::{apply_cleanse_only, apply_legacy_ops};
use crate::combat::rng::CombatRng;
use crate::combat::runtime::intent::CastId;
use crate::combat::sp::{RoundSpTracker, SpPool};
use crate::combat::state::{CombatPhase, CombatState, InFlightAction, UltEffect};
use crate::combat::status_effect::{StatusBag, StatusEffectKind};
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
    rng: &mut Option<ResMut<CombatRng>>,
    energy_q: &mut Query<(&mut Energy, Option<&mut RoundEnergyTracker>)>,
    cast_id: CastId,
    attacker_entity: Entity,
    target_entity: Entity,
    attacker_id: UnitId,
    target_id: UnitId,
) {
    let Ok([attacker, defender]) = actors.get_many_mut([attacker_entity, target_entity]) else {
        return;
    };

    let (
        _,
        attacker_team,
        attacker_unit,
        _attacker_kit,
        attacker_ult,
        _,
        _attacker_counterplay,
        attacker_ko,
        attacker_stunned,
        _attacker_commander,
        attacker_bag,
        mut attacker_streak,
        _attacker_round_flags,
        _attacker_slot,
        _attacker_dr,
    ) = attacker;
    let (
        _,
        defender_team,
        mut defender_unit,
        _,
        _,
        mut defender_tough,
        _defender_counterplay,
        defender_ko,
        _,
        defender_commander,
        mut defender_bag,
        _,
        mut defender_round_flags,
        _defender_slot,
        defender_dr,
    ) = defender;

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
        return;
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
        return;
    }
    if defender_ko.is_some() && inflight.action.revive_pct == 0 {
        log.push(LogEntry::ActionFailed {
            reason: "Target is KO".to_string(),
        });
        emit_combat_event(
            event_writer,
            CombatEventKind::OnActionFailed {
                reason: "Target is KO".to_string(),
            },
            attacker_id,
            target_id,
            inflight.follow_up_depth,
            cast_id,
        );
        set_phase(state, CombatPhase::WaitingAction);
        return;
    }
    if defender_ko.is_none() && inflight.action.revive_pct > 0 {
        log.push(LogEntry::ActionFailed {
            reason: "Target is not KO".to_string(),
        });
        emit_combat_event(
            event_writer,
            CombatEventKind::OnActionFailed {
                reason: "Target is not KO".to_string(),
            },
            attacker_id,
            target_id,
            inflight.follow_up_depth,
            cast_id,
        );
        set_phase(state, CombatPhase::WaitingAction);
        return;
    }

    let Some(mut attacker_ult) = attacker_ult else {
        return;
    };
    let mut local_streak = BasicStreak::default();
    let streak_ref: &mut BasicStreak = if let Some(ref mut s) = attacker_streak {
        &mut **s
    } else {
        &mut local_streak
    };

    let hp_before = defender_unit.hp_current;
    let low_hp_threshold = defender_unit.hp_max * 3 / 10;
    let ult_before = attacker_ult.current;

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
    let mut sp_tracker = RoundSpTracker::default();
    let defender_break_sealed = defender_round_flags
        .as_ref()
        .map(|f| f.break_sealed)
        .unwrap_or(false);
    let (outcome, core_events) = apply_legacy_ops(
        &inflight.action,
        &attacker_unit,
        &mut defender_unit,
        *defender_team,
        defender_tough.as_deref_mut(),
        &mut attacker_ult,
        sp,
        &mut sp_tracker,
        streak_ref,
        defender_commander.is_some(),
        defender_break_sealed,
        defender_bag.as_deref(),
        attacker_bag.as_deref(),
        defender_dr.as_deref(),
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
        return;
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
                if *attacker_team != *defender_team {
                    emit_combat_event(
                        event_writer,
                        CombatEventKind::OnEnemyKill,
                        attacker_id,
                        target_id,
                        inflight.follow_up_depth,
                        cast_id,
                    );
                }
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

    if outcome.broke {
        if let Some(ref mut flags) = defender_round_flags {
            flags.break_sealed = true;
        }
    }

    if hp_before > low_hp_threshold
        && defender_unit.hp_current <= low_hp_threshold
        && !defender_unit.is_ko()
    {
        emit_combat_event(
            event_writer,
            CombatEventKind::OnAllyLowHp,
            target_id,
            target_id,
            inflight.follow_up_depth,
            cast_id,
        );
    }

    if outcome.succeeded {
        if let Some((kind, duration)) = inflight.action.status_to_apply.clone() {
            if !outcome.ko {
                let tri = triangle_modifiers(attacker_unit.attribute, defender_unit.attribute);
                let threshold = (tri.status_acc_modifier * 100.0) as i32;
                let passes = match rng {
                    Some(r) => r.roll_pct(threshold),
                    None => CombatRng::from_seed(42).roll_pct(threshold),
                };
                if passes {
                    // Check first-apply before bag.apply() mutates it.
                    let is_first_apply_slowed = matches!(kind, StatusEffectKind::Slowed)
                        && defender_bag
                            .as_deref()
                            .map_or(true, |b| !b.has(&StatusEffectKind::Slowed));
                    if let Some(ref mut bag) = defender_bag {
                        bag.apply(kind.clone(), duration);
                    } else {
                        // Fallback: bag not yet in world — insert fresh bag with the status.
                        // This should not occur post-bootstrap seeding, but guards against
                        // units spawned without StatusBag (e.g. test fixtures).
                        let mut fresh = StatusBag::default();
                        fresh.apply(kind.clone(), duration);
                        commands.entity(target_entity).insert(fresh);
                    }
                    emit_combat_event(
                        event_writer,
                        CombatEventKind::OnStatusApplied { kind },
                        attacker_id,
                        target_id,
                        inflight.follow_up_depth,
                        cast_id,
                    );
                    if is_first_apply_slowed {
                        emit_combat_event(
                            event_writer,
                            CombatEventKind::DelayTurn {
                                target: target_id,
                                amount_pct: 30,
                            },
                            attacker_id,
                            target_id,
                            inflight.follow_up_depth,
                            cast_id,
                        );
                    }
                } else {
                    emit_combat_event(
                        event_writer,
                        CombatEventKind::OnStatusResisted { kind },
                        attacker_id,
                        target_id,
                        inflight.follow_up_depth,
                        cast_id,
                    );
                }
            }
        }
        if inflight.action.cleanse_count.is_some() && !outcome.ko {
            let cleanse_events = if let Some(ref mut bag) = defender_bag {
                let (_co, evs) = apply_cleanse_only(&inflight.action, &mut **bag, true);
                evs
            } else {
                vec![CombatEventKind::OnCleansed { kinds: vec![] }]
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
        dispatch_blueprint_transitions(inflight, log, event_writer, registry, cast_id);
    }

    set_phase(state, CombatPhase::WaitingAction);
}
