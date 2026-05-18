use bevy::prelude::*;
use std::collections::HashSet;

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::floating::FloatingDamage;
use crate::combat::kernel::{CombatBeatId, CombatKernelRegistry};
use crate::combat::log::{ActionLog, LogEntry};
use crate::combat::resolution::{apply_damage_only, compute_hop_damage, select_bounce_hop, TargetEntry, TargetableSnapshot};
use crate::combat::runtime::intent::CastId;
use crate::combat::state::InFlightAction;
use crate::combat::stun::Stunned;
use crate::combat::types::UnitId;
use crate::combat::unit::{Ko, SlotIndex};
use crate::data::skills_ron::{DamageCurve, RepeatPolicy};

use super::super::super::super::{ResolveActorsQuery, emit_combat_beat, emit_combat_event};

#[allow(clippy::too_many_arguments)]
pub(super) fn run_hop_loop(
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
    target_id: UnitId,
    hops: u8,
    selector: crate::data::skills_ron::BounceSelector,
    repeat: RepeatPolicy,
    actor_pairs: &[(Entity, UnitId)],
) {
        // Phase 2: per-hop damage loop.
        let mut already_hit: HashSet<UnitId> = HashSet::new();
        let mut last_slot: Option<u8> = None;
        let curve = &inflight.action.damage_curve;
        let base_damage = inflight.action.base_damage;

        // Determine the enemy team from the primary target.
        let enemy_team = actors
            .iter()
            .find_map(|(_, team, unit, ..)| {
                if unit.id == target_id {
                    Some(*team)
                } else {
                    None
                }
            })
            .unwrap_or(crate::combat::team::Team::Enemy);

        // Pre-loop guard: PerHop curve shorter than hops_planned.
        // Emits OnActionFailed once and clamps the loop to v.len() so the action
        // still resolves the hops it has coefficients for (D001: kernel never panics).
        let clamped_hops = if let DamageCurve::PerHop(v) = curve {
            let n = v.len();
            let h = hops as usize;
            if n < h {
                let reason = format!("DamageCurve::PerHop length {n} < hops_planned {h}");
                emit_combat_event(
                    event_writer,
                    CombatEventKind::OnActionFailed { reason },
                    attacker_id,
                    target_id,
                    inflight.follow_up_depth,
                    cast_id,
                );
                n
            } else {
                h
            }
        } else {
            hops as usize
        };

        for hop_k in 0..clamped_hops {
            // Rebuild snapshot each hop so that KOs from previous hops are reflected.
            let snapshot = {
                let entries = actors
                    .iter()
                    .map(|(_, team, unit, _, _, _, _, ko, _, _, _, _, _, slot, _)| {
                        let alive = ko.is_none() && unit.hp_current > 0;
                        let hp_per_mille = if unit.hp_max > 0 {
                            ((unit.hp_current.max(0) as u64 * 1000) / unit.hp_max as u64) as u32
                        } else {
                            0
                        };
                        TargetEntry {
                            id: unit.id,
                            team: *team,
                            slot_index: slot.map(|s: &SlotIndex| s.0).unwrap_or(0),
                            alive,
                            hp_per_mille,
                        }
                    })
                    .collect();
                TargetableSnapshot { entries }
            };

            // Select next hop target.
            let Some(def_id) = select_bounce_hop(
                selector,
                &snapshot,
                &already_hit,
                repeat,
                enemy_team,
                last_slot,
            ) else {
                break; // pool exhausted → truncate silently
            };

            // Update tracking state.
            if let Some(entry) = snapshot.entries.iter().find(|e| e.id == def_id) {
                last_slot = Some(entry.slot_index);
            }
            if repeat == RepeatPolicy::NoRepeat {
                already_hit.insert(def_id);
            }

            // Find the defender entity.
            let Some(def_entity) = actor_pairs
                .iter()
                .find_map(|(e, id)| if *id == def_id { Some(*e) } else { None })
            else {
                continue;
            };

            if def_entity == attacker_entity {
                continue;
            }

            // Build per-hop action with scaled damage.
            let hop_damage = compute_hop_damage(base_damage, curve, hop_k);
            let hop_action = crate::combat::state::ResolvedAction {
                base_damage: hop_damage,
                ..inflight.action.clone()
            };

            let Ok([att_row, mut def_row]) = actors.get_many_mut([attacker_entity, def_entity])
            else {
                continue;
            };

            let (_, _att_team_val, att_unit_val, _, _, _, _, _, _, _, att_bag_val, _, _, _, _) =
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
                ref mut _def_bag_val,
                _,
                ref mut def_flags_val,
                _,
                ref mut _def_dr_val2,
            ) = def_row;

            let hp_before = def_unit_val.hp_current;
            let low_hp_threshold = def_unit_val.hp_max * 3 / 10;
            let defender_break_sealed = def_flags_val
                .as_ref()
                .map(|f| f.break_sealed)
                .unwrap_or(false);

            let (outcome, core_events) = apply_damage_only(
                &hop_action,
                att_unit_val,
                def_unit_val,
                *def_team_val,
                def_tough_val.as_deref_mut(),
                def_cmdr_val.is_some(),
                defender_break_sealed,
                _def_bag_val.as_deref(),
                att_bag_val.as_deref(),
                _def_dr_val2.as_deref(),
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
                        // Emit OnEnemyKill (attacker vs enemy team).
                        emit_combat_event(
                            event_writer,
                            CombatEventKind::OnEnemyKill,
                            attacker_id,
                            def_id,
                            inflight.follow_up_depth,
                            cast_id,
                        );
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
        } // end hop loop
}
