use bevy::prelude::*;

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::runtime::applier::{IntentExecutionMeta, IntentQueue};
use crate::combat::runtime::intent::CastId;
use crate::combat::runtime::runner::{BeatRunner, StepOutcome};
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatPhase, InFlightAction, UltEffect};

#[allow(dead_code)]
fn intern_timeline_id(value: &str) -> &'static str {
    Box::leak(value.to_owned().into_boxed_str())
}

#[allow(dead_code)]
fn intern_compiled_timeline(
    timeline: &crate::combat::runtime::timeline::CompiledTimeline<String>,
) -> crate::combat::runtime::timeline::CompiledTimeline<&'static str> {
    use crate::combat::runtime::timeline::{
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
