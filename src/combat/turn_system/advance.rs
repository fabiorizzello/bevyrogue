use super::av::{AV_PER_SPEED, ActionValue, ActionValueUpdated, MAX_AV};
use super::*;
use crate::combat::buffs::DrBag;
use crate::combat::runtime::intent::CastId;
use crate::combat::{
    StatusBag,
    energy::RoundEnergyTracker,
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    round_flags::RoundFlags,
    speed::Speed,
    speed::SpeedModifier,
    state::{CombatPhase, CombatState},
    status_effect::StatusEffectKind,
    stun::Stunned,
    team::Team,
    toughness::{DamageKind, Toughness},
    turn_order::{TurnAdvanced, TurnOrder},
    types::{DamageTag, UnitId},
    ultimate::UltimateCharge,
    unit::{Commander, Ko, Unit},
};
use bevy::prelude::*;

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
            |(entity, unit, team, _, _, _, stunned, status_bag, _, _, _, _, _, _, _)| Snap {
                entity,
                id: unit.id,
                team: *team,
                is_stunned: stunned.is_some(),
                is_paralyzed: status_bag
                    .as_ref()
                    .map(|b| b.has(&StatusEffectKind::Paralyzed))
                    .unwrap_or(false),
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
