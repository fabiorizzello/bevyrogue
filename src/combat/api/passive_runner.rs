use std::sync::Arc;

use bevy::log;
use bevy::prelude::{Resource, World};

use crate::combat::{
    api::{
        applier::{intent_applier, IntentQueue},
        event_filter::EventFilter,
        intent::CastIdGen,
        runner::{BeatRunner, StepOutcome},
        signal::{Signal, SignalBus},
        skill_ctx::SkillCtxMode,
        timeline::CompiledTimeline,
    },
    types::UnitId,
};

/// Sibling to `BeatRunner` that consumes reactive signals and drives a timeline.
///
/// The runner is persistent, but each matched signal triggers a fresh atomic run
/// of the underlying timeline to completion.
pub struct PassiveRunner {
    pub timeline: Arc<CompiledTimeline>,
    pub owner: UnitId,
    pub filters: Vec<EventFilter>,
}

impl PassiveRunner {
    pub fn new(
        timeline: Arc<CompiledTimeline>,
        owner: UnitId,
        filters: Vec<EventFilter>,
    ) -> Self {
        Self {
            timeline,
            owner,
            filters,
        }
    }

    /// Returns `true` when any subscription matches the incoming signal.
    pub fn matches(&self, signal: &Signal) -> bool {
        self.filters.iter().any(|filter| filter.matches(signal))
    }

    /// React to a signal if it matches any subscription.
    pub fn react(
        &mut self,
        signal: &Signal,
        world: &mut World,
        regs: &crate::combat::api::registry::ExtRegistries,
        mode: SkillCtxMode,
        pending: &mut std::collections::VecDeque<crate::combat::api::intent::Intent>,
        cast_id_gen: &mut CastIdGen,
    ) {
        if !self.matches(signal) {
            return;
        }

        let cast_id = cast_id_gen.next();
        let primary_target = signal.primary_target(self.owner);

        let mut runner = BeatRunner::new(Arc::clone(&self.timeline), cast_id, self.owner, primary_target);

        for step_index in 0..1_024 {
            let outcome = runner.step(world, regs, mode, pending);
            intent_applier(world);

            if step_index > 0 && !runner.in_loop() && runner.cursor() == Some(runner.entry()) {
                return;
            }

            match outcome {
                StepOutcome::Done => return,
                StepOutcome::Halted => {
                    // BeatRunner already logs the circuit-breaker; this branch exists only
                    // to make the control flow explicit for future diagnostics.
                    log::debug!(
                        "PassiveRunner halted after running timeline={} for owner={:?}",
                        self.timeline.id,
                        self.owner
                    );
                    return;
                }
                StepOutcome::AwaitingCue => runner.resume_cue(),
                StepOutcome::Advanced | StepOutcome::LoopExited => {}
            }
        }

        panic!(
            "PassiveRunner::react exceeded max_steps=1024; possible infinite loop"
        );
    }
}

/// Resource that holds all active passive runners.
#[derive(Resource, Default)]
pub struct PassiveListeners {
    pub runners: Vec<PassiveRunner>,
}

/// System that drains SignalBus and dispatches to PassiveListeners.
pub fn passive_dispatch_system(world: &mut World) {
    use crate::combat::api::signal::SignalBus;

    let mut total_reacts = 0;

    loop {
        let signals: Vec<Signal> = world.resource_mut::<SignalBus>().drain().collect();
        if signals.is_empty() {
            break;
        }

        if total_reacts >= 256 {
            log::warn!(
                "passive_dispatch_system: signal-cascade circuit-breaker fired (total_reacts={})",
                total_reacts
            );
            break;
        }

        world.resource_scope(|world_l, mut listeners: bevy::prelude::Mut<PassiveListeners>| {
            world_l.resource_scope(|world_r, regs: bevy::prelude::Mut<crate::combat::api::registry::ExtRegistries>| {
                world_r.resource_scope(|world_q, mut queue: bevy::prelude::Mut<IntentQueue>| {
                    world_q.resource_scope(|world_c, mut cast_id_gen: bevy::prelude::Mut<CastIdGen>| {
                        for signal in &signals {
                            for runner in &mut listeners.runners {
                                runner.react(
                                    signal,
                                    world_c,
                                    &regs,
                                    SkillCtxMode::Execute,
                                    &mut queue.0,
                                    &mut cast_id_gen,
                                );
                            }
                            total_reacts += 1;
                        }
                    });
                });
            });
        });

        // Flush the intents produced by the reactive pass. Any new signals emitted
        // by the applier will be observed in the next outer loop iteration, keeping
        // same-frame cascades deterministic.
        intent_applier(world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::api::{
        event_filter::EventFilter,
        intent::CastId,
        registry::ExtRegistries,
        signal::{Signal, SignalPayload},
        timeline::{Beat, BeatEdge, BeatEvent, BeatKind},
    };
    use std::{collections::VecDeque, num::NonZeroU32};

    fn cast_id() -> CastId {
        CastId(NonZeroU32::new(1).unwrap())
    }

    #[test]
    fn blueprint_filter_matches_exact_signal() {
        let mut runner = PassiveRunner::new(
            Arc::new(CompiledTimeline {
                id: "passive_test",
                entry: "impact",
                beats: vec![Beat {
                    id: "impact",
                    kind: BeatKind::Impact,
                    hook: None,
                    selector: None,
                    presentation: None,
                    payload: None,
                }],
                edges: vec![],
            }),
            UnitId(1),
            vec![EventFilter::blueprint("renamon", "ult_used")],
        );

        let signal = Signal::Blueprint {
            owner: "renamon".to_string(),
            name: "ult_used".to_string(),
            payload: SignalPayload::Empty,
            cast_id: cast_id(),
        };
        assert!(runner.matches(&signal));

        let other = Signal::Blueprint {
            owner: "other".to_string(),
            name: "ult_used".to_string(),
            payload: SignalPayload::Empty,
            cast_id: cast_id(),
        };
        assert!(!runner.matches(&other));
    }

    #[test]
    fn loop_timeline_halts_at_circuit_breaker() {
        fn never(_evt: &BeatEvent, _ctx: &crate::combat::api::skill_ctx::SkillCtx<'_>) -> bool {
            false
        }

        fn loop_hook(evt: &BeatEvent, ctx: &mut crate::combat::api::skill_ctx::SkillCtx<'_>) {
            use std::sync::atomic::{AtomicU32, Ordering};

            static LOOP_CALLS: AtomicU32 = AtomicU32::new(0);
            let count = LOOP_CALLS.fetch_add(1, Ordering::Relaxed) + 1;
            ctx.enqueue(crate::combat::api::intent::Intent::SetBlueprintState {
                actor: ctx.caster,
                key: "loop/count".to_string(),
                value: count as i64,
                cast_id: evt.cast_id,
            });
        }

        let mut regs = ExtRegistries::default();
        regs.predicates.register("loop/never", never);
        regs.hooks.register("loop/tick", loop_hook);

        let timeline = Arc::new(CompiledTimeline {
            id: "loop_test",
            entry: "loop",
            beats: vec![Beat {
                id: "loop",
                kind: BeatKind::Loop {
                    body: vec![Beat {
                        id: "tick",
                        kind: BeatKind::Impact,
                        hook: Some("loop/tick"),
                        selector: None,
                        presentation: None,
                        payload: None,
                    }],
                    exit_when: "loop/never",
                },
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            }],
            edges: vec![],
        });

        let mut runner = PassiveRunner::new(
            timeline,
            UnitId(1),
            vec![EventFilter::custom(|signal| matches!(signal, Signal::CombatEvent(_)))],
        );

        let mut world = World::new();
        world.init_resource::<crate::combat::api::blueprint_state::BlueprintState>();
        let mut pending = VecDeque::new();
        let mut cast_id_gen = crate::combat::api::intent::CastIdGen::default();

        let signal = Signal::CombatEvent(crate::combat::events::CombatEvent {
            kind: crate::combat::events::CombatEventKind::OnActionDeclared {
                intent_kind: crate::combat::events::ActionIntentKind::Basic,
            },
            source: UnitId(1),
            target: UnitId(2),
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });

        runner.react(&signal, &mut world, &regs, SkillCtxMode::Execute, &mut pending, &mut cast_id_gen);

        let state = world.resource::<crate::combat::api::blueprint_state::BlueprintState>();
        assert_eq!(
            state.map.get(&(UnitId(1), "loop/count".to_string())),
            Some(&256),
            "loop timeline should stop at the 256-hop guard"
        );
    }
}
