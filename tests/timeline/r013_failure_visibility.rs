//! R013 — Failure-visibility coverage for the timeline pipeline.
//!
//! Three failure modes that must surface diagnostically (not panic, not silently
//! advance) so the next agent debugging a windowed session at 3am has a signal:
//!
//! 1. Cue-never-released → `SuspendedTimelineState` keeps the latched state
//!    visible (`active_status()` stays `Some`, phase stays `Resolving`) across
//!    arbitrarily many frames, instead of advancing without damage. Failure
//!    visible via the standard observability surface.
//! 2. Degenerate-instant-graph (empty `BeatKind::Loop` body) → rejected at
//!    `compile_skill_book_timelines` time with a typed
//!    `SkillTimelineCompileError` pointing at the offending beat. Strict failure
//!    visible at boot.
//! 3. Target dies mid multi-hit loop → `UnitDied` is emitted on the killing hop
//!    and the runner keeps iterating until the circuit-breaker trips, with the
//!    overshoot observable as additional `OnDamageDealt` events whose target
//!    HP was already ≤0. Documents the "loop continued past death" semantics so
//!    a future skill author can detect the case from the event log.

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevyrogue::combat::{
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    runtime::{
        CastIdGen, Clock, ExtRegistries, Intent, IntentQueue, SuspendedTimelineState,
        TimelineClock, register_kernel_builtins,
        runner::{BeatRunner, StepOutcome},
        skill_ctx::{SkillCtx, SkillCtxMode},
        timeline::{
            Beat, BeatEdge, BeatEvent, BeatKind, BeatPayload, CompiledTimeline, SelectorCtx,
            TimelineLibrary,
        },
    },
    sp::SpPool,
    state::{CombatPhase, CombatState},
    status_effect::StatusBag,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{
        ActionIntent, apply_av_ops_system, continue_suspended_timeline_system,
        resolve_action_system,
    },
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skill_timeline::{SkillTimeline, compile_skill_book_timelines},
    skills_ron::{
        SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting, TargetLife,
        TargetShape, TargetSide,
    },
};
use crate::common::app::minimal_intent_app;
use std::collections::VecDeque;
use std::sync::Arc;

// ────────────────────────────────────────────────────────────────────────────
// Test 1 — cue-never-released: suspension state stays observable.
// ────────────────────────────────────────────────────────────────────────────

const T1_CASTER: UnitId = UnitId(1);
const T1_TARGET: UnitId = UnitId(2);
const T1_CUE: &str = "r013/never_release";

fn t1_skill() -> SkillDef {
    SkillDef {
        id: SkillId("r013_never_release".into()),
        name: "r013_never_release".into(),
        damage_tag: DamageTag::Physical,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![],
        timeline: Some(SkillTimeline {
            entry: "cast".into(),
            beats: vec![
                Beat {
                    id: "cast".into(),
                    kind: BeatKind::Cast,
                    hook: None,
                    selector: None,
                    presentation: None,
                    payload: None,
                },
                Beat {
                    id: "impact".into(),
                    kind: BeatKind::Impact,
                    hook: Some("core/deal_damage".into()),
                    selector: Some("core/primary".into()),
                    presentation: Some(bevyrogue::combat::runtime::Presentation {
                        cue_id: T1_CUE.into(),
                        anim: None,
                        vfx: None,
                        sfx: None,
                    }),
                    payload: Some(BeatPayload::DealDamage {
                        amount: 17,
                        tag: DamageTag::Physical,
                        target: TargetShape::Single,
                    }),
                },
            ],
            edges: vec![BeatEdge {
                from: "cast".into(),
                to: "impact".into(),
                gate: Some("core/always".into()),
            }],
        }),
        ..Default::default()
    }
}

fn t1_build_app() -> App {
    let book = SkillBook(vec![t1_skill()]);
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .insert_resource(TimelineClock(Clock::Windowed))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SuspendedTimelineState>()
        .insert_resource(SpPool {
            current: 99,
            max: 99,
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(13))
        .insert_resource(TimelineLibrary::<String>::default())
        .init_resource::<ExtRegistries>()
        .add_message::<ActionIntent>()
        .add_message::<TurnAdvanced>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(
            Update,
            (
                resolve_action_system,
                continue_suspended_timeline_system,
                apply_av_ops_system,
            )
                .chain(),
        );
    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        register_kernel_builtins(&mut regs);
        let compiled = compile_skill_book_timelines(&book, &regs)
            .expect("r013 skill book must compile");
        app.world_mut()
            .resource_mut::<TimelineLibrary<String>>()
            .timelines = compiled;
    }

    let basic = SkillId("r013_never_release".into());
    app.world_mut().spawn((
        Unit {
            id: T1_CASTER,
            name: "Caster".into(),
            hp_max: 500,
            hp_current: 500,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        UnitSkills {
            basic: basic.clone(),
            skills: vec![basic.clone()],
            ultimate: basic,
            follow_up: None,
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 10,
        },
        Toughness::new(50, vec![]),
        StatusBag::default(),
    ));
    app.world_mut().spawn((
        Unit {
            id: T1_TARGET,
            name: "Target".into(),
            hp_max: 200,
            hp_current: 200,
            attribute: Attribute::Data,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        ActionValue(MAX_AV),
        Toughness::new(20, vec![]),
        StatusBag::default(),
    ));

    app
}

fn t1_target_hp(app: &mut App) -> i32 {
    let mut q = app.world_mut().query::<&Unit>();
    q.iter(app.world())
        .find(|u| u.id == T1_TARGET)
        .expect("target missing")
        .hp_current
}

#[test]
fn cue_never_released_keeps_suspension_visible_across_frames() {
    let mut app = t1_build_app();

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: T1_CASTER,
        target: T1_TARGET,
    });
    app.update();

    // The barrier must latch on the impact cue.
    let active = app
        .world()
        .resource::<SuspendedTimelineState>()
        .active_status()
        .expect("windowed action should suspend on the impact cue");
    assert_eq!(active.cue_id, T1_CUE);
    assert_eq!(active.beat_id, "impact");
    assert!(active.awaiting_release);
    assert!(!active.released);
    assert_eq!(t1_target_hp(&mut app), 200);
    assert_eq!(
        app.world().resource::<CombatState>().phase,
        CombatPhase::Resolving
    );

    // Tick many frames without ever calling `request_timeline_cue_release`.
    // The kernel must NOT silently advance, NOT panic, NOT timeout — the
    // observable surface must stay populated so an operator can see the stall.
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();
    for _ in 0..200 {
        app.update();
        let frame_events: Vec<CombatEvent> = cursor
            .read(app.world().resource::<Messages<CombatEvent>>())
            .cloned()
            .collect();
        let damage_count = frame_events
            .iter()
            .filter(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
            .count();
        assert_eq!(
            damage_count, 0,
            "damage must not land while the cue is never released"
        );
    }

    let still_active = app
        .world()
        .resource::<SuspendedTimelineState>()
        .active_status()
        .expect("suspension must remain observable after 200 idle frames");
    assert_eq!(still_active.cue_id, T1_CUE);
    assert_eq!(still_active.beat_id, "impact");
    assert!(still_active.awaiting_release);
    assert!(!still_active.released);
    assert_eq!(t1_target_hp(&mut app), 200);
    assert_eq!(
        app.world().resource::<CombatState>().phase,
        CombatPhase::Resolving,
        "phase must stay Resolving so the failure is visible to upstream observers"
    );
}

// ────────────────────────────────────────────────────────────────────────────
// Test 2 — degenerate-instant-graph: empty loop body rejected at compile time.
// ────────────────────────────────────────────────────────────────────────────

fn t2_skill_with_empty_loop() -> SkillDef {
    SkillDef {
        id: SkillId("r013_empty_loop".into()),
        name: "r013_empty_loop".into(),
        damage_tag: DamageTag::Physical,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![],
        timeline: Some(SkillTimeline {
            entry: "cast".into(),
            beats: vec![
                Beat {
                    id: "cast".into(),
                    kind: BeatKind::Cast,
                    hook: None,
                    selector: None,
                    presentation: None,
                    payload: None,
                },
                Beat {
                    id: "degenerate_loop".into(),
                    kind: BeatKind::Loop {
                        body: vec![], // empty body — degenerate instant graph
                        exit_when: "core/always".into(),
                    },
                    hook: None,
                    selector: None,
                    presentation: None,
                    payload: None,
                },
            ],
            edges: vec![BeatEdge {
                from: "cast".into(),
                to: "degenerate_loop".into(),
                gate: Some("core/always".into()),
            }],
        }),
        ..Default::default()
    }
}

#[test]
fn degenerate_instant_graph_empty_loop_body_rejected_at_compile() {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);

    let book = SkillBook(vec![t2_skill_with_empty_loop()]);
    let err = compile_skill_book_timelines(&book, &regs)
        .expect_err("empty loop body must be rejected before runtime");

    assert_eq!(err.skill_id, SkillId("r013_empty_loop".into()));
    assert_eq!(err.site, "beat degenerate_loop");
    assert!(
        err.detail.contains("at least one"),
        "diagnostic must point at the empty body: {}",
        err.detail
    );
}

// ────────────────────────────────────────────────────────────────────────────
// Test 3 — target dies mid multi-hit loop: UnitDied + observable overshoot.
// ────────────────────────────────────────────────────────────────────────────

const T3_HOP_DAMAGE: i32 = 1;
const T3_TARGET_HP: i32 = 3;

fn t3_target_selector(sctx: &SelectorCtx<'_>) -> Vec<UnitId> {
    vec![sctx.primary_target]
}

fn t3_one_damage_per_hop(ev: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    let target = ev
        .beat_targets
        .first()
        .copied()
        .unwrap_or(ctx.primary_target);
    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target,
        amount: T3_HOP_DAMAGE,
        tag: DamageTag::Physical,
        cast_id: ctx.cast_id,
    });
}

fn t3_never(_ev: &BeatEvent, _ctx: &SkillCtx<'_>) -> bool {
    false
}

fn t3_spawn_unit(app: &mut App, id: UnitId, team: Team, hp: i32) {
    app.world_mut().spawn((
        Unit {
            id,
            name: format!("u{}", id.0),
            hp_max: hp.max(1),
            hp_current: hp,
            attribute: Attribute::Data,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        team,
        StatusBag::default(),
        Toughness::new(50, vec![]),
    ));
}

#[test]
fn target_dead_mid_loop_emits_unit_died_and_observable_overshoot() {
    let mut app = minimal_intent_app();

    let caster = UnitId(101);
    let target = UnitId(102);
    t3_spawn_unit(&mut app, caster, Team::Ally, 500);
    t3_spawn_unit(&mut app, target, Team::Enemy, T3_TARGET_HP);

    let cast_id = app.world_mut().resource_mut::<CastIdGen>().next();

    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    regs.selectors.register("r013/target", t3_target_selector);
    regs.hooks
        .register("r013/one_damage_per_hop", t3_one_damage_per_hop);
    regs.predicates.register("r013/never", t3_never);

    let timeline = Arc::new(CompiledTimeline {
        id: "r013_target_dead_loop",
        entry: "loop_root",
        beats: vec![Beat {
            id: "loop_root",
            kind: BeatKind::Loop {
                body: vec![Beat {
                    id: "hit",
                    kind: BeatKind::Impact,
                    hook: Some("r013/one_damage_per_hop"),
                    selector: Some("r013/target"),
                    presentation: None,
                    payload: None,
                }],
                exit_when: "r013/never",
            },
            hook: None,
            selector: None,
            presentation: None,
            payload: None,
        }],
        edges: vec![],
    });

    let mut runner = BeatRunner::new(Arc::clone(&timeline), cast_id, caster, target);
    let mut pending: VecDeque<Intent> = VecDeque::new();
    let outcome = runner.run_to_completion(
        app.world_mut(),
        &regs,
        SkillCtxMode::Execute,
        &mut pending,
        1000,
    );
    assert_eq!(
        outcome,
        StepOutcome::Halted,
        "loop with `never` exit must halt at MAX_HOPS, not panic"
    );

    let queued_damage = pending
        .iter()
        .filter(|i| matches!(i, Intent::DealDamage { .. }))
        .count();
    assert!(
        queued_damage > T3_TARGET_HP as usize,
        "loop must keep producing intents past the killing hop (observable overshoot): \
         queued={queued_damage}, target_hp={T3_TARGET_HP}"
    );

    // Drain through the real applier so the death event is visible.
    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .extend(pending);
    app.update();

    let messages = app.world().resource::<Messages<CombatEvent>>();
    let mut cursor = messages.get_cursor();
    let events: Vec<CombatEvent> = cursor.read(messages).cloned().collect();

    // (a) UnitDied is emitted at least once — the death is visible in the event
    // stream rather than being silently swallowed. The current applier re-emits
    // UnitDied on every post-death hit; the contract guaranteed by this test is
    // "death is observable", not a specific dedup count. The repeated UnitDied
    // events are themselves a visible signature of "loop continued past death".
    let died_count = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::UnitDied { .. }))
        .count();
    assert!(
        died_count >= 1,
        "UnitDied must be visible in the event stream when target HP crosses zero, got {}",
        died_count
    );

    // (b) The applier keeps emitting OnDamageDealt past the killing hop because
    // the runner has no world-HP introspection inside the loop body. This is the
    // observable surface of R013's "target-dead-mid-loop" failure mode: a future
    // skill author seeing damage_events > target_hp in the JSONL stream knows
    // the loop overshot the death. The contract is "the overshoot is visible",
    // not "the applier gates on alive".
    let damage_events = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .count();
    assert!(
        damage_events > T3_TARGET_HP as usize,
        "loop must overshoot the killing hop so the failure is observable: \
         damage_events={damage_events}, target_hp={T3_TARGET_HP}"
    );

    // (c) Sanity: target really is at or below zero HP after the run.
    let mut q = app.world_mut().query::<&Unit>();
    let final_hp = q
        .iter(app.world())
        .find(|u| u.id == target)
        .expect("target unit missing")
        .hp_current;
    assert!(
        final_hp <= 0,
        "target should be dead after multi-hit loop, hp_current={final_hp}"
    );
}
