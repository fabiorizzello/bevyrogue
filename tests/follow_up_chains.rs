/// Tests for D046: follow-up chain semantics after removing the D026 one-hop engine cap.
///
/// D046 contract: chain bounding lives in the data (cooldown/stack/once-per-round flags),
/// not in the engine. A follow-up emitted at depth N can trigger another follow-up at depth
/// N+1. Chains terminate naturally when no follower's preconditions re-trigger.
///
/// Regression hedge: if a future executor re-introduces a depth cap, JSONL output will
/// stop showing events with follow_up_depth >= 2. The assertions below guard against that.
use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    runtime::{ExtRegistries, register_kernel_builtins, timeline::TimelineLibrary},
    blueprints::register_all_blueprint_exts,
    events::{CombatEvent, CombatEventKind},
    follow_up::{
        FollowUpIntent, FollowUpTrace, follow_up_listener_system, resolve_follow_up_action_system,
    },
    kit::{FollowUpConfig, FollowUpTrigger, UnitSkills},
    log::{ActionLog, LogEntry},
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoLineId, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skill_timeline::compile_skill_book_timelines,
    skills_ron::SkillBook,
    units_ron::{UnitDef, UnitRoster},
};

fn load_roster() -> UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

fn load_skill_book() -> SkillBook {
    bevyrogue::data::aggregate_skill_book()
}

fn pilot(roster: &UnitRoster, name: &str) -> UnitDef {
    roster
        .0
        .iter()
        .find(|unit| unit.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("missing pilot {name}"))
}

fn setup_app(skill_book: SkillBook) -> App {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(TimelineLibrary::<String>::default())
        .init_resource::<ExtRegistries>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<FollowUpIntent>()
        .add_message::<FollowUpTrace>()
        .add_systems(
            Update,
            (
                resolve_action_system,
                follow_up_listener_system,
                resolve_follow_up_action_system,
            )
                .chain(),
        );

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(skill_book.clone());
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        register_kernel_builtins(&mut regs);
        register_all_blueprint_exts(&mut regs);
        let compiled = compile_skill_book_timelines(&skill_book, &regs)
            .expect("follow_up_chains test book must compile");
        app.world_mut().resource_mut::<TimelineLibrary<String>>().timelines = compiled;
    }
    app.world_mut().resource_mut::<SpPool>().current = 999;
    app
}

fn spawn_from_def(
    app: &mut App,
    def: &UnitDef,
    hp_current: i32,
    toughness_current: i32,
    ultimate_current: i32,
) {
    app.world_mut().spawn((
        Unit {
            id: def.id,
            name: def.name.clone(),
            hp_max: def.hp_max,
            hp_current,
            attribute: def.attribute,
            resists: def.resists.clone(),
            evo_stage: def.evo_stage,
        },
        def.team,
        Toughness {
            max: def.toughness_max,
            current: toughness_current,
            weaknesses: def.weaknesses.clone(),
            broken: false,
            category: Default::default(),
        },
        UltimateCharge {
            current: ultimate_current,
            trigger: def.ultimate_trigger,
            cap: def.ultimate_cap,
            trigger_type: def.ultimate_accumulation_trigger,
            charge_per_event: def.ultimate_charge_per_event,
        },
        UnitSkills {
            basic: def.basic_skill.clone(),
            skills: def.skill_ids.clone(),
            ultimate: def.ultimate_skill.clone(),
            follow_up: def.follow_up.clone(),
        },
    ));
}

fn message_cursor<T: Message>(app: &mut App) -> MessageCursor<T> {
    app.world_mut().resource_mut::<Messages<T>>().get_cursor()
}

fn drain_messages<T: Message + Clone>(cursor: &mut MessageCursor<T>, app: &App) -> Vec<T> {
    cursor
        .read(app.world().resource::<Messages<T>>())
        .cloned()
        .collect()
}

fn basic_hit_count(app: &App) -> usize {
    app.world()
        .resource::<ActionLog>()
        .events
        .iter()
        .filter(|entry| matches!(entry, LogEntry::BasicHit { .. }))
        .count()
}

/// D046: A follow-up triggered at depth=0 (OnEnemyKill) fires at depth=1 and breaks a second
/// enemy, which then triggers a depth=2 follow-up from a unit with OnEnemyBreak. The chain
/// terminates naturally because the depth=2 action hits the already-broken enemy (no new
/// OnBreak emitted).
///
/// Setup:
/// - Agumon (UnitId 1, OnEnemyBreak follow-up)
/// - Impmon-like inline unit (UnitId 8, OnEnemyKill follow-up → dorumon_follow_up: Dark, ToughnessHit 13)
/// - Enemy A (UnitId 4, HP=30, high toughness=200): killed by Agumon's Ultimate (no break)
/// - Enemy B (UnitId 5, HP=200, toughness=10, Dark-weak): broken by Impmon's depth=1 hit but survives
#[test]
fn depth_chain_progresses_to_depth_two() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let agumon = pilot(&roster, "Agumon");

    // Inline fixture: unit with OnEnemyKill follow-up (mirrors the roster convention for
    // a Dark attacker that can break Enemy B via dorumon_follow_up)
    let impmon = UnitDef {
        id: UnitId(8),
        name: "TestImpmon".into(),
        role_tags: vec!["assault".into()],
        signature_traits: vec!["dark".into()],
        hp_max: 86,
        attribute: Attribute::Virus,
        team: Team::Ally,
        basic_damage_tag: DamageTag::Dark,
        basic_skill: SkillId("draconic_edge".into()),
        skill_ids: vec![SkillId("draconic_edge".into())],
        ultimate_skill: SkillId("dorumon_ult".into()),
        follow_up: Some(FollowUpConfig {
            trigger: FollowUpTrigger::OnEnemyKill,
            action: SkillId("dorumon_follow_up".into()),
        }),
        enemy_traits: vec![],
        charged_attack: None,
        form_identity: None,
        blueprint_metadata: Default::default(),
        resists: vec![],
        toughness_max: 42,
        weaknesses: vec![],
        ultimate_trigger: 100,
        ultimate_cap: 150,
        ultimate_accumulation_trigger: UltAccumulationTrigger::OnKill,
        ultimate_charge_per_event: 50,
        speed: 90,
        evo_stage: EvoStage::Child,
        evo_line: EvoLineId("test".into()),
        evolves_to: vec![],
        tempo_resistant: false,
        toughness_category: Default::default(),
    };

    // Enemy A: very low HP so Agumon's Ultimate kills it; high toughness so it does NOT break.
    // Enemy B: high HP (survives Impmon's follow-up) but low toughness (Dark-weak → breaks on ToughnessHit 13).
    spawn_from_def(&mut app, &agumon, 100, 50, 100);
    spawn_from_def(&mut app, &impmon, 86, 42, 0);
    // Enemy A: HP 30, toughness 200 (no break from Agumon Ult ToughnessHit 30)
    app.world_mut().spawn((
        Unit {
            id: UnitId(4),
            name: "EnemyA".into(),
            hp_max: 30,
            hp_current: 30,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        Toughness {
            max: 200,
            current: 200,
            weaknesses: vec![],
            broken: false,
            category: Default::default(),
        },
        UltimateCharge {
            current: 0,
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
    // Enemy B: HP 200 (survives dorumon_follow_up Damage 23), toughness 10 Dark-weak → breaks.
    app.world_mut().spawn((
        Unit {
            id: UnitId(5),
            name: "EnemyB".into(),
            hp_max: 200,
            hp_current: 200,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        Toughness {
            max: 10,
            current: 10,
            weaknesses: vec![DamageTag::Dark],
            broken: false,
            category: Default::default(),
        },
        UltimateCharge {
            current: 0,
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

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);

    // Update 1: Agumon Ultimate kills Enemy A → Impmon's OnEnemyKill follow-up fires at
    // depth=1, breaking Enemy B → OnBreak{depth=1} emitted.
    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: agumon.id,
        target: UnitId(4),
    });
    app.update();
    // MEM029: drain cursor before update 2 so events from update 1 are not pruned.
    let events_u1 = drain_messages(&mut event_cursor, &app);

    // Update 2: follow_up_listener reads OnBreak{depth=1} → Agumon's OnEnemyBreak follow-up
    // fires at depth=2 on Enemy B (alive, already broken → no new OnBreak).
    app.update();
    let events_u2 = drain_messages(&mut event_cursor, &app);

    // Update 3: no new triggers — chain terminates.
    app.update();
    let events_u3 = drain_messages(&mut event_cursor, &app);

    let all_events: Vec<&CombatEvent> = events_u1
        .iter()
        .chain(events_u2.iter())
        .chain(events_u3.iter())
        .collect();

    // --- depth=0: root kill ---
    assert!(
        all_events.iter().any(|e| {
            matches!(e.kind, CombatEventKind::OnEnemyKill)
                && e.source == agumon.id
                && e.target == UnitId(4)
                && e.follow_up_depth == 0
        }),
        "missing root kill at depth=0"
    );

    // --- depth=1: Impmon's follow-up breaks Enemy B ---
    assert!(
        events_u1.iter().any(|e| {
            matches!(e.kind, CombatEventKind::OnBreak { .. })
                && e.source == impmon.id
                && e.target == UnitId(5)
                && e.follow_up_depth == 1
        }),
        "Impmon's depth=1 follow-up must emit OnBreak{{depth=1}} on Enemy B"
    );

    // --- depth=2: Agumon's OnEnemyBreak follow-up fires — D046 unblocked ---
    assert!(
        events_u2
            .iter()
            .any(|e| e.follow_up_depth == 2 && e.source == agumon.id),
        "Agumon must fire a follow-up at depth=2 in update 2 (D046: no engine cap)"
    );

    // --- chain terminates: no depth > 2 events ---
    assert!(
        !all_events.iter().any(|e| e.follow_up_depth > 2),
        "chain must terminate after depth=2 — no depth>2 events expected"
    );

    // --- update 3 is idle: no new follow-up triggered ---
    assert!(
        !events_u3.iter().any(|e| e.follow_up_depth >= 1),
        "update 3 must be idle — chain already terminated"
    );

    // --- exactly 3 damage hits: root + depth=1 + depth=2 ---
    assert_eq!(
        basic_hit_count(&app),
        3,
        "expected 3 BasicHit entries: Agumon Ultimate, Impmon depth=1, Agumon depth=2"
    );
}

/// D046 negative: a follow-up that does NOT emit OnBreak (Greymon's follow-up on a target
/// that is already broken) does not trigger further chain steps. Greymon vs a single enemy:
/// initial break fires Greymon's depth=1 follow-up; that hit lands on a broken target with
/// no new OnBreak; the chain terminates after exactly one follow-up.
#[test]
fn chain_terminates_when_follow_up_cannot_retrigger() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let greymon = pilot(&roster, "Greymon");

    // Attacker with no follow-up — breaks the enemy with a fire hit.
    let breaker_def = UnitDef {
        id: UnitId(9),
        name: "Breaker".into(),
        role_tags: vec![],
        signature_traits: vec![],
        hp_max: 100,
        attribute: Attribute::Vaccine,
        team: Team::Ally,
        basic_damage_tag: DamageTag::Fire,
        basic_skill: SkillId("baby_flame".into()),
        skill_ids: vec![SkillId("baby_flame".into())],
        ultimate_skill: SkillId("agumon_ult".into()),
        follow_up: None,
        enemy_traits: vec![],
        charged_attack: None,
        form_identity: None,
        blueprint_metadata: Default::default(),
        resists: vec![],
        toughness_max: 50,
        weaknesses: vec![],
        ultimate_trigger: 100,
        ultimate_cap: 150,
        ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
        ultimate_charge_per_event: 25,
        speed: 100,
        evo_stage: EvoStage::Child,
        evo_line: EvoLineId("test".into()),
        evolves_to: vec![],
        tempo_resistant: false,
        toughness_category: Default::default(),
    };

    // Enemy: HP 200 (survives both hits), low toughness (Fire-weak → breaks on first hit).
    spawn_from_def(&mut app, &greymon, 130, 60, 0);
    app.world_mut().spawn((
        Unit {
            id: UnitId(9),
            name: breaker_def.name.clone(),
            hp_max: breaker_def.hp_max,
            hp_current: breaker_def.hp_max,
            attribute: breaker_def.attribute,
            resists: vec![],
            evo_stage: breaker_def.evo_stage,
        },
        breaker_def.team,
        Toughness {
            max: 50,
            current: 50,
            weaknesses: vec![],
            broken: false,
            category: Default::default(),
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: breaker_def.basic_skill.clone(),
            skills: breaker_def.skill_ids.clone(),
            ultimate: breaker_def.ultimate_skill.clone(),
            follow_up: breaker_def.follow_up.clone(),
        },
    ));
    app.world_mut().spawn((
        Unit {
            id: UnitId(10),
            name: "Enemy".into(),
            hp_max: 200,
            hp_current: 200,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        Toughness {
            max: 10,
            current: 10,
            weaknesses: vec![DamageTag::Fire],
            broken: false,
            category: Default::default(),
        },
        UltimateCharge {
            current: 0,
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

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);

    // Fire a Skill (not Ultimate) to avoid the ult_ready gate (breaker has current=0).
    // agumon_ult has ToughnessHit(30) on a Fire attack — enough to break toughness=10 Fire-weak.
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(9),
        skill_id: SkillId("agumon_ult".into()),
        target: UnitId(10),
    });
    // Update 1: breaker fires → OnBreak{depth=0} emitted → follow_up_listener sees it →
    // Greymon's OnEnemyBreak follow-up fires at depth=1 in the SAME update.
    // (resolve_follow_up_action_system runs after follow_up_listener in the same chain.)
    app.update();
    // MEM029: drain before next update
    let events_u1 = drain_messages(&mut event_cursor, &app);

    // Update 2: follow_up_listener reads depth=1 events from update 1. No OnBreak at depth=1
    // was emitted (enemy already broken → apply_hit returns false). No new follow-up scheduled.
    app.update();
    let events_u2 = drain_messages(&mut event_cursor, &app);

    // Update 3: no new triggers expected.
    app.update();
    let events_u3 = drain_messages(&mut event_cursor, &app);

    // Break happened at depth=0 in update 1.
    assert!(
        events_u1.iter().any(|e| {
            matches!(e.kind, CombatEventKind::OnBreak { .. })
                && e.target == UnitId(10)
                && e.follow_up_depth == 0
        }),
        "initial break at depth=0 must be present in update 1"
    );

    // Greymon fires at depth=1 in the SAME update as the break (within the same system chain).
    assert!(
        events_u1
            .iter()
            .any(|e| e.follow_up_depth == 1 && e.source == greymon.id),
        "Greymon must fire follow-up at depth=1 (in update 1, same cycle as the break)"
    );

    // No depth=2 events anywhere — chain terminates because greymon_follow_up
    // hits a broken target and does not re-emit OnBreak.
    assert!(
        !events_u1
            .iter()
            .chain(events_u2.iter())
            .chain(events_u3.iter())
            .any(|e| e.follow_up_depth >= 2),
        "chain must terminate after depth=1 — greymon_follow_up does not re-emit OnBreak on a broken target"
    );
}
