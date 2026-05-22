use std::sync::Arc;

use crate::common::app::passive_dispatch_app;
use bevy::prelude::*;
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    runtime::{
        BlueprintState, CastId, EventFilter, ExtRegistries, Intent, PassiveListeners,
        PassiveRunner, Signal, SignalBus, SignalPayload, SignalTaxonomy,
    },
    types::UnitId,
};

const OWNER: UnitId = UnitId(10);
const TARGET: UnitId = UnitId(11);
const TRACE_KEY: &str = "trace/order";
const ULT_KEY: &str = "ult/seen";
const LOOP_KEY: &str = "loop/count";

fn register_hooks(app: &mut App) {
    let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
    regs.hooks.register("trace/composite", composite_hook);
    regs.hooks.register("trace/ult_seen", ult_seen_hook);
    regs.hooks.register("trace/cascade_seen", cascade_seen_hook);
    regs.hooks.register("trace/loop_tick", loop_tick_hook);
    regs.predicates.register("trace/loop_never", never_exit);

    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register("cascade", "follow");
}

fn register_passives(app: &mut App) {
    let composite_filter = EventFilter::all([
        EventFilter::any([
            EventFilter::combat(|event| {
                matches!(&event.kind, CombatEventKind::UltimateUsed { .. })
            }),
            EventFilter::blueprint("kernel", "ult_used"),
        ]),
        EventFilter::custom(|signal| matches!(signal, Signal::CombatEvent(_))),
        EventFilter::not(EventFilter::blueprint("kernel", "ult_used")),
    ]);

    let mut listeners = app.world_mut().resource_mut::<PassiveListeners>();
    listeners.runners.push(PassiveRunner::new(
        build_single_beat_timeline("trace/composite", "trace/composite"),
        OWNER,
        vec![composite_filter],
    ));
    listeners.runners.push(PassiveRunner::new(
        build_single_beat_timeline("trace/ult_seen", "trace/ult_seen"),
        OWNER,
        vec![EventFilter::blueprint("kernel", "ult_used")],
    ));
    listeners.runners.push(PassiveRunner::new(
        build_single_beat_timeline("trace/cascade_seen", "trace/cascade_seen"),
        OWNER,
        vec![EventFilter::blueprint("cascade", "follow")],
    ));
    listeners.runners.push(PassiveRunner::new(
        build_loop_timeline(),
        OWNER,
        vec![EventFilter::custom(|signal| {
            matches!(signal, Signal::CombatEvent(event) if matches!(&event.kind, CombatEventKind::UltimateUsed { .. }))
        })],
    ));
}

fn build_single_beat_timeline(
    id: &'static str,
    hook: &'static str,
) -> Arc<bevyrogue::combat::runtime::CompiledTimeline> {
    Arc::new(bevyrogue::combat::runtime::CompiledTimeline {
        id,
        entry: "start",
        beats: vec![bevyrogue::combat::runtime::Beat {
            id: "start",
            kind: bevyrogue::combat::runtime::BeatKind::Impact,
            hook: Some(hook),
            selector: None,
            presentation: None,
            payload: None,
        }],
        edges: vec![],
    })
}

fn build_loop_timeline() -> Arc<bevyrogue::combat::runtime::CompiledTimeline> {
    Arc::new(bevyrogue::combat::runtime::CompiledTimeline {
        id: "trace/loop",
        entry: "loop",
        beats: vec![bevyrogue::combat::runtime::Beat {
            id: "loop",
            kind: bevyrogue::combat::runtime::BeatKind::Loop {
                body: vec![bevyrogue::combat::runtime::Beat {
                    id: "tick",
                    kind: bevyrogue::combat::runtime::BeatKind::Impact,
                    hook: Some("trace/loop_tick"),
                    selector: None,
                    presentation: None,
                    payload: None,
                }],
                exit_when: "trace/loop_never",
            },
            hook: None,
            selector: None,
            presentation: None,
            payload: None,
        }],
        edges: vec![],
    })
}

fn composite_hook(
    evt: &bevyrogue::combat::runtime::BeatEvent,
    ctx: &mut bevyrogue::combat::runtime::SkillCtx<'_>,
) {
    ctx.enqueue(Intent::SetBlueprintState {
        actor: ctx.caster,
        key: TRACE_KEY.to_string(),
        value: 1,
        cast_id: evt.cast_id,
    });
    ctx.enqueue(Intent::BlueprintSignal {
        source: ctx.caster,
        owner: "cascade",
        name: "follow",
        payload: SignalPayload::Empty,
        cast_id: evt.cast_id,
    });
}

fn ult_seen_hook(
    evt: &bevyrogue::combat::runtime::BeatEvent,
    ctx: &mut bevyrogue::combat::runtime::SkillCtx<'_>,
) {
    ctx.enqueue(Intent::SetBlueprintState {
        actor: ctx.caster,
        key: ULT_KEY.to_string(),
        value: evt.hop_index as i64 + 1,
        cast_id: evt.cast_id,
    });
}

fn cascade_seen_hook(
    evt: &bevyrogue::combat::runtime::BeatEvent,
    ctx: &mut bevyrogue::combat::runtime::SkillCtx<'_>,
) {
    ctx.enqueue(Intent::SetBlueprintState {
        actor: ctx.caster,
        key: TRACE_KEY.to_string(),
        value: evt.hop_index as i64 + 2,
        cast_id: evt.cast_id,
    });
}

fn loop_tick_hook(
    evt: &bevyrogue::combat::runtime::BeatEvent,
    ctx: &mut bevyrogue::combat::runtime::SkillCtx<'_>,
) {
    use std::sync::atomic::{AtomicU32, Ordering};

    static LOOP_CALLS: AtomicU32 = AtomicU32::new(0);
    let count = LOOP_CALLS.fetch_add(1, Ordering::Relaxed) + 1;

    ctx.enqueue(Intent::SetBlueprintState {
        actor: ctx.caster,
        key: LOOP_KEY.to_string(),
        value: count as i64,
        cast_id: evt.cast_id,
    });
}

fn never_exit(
    _evt: &bevyrogue::combat::runtime::BeatEvent,
    _ctx: &bevyrogue::combat::runtime::SkillCtx<'_>,
) -> bool {
    false
}

fn write_ultimate_used(app: &mut App) {
    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::UltimateUsed { unit_id: TARGET },
        source: TARGET,
        target: TARGET,
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
}

#[test]
fn composite_matching_same_frame_cascade_and_loop_breaker_all_hold_together() {
    let mut app = passive_dispatch_app();
    register_hooks(&mut app);
    register_passives(&mut app);

    write_ultimate_used(&mut app);
    app.update();

    let state = app.world().resource::<BlueprintState>();
    assert_eq!(state.map.get(&(OWNER, TRACE_KEY.to_string())), Some(&2));
    assert_eq!(state.map.get(&(OWNER, ULT_KEY.to_string())), Some(&1));
    assert_eq!(state.map.get(&(OWNER, LOOP_KEY.to_string())), Some(&256));

    let drained: Vec<_> = app
        .world_mut()
        .resource_mut::<SignalBus>()
        .drain()
        .collect();
    assert!(
        drained.is_empty(),
        "SignalBus should be empty after passive dispatch, got: {:?}",
        drained
    );
}
