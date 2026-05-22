//! Regression: a timeline-backed Basic attack must grant +1 SP, matching the
//! legacy executor (`apply_legacy_ops`: UltEffect::GainFromBasic => sp.gain(1)).
//!
//! The bug: `finalize_timeline_action` charged the ult gauge on GainFromBasic
//! but skipped the SP gain, so any Digimon whose basic carries a `timeline`
//! (e.g. Agumon) silently failed to accrue SP on basics. Legacy-path basics
//! were unaffected and already covered by `combat_resolution_apply.rs`.
//!
//! This drives the real `resolve_action_system` -> `run_timeline_backed_action`
//! pipeline end-to-end, so it exercises `finalize_timeline_action` directly.

use bevy::prelude::*;
use bevyrogue::combat::{
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    events::CombatEvent,
    kit::UnitSkills,
    rng::CombatRng,
    runtime::{
        Clock, ExtRegistries, SuspendedTimelineState, TimelineClock, register_kernel_builtins,
        timeline::{Beat, BeatEdge, BeatKind, BeatPayload, TimelineLibrary},
    },
    sp::SpPool,
    state::CombatState,
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
const BASIC_ID: &str = "timeline_basic_sp";
const DAMAGE_AMOUNT: i32 = 17;
const CHARGE_PER_EVENT: i32 = 10;

/// A minimal timeline-backed Basic: Cast -> Impact (deal damage), no Loop, so
/// the runner reaches `StepOutcome::Done` in one frame and `finalize` runs.
fn timeline_basic_skill() -> SkillDef {
    SkillDef {
        id: SkillId(BASIC_ID.into()),
        name: BASIC_ID.into(),
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
                    presentation: None,
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

fn build_app(book: SkillBook, start_sp: i32) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .insert_resource(TimelineClock(Clock::HeadlessAuto))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SuspendedTimelineState>()
        .insert_resource(SpPool {
            current: start_sp,
            max: 5,
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
        let compiled = compile_skill_book_timelines(&book, &regs)
            .expect("timeline-backed basic must compile");
        app.world_mut()
            .resource_mut::<TimelineLibrary<String>>()
            .timelines = compiled;
    }

    app
}

fn spawn_actor(app: &mut App) {
    let basic = SkillId(BASIC_ID.into());
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
            charge_per_event: CHARGE_PER_EVENT,
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

fn ult_current(app: &mut App) -> i32 {
    let mut q = app.world_mut().query::<(&Unit, &UltimateCharge)>();
    q.iter(app.world())
        .find(|(u, _)| u.id == CASTER_ID)
        .map(|(_, c)| c.current)
        .expect("caster ult charge missing")
}

use bevyrogue::combat::log::ActionLog;

#[test]
fn timeline_backed_basic_grants_one_sp() {
    let book = SkillBook(vec![timeline_basic_skill()]);
    let mut app = build_app(book, 1);
    spawn_actor(&mut app);
    spawn_target(&mut app);

    assert_eq!(app.world().resource::<SpPool>().current, 1, "precondition");

    fire_basic(&mut app);

    // The basic resolved through the timeline executor (ult gauge charged)...
    assert_eq!(
        ult_current(&mut app),
        CHARGE_PER_EVENT,
        "basic must charge the ult gauge, proving it resolved via finalize"
    );
    // ...and SP must have incremented by exactly 1 — the regression.
    assert_eq!(
        app.world().resource::<SpPool>().current,
        2,
        "timeline-backed basic must grant +1 SP (parity with legacy apply_legacy_ops)"
    );
}

#[test]
fn timeline_backed_basic_sp_gain_clamps_at_max() {
    let book = SkillBook(vec![timeline_basic_skill()]);
    let mut app = build_app(book, 5); // already at SpPool.max
    spawn_actor(&mut app);
    spawn_target(&mut app);

    fire_basic(&mut app);

    assert_eq!(
        app.world().resource::<SpPool>().current,
        5,
        "SP gain from basic must clamp at SpPool.max (no overflow past 5)"
    );
}
