use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use bevyrogue::combat::{
    api::timeline::TimelineLibrary,
    api::{ExtRegistries, SignalBus, SignalTaxonomy, register_kernel_builtins},
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    blueprints::register_all_blueprint_exts,
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
    SkillBookHandle,
    skill_timeline::compile_skill_book_timelines,
    skills_ron::{SkillBook, validate_skill_book},
};
use std::collections::HashSet;

fn canonical_book() -> SkillBook {
    bevyrogue::data::aggregate_skill_book()
}

fn canonical_regs() -> ExtRegistries {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    register_all_blueprint_exts(&mut regs);
    regs
}

/// All child-roster active skills that must be timeline-backed after S06 migration.
const CHILD_ROSTER_ACTIVE: &[&str] = &[
    // child basics / skill_ids
    "baby_flame",
    "bubble_blast",
    "draconic_edge",
    "diamond_storm",
    "holy_breeze",
    "patamon_revive",
    "tentomon_basic",
    "petit_thunder",
    // child follow-ups
    "agumon_follow_up",
    "gabumon_follow_up",
    "dorumon_follow_up",
    "renamon_follow_up",
    "patamon_follow_up",
    "tentomon_follow_up",
    // previously migrated ultimate
    "renamon_ult",
];

#[test]
fn child_roster_active_skills_all_have_compiled_timelines() {
    let book = canonical_book();
    validate_skill_book(&book).expect("canonical skills.ron must validate");
    let compiled = compile_skill_book_timelines(&book, &canonical_regs())
        .expect("timeline compilation must succeed");

    let ids: HashSet<_> = compiled.iter().map(|t| t.id.as_str()).collect();

    for skill_id in CHILD_ROSTER_ACTIVE {
        assert!(
            ids.contains(*skill_id),
            "child-roster skill `{skill_id}` must have a compiled timeline after S06 migration"
        );
    }
}

// ── runtime execution test ────────────────────────────────────────────────────

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
        .insert_resource(CombatRng::from_seed(42))
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

fn spawn_caster(app: &mut App, skill_id: &str) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id: UnitId(1),
                name: "Caster".into(),
                hp_max: 500,
                hp_current: 500,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Child,
            },
            Team::Ally,
            UnitSkills {
                basic: SkillId(skill_id.into()),
                skills: vec![SkillId(skill_id.into())],
                ultimate: SkillId(skill_id.into()),
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

fn spawn_enemy(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id: UnitId(2),
                name: "Enemy".into(),
                hp_max: 300,
                hp_current: 300,
                attribute: Attribute::Virus,
                resists: vec![],
                evo_stage: EvoStage::Child,
            },
            Team::Enemy,
            ActionValue(MAX_AV),
            Toughness::new(10, vec![DamageTag::Fire]),
            StatusBag::default(),
        ))
        .id()
}

fn collect_events(app: &App, cursor: &mut MessageCursor<CombatEvent>) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn fire_skill(app: &mut App, attacker: UnitId, skill_id: &str, target: UnitId) {
    app.world_mut().write_message(ActionIntent::Skill {
        attacker,
        skill_id: SkillId(skill_id.into()),
        target,
    });
    app.update();
}

#[test]
fn baby_flame_timeline_path_delivers_damage_before_break_then_signal() {
    let book = canonical_book();
    validate_skill_book(&book).expect("canonical skills.ron must validate");

    let mut app = build_app(&book);
    // register signal so taxonomy check passes
    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register("agumon", "apply_heated");

    let _caster = spawn_caster(&mut app, "baby_flame");
    let _enemy = spawn_enemy(&mut app);

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();

    fire_skill(&mut app, UnitId(1), "baby_flame", UnitId(2));

    let events = collect_events(&app, &mut cursor);
    let dump: Vec<_> = events.iter().map(|e| format!("{:?}", e.kind)).collect();

    let pos_damage = events
        .iter()
        .position(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .expect("damage event must fire");

    let pos_break = events
        .iter()
        .position(|e| matches!(e.kind, CombatEventKind::OnBreak { .. }))
        .expect("break event must fire");

    let pos_signal = events
        .iter()
        .position(|e| {
            matches!(
                &e.kind,
                CombatEventKind::OnKernelTransition {
                    transition: CombatKernelTransition::Blueprint { owner, name, .. }
                } if owner == "agumon" && name == "apply_heated"
            )
        })
        .expect("apply_heated signal must fire");

    assert!(
        pos_damage < pos_break,
        "damage must precede break: {dump:?}"
    );
    assert!(
        pos_break < pos_signal,
        "break must precede signal: {dump:?}"
    );

    // target should have taken damage
    let mut q = app.world_mut().query::<(&Unit, &Team)>();
    let took_damage = q
        .iter(app.world())
        .any(|(unit, team)| *team == Team::Enemy && unit.hp_current < 300);
    assert!(took_damage, "baby_flame must deal damage via timeline path");
}

// ── negative tests ────────────────────────────────────────────────────────────

#[test]
fn dangling_hook_in_child_roster_skill_fails_at_compile_with_skill_and_site() {
    // Inject a typo into the first deal_damage hook (baby_flame's impact_damage beat)
    let bad_ron =
        bevyrogue::data::aggregate_skill_book_ron_text().replacen("core/deal_damage", "core/deal_dmge", 1);
    let book: SkillBook = ron::from_str(&bad_ron).expect("parse tweaked skills.ron");

    let err = compile_skill_book_timelines(&book, &canonical_regs())
        .expect_err("dangling hook must fail at compile, not runtime");

    // baby_flame is the first timeline-backed skill in the catalog order
    assert_eq!(err.skill_id, SkillId("baby_flame".into()));
    assert_eq!(err.site, "beat impact_damage");
    assert!(
        err.detail.contains("core/deal_dmge"),
        "error detail must name the missing hook: {}",
        err.detail
    );
}

#[test]
fn dangling_selector_in_child_roster_skill_fails_at_compile() {
    // Inject a typo into the first selector (bubble_blast or baby_flame)
    let bad_ron =
        bevyrogue::data::aggregate_skill_book_ron_text().replacen("core/primary", "core/priimary", 1);
    let book: SkillBook = ron::from_str(&bad_ron).expect("parse tweaked skills.ron");

    let err = compile_skill_book_timelines(&book, &canonical_regs())
        .expect_err("dangling selector must fail at compile");

    assert!(
        !err.skill_id.0.is_empty(),
        "error must name the owning skill"
    );
    assert!(
        err.detail.contains("core/priimary"),
        "error detail must name the missing selector: {}",
        err.detail
    );
}

#[test]
fn timeline_skill_execution_gracefully_handles_all_enemies_dead() {
    let book = canonical_book();
    let mut app = build_app(&book);
    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register("agumon", "apply_heated");

    let _caster = spawn_caster(&mut app, "baby_flame");
    // spawn a dead enemy (hp_current = 0) — no valid alive target
    app.world_mut().spawn((
        Unit {
            id: UnitId(2),
            name: "DeadEnemy".into(),
            hp_max: 100,
            hp_current: 0,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: EvoStage::Child,
        },
        Team::Enemy,
        ActionValue(MAX_AV),
        Toughness::new(10, vec![]),
        StatusBag::default(),
        bevyrogue::combat::unit::Ko,
    ));

    // firing the skill with no alive targets must not panic
    fire_skill(&mut app, UnitId(1), "baby_flame", UnitId(2));
}
