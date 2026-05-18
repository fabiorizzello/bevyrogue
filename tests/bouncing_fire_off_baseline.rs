use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use bevyrogue::combat::runtime::timeline::TimelineLibrary;
use bevyrogue::combat::{
    runtime::{ExtRegistries, SignalBus, SignalTaxonomy, register_kernel_builtins},
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    blueprints::{agumon::TalentRanks, register_all_blueprint_exts},
    events::{CombatEvent, CombatEventKind},
    kernel::CombatKernelTransition,
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    sp::SpPool,
    state::CombatState,
    status_effect::StatusBag,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, apply_av_ops_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle, skill_timeline::compile_skill_book_timelines, skills_ron::SkillBook,
};

const AGUMON_ID: UnitId = UnitId(1);
const ENEMY_A_ID: UnitId = UnitId(2);
const ENEMY_B_ID: UnitId = UnitId(3);

fn canonical_book() -> SkillBook {
    bevyrogue::data::aggregate_skill_book()
}

fn build_app(book: &SkillBook) -> App {
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
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(7))
        .insert_resource(TimelineLibrary::<String>::default())
        .init_resource::<SignalBus>()
        .init_resource::<ExtRegistries>()
        .init_resource::<SignalTaxonomy>()
        .init_resource::<TalentRanks>()
        .add_message::<ActionIntent>()
        .add_message::<TurnAdvanced>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, (resolve_action_system, apply_av_ops_system).chain());

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

    // Required for apply_blueprint_signal to accept this owner/name pair.
    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register("agumon", "apply_heated");

    app
}

fn spawn_agumon(app: &mut App) {
    app.world_mut().spawn((
        Unit {
            id: AGUMON_ID,
            name: "Agumon".into(),
            hp_max: 500,
            hp_current: 500,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Child,
        },
        Team::Ally,
        UnitSkills {
            basic: SkillId("baby_flame".into()),
            skills: vec![SkillId("baby_flame".into())],
            ultimate: SkillId("baby_flame".into()),
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

fn spawn_enemy(app: &mut App, id: UnitId) {
    // Free attribute → neutral triangle vs Vaccine; empty resists → neutral tag mod.
    // Toughness has Fire weakness so baby_flame's BreakToughness(10) > max(8) triggers OnBreak.
    app.world_mut().spawn((
        Unit {
            id,
            name: "Enemy".into(),
            hp_max: 300,
            hp_current: 300,
            attribute: Attribute::Free,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        ActionValue(MAX_AV),
        Toughness::new(8, vec![DamageTag::Fire]),
        StatusBag::default(),
    ));
}

fn fire_baby_flame(app: &mut App) {
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: AGUMON_ID,
        skill_id: SkillId("baby_flame".into()),
        target: ENEMY_A_ID,
    });
    app.update();
}

fn collect_events(app: &App, cursor: &mut MessageCursor<CombatEvent>) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

/// OFF=baseline: TalentRanks default (rank 0) → has_bouncing_fire gate is false →
/// bounce_loop beat never entered → exactly one DamageDealt event targeting the primary enemy.
#[test]
fn baby_flame_off_baseline_emits_single_damage_and_no_bounce() {
    let book = canonical_book();
    let mut app = build_app(&book);
    spawn_agumon(&mut app);
    spawn_enemy(&mut app, ENEMY_A_ID);

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_baby_flame(&mut app);

    let events = collect_events(&app, &mut cursor);
    let dump: Vec<_> = events.iter().map(|e| format!("{:?}", e.kind)).collect();

    let damage_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .collect();
    assert_eq!(
        damage_events.len(),
        1,
        "OFF baseline: expected exactly 1 DamageDealt event (no bounce); dump={dump:?}"
    );
    assert_eq!(
        damage_events[0].target, ENEMY_A_ID,
        "OFF baseline: damage must target the primary enemy"
    );

    // apply_heated blueprint signal must flow through the kernel transition path.
    assert!(
        events.iter().any(|e| matches!(
            &e.kind,
            CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::Blueprint { owner, name, .. },
            } if owner == "agumon" && name == "apply_heated"
        )),
        "OFF baseline: apply_heated Blueprint transition must be emitted; dump={dump:?}"
    );
}

/// ON (rank 1): Bouncing Fire enabled → bounce_loop entered → primary hit + exactly one hop
/// to the second enemy target, with both enemies registered in the cast_hit_set so the loop
/// exit predicate fires after a single pass.
#[test]
fn baby_flame_rank1_bouncing_fire_produces_one_hop_to_second_target() {
    let book = canonical_book();
    let mut app = build_app(&book);

    app.world_mut()
        .resource_mut::<TalentRanks>()
        .0
        .insert("agumon::bouncing_fire".into(), 1);

    spawn_agumon(&mut app);
    spawn_enemy(&mut app, ENEMY_A_ID);
    spawn_enemy(&mut app, ENEMY_B_ID);

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_baby_flame(&mut app);

    let events = collect_events(&app, &mut cursor);
    let dump: Vec<_> = events.iter().map(|e| format!("{:?}", e.kind)).collect();

    let damage_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .collect();

    assert_eq!(
        damage_events.len(),
        2,
        "ON rank1: expected 2 DamageDealt events (primary + 1 bounce hop); dump={dump:?}"
    );

    let targets: Vec<UnitId> = damage_events.iter().map(|e| e.target).collect();
    assert!(
        targets.contains(&ENEMY_A_ID),
        "ON rank1: primary enemy must be hit; targets={targets:?}"
    );
    assert!(
        targets.contains(&ENEMY_B_ID),
        "ON rank1: bounce enemy must be hit; targets={targets:?}"
    );

    // apply_heated blueprint signal still emitted even with the bounce active.
    assert!(
        events.iter().any(|e| matches!(
            &e.kind,
            CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::Blueprint { owner, name, .. },
            } if owner == "agumon" && name == "apply_heated"
        )),
        "ON rank1: apply_heated Blueprint transition must be emitted; dump={dump:?}"
    );
}
