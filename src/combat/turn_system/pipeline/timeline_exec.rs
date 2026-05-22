use std::collections::VecDeque;

use bevy::prelude::*;

use crate::combat::energy::Energy;
use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::kernel::{CombatKernelRegistry, CombatKernelTransition};
use crate::combat::runtime::applier::{IntentExecutionMeta, IntentQueue};
use crate::combat::runtime::cue_barrier::{
    SuspendedTimeline, SuspendedTimelineState, TimelineClock,
};
use crate::combat::runtime::intent::CastId;
use crate::combat::runtime::post_action::{
    PostActionContext, PostActionUnitDied, PostActionUnitSnapshot, dispatch_post_action_reactions,
};
use crate::combat::runtime::runner::{BeatRunner, StepOutcome};
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatPhase, InFlightAction, UltEffect};
use crate::combat::status_effect::StatusBag;
use crate::combat::team::Team;
use crate::combat::types::UnitId;
use crate::combat::unit::{SlotIndex, Unit};

use super::super::set_phase;

/// Compact pre/post-cast snapshot of a unit used to construct the
/// `PostActionContext` for timeline-backed actions.
#[derive(Clone)]
struct PostActionUnitSlice {
    unit_id: UnitId,
    team: Team,
    slot_index: Option<u8>,
    hp_current: i32,
    hp_max: i32,
    alive: bool,
    heated_remaining: u32,
    status_remaining: Vec<crate::combat::status_effect::StatusEffectKind>,
}

fn snapshot_post_action_units(world: &mut World) -> Vec<PostActionUnitSlice> {
    let mut q = world.query::<(
        &Unit,
        &Team,
        Option<&SlotIndex>,
        Option<&StatusBag>,
    )>();
    q.iter(world)
        .map(|(unit, team, slot, status)| {
            let (status_remaining, heated_remaining) = match status {
                Some(b) => (
                    b.iter().map(|inst| inst.kind.clone()).collect(),
                    b.get_dur(&crate::combat::status_effect::StatusEffectKind::Heated)
                        .unwrap_or(0),
                ),
                None => (vec![], 0),
            };
            PostActionUnitSlice {
                unit_id: unit.id,
                team: *team,
                slot_index: slot.map(|s| s.0),
                hp_current: unit.hp_current,
                hp_max: unit.hp_max,
                alive: unit.hp_current > 0,
                heated_remaining,
                status_remaining,
            }
        })
        .collect()
}

const MAX_TIMELINE_STEPS: usize = 1024;

pub(crate) fn run_timeline_backed_action(
    world: &mut World,
    inflight: InFlightAction,
    cast_id: CastId,
) {
    world.init_resource::<TimelineClock>();
    world.init_resource::<SuspendedTimelineState>();

    set_phase(
        &mut world.resource_mut::<crate::combat::state::CombatState>(),
        CombatPhase::Resolving,
    );

    let mut fallback_regs = None;
    let regs_ptr = resolve_regs_ptr(world, &mut fallback_regs);

    let Some(timeline) = crate::combat::preview::resolve_compiled_skill_timeline(
        world,
        &inflight.action.skill_id,
        unsafe { &*regs_ptr },
    ) else {
        fail_timeline_action(
            world,
            &inflight,
            cast_id,
            format!(
                "timeline-backed skill missing compiled timeline: {:?}",
                inflight.action.skill_id
            ),
        );
        return;
    };

    if !preflight_timeline_action(world, &inflight, cast_id) {
        return;
    }

    let clock = world
        .get_resource::<TimelineClock>()
        .copied()
        .unwrap_or_default()
        .0;
    let mut pending = VecDeque::new();
    let mut runner = BeatRunner::new(
        timeline,
        cast_id,
        inflight.action.source,
        inflight.action.target,
    )
    .with_clock(clock);
    let outcome = unsafe {
        runner.run_to_completion(
            world,
            &*regs_ptr,
            crate::combat::runtime::SkillCtxMode::Execute,
            &mut pending,
            MAX_TIMELINE_STEPS,
        )
    };

    handle_timeline_outcome(world, inflight, cast_id, runner, pending, outcome);
}

pub fn continue_suspended_timeline_system(world: &mut World) {
    continue_suspended_timeline(world);
}

pub fn continue_suspended_timeline(world: &mut World) {
    world.init_resource::<SuspendedTimelineState>();

    {
        let mut barrier = world.resource_mut::<SuspendedTimelineState>();
        barrier.tick_timeout();
    }

    let Some(mut suspended) = ({
        let mut barrier = world.resource_mut::<SuspendedTimelineState>();
        barrier.take_released()
    }) else {
        return;
    };

    let mut fallback_regs = None;
    let regs_ptr = resolve_regs_ptr(world, &mut fallback_regs);

    suspended.runner.resume_cue();
    let outcome = unsafe {
        suspended.runner.run_to_completion(
            world,
            &*regs_ptr,
            crate::combat::runtime::SkillCtxMode::Execute,
            &mut suspended.pending,
            MAX_TIMELINE_STEPS,
        )
    };

    handle_timeline_outcome(
        world,
        suspended.inflight,
        suspended.cast_id,
        suspended.runner,
        suspended.pending,
        outcome,
    );
}

fn handle_timeline_outcome(
    world: &mut World,
    inflight: InFlightAction,
    cast_id: CastId,
    runner: BeatRunner,
    pending: VecDeque<crate::combat::runtime::Intent>,
    outcome: StepOutcome,
) {
    match outcome {
        StepOutcome::Done => finalize_timeline_action(world, inflight, cast_id, pending),
        StepOutcome::AwaitingCue => {
            world
                .resource_mut::<SuspendedTimelineState>()
                .suspend(SuspendedTimeline::new(runner, pending, inflight, cast_id));
            set_phase(
                &mut world.resource_mut::<crate::combat::state::CombatState>(),
                CombatPhase::Resolving,
            );
        }
        other => fail_timeline_action(
            world,
            &inflight,
            cast_id,
            format!("timeline execution halted: {other:?}"),
        ),
    }
}

fn preflight_timeline_action(
    world: &mut World,
    inflight: &InFlightAction,
    cast_id: CastId,
) -> bool {
    let attacker_id = inflight.action.source;

    if inflight.action.sp_cost > 0 {
        let current_sp = world
            .get_resource::<SpPool>()
            .map(|sp| sp.current)
            .unwrap_or(0);
        if current_sp < inflight.action.sp_cost {
            preflight_fail_timeline_action(world, inflight, cast_id, "SP shortfall".to_string());
            return false;
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
            preflight_fail_timeline_action(world, inflight, cast_id, "SP shortfall".to_string());
            return false;
        }
    }

    true
}

fn preflight_fail_timeline_action(
    world: &mut World,
    inflight: &InFlightAction,
    cast_id: CastId,
    reason: String,
) {
    let source = inflight.action.source;
    let target = inflight.action.target;
    let follow_up_depth = inflight.follow_up_depth;

    let mut events = world.resource_mut::<bevy::ecs::message::Messages<CombatEvent>>();
    events.write(CombatEvent {
        kind: CombatEventKind::OnActionFailed {
            reason: reason.clone(),
        },
        source,
        target,
        follow_up_depth,
        cast_id,
    });
    events.write(CombatEvent {
        kind: CombatEventKind::OnActionApplied,
        source,
        target,
        follow_up_depth,
        cast_id,
    });
    events.write(CombatEvent {
        kind: CombatEventKind::OnActionResolved,
        source,
        target,
        follow_up_depth,
        cast_id,
    });
    drop(events);

    world.resource_mut::<SuspendedTimelineState>().note_failure(
        cast_id,
        &inflight.action.skill_id,
        &reason,
    );
    set_phase(
        &mut world.resource_mut::<crate::combat::state::CombatState>(),
        CombatPhase::WaitingAction,
    );
}

fn fail_timeline_action(
    world: &mut World,
    inflight: &InFlightAction,
    cast_id: CastId,
    reason: String,
) {
    world
        .resource_mut::<bevy::ecs::message::Messages<CombatEvent>>()
        .write(CombatEvent {
            kind: CombatEventKind::OnActionFailed {
                reason: reason.clone(),
            },
            source: inflight.action.source,
            target: inflight.action.target,
            follow_up_depth: inflight.follow_up_depth,
            cast_id,
        });

    world.resource_mut::<SuspendedTimelineState>().note_failure(
        cast_id,
        &inflight.action.skill_id,
        &reason,
    );
    set_phase(
        &mut world.resource_mut::<crate::combat::state::CombatState>(),
        CombatPhase::WaitingAction,
    );
}

fn finalize_timeline_action(
    world: &mut World,
    inflight: InFlightAction,
    cast_id: CastId,
    pending: VecDeque<crate::combat::runtime::Intent>,
) {
    prepare_timeline_intent_runtime(world, cast_id, inflight.follow_up_depth);

    // Snapshot pre-cast unit state so we can compute the `UnitDied` payload
    // for the primary target (heated/status remaining at moment of KO) and
    // detect who actually died during this cast.
    let pre_units = snapshot_post_action_units(world);

    world.resource_mut::<IntentQueue>().0.extend(pending);
    crate::combat::runtime::applier::intent_applier(world);

    // Dispatch post-action reactions (e.g. Baby Burner reactive detonate).
    dispatch_timeline_post_action(world, &inflight, cast_id, &pre_units);

    world
        .resource_mut::<IntentExecutionMeta>()
        .follow_up_depths
        .remove(&cast_id);

    let attacker_id = inflight.action.source;
    let ult_effect = inflight.action.ult_effect.clone();

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

        // Basic attacks grant +1 SP. The legacy executor does this in
        // `apply_legacy_ops` (UltEffect::GainFromBasic => sp.gain(1)); the
        // timeline executor must mirror it or timeline-backed basics (e.g.
        // Agumon) silently skip the SP gain. `finalize_timeline_action` runs
        // only on StepOutcome::Done, so this fires only on a successful basic,
        // matching legacy semantics. SpPool.gain clamps at SpPool.max.
        if matches!(ult_effect, UltEffect::GainFromBasic) {
            world.resource_mut::<SpPool>().gain(1);
        }

        // S07/T03: when the attacker is energy-backed, drain `Energy.current`
        // alongside the legacy `UltimateCharge.current = 0` so an Ult cast
        // consumes the new energy resource. Legacy `UltimateCharge` is kept at
        // 0 above for back-compat until the old gauge is fully smantellato.
        if matches!(ult_effect, UltEffect::Reset) {
            let mut q = world.query::<(
                &crate::combat::unit::Unit,
                &mut crate::combat::energy::Energy,
                &crate::combat::ult_gauge::UltGaugeMetadata,
            )>();
            for (unit, mut energy, meta) in q.iter_mut(world) {
                if unit.id == attacker_id {
                    crate::combat::ult_gauge::drain_energy_on_ult_reset(
                        Some(meta),
                        Some(energy.as_mut()),
                    );
                    break;
                }
            }
        }
    }

    if inflight.action.energy_grant > 0 {
        let mut q = world.query::<(&crate::combat::unit::Unit, &mut Energy)>();
        for (unit, mut energy) in q.iter_mut(world) {
            if unit.id != attacker_id {
                continue;
            }
            let applied = energy.gain_capped(inflight.action.energy_grant);
            if applied > 0 {
                world
                    .resource_mut::<bevy::ecs::message::Messages<CombatEvent>>()
                    .write(CombatEvent {
                        kind: CombatEventKind::EnergyGained {
                            unit_id: attacker_id,
                            amount: applied,
                        },
                        source: attacker_id,
                        target: attacker_id,
                        follow_up_depth: inflight.follow_up_depth,
                        cast_id,
                    });
            }
            break;
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
    drop(event_writer);

    world
        .resource_mut::<SuspendedTimelineState>()
        .note_completion(cast_id, &inflight.action.skill_id);
    set_phase(
        &mut world.resource_mut::<crate::combat::state::CombatState>(),
        CombatPhase::WaitingAction,
    );
}

fn prepare_timeline_intent_runtime(world: &mut World, cast_id: CastId, follow_up_depth: u8) {
    world.init_resource::<IntentQueue>();
    world.init_resource::<IntentExecutionMeta>();
    world.init_resource::<crate::combat::runtime::signal::SignalBus>();
    world.init_resource::<crate::combat::runtime::signal::SignalTaxonomy>();
    world.init_resource::<crate::combat::runtime::blueprint_state::BlueprintState>();

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
        let mut taxonomy = world.resource_mut::<crate::combat::runtime::signal::SignalTaxonomy>();
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
        .insert(cast_id, follow_up_depth);
}

fn dispatch_timeline_post_action(
    world: &mut World,
    inflight: &InFlightAction,
    cast_id: CastId,
    pre_units: &[PostActionUnitSlice],
) {
    let mut fallback_regs = None;
    let regs_ptr = resolve_regs_ptr(world, &mut fallback_regs);

    let post_units = snapshot_post_action_units(world);

    let primary_target = inflight.action.target;
    let unit_died = pre_units
        .iter()
        .find(|u| u.unit_id == primary_target && u.alive)
        .and_then(|pre| {
            post_units
                .iter()
                .find(|u| u.unit_id == primary_target && !u.alive)
                .map(|_| {
                    PostActionUnitDied::new(pre.status_remaining.clone(), pre.heated_remaining)
                })
        });

    let roster: Vec<PostActionUnitSnapshot> = post_units
        .iter()
        .map(|u| {
            PostActionUnitSnapshot::new(
                u.unit_id,
                u.team,
                u.slot_index,
                u.hp_current,
                u.hp_max,
                u.alive,
            )
        })
        .collect();

    let ctx = PostActionContext::new(
        inflight.action.skill_id.clone(),
        inflight.action.source,
        primary_target,
        cast_id,
        inflight.follow_up_depth,
        unit_died,
        roster,
    );

    // SAFETY: resolve_regs_ptr returns either a pointer into `world` or into
    // the local `fallback_regs`. `dispatch_post_action_reactions` only reads.
    let queue = unsafe { dispatch_post_action_reactions(&*regs_ptr, &ctx) };

    if !queue.intents.is_empty() {
        world.resource_mut::<IntentQueue>().0.extend(queue.intents);
        crate::combat::runtime::applier::intent_applier(world);
    }

    if !queue.transitions.is_empty() {
        // Expand each transition through the kernel registry first (immutable
        // read), then write the resulting events. The registry and the
        // messages resource cannot be borrowed simultaneously.
        let registry_ptr = world
            .get_resource::<CombatKernelRegistry>()
            .map(|r| r as *const CombatKernelRegistry);
        let mut expanded: Vec<CombatKernelTransition> = Vec::new();
        for transition in queue.transitions {
            match registry_ptr {
                Some(p) => unsafe { expanded.extend((*p).dispatch(transition)) },
                None => expanded.push(transition),
            }
        }
        let mut events = world.resource_mut::<bevy::ecs::message::Messages<CombatEvent>>();
        for transition in expanded {
            events.write(CombatEvent {
                kind: CombatEventKind::OnKernelTransition { transition },
                source: inflight.action.source,
                target: primary_target,
                follow_up_depth: inflight.follow_up_depth,
                cast_id,
            });
        }
    }

    // Touch fallback_regs to keep its lifetime in scope until regs_ptr is unused.
    let _ = (regs_ptr, &fallback_regs);
}

fn resolve_regs_ptr<'a>(
    world: &mut World,
    fallback_regs: &'a mut Option<crate::combat::runtime::registry::ExtRegistries>,
) -> *const crate::combat::runtime::registry::ExtRegistries {
    if let Some(regs) = world.get_resource::<crate::combat::runtime::registry::ExtRegistries>() {
        regs as *const _
    } else {
        let mut regs = crate::combat::runtime::registry::ExtRegistries::default();
        crate::combat::runtime::builtins::register_kernel_builtins(&mut regs);
        *fallback_regs = Some(regs);
        fallback_regs
            .as_ref()
            .expect("fallback ext registries initialized") as *const _
    }
}
