use std::sync::Arc;

use bevy::log;
use bevy::prelude::{Resource, World};

use crate::combat::{
    runtime::{
        applier::{IntentQueue, intent_applier},
        event_filter::EventFilter,
        intent::CastIdGen,
        runner::{BeatRunner, StepOutcome},
        signal::Signal,
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
    pub fn new(timeline: Arc<CompiledTimeline>, owner: UnitId, filters: Vec<EventFilter>) -> Self {
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
        regs: &crate::combat::runtime::registry::ExtRegistries,
        mode: SkillCtxMode,
        pending: &mut std::collections::VecDeque<crate::combat::runtime::intent::Intent>,
        cast_id_gen: &mut CastIdGen,
    ) {
        if !self.matches(signal) {
            return;
        }

        let cast_id = cast_id_gen.next();
        let primary_target = signal.primary_target(self.owner);

        let mut runner = BeatRunner::new(
            Arc::clone(&self.timeline),
            cast_id,
            self.owner,
            primary_target,
        );

        for step_index in 0..1_024 {
            let outcome = runner.step(world, regs, mode, pending);

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

        panic!("PassiveRunner::react exceeded max_steps=1024; possible infinite loop");
    }
}

/// Resource that holds all active passive runners.
#[derive(Resource, Default)]
pub struct PassiveListeners {
    pub runners: Vec<PassiveRunner>,
}

/// System that drains SignalBus and dispatches to PassiveListeners.
pub fn passive_dispatch_system(world: &mut World) {
    use crate::combat::runtime::signal::SignalBus;

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
            world_l.resource_scope(|world_r, regs: bevy::prelude::Mut<crate::combat::runtime::registry::ExtRegistries>| {
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
