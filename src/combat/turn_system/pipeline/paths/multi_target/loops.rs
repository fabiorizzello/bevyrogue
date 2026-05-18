use bevy::prelude::*;

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::floating::FloatingDamage;
use crate::combat::kernel::{CombatBeatId, CombatKernelRegistry};
use crate::combat::log::{ActionLog, LogEntry};
use crate::combat::resolution::{apply_cleanse_only, apply_damage_only, apply_heal_only};
use crate::combat::runtime::intent::CastId;
use crate::combat::state::InFlightAction;
use crate::combat::stun::Stunned;
use crate::combat::types::UnitId;
use crate::combat::unit::Ko;
use crate::data::skills_ron::TargetShape;

use super::super::super::super::{ResolveActorsQuery, emit_combat_beat, emit_combat_event};

#[allow(clippy::too_many_arguments)]
pub(super) fn run_target_loop(
    commands: &mut Commands,
    inflight: &InFlightAction,
    log: &mut ResMut<ActionLog>,
    time: &Res<Time>,
    event_writer: &mut MessageWriter<CombatEvent>,
    registry: Option<&CombatKernelRegistry>,
    actors: &mut ResolveActorsQuery,
    cast_id: CastId,
    attacker_entity: Entity,
    attacker_id: UnitId,
    target_ids: &[UnitId],
    actor_pairs: &[(Entity, UnitId)],
) {
        for &def_id in target_ids {
            let Some(def_entity) = actor_pairs
                .iter()
                .find_map(|(e, id)| if *id == def_id { Some(*e) } else { None })
            else {
                continue;
            };

            // AllAllies fan-out: heal or cleanse — only the defender unit/bag is needed.
            if inflight.action.target_shape == TargetShape::AllAllies {
                let Ok((_, _, mut def_unit, _, _, _, _, def_ko, _, _, mut def_bag, _, _, _, _)) =
                    actors.get_mut(def_entity)
                else {
                    continue;
                };
                let dispatch_events: Vec<CombatEventKind> = if inflight.action.heal_pct > 0 {
                    let (_outcome, evs) = apply_heal_only(&inflight.action, &mut def_unit);
                    evs
                } else if inflight.action.cleanse_count.is_some() {
                    let defender_alive = def_ko.is_none() && def_unit.hp_current > 0;
                    if let Some(ref mut bag) = def_bag {
                        let (_outcome, evs) =
                            apply_cleanse_only(&inflight.action, &mut **bag, defender_alive);
                        evs
                    } else if defender_alive {
                        vec![CombatEventKind::OnCleansed { kinds: vec![] }]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                for kind in dispatch_events {
                    emit_combat_event(
                        event_writer,
                        kind,
                        inflight.action.source,
                        def_id,
                        inflight.follow_up_depth,
                        cast_id,
                    );
                }
                continue;
            }

            if def_entity == attacker_entity {
                continue;
            }

            let Ok([att_row, mut def_row]) = actors.get_many_mut([attacker_entity, def_entity])
            else {
                continue;
            };

            let (_, att_team_val, att_unit_val, _, _, _, _, _, _, _, att_bag_val, _, _, _, _) =
                &att_row;
            let (
                _,
                def_team_val,
                ref mut def_unit_val,
                _,
                _,
                ref mut def_tough_val,
                _,
                _,
                _,
                def_cmdr_val,
                ref mut def_bag_val,
                _,
                ref mut def_flags_val,
                _,
                ref mut def_dr_val,
            ) = def_row;

            let hp_before = def_unit_val.hp_current;
            let low_hp_threshold = def_unit_val.hp_max * 3 / 10;
            let defender_break_sealed = def_flags_val
                .as_ref()
                .map(|f| f.break_sealed)
                .unwrap_or(false);

            let (outcome, core_events) = apply_damage_only(
                &inflight.action,
                att_unit_val,
                def_unit_val,
                *def_team_val,
                def_tough_val.as_deref_mut(),
                def_cmdr_val.is_some(),
                defender_break_sealed,
                def_bag_val.as_deref(),
                att_bag_val.as_deref(),
                def_dr_val.as_deref(),
            );

            for kind in core_events {
                let hit_taken_amount = if let CombatEventKind::OnDamageDealt { amount, .. } = &kind
                {
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
                            target: def_id,
                            amount: *amount,
                            kind: *dkind,
                        });
                        commands.spawn(FloatingDamage {
                            target: def_id,
                            amount: *amount,
                            kind: *dkind,
                            spawn_time: time.elapsed_secs(),
                        });
                    }
                    CombatEventKind::OnBreak { damage_tag } => {
                        commands
                            .entity(def_entity)
                            .insert(Stunned { turns_left: 1 });
                        log.push(LogEntry::Break {
                            target: def_id,
                            damage_tag: *damage_tag,
                        });
                    }
                    CombatEventKind::UnitDied { .. } => {
                        commands.entity(def_entity).insert(Ko);
                        log.push(LogEntry::Ko { target: def_id });
                        if **att_team_val != *def_team_val {
                            emit_combat_event(
                                event_writer,
                                CombatEventKind::OnEnemyKill,
                                attacker_id,
                                def_id,
                                inflight.follow_up_depth,
                                cast_id,
                            );
                        }
                    }
                    _ => {}
                }
                emit_combat_event(
                    event_writer,
                    kind,
                    inflight.action.source,
                    def_id,
                    inflight.follow_up_depth,
                    cast_id,
                );

                if let Some(amount) = hit_taken_amount {
                    emit_combat_event(
                        event_writer,
                        CombatEventKind::OnHitTaken { amount },
                        attacker_id,
                        def_id,
                        inflight.follow_up_depth,
                        cast_id,
                    );
                    emit_combat_beat(
                        event_writer,
                        registry,
                        CombatBeatId::Damage,
                        attacker_id,
                        def_id,
                        inflight.follow_up_depth,
                        cast_id,
                    );
                }
            }

            if outcome.broke {
                if let Some(flags) = def_flags_val {
                    flags.break_sealed = true;
                }
            }

            if hp_before > low_hp_threshold
                && def_unit_val.hp_current <= low_hp_threshold
                && !def_unit_val.is_ko()
            {
                emit_combat_event(
                    event_writer,
                    CombatEventKind::OnAllyLowHp,
                    def_id,
                    def_id,
                    inflight.follow_up_depth,
                    cast_id,
                );
            }
        }
}
