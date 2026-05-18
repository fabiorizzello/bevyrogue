use bevy::prelude::*;

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::runtime::applier::{IntentExecutionMeta, IntentQueue};
use crate::combat::runtime::intent::CastId;
use crate::combat::runtime::runner::{BeatRunner, StepOutcome};
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatPhase, InFlightAction, UltEffect};

pub(crate) fn run_timeline_backed_action(
    world: &mut World,
    inflight: InFlightAction,
    cast_id: CastId,
) {
    let mut _fallback_regs = None;
    let regs_ptr: *const crate::combat::runtime::registry::ExtRegistries =
        if let Some(regs) = world.get_resource::<crate::combat::runtime::registry::ExtRegistries>() {
            regs as *const _
        } else {
            let mut regs = crate::combat::runtime::registry::ExtRegistries::default();
            crate::combat::runtime::builtins::register_kernel_builtins(&mut regs);
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
            crate::combat::runtime::SkillCtxMode::Execute,
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
        .insert(cast_id, inflight.follow_up_depth);
    world.resource_mut::<IntentQueue>().0.extend(pending);
    crate::combat::runtime::applier::intent_applier(world);
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
