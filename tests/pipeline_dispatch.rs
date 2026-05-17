use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    api::{ExtRegistries, register_kernel_builtins, timeline::TimelineLibrary},
    blueprints::register_all_blueprint_exts,
    events::{ActionIntentKind, CombatEvent, CombatEventKind},
    follow_up::{
        FollowUpIntent, FollowUpTrace, follow_up_listener_system, resolve_follow_up_action_system,
    },
    kit::UnitSkills,
    log::ActionLog,
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, EvoStage, SkillId, UnitId},
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
    ron::from_str(include_str!("../assets/data/units.ron")).expect("parse units.ron")
}

fn load_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
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
            .expect("pipeline_dispatch test book must compile");
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
            evo_stage: EvoStage::Adult,
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

/// Spawn a passive enemy that will never act in our tests.
/// Uses "enemy_skill_fire" (in skills.ron) as a placeholder skill so lookup
/// won't fail if it were ever resolved — but these tests never trigger enemy actions.
fn spawn_sturdy_enemy(app: &mut App, id: UnitId, name: &str, hp: i32, toughness: i32) {
    app.world_mut().spawn((
        Unit {
            id,
            name: name.into(),
            hp_max: hp,
            hp_current: hp,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        Toughness {
            max: toughness,
            current: toughness,
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
            basic: SkillId("enemy_skill_fire".into()),
            skills: vec![],
            ultimate: SkillId("enemy_ult_fire".into()),
            follow_up: None,
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

fn pos_of<F: Fn(&CombatEvent) -> bool>(events: &[CombatEvent], pred: F) -> Option<usize> {
    events.iter().position(pred)
}

fn event_dump(events: &[CombatEvent]) -> String {
    events
        .iter()
        .enumerate()
        .map(|(i, e)| format!("[{}] depth={} {:?}", i, e.follow_up_depth, e.kind))
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Test 1: 4 lifecycle events emitted in positional order for a root Basic
// ---------------------------------------------------------------------------

#[test]
fn lifecycle_root_action_emits_4_events_in_order() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let agumon = pilot(&roster, "Agumon"); // baby_flame basic, OnEnemyBreak follow-up
    spawn_from_def(&mut app, &agumon, 100, 50, 0);
    // Sturdy enemy: high HP and toughness — no break, no KO from one Basic
    spawn_sturdy_enemy(&mut app, UnitId(20), "SturdyDummy", 500, 500);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: agumon.id,
        target: UnitId(20),
    });
    app.update();

    let events = drain_messages(&mut cursor, &app);
    let dump = event_dump(&events);

    let pos_declared = pos_of(&events, |e| {
        matches!(&e.kind, CombatEventKind::OnActionDeclared { intent_kind } if *intent_kind == ActionIntentKind::Basic)
            && e.follow_up_depth == 0
    });
    let pos_preapp = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionPreApp) && e.follow_up_depth == 0
    });
    let pos_applied = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionApplied) && e.follow_up_depth == 0
    });
    let pos_resolved = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionResolved) && e.follow_up_depth == 0
    });

    assert!(
        pos_declared.is_some(),
        "OnActionDeclared{{Basic,depth=0}} missing\n{dump}"
    );
    assert!(
        pos_preapp.is_some(),
        "OnActionPreApp(depth=0) missing\n{dump}"
    );
    assert!(
        pos_applied.is_some(),
        "OnActionApplied(depth=0) missing\n{dump}"
    );
    assert!(
        pos_resolved.is_some(),
        "OnActionResolved(depth=0) missing\n{dump}"
    );

    let (d, p, a, r) = (
        pos_declared.unwrap(),
        pos_preapp.unwrap(),
        pos_applied.unwrap(),
        pos_resolved.unwrap(),
    );

    assert!(
        d < p,
        "OnActionDeclared must precede OnActionPreApp\n{dump}"
    );
    assert!(p < a, "OnActionPreApp must precede OnActionApplied\n{dump}");
    assert!(
        a < r,
        "OnActionApplied must precede OnActionResolved\n{dump}"
    );

    // Core events must not appear before Declared
    let core_before = events[..d].iter().any(|e| {
        matches!(
            e.kind,
            CombatEventKind::OnDamageDealt { .. }
                | CombatEventKind::OnBreak { .. }
                | CombatEventKind::UnitDied { .. }
        )
    });
    assert!(
        !core_before,
        "core event appeared before OnActionDeclared\n{dump}"
    );

    // Core events must not appear after Resolved (no stray follow-up at depth=0)
    let core_after = events[r + 1..].iter().any(|e| {
        e.follow_up_depth == 0
            && matches!(
                e.kind,
                CombatEventKind::OnDamageDealt { .. }
                    | CombatEventKind::OnBreak { .. }
                    | CombatEventKind::UnitDied { .. }
            )
    });
    assert!(
        !core_after,
        "core event (depth=0) appeared after OnActionResolved\n{dump}"
    );
}

// ---------------------------------------------------------------------------
// Test 2: follow-up produces a second declared→resolved cycle at depth = 1
// ---------------------------------------------------------------------------

#[test]
fn lifecycle_follow_up_action_emits_second_cycle_with_depth_1() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let agumon = pilot(&roster, "Agumon"); // OnEnemyBreak → agumon_follow_up
    spawn_from_def(&mut app, &agumon, 100, 50, 0);

    // Enemy with toughness=5 and Fire weakness — Agumon's baby_flame (Fire, ToughnessHit=10)
    // drains toughness to -5 and the Fire weakness satisfies apply_hit's break condition.
    app.world_mut().spawn((
        Unit {
            id: UnitId(20),
            name: "FragileDummy".into(),
            hp_max: 500,
            hp_current: 500,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        Toughness {
            max: 5,
            current: 5,
            weaknesses: vec![bevyrogue::combat::types::DamageTag::Fire],
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
            basic: SkillId("enemy_skill_fire".into()),
            skills: vec![],
            ultimate: SkillId("enemy_ult_fire".into()),
            follow_up: None,
        },
    ));

    let mut cursor = message_cursor::<CombatEvent>(&mut app);

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: agumon.id,
        target: UnitId(20),
    });
    app.update();

    let events = drain_messages(&mut cursor, &app);
    let dump = event_dump(&events);

    // Prerequisite: the break must have fired to enable the follow-up
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, CombatEventKind::OnBreak { .. }) && e.follow_up_depth == 0),
        "expected OnBreak at depth=0 to trigger Agumon follow-up\n{dump}"
    );

    // --- Root cycle (depth = 0) ---
    let pos_d0 = pos_of(&events, |e| {
        matches!(&e.kind, CombatEventKind::OnActionDeclared { intent_kind } if *intent_kind == ActionIntentKind::Basic)
            && e.follow_up_depth == 0
    })
    .expect("missing OnActionDeclared(depth=0)");

    let pos_p0 = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionPreApp) && e.follow_up_depth == 0
    })
    .expect("missing OnActionPreApp(depth=0)");

    let pos_a0 = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionApplied) && e.follow_up_depth == 0
    })
    .expect("missing OnActionApplied(depth=0)");

    let pos_r0 = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionResolved) && e.follow_up_depth == 0
    })
    .expect("missing OnActionResolved(depth=0)");

    // --- Follow-up cycle (depth = 1) ---
    let pos_d1 = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionDeclared { .. }) && e.follow_up_depth == 1
    })
    .expect("missing OnActionDeclared(depth=1) — follow-up was not scheduled");

    let pos_p1 = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionPreApp) && e.follow_up_depth == 1
    })
    .expect("missing OnActionPreApp(depth=1)");

    let pos_a1 = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionApplied) && e.follow_up_depth == 1
    })
    .expect("missing OnActionApplied(depth=1)");

    let pos_r1 = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionResolved) && e.follow_up_depth == 1
    })
    .expect("missing OnActionResolved(depth=1)");

    // Full ordering: d0 < p0 < a0 < r0 < d1 < p1 < a1 < r1
    assert!(pos_d0 < pos_p0, "d0 < p0\n{dump}");
    assert!(pos_p0 < pos_a0, "p0 < a0\n{dump}");
    assert!(pos_a0 < pos_r0, "a0 < r0\n{dump}");
    assert!(
        pos_r0 < pos_d1,
        "root Resolved must precede follow-up Declared\n{dump}"
    );
    assert!(pos_d1 < pos_p1, "d1 < p1\n{dump}");
    assert!(pos_p1 < pos_a1, "p1 < a1\n{dump}");
    assert!(pos_a1 < pos_r1, "a1 < r1\n{dump}");
}

// ---------------------------------------------------------------------------
// Test 3: lifecycle events still fully emitted when the action fails (SP)
// ---------------------------------------------------------------------------

#[test]
fn lifecycle_emitted_even_when_action_fails_for_sp_shortfall() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let agumon = pilot(&roster, "Agumon");
    spawn_from_def(&mut app, &agumon, 100, 50, 0);
    spawn_sturdy_enemy(&mut app, UnitId(20), "SturdyDummy", 500, 500);

    // Force SP to zero so baby_flame (sp_cost=4) cannot be afforded
    app.world_mut().resource_mut::<SpPool>().current = 0;

    let mut cursor = message_cursor::<CombatEvent>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: agumon.id,
        skill_id: SkillId("baby_flame".into()),
        target: UnitId(20),
    });
    app.update();

    let events = drain_messages(&mut cursor, &app);
    let dump = event_dump(&events);

    let pos_declared = pos_of(&events, |e| {
        matches!(&e.kind, CombatEventKind::OnActionDeclared { intent_kind } if *intent_kind == ActionIntentKind::Skill)
            && e.follow_up_depth == 0
    });
    let pos_preapp = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionPreApp) && e.follow_up_depth == 0
    });
    let pos_failed = pos_of(&events, |e| {
        matches!(&e.kind, CombatEventKind::OnActionFailed { reason } if reason.contains("SP"))
            && e.follow_up_depth == 0
    });
    let pos_applied = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionApplied) && e.follow_up_depth == 0
    });
    let pos_resolved = pos_of(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionResolved) && e.follow_up_depth == 0
    });

    assert!(
        pos_declared.is_some(),
        "OnActionDeclared{{Skill}} missing\n{dump}"
    );
    assert!(pos_preapp.is_some(), "OnActionPreApp missing\n{dump}");
    assert!(
        pos_failed.is_some(),
        "OnActionFailed{{SP}} missing — lifecycle must report SP shortfall\n{dump}"
    );
    assert!(
        pos_applied.is_some(),
        "OnActionApplied missing — must be emitted even after failure\n{dump}"
    );
    assert!(
        pos_resolved.is_some(),
        "OnActionResolved missing — must close the lifecycle even after failure\n{dump}"
    );

    let (d, p, f, a, r) = (
        pos_declared.unwrap(),
        pos_preapp.unwrap(),
        pos_failed.unwrap(),
        pos_applied.unwrap(),
        pos_resolved.unwrap(),
    );

    assert!(d < p, "Declared must precede PreApp\n{dump}");
    assert!(p < f, "PreApp must precede OnActionFailed\n{dump}");
    assert!(f < a, "OnActionFailed must precede Applied\n{dump}");
    assert!(a < r, "Applied must precede Resolved\n{dump}");

    // No damage events — action aborted before effects
    let has_damage = events
        .iter()
        .any(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }));
    assert!(
        !has_damage,
        "OnDamageDealt must not appear when action fails due to SP shortfall\n{dump}"
    );
}
