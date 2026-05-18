use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use bevyrogue::combat::{
    runtime::timeline::TimelineLibrary,
    runtime::{ExtRegistries, SignalBus, SignalTaxonomy, register_kernel_builtins},
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    blueprints::register_all_blueprint_exts,
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    sp::SpPool,
    state::CombatState,
    status_effect::{StatusBag, StatusEffectKind},
    stun::Stunned,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, apply_av_ops_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
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

fn canonical_regs() -> ExtRegistries {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    register_all_blueprint_exts(&mut regs);
    regs
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
                name: "Tentomon".into(),
                hp_max: 500,
                hp_current: 500,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            UnitSkills {
                basic: SkillId("petit_thunder".into()),
                skills: vec![SkillId("petit_thunder".into())],
                ultimate: SkillId("petit_thunder".into()),
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
            Toughness::new(8, vec![DamageTag::Electric]),
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

fn event_pos(events: &[CombatEvent], predicate: impl Fn(&CombatEvent) -> bool) -> usize {
    events
        .iter()
        .position(predicate)
        .expect("expected event not found")
}

#[test]
fn petit_thunder_timeline_backed_path_preserves_break_status_and_signal_order() {
    let book = canonical_book();
    validate_skill_book(&book).expect("canonical skills.ron must validate");

    let mut app = build_app(&book);
    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register("tentomon", "build_static_charge");
    let _caster = spawn_caster(&mut app);
    let target = spawn_target(&mut app);

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_skill(&mut app, "petit_thunder");

    let events = collect_events(&app, &mut cursor);
    let dump: Vec<_> = events.iter().map(|e| format!("{:?}", e.kind)).collect();

    let pos_damage = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::OnDamageDealt { .. })
    });
    let pos_break = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::OnBreak { .. })
    });
    let pos_status = event_pos(
        &events,
        |e| matches!(&e.kind, CombatEventKind::OnStatusApplied { kind } if *kind == StatusEffectKind::Paralyzed),
    );
    let pos_applied = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionApplied)
    });
    let pos_resolved = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionResolved)
    });
    let pos_signal = event_pos(&events, |e| {
        matches!(
            &e.kind,
            CombatEventKind::OnKernelTransition {
                transition: bevyrogue::combat::kernel::CombatKernelTransition::Blueprint { owner, name, .. }
            } if owner == "tentomon" && name == "build_static_charge"
        )
    });

    assert!(
        pos_damage < pos_break,
        "damage must precede break: {dump:?}"
    );
    assert!(
        pos_break < pos_status,
        "break must precede paralyze: {dump:?}"
    );
    assert!(
        pos_status < pos_signal,
        "status must precede signal: {dump:?}"
    );
    assert!(
        pos_signal < pos_applied,
        "signal must precede action applied: {dump:?}"
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
    assert!(
        app.world().get::<Stunned>(target).is_some(),
        "BreakToughness should stun the target"
    );

    let status_bag = app
        .world()
        .get::<StatusBag>(target)
        .expect("target status bag missing");
    assert!(
        status_bag.has(&StatusEffectKind::Paralyzed),
        "Paralyzed must be applied"
    );
}
