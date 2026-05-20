use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use bevyrogue::combat::{
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::{ActionLog, LogEntry},
    rng::CombatRng,
    runtime::{
        Clock, CueReleaseResult, ExtRegistries, SuspendedTimelineState, TimelineClock,
        register_kernel_builtins, request_timeline_cue_release,
        timeline::{Beat, BeatEdge, BeatKind, BeatPayload, TimelineLibrary},
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

const CASTER_ID: UnitId = UnitId(1);
const TARGET_ID: UnitId = UnitId(2);
const DAMAGE_AMOUNT: i32 = 17;
const IMPACT_CUE: &str = "demo/basic/impact";
const HALT_CUE: &str = "demo/basic/halt";

fn barrier_basic_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
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
                        cue_id: IMPACT_CUE.into(),
                        anim: Some("sharp_claws_strike".into()),
                        vfx: None,
                        sfx: None,
                    }),
                    payload: Some(BeatPayload::DealDamage {
                        amount: DAMAGE_AMOUNT,
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

fn halted_after_release_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
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
                    presentation: Some(bevyrogue::combat::runtime::Presentation {
                        cue_id: HALT_CUE.into(),
                        anim: Some("sharp_claws_strike".into()),
                        vfx: None,
                        sfx: None,
                    }),
                    payload: None,
                },
                Beat {
                    id: "stall_loop".into(),
                    kind: BeatKind::Loop {
                        body: vec![Beat {
                            id: "loop_phase".into(),
                            kind: BeatKind::Phase,
                            hook: None,
                            selector: None,
                            presentation: None,
                            payload: None,
                        }],
                        exit_when: "test/never".into(),
                    },
                    hook: None,
                    selector: None,
                    presentation: None,
                    payload: None,
                },
            ],
            edges: vec![BeatEdge {
                from: "cast".into(),
                to: "stall_loop".into(),
                gate: Some("core/always".into()),
            }],
        }),
        ..Default::default()
    }
}

fn build_app(book: SkillBook, clock: Clock, add_never_predicate: bool) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .insert_resource(TimelineClock(clock))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SuspendedTimelineState>()
        .insert_resource(SpPool {
            current: 99,
            max: 99,
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(7))
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
        if add_never_predicate {
            regs.predicates.register("test/never", |_ev, _ctx| false);
        }
        let compiled = compile_skill_book_timelines(&book, &regs)
            .expect("timeline-backed cue barrier test book must compile");
        app.world_mut()
            .resource_mut::<TimelineLibrary<String>>()
            .timelines = compiled;
    }

    app
}

fn spawn_actor(app: &mut App, basic_skill: SkillId) {
    app.world_mut().spawn((
        Unit {
            id: CASTER_ID,
            name: "Caster".into(),
            hp_max: 500,
            hp_current: 500,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        UnitSkills {
            basic: basic_skill.clone(),
            skills: vec![basic_skill.clone()],
            ultimate: basic_skill,
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
}

fn spawn_target(app: &mut App) {
    app.world_mut().spawn((
        Unit {
            id: TARGET_ID,
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
}

fn fire_basic(app: &mut App) {
    app.world_mut().write_message(ActionIntent::Basic {
        attacker: CASTER_ID,
        target: TARGET_ID,
    });
    app.update();
}

fn collect_events(app: &App, cursor: &mut MessageCursor<CombatEvent>) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn target_hp(app: &mut App) -> i32 {
    let mut q = app.world_mut().query::<&Unit>();
    q.iter(app.world())
        .find(|unit| unit.id == TARGET_ID)
        .expect("target unit missing")
        .hp_current
}

fn normalized_event_kinds(events: &[CombatEvent]) -> Vec<String> {
    events.iter().map(|event| format!("{:?}", event.kind)).collect()
}

fn expected_barrier_basic_final_hp() -> i32 {
    let book = SkillBook(vec![barrier_basic_skill("barrier_basic")]);
    let mut app = build_app(book, Clock::HeadlessAuto, false);
    spawn_actor(&mut app, SkillId("barrier_basic".into()));
    spawn_target(&mut app);
    fire_basic(&mut app);
    target_hp(&mut app)
}

fn damage_event_count(events: &[CombatEvent]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event.kind, CombatEventKind::OnDamageDealt { .. }))
        .count()
}

#[test]
fn windowed_basic_action_suspends_until_release_then_matches_headless() {
    let book = SkillBook(vec![barrier_basic_skill("barrier_basic")]);

    let mut headless = build_app(book.clone(), Clock::HeadlessAuto, false);
    spawn_actor(&mut headless, SkillId("barrier_basic".into()));
    spawn_target(&mut headless);
    let mut headless_cursor = headless
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_basic(&mut headless);
    let headless_events = collect_events(&headless, &mut headless_cursor);
    let expected_hp = target_hp(&mut headless);

    let mut windowed = build_app(book, Clock::Windowed, false);
    spawn_actor(&mut windowed, SkillId("barrier_basic".into()));
    spawn_target(&mut windowed);
    let mut windowed_cursor = windowed
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();

    fire_basic(&mut windowed);

    let first_frame_events = collect_events(&windowed, &mut windowed_cursor);
    assert_eq!(target_hp(&mut windowed), 200, "damage must not land before cue release");
    assert_eq!(damage_event_count(&first_frame_events), 0);
    assert!(
        first_frame_events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionDeclared { .. })),
        "declaration should still happen before the barrier"
    );
    assert!(
        first_frame_events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionPreApp)),
        "pre-app seam should still be emitted before the barrier"
    );
    assert!(
        !first_frame_events
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnSkillCast { .. })),
        "skill cast must wait until the suspended timeline resumes"
    );
    assert!(
        windowed
            .world()
            .resource::<ActionLog>()
            .events
            .iter()
            .all(|entry| !matches!(entry, LogEntry::BasicHit { .. })),
        "ActionLog must not show damage while the barrier is awaiting release"
    );
    let barrier = windowed.world().resource::<SuspendedTimelineState>();
    let active = barrier
        .active_status()
        .expect("windowed action should suspend on the impact cue");
    assert_eq!(active.cue_id, IMPACT_CUE);
    assert_eq!(active.beat_id, "impact");
    assert_eq!(windowed.world().resource::<CombatState>().phase, CombatPhase::Resolving);

    assert_eq!(
        request_timeline_cue_release(windowed.world_mut(), IMPACT_CUE),
        CueReleaseResult::Released
    );
    assert_eq!(
        windowed
            .world()
            .resource::<SuspendedTimelineState>()
            .last_release_result(),
        Some(CueReleaseResult::Released)
    );

    windowed.update();
    let resumed_events = collect_events(&windowed, &mut windowed_cursor);
    assert_eq!(target_hp(&mut windowed), expected_hp);
    assert_eq!(damage_event_count(&resumed_events), 1);
    assert!(
        windowed
            .world()
            .resource::<SuspendedTimelineState>()
            .active_status()
            .is_none(),
        "suspension should clear after resume reaches Done"
    );
    assert_eq!(
        windowed.world().resource::<CombatState>().phase,
        CombatPhase::WaitingAction
    );

    let mut total_windowed = first_frame_events;
    total_windowed.extend(resumed_events);
    assert_eq!(
        normalized_event_kinds(&total_windowed),
        normalized_event_kinds(&headless_events),
        "windowed and headless should converge to the same final combat event stream"
    );

    windowed.update();
    let after_done_events = collect_events(&windowed, &mut windowed_cursor);
    assert!(after_done_events.is_empty(), "resume must not replay damage on later frames");
    assert_eq!(target_hp(&mut windowed), expected_hp);
}

#[test]
fn release_before_suspension_is_a_diagnostic_no_op() {
    let book = SkillBook(vec![barrier_basic_skill("barrier_basic")]);
    let mut app = build_app(book, Clock::Windowed, false);
    spawn_actor(&mut app, SkillId("barrier_basic".into()));
    spawn_target(&mut app);

    assert_eq!(
        request_timeline_cue_release(app.world_mut(), IMPACT_CUE),
        CueReleaseResult::NoSuspendedTimeline
    );
    let barrier = app.world().resource::<SuspendedTimelineState>();
    assert_eq!(
        barrier.last_release_result(),
        Some(CueReleaseResult::NoSuspendedTimeline)
    );
    assert!(
        barrier
            .last_message()
            .expect("no-op release should leave a diagnostic")
            .contains("no suspended timeline")
    );

    fire_basic(&mut app);
    let active = app
        .world()
        .resource::<SuspendedTimelineState>()
        .active_status()
        .expect("the later action should still suspend normally");
    assert_eq!(active.cue_id, IMPACT_CUE);
}

#[test]
fn duplicate_release_is_no_op_and_resume_only_applies_damage_once() {
    let expected_hp = expected_barrier_basic_final_hp();
    let book = SkillBook(vec![barrier_basic_skill("barrier_basic")]);
    let mut app = build_app(book, Clock::Windowed, false);
    spawn_actor(&mut app, SkillId("barrier_basic".into()));
    spawn_target(&mut app);
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();

    fire_basic(&mut app);
    let _ = collect_events(&app, &mut cursor);

    assert_eq!(
        request_timeline_cue_release(app.world_mut(), IMPACT_CUE),
        CueReleaseResult::Released
    );
    assert_eq!(
        request_timeline_cue_release(app.world_mut(), IMPACT_CUE),
        CueReleaseResult::DuplicateRelease
    );

    app.update();
    let resumed = collect_events(&app, &mut cursor);
    assert_eq!(damage_event_count(&resumed), 1);
    assert_eq!(target_hp(&mut app), expected_hp);

    assert_eq!(
        request_timeline_cue_release(app.world_mut(), IMPACT_CUE),
        CueReleaseResult::NoSuspendedTimeline
    );
    app.update();
    let after = collect_events(&app, &mut cursor);
    assert_eq!(damage_event_count(&after), 0);
    assert_eq!(target_hp(&mut app), expected_hp);
}

#[test]
fn action_spam_is_ignored_while_barrier_keeps_phase_resolving() {
    let expected_hp = expected_barrier_basic_final_hp();
    let book = SkillBook(vec![barrier_basic_skill("barrier_basic")]);
    let mut app = build_app(book, Clock::Windowed, false);
    spawn_actor(&mut app, SkillId("barrier_basic".into()));
    spawn_target(&mut app);
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();

    fire_basic(&mut app);
    let _ = collect_events(&app, &mut cursor);
    assert_eq!(app.world().resource::<CombatState>().phase, CombatPhase::Resolving);

    for _ in 0..10 {
        app.world_mut().write_message(ActionIntent::Basic {
            attacker: CASTER_ID,
            target: TARGET_ID,
        });
    }
    app.update();
    let spam_frame = collect_events(&app, &mut cursor);
    assert_eq!(damage_event_count(&spam_frame), 0);
    assert_eq!(target_hp(&mut app), 200);
    assert!(
        app.world()
            .resource::<SuspendedTimelineState>()
            .active_status()
            .is_some(),
        "spam must not replace or stack suspended runners"
    );

    assert_eq!(
        request_timeline_cue_release(app.world_mut(), IMPACT_CUE),
        CueReleaseResult::Released
    );
    app.update();
    let resolved = collect_events(&app, &mut cursor);
    assert_eq!(damage_event_count(&resolved), 1);
    assert_eq!(target_hp(&mut app), expected_hp);
    assert_eq!(
        resolved
            .iter()
            .filter(|event| matches!(event.kind, CombatEventKind::OnActionResolved))
            .count(),
        1,
        "only the original action should resolve after the queued spam is ignored"
    );
}

#[test]
fn halted_resume_emits_failure_and_clears_suspension_without_mutating_world() {
    let book = SkillBook(vec![halted_after_release_skill("halt_basic")]);
    let mut app = build_app(book, Clock::Windowed, true);
    spawn_actor(&mut app, SkillId("halt_basic".into()));
    spawn_target(&mut app);
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();

    fire_basic(&mut app);
    let first_frame = collect_events(&app, &mut cursor);
    assert_eq!(damage_event_count(&first_frame), 0);
    assert!(
        app.world()
            .resource::<SuspendedTimelineState>()
            .active_status()
            .is_some(),
        "halt test should suspend before entering the looping body"
    );

    assert_eq!(
        request_timeline_cue_release(app.world_mut(), HALT_CUE),
        CueReleaseResult::Released
    );
    app.update();

    let resumed = collect_events(&app, &mut cursor);
    assert!(
        resumed
            .iter()
            .any(|event| matches!(event.kind, CombatEventKind::OnActionFailed { .. })),
        "halted runner should surface OnActionFailed after release"
    );
    assert_eq!(damage_event_count(&resumed), 0);
    assert_eq!(target_hp(&mut app), 200);
    assert!(
        app.world()
            .resource::<SuspendedTimelineState>()
            .active_status()
            .is_none(),
        "halted resume must clear the suspended runner"
    );
    assert_eq!(app.world().resource::<CombatState>().phase, CombatPhase::WaitingAction);
}
