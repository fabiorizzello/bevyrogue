//! Unit-level contracts for `combat::runtime::passive_runner`:
//!
//! * `PassiveRunner::matches` — owner/name filter precision (no integration
//!   counterpart asserts `runner.matches` directly).
//! * `PassiveRunner::react` — 256-hop circuit-breaker fires on a never-exiting
//!   loop (integration tests in `tests/passive_event_filters.rs` and
//!   `tests/timeline_circuit_breaker.rs` exercise the BeatRunner-level breaker
//!   but not via the PassiveRunner::react entry path).
//!
//! Relocated from `src/combat/runtime/passive_runner.rs` per R003.

use std::sync::Arc;
use std::{collections::VecDeque, num::NonZeroU32};

use bevy::prelude::World;
use bevyrogue::combat::{
    events::{ActionIntentKind, CombatEvent, CombatEventKind},
    runtime::{
        BlueprintState, CastId, CastIdGen, EventFilter, ExtRegistries, Intent, IntentQueue,
        PassiveRunner, Signal, SignalPayload, SkillCtxMode,
        applier::intent_applier,
        skill_ctx::SkillCtx,
        timeline::{Beat, BeatEvent, BeatKind, CompiledTimeline},
    },
    types::UnitId,
};

fn cast_id() -> CastId {
    CastId(NonZeroU32::new(1).unwrap())
}

#[test]
fn blueprint_filter_matches_exact_signal() {
    let runner = PassiveRunner::new(
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
    fn never(_evt: &BeatEvent, _ctx: &SkillCtx<'_>) -> bool {
        false
    }

    fn loop_hook(evt: &BeatEvent, ctx: &mut SkillCtx<'_>) {
        use std::sync::atomic::{AtomicU32, Ordering};

        static LOOP_CALLS: AtomicU32 = AtomicU32::new(0);
        let count = LOOP_CALLS.fetch_add(1, Ordering::Relaxed) + 1;
        ctx.enqueue(Intent::SetBlueprintState {
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
        vec![EventFilter::custom(|signal| {
            matches!(signal, Signal::CombatEvent(_))
        })],
    );

    let mut world = World::new();
    world.init_resource::<BlueprintState>();
    let mut pending = VecDeque::new();
    let mut cast_id_gen = CastIdGen::default();

    let signal = Signal::CombatEvent(CombatEvent {
        kind: CombatEventKind::OnActionDeclared {
            intent_kind: ActionIntentKind::Basic,
        },
        source: UnitId(1),
        target: UnitId(2),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });

    runner.react(
        &signal,
        &mut world,
        &regs,
        SkillCtxMode::Execute,
        &mut pending,
        &mut cast_id_gen,
    );
    world.insert_resource(IntentQueue(pending));
    intent_applier(&mut world);

    let state = world.resource::<BlueprintState>();
    assert_eq!(
        state.map.get(&(UnitId(1), "loop/count".to_string())),
        Some(&256),
        "loop timeline should stop at the 256-hop guard"
    );
}
