//! M010 action pipeline (WIP). Multi-phase action lifecycle:
//! Declaration → PreApp → App → Resolution.
//!
//! See `.gsd/M010-HANDOFF.md` for integration status. The functions here
//! are the scaffolding; wire-up into the Bevy schedule is incomplete.

use bevy::prelude::*;

use crate::combat::api::applier::{IntentExecutionMeta, IntentQueue};
use crate::combat::api::intent::CastId;
use crate::combat::api::runner::{BeatRunner, StepOutcome};

use crate::combat::damage::triangle_modifiers;
use crate::combat::energy::{Energy, EnergyGainSource, RoundEnergyTracker};
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::floating::FloatingDamage;
use crate::combat::kernel::{CombatBeatId, CombatKernelRegistry};
use crate::combat::log::{ActionLog, LogEntry};
use crate::combat::resolution::{TargetEntry, TargetableSnapshot};
use crate::combat::resolution::{
    apply_cleanse_only, apply_damage_only, apply_heal_only, apply_legacy_ops, compute_hop_damage,
    resolve_action, resolve_targets, select_bounce_hop, target_shape_rejection_reason,
};
use crate::combat::rng::CombatRng;
use crate::combat::sp::{RoundSpTracker, SpPool};
use crate::combat::state::{CombatPhase, CombatState, InFlightAction, UltEffect};
use crate::combat::status_effect::{StatusBag, StatusEffectKind};
use crate::combat::stun::Stunned;
use crate::combat::turn_order::TurnOrder;
use crate::combat::types::{EvoStage, UnitId};
use crate::combat::unit::{BasicStreak, Ko, SlotIndex};
use crate::data::{
    SkillBookHandle,
    skills_ron::{DamageCurve, RepeatPolicy, SkillBook, TargetShape},
};
use std::collections::HashSet;

use super::{
    ActionIntent, ResolveActorsQuery, emit_combat_beat, emit_combat_event, emit_kernel_transition,
    set_phase,
};

pub(crate) fn step_declaration(
    _commands: &mut Commands,
    intent: &ActionIntent,
    follow_up_depth: u8,
    _state: &mut ResMut<CombatState>,
    follow_up_origin_kind: super::super::follow_up::FollowUpOriginKind,
    skill_books: &Res<Assets<SkillBook>>,
    skill_book_handle: Option<&Res<SkillBookHandle>>,
    log: &mut ResMut<ActionLog>,
    event_writer: &mut MessageWriter<CombatEvent>,
    actors: &mut ResolveActorsQuery,
) -> Option<InFlightAction> {
    let (attacker_id, _target_id) = match intent {
        ActionIntent::Basic { attacker, target }
        | ActionIntent::Skill {
            attacker, target, ..
        }
        | ActionIntent::Ultimate { attacker, target } => (*attacker, *target),
    };

    let (_entity, kit) =
        actors
            .iter()
            .find_map(|(entity, _, unit, kit, _, _, _, _, _, _, _, _, _, _, _)| {
                if unit.id == attacker_id {
                    Some((entity, kit))
                } else {
                    None
                }
            })?;

    let Some(kit) = kit else {
        return None;
    };
    let skill_book = skill_book_handle.and_then(|h| skill_books.get(&h.0));
    let mut action = resolve_action(intent, kit, skill_book)?;

    if follow_up_origin_kind == super::super::follow_up::FollowUpOriginKind::FormIdentity
        && action.target_shape == TargetShape::SelfOnly
        && action.base_damage == 0
        && action.toughness_damage == 0
        && action.revive_pct == 0
    {
        action.target = action.source;
    } else if follow_up_origin_kind != super::super::follow_up::FollowUpOriginKind::FormIdentity
        && let Some(reason) = target_shape_rejection_reason(action.target_shape)
    {
        log.push(LogEntry::ActionFailed {
            reason: reason.clone(),
        });
        emit_combat_event(
            event_writer,
            CombatEventKind::OnActionFailed { reason },
            action.source,
            action.target,
            follow_up_depth,
            CastId::ROOT,
        );
        return None;
    }

    let inflight = InFlightAction {
        action,
        interrupted: false,
        follow_up_depth,
    };

    Some(inflight)
}

fn dispatch_blueprint_transitions(
    inflight: &InFlightAction,
    log: &mut ResMut<ActionLog>,
    event_writer: &mut MessageWriter<CombatEvent>,
    registry: Option<&CombatKernelRegistry>,
    cast_id: CastId,
) {
    match crate::combat::blueprints::transitions_for_action_checked(&inflight.action) {
        Ok(transitions) => {
            for transition in transitions {
                emit_kernel_transition(
                    event_writer,
                    registry,
                    transition,
                    inflight.action.source,
                    inflight.action.target,
                    inflight.follow_up_depth,
                    cast_id,
                );
            }
        }
        Err(error) => {
            let reason = error.to_string();
            log.push(LogEntry::ActionFailed {
                reason: reason.clone(),
            });
            emit_combat_event(
                event_writer,
                CombatEventKind::OnActionFailed { reason },
                inflight.action.source,
                inflight.action.target,
                inflight.follow_up_depth,
                cast_id,
            );
        }
    }
}

#[allow(dead_code)]
fn intern_timeline_id(value: &str) -> &'static str {
    Box::leak(value.to_owned().into_boxed_str())
}

#[allow(dead_code)]
fn intern_compiled_timeline(
    timeline: &crate::combat::api::timeline::CompiledTimeline<String>,
) -> crate::combat::api::timeline::CompiledTimeline<&'static str> {
    use crate::combat::api::timeline::{
        Beat, BeatEdge, BeatKind, BeatPayload, CompiledTimeline, Presentation,
    };

    fn intern_payload(payload: &BeatPayload) -> BeatPayload {
        match payload {
            BeatPayload::DealDamage {
                amount,
                tag,
                target,
            } => BeatPayload::DealDamage {
                amount: *amount,
                tag: *tag,
                target: target.clone(),
            },
            BeatPayload::BreakToughness {
                amount,
                tag,
                target,
            } => BeatPayload::BreakToughness {
                amount: *amount,
                tag: *tag,
                target: target.clone(),
            },
            BeatPayload::ApplyStatus {
                kind,
                duration,
                target,
            } => BeatPayload::ApplyStatus {
                kind: kind.clone(),
                duration: *duration,
                target: target.clone(),
            },
            BeatPayload::DelayTurn { amount_pct, target } => BeatPayload::DelayTurn {
                amount_pct: *amount_pct,
                target: target.clone(),
            },
            BeatPayload::AdvanceTurn { amount_pct, target } => BeatPayload::AdvanceTurn {
                amount_pct: *amount_pct,
                target: target.clone(),
            },
            BeatPayload::ApplyBuff {
                kind,
                duration,
                target,
            } => BeatPayload::ApplyBuff {
                kind: kind.clone(),
                duration: *duration,
                target: target.clone(),
            },
            BeatPayload::Revive { pct, target } => BeatPayload::Revive {
                pct: *pct,
                target: target.clone(),
            },
            BeatPayload::GrantFreeSkill { count } => BeatPayload::GrantFreeSkill { count: *count },
            BeatPayload::GrantEnergy { amount } => BeatPayload::GrantEnergy { amount: *amount },
            BeatPayload::SelfAdvance { amount_pct } => BeatPayload::SelfAdvance {
                amount_pct: *amount_pct,
            },
            BeatPayload::BlueprintSignal {
                owner,
                name,
                payload,
            } => BeatPayload::BlueprintSignal {
                owner: owner.clone(),
                name: name.clone(),
                payload: payload.clone(),
            },
        }
    }

    fn intern_presentation(p: &Presentation<String>) -> Presentation<&'static str> {
        Presentation {
            cue_id: intern_timeline_id(&p.cue_id),
            anim: p.anim.as_deref().map(intern_timeline_id),
            vfx: p.vfx.as_deref().map(intern_timeline_id),
            sfx: p.sfx.as_deref().map(intern_timeline_id),
        }
    }

    fn intern_beat(beat: &Beat<String>) -> Beat<&'static str> {
        Beat {
            id: intern_timeline_id(&beat.id),
            kind: match &beat.kind {
                BeatKind::Cast => BeatKind::Cast,
                BeatKind::Phase => BeatKind::Phase,
                BeatKind::Impact => BeatKind::Impact,
                BeatKind::Aftermath => BeatKind::Aftermath,
                BeatKind::Loop { body, exit_when } => BeatKind::Loop {
                    body: body.iter().map(intern_beat).collect(),
                    exit_when: intern_timeline_id(exit_when),
                },
            },
            hook: beat.hook.as_deref().map(intern_timeline_id),
            selector: beat.selector.as_deref().map(intern_timeline_id),
            presentation: beat.presentation.as_ref().map(intern_presentation),
            payload: beat.payload.as_ref().map(intern_payload),
        }
    }

    CompiledTimeline {
        id: intern_timeline_id(&timeline.id),
        entry: intern_timeline_id(&timeline.entry),
        beats: timeline.beats.iter().map(intern_beat).collect(),
        edges: timeline
            .edges
            .iter()
            .map(|edge| BeatEdge {
                from: intern_timeline_id(&edge.from),
                to: intern_timeline_id(&edge.to),
                gate: edge.gate.as_deref().map(intern_timeline_id),
            })
            .collect(),
    }
}

pub(crate) fn run_timeline_backed_action(
    world: &mut World,
    inflight: InFlightAction,
    cast_id: CastId,
) {
    let mut _fallback_regs = None;
    let regs_ptr: *const crate::combat::api::registry::ExtRegistries =
        if let Some(regs) = world.get_resource::<crate::combat::api::registry::ExtRegistries>() {
            regs as *const _
        } else {
            let mut regs = crate::combat::api::registry::ExtRegistries::default();
            crate::combat::api::builtins::register_kernel_builtins(&mut regs);
            _fallback_regs = Some(regs);
            _fallback_regs
                .as_ref()
                .expect("fallback ext registries initialized") as *const _
        };

    let Some(timeline) = crate::combat::preview::resolve_compiled_skill_timeline(
        world,
        &inflight.action.skill_id,
        unsafe { &*regs_ptr },
    ) else {
        return;
    };

    let attacker_id = inflight.action.source;

    if inflight.action.sp_cost > 0 {
        let current_sp = world
            .get_resource::<SpPool>()
            .map(|sp| sp.current)
            .unwrap_or(0);
        if current_sp < inflight.action.sp_cost {
            let mut events = world.resource_mut::<bevy::ecs::message::Messages<CombatEvent>>();
            events.write(CombatEvent {
                kind: CombatEventKind::OnActionFailed {
                    reason: "SP shortfall".to_string(),
                },
                source: inflight.action.source,
                target: inflight.action.target,
                follow_up_depth: inflight.follow_up_depth,
                cast_id,
            });
            events.write(CombatEvent {
                kind: CombatEventKind::OnActionApplied,
                source: inflight.action.source,
                target: inflight.action.target,
                follow_up_depth: inflight.follow_up_depth,
                cast_id,
            });
            events.write(CombatEvent {
                kind: CombatEventKind::OnActionResolved,
                source: inflight.action.source,
                target: inflight.action.target,
                follow_up_depth: inflight.follow_up_depth,
                cast_id,
            });
            world
                .resource_mut::<crate::combat::state::CombatState>()
                .phase = CombatPhase::WaitingAction;
            return;
        }
    }

    if matches!(inflight.action.ult_effect, UltEffect::Reset) {
        let mut q = world.query::<(
            &crate::combat::unit::Unit,
            &crate::combat::ultimate::UltimateCharge,
        )>();
        let ult_ready = q
            .iter(world)
            .find_map(|(unit, ult)| (unit.id == attacker_id).then_some(ult.ready()))
            .unwrap_or(false);
        if !ult_ready {
            let mut events = world.resource_mut::<bevy::ecs::message::Messages<CombatEvent>>();
            events.write(CombatEvent {
                kind: CombatEventKind::OnActionFailed {
                    reason: "SP shortfall".to_string(),
                },
                source: inflight.action.source,
                target: inflight.action.target,
                follow_up_depth: inflight.follow_up_depth,
                cast_id,
            });
            events.write(CombatEvent {
                kind: CombatEventKind::OnActionApplied,
                source: inflight.action.source,
                target: inflight.action.target,
                follow_up_depth: inflight.follow_up_depth,
                cast_id,
            });
            events.write(CombatEvent {
                kind: CombatEventKind::OnActionResolved,
                source: inflight.action.source,
                target: inflight.action.target,
                follow_up_depth: inflight.follow_up_depth,
                cast_id,
            });
            world
                .resource_mut::<crate::combat::state::CombatState>()
                .phase = CombatPhase::WaitingAction;
            return;
        }
    }

    let mut pending = std::collections::VecDeque::new();
    let mut runner = BeatRunner::new(
        timeline,
        cast_id,
        inflight.action.source,
        inflight.action.target,
    );
    let outcome = unsafe {
        runner.run_to_completion(
            world,
            &*regs_ptr,
            crate::combat::api::SkillCtxMode::Execute,
            &mut pending,
            1024,
        )
    };

    if outcome != StepOutcome::Done {
        let reason = format!("timeline execution halted: {outcome:?}");
        world
            .resource_mut::<bevy::ecs::message::Messages<CombatEvent>>()
            .write(CombatEvent {
                kind: CombatEventKind::OnActionFailed { reason },
                source: inflight.action.source,
                target: inflight.action.target,
                follow_up_depth: inflight.follow_up_depth,
                cast_id,
            });
        world
            .resource_mut::<crate::combat::state::CombatState>()
            .phase = CombatPhase::WaitingAction;
        return;
    }

    world.init_resource::<IntentQueue>();
    world.init_resource::<IntentExecutionMeta>();
    world.init_resource::<crate::combat::api::signal::SignalBus>();
    world.init_resource::<crate::combat::api::signal::SignalTaxonomy>();
    world.init_resource::<crate::combat::api::blueprint_state::BlueprintState>();
    let fallback_signal_pairs: Vec<(String, String)> = world
        .get_resource::<bevy::prelude::Assets<crate::data::skills_ron::SkillBook>>()
        .and_then(|assets| {
            world
                .get_resource::<crate::data::SkillBookHandle>()
                .and_then(|handle| assets.get(&handle.0))
        })
        .map(|book| {
            book.0
                .iter()
                .flat_map(|skill| {
                    skill
                        .custom_signals
                        .iter()
                        .map(|custom| (custom.owner.clone(), custom.signal.clone()))
                })
                .collect()
        })
        .unwrap_or_default();
    {
        let mut taxonomy = world.resource_mut::<crate::combat::api::signal::SignalTaxonomy>();
        taxonomy.register("kernel", "ult_used");
        for (owner, signal) in fallback_signal_pairs {
            let owner: &'static str = Box::leak(owner.into_boxed_str());
            let signal: &'static str = Box::leak(signal.into_boxed_str());
            taxonomy.register(owner, signal);
        }
    }
    world
        .resource_mut::<IntentExecutionMeta>()
        .follow_up_depths
        .insert(cast_id, inflight.follow_up_depth);
    world.resource_mut::<IntentQueue>().0.extend(pending);
    crate::combat::api::applier::intent_applier(world);
    world
        .resource_mut::<IntentExecutionMeta>()
        .follow_up_depths
        .remove(&cast_id);

    let attacker_id = inflight.action.source;
    let ult_effect = inflight.action.ult_effect;

    if matches!(ult_effect, UltEffect::Reset) || matches!(ult_effect, UltEffect::GainFromBasic) {
        let mut q = world.query::<(
            &crate::combat::unit::Unit,
            &mut crate::combat::ultimate::UltimateCharge,
        )>();
        for (unit, mut ult) in q.iter_mut(world) {
            if unit.id == attacker_id {
                match ult_effect {
                    UltEffect::Reset => {
                        ult.current = 0;
                    }
                    UltEffect::GainFromBasic => {
                        let cpe = ult.charge_per_event;
                        ult.try_add(cpe);
                    }
                    UltEffect::None => {}
                }
                break;
            }
        }
    }

    if inflight.action.sp_cost > 0 {
        let mut sp = world.resource_mut::<SpPool>();
        sp.spend(inflight.action.sp_cost);
    }

    let mut event_writer = world.resource_mut::<bevy::ecs::message::Messages<CombatEvent>>();
    event_writer.write(CombatEvent {
        kind: CombatEventKind::OnSkillCast {
            skill_id: inflight.action.skill_id.clone(),
        },
        source: inflight.action.source,
        target: inflight.action.target,
        follow_up_depth: inflight.follow_up_depth,
        cast_id,
    });
    if matches!(ult_effect, UltEffect::Reset) {
        event_writer.write(CombatEvent {
            kind: CombatEventKind::UltimateUsed {
                unit_id: attacker_id,
            },
            source: attacker_id,
            target: inflight.action.target,
            follow_up_depth: inflight.follow_up_depth,
            cast_id,
        });
    }
    event_writer.write(CombatEvent {
        kind: CombatEventKind::OnActionApplied,
        source: inflight.action.source,
        target: inflight.action.target,
        follow_up_depth: inflight.follow_up_depth,
        cast_id,
    });
    event_writer.write(CombatEvent {
        kind: CombatEventKind::OnActionResolved,
        source: inflight.action.source,
        target: inflight.action.target,
        follow_up_depth: inflight.follow_up_depth,
        cast_id,
    });

    world
        .resource_mut::<crate::combat::state::CombatState>()
        .phase = CombatPhase::WaitingAction;
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn step_app(
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
) {
    if inflight.interrupted {
        return;
    }

    let attacker_id = inflight.action.source;
    let target_id = inflight.action.target;

    let attacker_entity = actors.iter().find_map(|(entity, _, unit, ..)| {
        if unit.id == attacker_id {
            Some(entity)
        } else {
            None
        }
    });
    let target_entity = actors.iter().find_map(|(entity, _, unit, ..)| {
        if unit.id == target_id {
            Some(entity)
        } else {
            None
        }
    });
    let (Some(attacker_entity), Some(target_entity)) = (attacker_entity, target_entity) else {
        return;
    };

    // === MULTI-TARGET PATH (Blast / AllEnemies / AllAllies) ===
    if matches!(
        inflight.action.target_shape,
        TargetShape::Blast | TargetShape::AllEnemies | TargetShape::AllAllies
    ) {
        // Phase 0: build entity→id map and snapshot (read-only pass, released before mut borrows)
        let actor_pairs: Vec<(Entity, UnitId)> = actors
            .iter()
            .map(|(entity, _, unit, ..)| (entity, unit.id))
            .collect();
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

        let target_ids = resolve_targets(
            &inflight.action.target_shape,
            inflight.action.target,
            &snapshot,
        );

        if target_ids.is_empty() {
            set_phase(state, CombatPhase::WaitingAction);
            return;
        }

        // Phase 1: attacker validation + resource consumption (mut borrow released after block)
        {
            let Ok((
                _,
                _,
                attacker_unit,
                _,
                att_ult_opt,
                _,
                _,
                att_ko,
                att_stun,
                _,
                _,
                mut att_streak_opt,
                _,
                _,
                _,
            )) = actors.get_mut(attacker_entity)
            else {
                return;
            };

            if att_stun.is_some() {
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
            if att_ko.is_some() {
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

            let Some(mut att_ult) = att_ult_opt else {
                return;
            };

            // SP cost with Child streak discount (hoisted from apply_effects)
            let effective_sp_cost = if matches!(inflight.action.ult_effect, UltEffect::None)
                && inflight.action.sp_cost > 0
                && attacker_unit.evo_stage == EvoStage::Child
                && att_streak_opt
                    .as_deref()
                    .map(|s| s.qualifies_for_discount())
                    .unwrap_or(false)
            {
                if let Some(ref mut streak) = att_streak_opt {
                    streak.reset();
                }
                (inflight.action.sp_cost - 1).max(0)
            } else {
                inflight.action.sp_cost
            };

            if effective_sp_cost > 0 && !sp.spend(effective_sp_cost) {
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

            if matches!(inflight.action.ult_effect, UltEffect::Reset) && !att_ult.ready() {
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
        } // attacker mut borrow released

        // Phase 2: per-target damage loop
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

        for &def_id in &target_ids {
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

        // Phase 3: post-loop attacker resource effects + once-per-cast events
        let ult_delta = {
            let Ok((
                _,
                _,
                _,
                _,
                att_ult_opt,
                _,
                _,
                _,
                _,
                _,
                att_bag_opt,
                mut att_streak_opt,
                _,
                _,
                _,
            )) = actors.get_mut(attacker_entity)
            else {
                set_phase(state, CombatPhase::WaitingAction);
                return;
            };

            let Some(mut att_ult) = att_ult_opt else {
                set_phase(state, CombatPhase::WaitingAction);
                return;
            };

            let before = att_ult.current;

            match inflight.action.ult_effect {
                UltEffect::GainFromBasic => {
                    sp.gain(1);
                    let cpe = att_ult.charge_per_event;
                    att_ult.try_add(cpe);
                    if let Some(ref mut streak) = att_streak_opt {
                        streak.increment();
                    }
                }
                UltEffect::None => {}
                UltEffect::Reset => {
                    att_ult.current = 0;
                }
            }

            if inflight.action.ult_effect != UltEffect::Reset {
                if let Some(bag) = att_bag_opt.as_deref() {
                    if bag.has(&StatusEffectKind::Blessed) {
                        att_ult.try_add(1);
                    }
                }
            }

            att_ult.current - before
        };

        // Once-per-cast events (not per-target)
        emit_combat_event(
            event_writer,
            CombatEventKind::OnSkillCast {
                skill_id: inflight.action.skill_id.clone(),
            },
            attacker_id,
            target_id,
            inflight.follow_up_depth,
            cast_id,
        );

        if inflight.action.advance_pct != 0 {
            emit_combat_event(
                event_writer,
                CombatEventKind::AdvanceTurn {
                    target: inflight.action.target,
                    amount_pct: inflight.action.advance_pct,
                },
                attacker_id,
                target_id,
                inflight.follow_up_depth,
                cast_id,
            );
        }
        if inflight.action.delay_pct != 0 {
            emit_combat_event(
                event_writer,
                CombatEventKind::DelayTurn {
                    target: inflight.action.target,
                    amount_pct: inflight.action.delay_pct,
                },
                attacker_id,
                target_id,
                inflight.follow_up_depth,
                cast_id,
            );
        }
        if inflight.action.self_advance_pct != 0 {
            let capped = (inflight.action.self_advance_pct.max(0) as u32).min(50);
            if capped != 0 {
                emit_combat_event(
                    event_writer,
                    CombatEventKind::AdvanceTurn {
                        target: inflight.action.source,
                        amount_pct: capped,
                    },
                    attacker_id,
                    attacker_id,
                    inflight.follow_up_depth,
                    cast_id,
                );
            }
        }

        if matches!(inflight.action.ult_effect, UltEffect::GainFromBasic) && ult_delta > 0 {
            emit_combat_event(
                event_writer,
                CombatEventKind::UltGain {
                    unit_id: attacker_id,
                    amount: ult_delta,
                },
                attacker_id,
                attacker_id,
                inflight.follow_up_depth,
                cast_id,
            );
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

        if inflight.action.energy_grant > 0 {
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

        dispatch_blueprint_transitions(inflight, log, event_writer, registry, cast_id);
        set_phase(state, CombatPhase::WaitingAction);
        return;
    }
    // === END MULTI-TARGET PATH ===

    // === BOUNCE PATH ===
    if let TargetShape::Bounce {
        hops,
        selector,
        repeat,
    } = inflight.action.target_shape
    {
        // Phase 0: build entity→id map (read-only pass released before mut borrows).
        let actor_pairs: Vec<(Entity, UnitId)> = actors
            .iter()
            .map(|(entity, _, unit, ..)| (entity, unit.id))
            .collect();

        // Phase 1: attacker validation + resource consumption (hoisted, paid once pre-loop).
        {
            let Ok((
                _,
                _,
                attacker_unit,
                _,
                att_ult_opt,
                _,
                _,
                att_ko,
                att_stun,
                _,
                _,
                mut att_streak_opt,
                _,
                _,
                _,
            )) = actors.get_mut(attacker_entity)
            else {
                return;
            };

            if att_stun.is_some() {
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
            if att_ko.is_some() {
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

            let Some(mut att_ult) = att_ult_opt else {
                return;
            };

            // SP cost with Child streak discount (hoisted from apply_effects).
            let effective_sp_cost = if matches!(inflight.action.ult_effect, UltEffect::None)
                && inflight.action.sp_cost > 0
                && attacker_unit.evo_stage == EvoStage::Child
                && att_streak_opt
                    .as_deref()
                    .map(|s| s.qualifies_for_discount())
                    .unwrap_or(false)
            {
                if let Some(ref mut streak) = att_streak_opt {
                    streak.reset();
                }
                (inflight.action.sp_cost - 1).max(0)
            } else {
                inflight.action.sp_cost
            };

            if effective_sp_cost > 0 && !sp.spend(effective_sp_cost) {
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

            if matches!(inflight.action.ult_effect, UltEffect::Reset) && !att_ult.ready() {
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
        } // attacker mut borrow released

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

        // Phase 3: post-loop attacker resource effects + once-per-cast events.
        let ult_delta = {
            let Ok((
                _,
                _,
                _,
                _,
                att_ult_opt,
                _,
                _,
                _,
                _,
                _,
                att_bag_opt,
                mut att_streak_opt,
                _,
                _,
                _,
            )) = actors.get_mut(attacker_entity)
            else {
                set_phase(state, CombatPhase::WaitingAction);
                return;
            };

            let Some(mut att_ult) = att_ult_opt else {
                set_phase(state, CombatPhase::WaitingAction);
                return;
            };

            let before = att_ult.current;

            match inflight.action.ult_effect {
                UltEffect::GainFromBasic => {
                    sp.gain(1);
                    let cpe = att_ult.charge_per_event;
                    att_ult.try_add(cpe);
                    if let Some(ref mut streak) = att_streak_opt {
                        streak.increment();
                    }
                }
                UltEffect::None => {}
                UltEffect::Reset => {
                    att_ult.current = 0;
                }
            }

            if inflight.action.ult_effect != UltEffect::Reset {
                if let Some(bag) = att_bag_opt.as_deref() {
                    if bag.has(&StatusEffectKind::Blessed) {
                        att_ult.try_add(1);
                    }
                }
            }

            att_ult.current - before
        };

        // Once-per-cast events.
        emit_combat_event(
            event_writer,
            CombatEventKind::OnSkillCast {
                skill_id: inflight.action.skill_id.clone(),
            },
            attacker_id,
            target_id,
            inflight.follow_up_depth,
            cast_id,
        );

        if inflight.action.advance_pct != 0 {
            emit_combat_event(
                event_writer,
                CombatEventKind::AdvanceTurn {
                    target: inflight.action.target,
                    amount_pct: inflight.action.advance_pct,
                },
                attacker_id,
                target_id,
                inflight.follow_up_depth,
                cast_id,
            );
        }
        if inflight.action.delay_pct != 0 {
            emit_combat_event(
                event_writer,
                CombatEventKind::DelayTurn {
                    target: inflight.action.target,
                    amount_pct: inflight.action.delay_pct,
                },
                attacker_id,
                target_id,
                inflight.follow_up_depth,
                cast_id,
            );
        }
        if inflight.action.self_advance_pct != 0 {
            let capped = (inflight.action.self_advance_pct.max(0) as u32).min(50);
            if capped != 0 {
                emit_combat_event(
                    event_writer,
                    CombatEventKind::AdvanceTurn {
                        target: inflight.action.source,
                        amount_pct: capped,
                    },
                    attacker_id,
                    attacker_id,
                    inflight.follow_up_depth,
                    cast_id,
                );
            }
        }

        if matches!(inflight.action.ult_effect, UltEffect::GainFromBasic) && ult_delta > 0 {
            emit_combat_event(
                event_writer,
                CombatEventKind::UltGain {
                    unit_id: attacker_id,
                    amount: ult_delta,
                },
                attacker_id,
                attacker_id,
                inflight.follow_up_depth,
                cast_id,
            );
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

        if inflight.action.energy_grant > 0 {
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

        dispatch_blueprint_transitions(inflight, log, event_writer, registry, cast_id);
        set_phase(state, CombatPhase::WaitingAction);
        return;
    }
    // === END BOUNCE PATH ===

    if attacker_entity == target_entity
        && inflight.action.base_damage == 0
        && inflight.action.toughness_damage == 0
        && inflight.action.revive_pct == 0
    {
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
            return;
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
            return;
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
            return;
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
                    let (_co, evs) =
                        apply_cleanse_only(&inflight.action, &mut **bag, defender_alive);
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
        return;
    }

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
        mut defender_dr,
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
