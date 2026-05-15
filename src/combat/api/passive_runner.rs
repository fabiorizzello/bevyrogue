use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

use bevy::log;
use bevy::prelude::{Resource, World};

use crate::combat::{
    api::{
        intent::{CastIdGen, Intent},
        registry::ExtRegistries,
        runner_common::{fire_beat, next_beat, RunnerParams, find_beat},
        signal::{Signal, SignalPayload},
        skill_ctx::SkillCtxMode,
        timeline::{BeatEvent, CompiledTimeline},
    },
    types::UnitId,
};

/// Maximum loop iterations before the circuit breaker fires.
const MAX_HOPS: u32 = 256;

/// Sibling to BeatRunner that consumes Signals and drives a timeline.
///
/// Unlike BeatRunner, PassiveRunner is persistent and has no persistent cursor.
/// Each matching signal triggers a fresh atomic drive of the timeline to completion.
pub struct PassiveRunner {
    pub timeline: Arc<CompiledTimeline>,
    pub owner: UnitId,
    pub triggers: Vec<(&'static str, &'static str)>,
    pub cast_hit_set: HashSet<UnitId>,
    pub last_beat_targets: Vec<UnitId>,
}

impl PassiveRunner {
    pub fn new(
        timeline: Arc<CompiledTimeline>,
        owner: UnitId,
        triggers: Vec<(&'static str, &'static str)>,
    ) -> Self {
        Self {
            timeline,
            owner,
            triggers,
            cast_hit_set: HashSet::new(),
            last_beat_targets: Vec::new(),
        }
    }

    /// React to a signal if it matches any of our triggers.
    pub fn react(
        &mut self,
        signal: &Signal,
        world: &World,
        regs: &ExtRegistries,
        mode: SkillCtxMode,
        pending: &mut VecDeque<Intent>,
        cast_id_gen: &mut CastIdGen,
    ) {
        let Signal::Blueprint { owner, name, payload, .. } = signal;

        let matched = self.triggers.iter().any(|(t_owner, t_name)| {
            *t_owner == owner.as_str() && *t_name == name.as_str()
        });

        if !matched {
            return;
        }

        let cast_id = cast_id_gen.next();
        let primary_target = match payload {
            SignalPayload::UnitTarget(unit) => *unit,
            _ => self.owner,
        };

        // Reset per-cast state
        self.cast_hit_set.clear();
        self.last_beat_targets.clear();

        let mut cursor = Some(self.timeline.entry);
        let mut hop_count = 0;

        while let Some(beat_id) = cursor {
            if hop_count >= MAX_HOPS {
                log::warn!(
                    "PassiveRunner circuit-breaker: owner={:?} timeline={} halted at hop_count={}",
                    self.owner,
                    self.timeline.id,
                    hop_count
                );
                break;
            }

            let beat = find_beat(&self.timeline, beat_id).clone();
            
            // Note: Passive timelines are assumed to be linear for S04 (no Loop support).
            // If they have Loop, it would need LoopFrame tracking here.
            
            let params = RunnerParams {
                timeline: &self.timeline,
                caster: self.owner,
                primary_target,
                cast_id,
                cast_hit_set: &mut self.cast_hit_set,
                world,
                regs,
                mode,
                pending,
            };

            let beat_targets = fire_beat(&beat, 0, params);
            self.last_beat_targets = beat_targets;

            let gate_evt = BeatEvent {
                cast_id,
                beat_id,
                hop_index: 0,
                beat_targets: self.last_beat_targets.clone(),
            };

            let mut params = RunnerParams {
                timeline: &self.timeline,
                caster: self.owner,
                primary_target,
                cast_id,
                cast_hit_set: &mut self.cast_hit_set,
                world,
                regs,
                mode,
                pending,
            };

            cursor = next_beat(beat_id, &gate_evt, &mut params);
            hop_count += 1;
        }
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
    use crate::combat::api::applier::{intent_applier, IntentQueue};

    let mut total_reacts = 0;

    loop {
        let signals: Vec<Signal> = world.resource_mut::<SignalBus>().drain().collect();
        if signals.is_empty() {
            break;
        }

        if total_reacts >= MAX_HOPS {
            log::warn!(
                "passive_dispatch_system: signal-cascade circuit-breaker fired (total_reacts={})",
                total_reacts
            );
            break;
        }

        world.resource_scope(|world_l, mut listeners: bevy::prelude::Mut<PassiveListeners>| {
            world_l.resource_scope(|world_r, regs: bevy::prelude::Mut<ExtRegistries>| {
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

        // After one pass of reacts, if new BlueprintSignals were enqueued, 
        // we might want to apply them immediately to allow same-frame cascade.
        // We call intent_applier to flush the queue and potentially populate SignalBus again.
        intent_applier(world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::api::{
        intent::CastId,
        registry::ExtRegistries,
        signal::Signal,
        timeline::{Beat, BeatEdge, BeatKind},
    };
    use std::num::NonZeroU32;

    fn cast_id() -> CastId {
        CastId(NonZeroU32::new(1).unwrap())
    }

    #[test]
    fn test_passive_trigger_match() {
        let timeline = Arc::new(CompiledTimeline {
            id: "passive_test",
            entry: "impact",
            beats: vec![Beat {
                id: "impact",
                kind: BeatKind::Impact,
                hook: Some("test_hook"),
                selector: None,
                presentation: None,
                payload: None,
            }],
            edges: vec![],
        });

        use std::sync::atomic::{AtomicU32, Ordering};
        static HOOK_CALLS: AtomicU32 = AtomicU32::new(0);
        fn test_hook(_evt: &BeatEvent, _ctx: &mut crate::combat::api::skill_ctx::SkillCtx<'_>) {
            HOOK_CALLS.fetch_add(1, Ordering::Relaxed);
        }

        let mut regs = ExtRegistries::default();
        regs.hooks.register("test_hook", test_hook);

        let mut runner = PassiveRunner::new(
            timeline,
            UnitId(1),
            vec![("renamon", "ult_used")],
        );

        let world = World::new();
        let mut pending = VecDeque::new();
        let mut cast_id_gen = CastIdGen::default();

        // 1. Non-matching signal
        let sig_none = Signal::Blueprint {
            owner: "other".to_string(),
            name: "ult_used".to_string(),
            payload: SignalPayload::Empty,
            cast_id: cast_id(),
        };
        HOOK_CALLS.store(0, Ordering::Relaxed);
        runner.react(&sig_none, &world, &regs, SkillCtxMode::Execute, &mut pending, &mut cast_id_gen);
        assert_eq!(HOOK_CALLS.load(Ordering::Relaxed), 0);

        // 2. Matching signal
        let sig_match = Signal::Blueprint {
            owner: "renamon".to_string(),
            name: "ult_used".to_string(),
            payload: SignalPayload::Empty,
            cast_id: cast_id(),
        };
        runner.react(&sig_match, &world, &regs, SkillCtxMode::Execute, &mut pending, &mut cast_id_gen);
        assert_eq!(HOOK_CALLS.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_passive_circuit_breaker() {
        let timeline = Arc::new(CompiledTimeline {
            id: "loop_test",
            entry: "impact",
            beats: vec![Beat {
                id: "impact",
                kind: BeatKind::Impact,
                hook: Some("signal_hook"),
                selector: None,
                presentation: None,
                payload: None,
            }],
            // Self-loop
            edges: vec![BeatEdge { from: "impact", to: "impact", gate: None }],
        });

        fn signal_hook(_evt: &BeatEvent, _ctx: &mut crate::combat::api::skill_ctx::SkillCtx<'_>) {
            // No-op, just needed to fire
        }

        let mut regs = ExtRegistries::default();
        regs.hooks.register("signal_hook", signal_hook);

        let mut runner = PassiveRunner::new(
            timeline,
            UnitId(1),
            vec![("test", "loop")],
        );

        let world = World::new();
        let mut pending = VecDeque::new();
        let mut cast_id_gen = CastIdGen::default();

        let sig = Signal::Blueprint {
            owner: "test".to_string(),
            name: "loop".to_string(),
            payload: SignalPayload::Empty,
            cast_id: cast_id(),
        };

        // Should halt at MAX_HOPS=256 without crashing
        runner.react(&sig, &world, &regs, SkillCtxMode::Execute, &mut pending, &mut cast_id_gen);
    }
}
