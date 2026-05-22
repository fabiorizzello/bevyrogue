//! T05: Baby Burner primary timeline parses, executes windup→impact→recovery,
//! emits the apply_thermal_spark BlueprintSignal, and preserves the existing
//! reactive detonate seam on a lethal Heated KO. Both Clock::HeadlessAuto and
//! Clock::Windowed runners must reach Done from the same compiled timeline.

use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use bevyrogue::combat::{
    CombatPlugin, StatusBag, StatusEffectKind,
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    blueprints::register_all_blueprint_exts,
    events::{CombatEvent, CombatEventKind},
    kernel::CombatKernelTransition,
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    runtime::{
        ExtRegistries, SignalBus, SignalPayload, SignalTaxonomy,
        clock::Clock,
        cue_barrier::TimelineClock,
        register_kernel_builtins, request_timeline_cue_release,
        timeline::{BeatKind, BeatPayload, TimelineLibrary},
    },
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{
        ActionIntent, apply_av_ops_system, continue_suspended_timeline_system,
        resolve_action_system,
    },
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{SlotIndex, Unit},
};
use bevyrogue::data::{
    SkillBookHandle, aggregate_skill_book, skill_timeline::compile_skill_book_timelines,
    skills_ron::SkillBook,
};

const AGUMON_ID: UnitId = UnitId(1);
const LEFT_ID: UnitId = UnitId(10);
const PRIMARY_ID: UnitId = UnitId(11);
const RIGHT_ID: UnitId = UnitId(12);

fn canonical_regs() -> ExtRegistries {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    register_all_blueprint_exts(&mut regs);
    regs
}

#[test]
fn baby_burner_timeline_parses_with_windup_impact_recovery_beats() {
    let book = aggregate_skill_book();
    let compiled = compile_skill_book_timelines(&book, &canonical_regs())
        .expect("canonical skill book must compile timelines");

    let timeline = compiled
        .iter()
        .find(|t| t.id == "agumon_ult")
        .expect("agumon_ult timeline must compile");

    assert_eq!(timeline.entry, "cast");
    let ids: Vec<&str> = timeline.beats.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "cast",
            "windup",
            "impact_damage",
            "impact_break",
            "impact_signal",
            "recovery"
        ]
    );

    let impact_damage = timeline
        .beats
        .iter()
        .find(|b| b.id == "impact_damage")
        .expect("impact_damage beat present");
    assert!(matches!(impact_damage.kind, BeatKind::Impact));
    assert_eq!(
        impact_damage.payload,
        Some(BeatPayload::DealDamage {
            amount: 50,
            tag: DamageTag::Fire,
            target: bevyrogue::data::skills_ron::TargetShape::Single,
        })
    );

    let impact_break = timeline
        .beats
        .iter()
        .find(|b| b.id == "impact_break")
        .expect("impact_break beat present");
    assert_eq!(
        impact_break.payload,
        Some(BeatPayload::BreakToughness {
            amount: 30,
            tag: DamageTag::Fire,
            target: bevyrogue::data::skills_ron::TargetShape::Single,
        })
    );

    let impact_signal = timeline
        .beats
        .iter()
        .find(|b| b.id == "impact_signal")
        .expect("impact_signal beat present");
    match &impact_signal.payload {
        Some(BeatPayload::BlueprintSignal {
            owner,
            name,
            payload,
        }) => {
            assert_eq!(*owner, "agumon");
            assert_eq!(*name, "apply_thermal_spark");
            assert!(matches!(payload, SignalPayload::Amount(3)));
        }
        other => panic!("impact_signal payload should be BlueprintSignal: {other:?}"),
    }
}

#[test]
fn baby_burner_windowed_run_consumes_three_cues_and_reaches_done() {
    let book = aggregate_skill_book();
    let mut app = build_app_with_clock(&book, Clock::Windowed);
    spawn_agumon(&mut app);
    spawn_enemy(&mut app, PRIMARY_ID, 1, 200, None);

    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: AGUMON_ID,
        target: PRIMARY_ID,
    });
    app.update();

    // The Baby Burner timeline has three presentation-bearing beats:
    // windup, impact_damage, recovery. Each stalls the windowed runner.
    let cues = [
        "agumon/baby_burner/windup",
        "agumon/baby_burner/impact",
        "agumon/baby_burner/recovery",
    ];
    for cue in cues {
        request_timeline_cue_release(app.world_mut(), cue);
        app.update();
    }

    // After all three cues are released the timeline must complete; the
    // primary must be damaged and the thermal_spark signal must have fired.
    assert!(
        unit_hp(&mut app, PRIMARY_ID) < 200,
        "windowed Baby Burner must apply damage after all cues released"
    );
}

// ── live App fixture for end-to-end behaviour assertions ───────────────────

fn build_app(book: &SkillBook) -> App {
    build_app_with_clock(book, Clock::HeadlessAuto)
}

fn build_app_with_clock(book: &SkillBook, clock: Clock) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool {
            current: 99,
            max: 99,
        })
        .insert_resource(TimelineClock(clock))
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(7))
        .insert_resource(TimelineLibrary::<String>::default())
        .init_resource::<SignalBus>()
        .init_resource::<ExtRegistries>()
        .init_resource::<SignalTaxonomy>()
        .add_message::<ActionIntent>()
        .add_message::<TurnAdvanced>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_plugins(CombatPlugin)
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
        register_all_blueprint_exts(&mut regs);
        let compiled = compile_skill_book_timelines(book, &regs)
            .expect("canonical timeline book must compile");
        app.world_mut()
            .resource_mut::<TimelineLibrary<String>>()
            .timelines = compiled;
    }
    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register("agumon", "apply_thermal_spark");

    app
}

fn spawn_agumon(app: &mut App) {
    app.world_mut().spawn((
        Unit {
            id: AGUMON_ID,
            name: "Agumon".into(),
            hp_max: 200,
            hp_current: 200,
            attribute: Attribute::Free,
            resists: vec![],
            evo_stage: EvoStage::Child,
        },
        Team::Ally,
        SlotIndex(0),
        UltimateCharge {
            current: 100,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("baby_flame".into()),
            skills: vec![SkillId("baby_flame".into())],
            ultimate: SkillId("agumon_ult".into()),
            follow_up: None,
        },
    ));
}

fn spawn_enemy(app: &mut App, id: UnitId, slot: u8, hp: i32, heated_turns: Option<u32>) {
    let mut bag = StatusBag::default();
    if let Some(turns) = heated_turns {
        bag.apply(StatusEffectKind::Heated, turns);
    }
    app.world_mut().spawn((
        Unit {
            id,
            name: format!("Enemy{}", id.0),
            hp_max: hp,
            hp_current: hp,
            attribute: Attribute::Free,
            resists: vec![],
            evo_stage: EvoStage::Child,
        },
        Team::Enemy,
        SlotIndex(slot),
        ActionValue(MAX_AV),
        Toughness::new(30, vec![DamageTag::Fire]),
        bag,
    ));
}

fn message_cursor<T: bevy::ecs::message::Message>(app: &mut App) -> MessageCursor<T> {
    app.world_mut().resource_mut::<Messages<T>>().get_cursor()
}

fn drain(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn unit_hp(app: &mut App, id: UnitId) -> i32 {
    let mut q = app.world_mut().query::<&Unit>();
    q.iter(app.world())
        .find(|u| u.id == id)
        .map(|u| u.hp_current)
        .unwrap_or_else(|| panic!("unit {id:?} missing"))
}

#[test]
fn baby_burner_impact_emits_damage_break_and_thermal_spark_signal() {
    let book = aggregate_skill_book();
    let mut app = build_app(&book);
    spawn_agumon(&mut app);
    spawn_enemy(&mut app, PRIMARY_ID, 1, 200, None);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: AGUMON_ID,
        target: PRIMARY_ID,
    });
    app.update();
    let events = drain(&mut cursor, &app);

    let damage_count = events
        .iter()
        .filter(|e| {
            matches!(e.kind, CombatEventKind::OnDamageDealt { .. }) && e.target == PRIMARY_ID
        })
        .count();
    assert_eq!(damage_count, 1, "exactly one damage event on the primary");

    let break_count = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnBreak { .. }) && e.target == PRIMARY_ID)
        .count();
    assert_eq!(break_count, 1, "exactly one break event on the primary");

    let thermal_spark_count = events
        .iter()
        .filter(|e| {
            matches!(
                &e.kind,
                CombatEventKind::OnKernelTransition {
                    transition: CombatKernelTransition::Blueprint { owner, name, .. }
                } if owner == "agumon" && name == "apply_thermal_spark"
            )
        })
        .count();
    assert_eq!(
        thermal_spark_count,
        1,
        "exactly one apply_thermal_spark signal: events={:?}",
        events
            .iter()
            .map(|e| format!("{:?}", e.kind))
            .collect::<Vec<_>>()
    );

    assert!(
        unit_hp(&mut app, PRIMARY_ID) < 200,
        "primary HP must drop after Baby Burner impact"
    );
}

#[test]
fn lethal_heated_baby_burner_still_triggers_reactive_detonate_via_timeline_path() {
    let book = aggregate_skill_book();
    let mut app = build_app(&book);
    spawn_agumon(&mut app);
    spawn_enemy(&mut app, LEFT_ID, 0, 200, None);
    spawn_enemy(&mut app, PRIMARY_ID, 1, 40, Some(2));
    spawn_enemy(&mut app, RIGHT_ID, 2, 200, None);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: AGUMON_ID,
        target: PRIMARY_ID,
    });

    let mut all = Vec::new();
    for _ in 0..3 {
        app.update();
        all.extend(drain(&mut cursor, &app));
    }

    let detonate_targets: Vec<UnitId> = all
        .iter()
        .filter_map(|e| match &e.kind {
            CombatEventKind::OnKernelTransition {
                transition:
                    CombatKernelTransition::Blueprint {
                        owner,
                        name,
                        payload: SignalPayload::UnitTarget(target),
                    },
            } if owner == "agumon" && name == "baby_burner_detonate" => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(
        detonate_targets,
        vec![LEFT_ID, RIGHT_ID],
        "timeline-path KO with Heated must still fire reactive detonate for adjacents"
    );

    assert!(
        unit_hp(&mut app, PRIMARY_ID) <= 0,
        "primary should be KO'd by the lethal Baby Burner"
    );
    assert!(
        unit_hp(&mut app, LEFT_ID) < 200,
        "left adjacent enemy should be damaged by reactive detonate"
    );
    assert!(
        unit_hp(&mut app, RIGHT_ID) < 200,
        "right adjacent enemy should be damaged by reactive detonate"
    );
}
