use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use bevyrogue::combat::{
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    blueprints::register_all_blueprint_exts,
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    runtime::timeline::TimelineLibrary,
    runtime::{ExtRegistries, SignalBus, SignalTaxonomy, register_kernel_builtins},
    sp::SpPool,
    state::CombatState,
    status_effect::{StatusBag, StatusEffectKind},
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, apply_av_ops_system, resolve_action_system},
    types::{Attribute, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skill_timeline::compile_skill_book_timelines,
    skills_ron::{SkillBook, validate_skill_book},
};

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

    app
}

fn spawn_caster(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id: UnitId(1),
                name: "Renamon".into(),
                hp_max: 500,
                hp_current: 500,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            UnitSkills {
                basic: SkillId("renamon_ult".into()),
                skills: vec![SkillId("renamon_ult".into())],
                ultimate: SkillId("renamon_ult".into()),
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
        ))
        .id()
}

fn spawn_target(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id: UnitId(2),
                name: "Target".into(),
                hp_max: 200,
                hp_current: 200,
                attribute: Attribute::Virus,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            ActionValue(MAX_AV),
            Toughness::new(30, vec![]),
            StatusBag::default(),
        ))
        .id()
}

fn fire_skill(app: &mut App, skill_id: &str) {
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId(skill_id.into()),
        target: UnitId(2),
    });
    app.update();
}

fn collect_events(app: &App, cursor: &mut MessageCursor<CombatEvent>) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn event_pos(
    events: &[CombatEvent],
    predicate: impl Fn(&CombatEvent) -> bool,
    label: &str,
) -> usize {
    events.iter().position(predicate).unwrap_or_else(|| {
        panic!(
            "expected {label} event not found; events={:?}",
            events
                .iter()
                .map(|e| format!("{:?}", e.kind))
                .collect::<Vec<_>>()
        )
    })
}

#[test]
fn renamon_ult_timeline_backed_path_applies_delay_and_ally_blessed() {
    let book = canonical_book();
    validate_skill_book(&book).expect("canonical skills.ron must validate");

    let mut app = build_app(&book);
    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register("renamon", "commit_precision_press");
    let caster = spawn_caster(&mut app);
    let target = spawn_target(&mut app);

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_skill(&mut app, "renamon_ult");

    let events = collect_events(&app, &mut cursor);
    let dump: Vec<_> = events.iter().map(|e| format!("{:?}", e.kind)).collect();

    let pos_damage = event_pos(
        &events,
        |e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }),
        "damage",
    );
    let pos_delay = event_pos(
        &events,
        |e| matches!(e.kind, CombatEventKind::DelayTurn { amount_pct: 50, .. }),
        "delay",
    );
    let pos_blessed = event_pos(
        &events,
        |e| matches!(&e.kind, CombatEventKind::OnStatusApplied { kind } if *kind == StatusEffectKind::Blessed),
        "blessed status",
    );
    let pos_applied = event_pos(
        &events,
        |e| matches!(e.kind, CombatEventKind::OnActionApplied),
        "action applied",
    );
    let pos_resolved = event_pos(
        &events,
        |e| matches!(e.kind, CombatEventKind::OnActionResolved),
        "action resolved",
    );

    assert!(
        pos_damage < pos_delay,
        "damage must precede enemy delay: {dump:?}"
    );
    assert!(
        pos_delay < pos_blessed,
        "enemy delay must precede caster buff: {dump:?}"
    );
    assert!(
        pos_blessed < pos_applied,
        "buff must precede action applied: {dump:?}"
    );
    assert!(
        pos_applied < pos_resolved,
        "action applied must precede resolved: {dump:?}"
    );

    let target_unit = app
        .world()
        .get::<Unit>(target)
        .expect("target unit missing");
    assert!(
        target_unit.hp_current < 200,
        "timeline skill should deal damage"
    );
    assert_eq!(
        app.world()
            .get::<ActionValue>(target)
            .expect("target AV missing")
            .0,
        5000,
        "DelayTurn should flow through apply_av_ops_system"
    );

    let caster_bag = app
        .world()
        .get::<StatusBag>(caster)
        .expect("caster status bag missing");
    assert!(
        caster_bag.has(&StatusEffectKind::Blessed),
        "caster must receive Blessed"
    );
}
