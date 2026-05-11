use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    follow_up::{
        FollowUpDecision, FollowUpIntent, FollowUpTrace, follow_up_listener_system,
        resolve_follow_up_action_system,
    },
    kit::{FollowUpTrigger, UnitSkills},
    log::{ActionLog, LogEntry},
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
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
    let handle = assets.add(skill_book);
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
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

fn spawn_custom_enemy(
    app: &mut App,
    id: UnitId,
    name: &str,
    hp_current: i32,
    toughness_current: i32,
    attribute: Attribute,
    basic_skill: &str,
    ultimate_skill: &str,
    weaknesses: Vec<DamageTag>,
) {
    app.world_mut().spawn((
        Unit {
            id,
            name: name.into(),
            hp_max: hp_current,
            hp_current,
            attribute,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        Toughness {
            max: toughness_current,
            current: toughness_current,
            weaknesses,
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
            basic: SkillId(basic_skill.into()),
            skills: vec![SkillId(basic_skill.into())],
            ultimate: SkillId(ultimate_skill.into()),
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

fn action_log_entries(app: &App) -> Vec<LogEntry> {
    app.world()
        .resource::<ActionLog>()
        .events
        .iter()
        .cloned()
        .collect()
}

fn event_kind_json(event: &CombatEventKind) -> String {
    match event {
        CombatEventKind::OnSkillCast { skill_id } => {
            format!(
                "{{\"kind\":\"OnSkillCast\",\"skill_id\":\"{}\"}}",
                skill_id.0
            )
        }
        CombatEventKind::OnDamageDealt { amount, kind, .. } => format!(
            "{{\"kind\":\"OnDamageDealt\",\"amount\":{},\"damage_kind\":\"{:?}\"}}",
            amount, kind
        ),
        CombatEventKind::OnBreak { damage_tag } => {
            format!("{{\"kind\":\"OnBreak\",\"element\":\"{:?}\"}}", damage_tag)
        }
        CombatEventKind::OnKO => "{\"kind\":\"OnKO\"}".to_string(),
        CombatEventKind::OnAllyLowHp => "{\"kind\":\"OnAllyLowHp\"}".to_string(),
        CombatEventKind::OnEnemyKill => "{\"kind\":\"OnEnemyKill\"}".to_string(),
        CombatEventKind::OnRevive { hp_after } => {
            format!("{{\"kind\":\"OnRevive\",\"hp_after\":{}}}", hp_after)
        }
        CombatEventKind::OnActionFailed { reason } => {
            format!("{{\"kind\":\"OnActionFailed\",\"reason\":\"{}\"}}", reason)
        }
        CombatEventKind::OnStatusTick { kind, turns_left } => format!(
            "{{\"kind\":\"OnStatusTick\",\"effect\":\"{:?}\",\"turns_left\":{}}}",
            kind, turns_left
        ),
        CombatEventKind::OnStatusExpired { kind } => {
            format!("{{\"kind\":\"OnStatusExpired\",\"effect\":\"{:?}\"}}", kind)
        }
        CombatEventKind::PartySelected { .. } => "{\"kind\":\"PartySelected\"}".to_string(),
        CombatEventKind::TurnOrderSeeded { .. } => "{\"kind\":\"TurnOrderSeeded\"}".to_string(),
        CombatEventKind::UltGain { unit_id, amount } => {
            format!(
                "{{\"kind\":\"UltGain\",\"unit_id\":{},\"amount\":{}}}",
                unit_id.0, amount
            )
        }
        CombatEventKind::OnHitTaken { amount } => {
            format!("{{\"kind\":\"OnHitTaken\",\"amount\":{}}}", amount)
        }
        CombatEventKind::OnStatusApplied { kind } => {
            format!("{{\"kind\":\"OnStatusApplied\",\"effect\":\"{:?}\"}}", kind)
        }
        CombatEventKind::TurnAdvance { target, amount_pct } => format!(
            "{{\"kind\":\"TurnAdvance\",\"target\":{},\"amount_pct\":{}}}",
            target.0, amount_pct
        ),
        _ => "{\"kind\":\"Other\"}".to_string(),
    }
}

fn trace_jsonl(events: &[CombatEvent], traces: &[FollowUpTrace], log: &[LogEntry]) -> String {
    let mut lines = Vec::new();

    for event in events {
        lines.push(format!(
            "{{\"type\":\"combat_event\",\"source\":{},\"target\":{},\"follow_up_depth\":{},\"event\":{}}}",
            event.source.0,
            event.target.0,
            event.follow_up_depth,
            event_kind_json(&event.kind)
        ));
    }

    for trace in traces {
        let decision = match &trace.decision {
            FollowUpDecision::Scheduled => "{\"decision\":\"scheduled\"}".to_string(),
            FollowUpDecision::Suppressed { reason } => format!(
                "{{\"decision\":\"suppressed\",\"reason\":\"{:?}\"}}",
                reason
            ),
        };
        let target = trace
            .follow_up_target
            .map(|id| id.0.to_string())
            .unwrap_or_else(|| "null".to_string());
        lines.push(format!(
            "{{\"type\":\"follow_up_trace\",\"follower\":{},\"trigger\":\"{:?}\",\"action\":\"{}\",\"origin_source\":{},\"origin_target\":{},\"origin_kind\":{},\"follow_up_target\":{},\"decision\":{}}}",
            trace.follower.0,
            trace.trigger,
            trace.action.0,
            trace.origin_source.0,
            trace.origin_target.0,
            event_kind_json(&trace.origin_kind),
            target,
            decision
        ));
    }

    for entry in log {
        let payload = match entry {
            LogEntry::BasicHit {
                attacker,
                target,
                amount,
                kind,
            } => format!(
                "{{\"kind\":\"BasicHit\",\"attacker\":{},\"target\":{},\"amount\":{},\"damage_kind\":\"{:?}\"}}",
                attacker.0, target.0, amount, kind
            ),
            LogEntry::Break { target, damage_tag } => format!(
                "{{\"kind\":\"Break\",\"target\":{},\"element\":\"{:?}\"}}",
                target.0, damage_tag
            ),
            LogEntry::Ko { target } => {
                format!("{{\"kind\":\"Ko\",\"target\":{}}}", target.0)
            }
            LogEntry::Revive { target, hp_after } => format!(
                "{{\"kind\":\"Revive\",\"target\":{},\"hp_after\":{}}}",
                target.0, hp_after
            ),
            LogEntry::ActionFailed { reason } => {
                format!("{{\"kind\":\"ActionFailed\",\"reason\":\"{}\"}}", reason)
            }
            LogEntry::TurnAdvance { target, amount_pct } => format!(
                "{{\"kind\":\"TurnAdvance\",\"target\":{},\"amount_pct\":{}}}",
                target.0, amount_pct
            ),
        };
        lines.push(format!("{{\"type\":\"action_log\",\"entry\":{payload}}}"));
    }

    lines.join("\n")
}

fn unit_hp(app: &mut App, unit_id: UnitId) -> i32 {
    let mut query = app.world_mut().query::<&Unit>();
    query
        .iter(app.world())
        .find(|unit| unit.id == unit_id)
        .unwrap_or_else(|| panic!("missing unit {:?}", unit_id))
        .hp_current
}

#[test]
fn s10_agumon_break_follow_up_uses_real_pilot_config() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let agumon = pilot(&roster, "Agumon");
    let renamon = pilot(&roster, "Renamon");

    spawn_from_def(&mut app, &agumon, 100, 50, 0);
    spawn_from_def(&mut app, &renamon, 90, 45, 0);
    spawn_custom_enemy(
        &mut app,
        UnitId(4),
        "Tentomon",
        100,
        10,
        Attribute::Vaccine,
        "enemy_skill_fire",
        "tentomon_ult",
        vec![DamageTag::Fire],
    );

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);
    let mut trace_cursor = message_cursor::<FollowUpTrace>(&mut app);

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: agumon.id,
        target: UnitId(4),
    });
    app.update();

    let events = drain_messages(&mut event_cursor, &app);
    let traces = drain_messages(&mut trace_cursor, &app);
    let log = action_log_entries(&app);
    let trace = trace_jsonl(&events, &traces, &log);

    assert!(
        events.iter().any(|event| {
            matches!(event.kind, CombatEventKind::OnBreak { .. })
                && event.source == agumon.id
                && event.target == UnitId(4)
                && event.follow_up_depth == 0
        }),
        "missing root break event\n{trace}"
    );
    assert!(
        traces.iter().any(|entry| {
            entry.follower == agumon.id
                && entry.trigger == FollowUpTrigger::OnEnemyBreak
                && entry.origin_source == agumon.id
                && entry.origin_target == UnitId(4)
                && entry.follow_up_target == Some(UnitId(4))
                && entry.decision == FollowUpDecision::Scheduled
        }),
        "missing Agumon follow-up scheduling evidence\n{trace}"
    );
    assert!(
        events.iter().any(|event| {
            matches!(
                &event.kind,
                CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("agumon_follow_up".into())
            ) && event.source == agumon.id
                && event.target == UnitId(4)
                && event.follow_up_depth == 1
        }),
        "missing Agumon follow-up event\n{trace}"
    );
    assert_eq!(
        log.iter()
            .filter(|entry| {
                matches!(
                    entry,
                    LogEntry::BasicHit {
                        attacker,
                        target,
                        ..
                    } if *attacker == agumon.id && *target == UnitId(4)
                )
            })
            .count(),
        2,
        "expected root hit plus Agumon follow-up\n{trace}"
    );
    assert_eq!(
        unit_hp(&mut app, UnitId(4)),
        49,
        "follow-up should leave the enemy at the expected deterministic HP\n{trace}"
    );
}

#[test]
fn s10_renamon_low_hp_follow_up_targets_the_attacker() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let agumon = pilot(&roster, "Agumon");
    let renamon = pilot(&roster, "Renamon");

    spawn_from_def(&mut app, &agumon, 40, 50, 0);
    spawn_from_def(&mut app, &renamon, 90, 45, 0);
    spawn_custom_enemy(
        &mut app,
        UnitId(4),
        "Tentomon",
        100,
        50,
        Attribute::Vaccine,
        "enemy_skill_fire",
        "tentomon_ult",
        vec![DamageTag::Fire],
    );

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);
    let mut trace_cursor = message_cursor::<FollowUpTrace>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(4),
        skill_id: SkillId("enemy_skill_fire".into()),
        target: agumon.id,
    });
    app.update();

    let events = drain_messages(&mut event_cursor, &app);
    let traces = drain_messages(&mut trace_cursor, &app);
    let log = action_log_entries(&app);
    let trace = trace_jsonl(&events, &traces, &log);

    assert!(
        events.iter().any(|event| {
            matches!(event.kind, CombatEventKind::OnAllyLowHp)
                && event.source == agumon.id
                && event.target == agumon.id
                && event.follow_up_depth == 0
        }),
        "missing low-HP threshold event\n{trace}"
    );
    assert!(
        traces.iter().any(|entry| {
            entry.follower == renamon.id
                && entry.trigger == FollowUpTrigger::OnAllyLowHp
                && entry.origin_source == agumon.id
                && entry.origin_target == agumon.id
                && entry.follow_up_target == Some(UnitId(4))
                && entry.decision == FollowUpDecision::Scheduled
        }),
        "missing Renamon follow-up scheduling evidence\n{trace}"
    );
    assert!(
        events.iter().any(|event| {
            matches!(
                &event.kind,
                CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("renamon_follow_up".into())
            ) && event.source == renamon.id
                && event.target == UnitId(4)
                && event.follow_up_depth == 1
        }),
        "missing Renamon follow-up event\n{trace}"
    );
    assert!(
        log.iter().any(|entry| {
            matches!(
                entry,
                LogEntry::BasicHit {
                    attacker,
                    target,
                    ..
                } if *attacker == UnitId(4) && *target == agumon.id
            )
        }),
        "missing enemy opening hit\n{trace}"
    );
    assert!(
        log.iter().any(|entry| {
            matches!(
                entry,
                LogEntry::BasicHit {
                    attacker,
                    target,
                    ..
                } if *attacker == renamon.id && *target == UnitId(4)
            )
        }),
        "missing Renamon follow-up hit\n{trace}"
    );
}

#[test]
fn s10_control_scenario_without_matching_trigger_has_no_follow_up_action() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let agumon = pilot(&roster, "Agumon");

    spawn_from_def(&mut app, &agumon, 100, 50, 0);
    spawn_custom_enemy(
        &mut app,
        UnitId(4),
        "Tentomon",
        100,
        50,
        Attribute::Vaccine,
        "enemy_skill_fire",
        "tentomon_ult",
        vec![DamageTag::Fire],
    );

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);
    let mut trace_cursor = message_cursor::<FollowUpTrace>(&mut app);

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: agumon.id,
        target: UnitId(4),
    });
    app.update();

    let events = drain_messages(&mut event_cursor, &app);
    let traces = drain_messages(&mut trace_cursor, &app);
    let log = action_log_entries(&app);
    let trace = trace_jsonl(&events, &traces, &log);

    assert!(
        !events.iter().any(|event| event.follow_up_depth == 1),
        "control scenario should not emit follow-up events\n{trace}"
    );
    assert!(
        !traces
            .iter()
            .any(|entry| entry.decision == FollowUpDecision::Scheduled),
        "control scenario should not schedule a follow-up\n{trace}"
    );
    assert_eq!(
        log.iter()
            .filter(|entry| matches!(entry, LogEntry::BasicHit { .. }))
            .count(),
        1,
        "control scenario should keep exactly one hit in the log\n{trace}"
    );
}

// ==================== T02 — 7 nuovi test per i follow-up aggiunti in T01 ====================

// --- OnAllyLowHp group (Gabumon, Patamon, Plotmon) ---

#[test]
fn gabumon_triggers_follow_up_on_ally_low_hp() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let gabumon = pilot(&roster, "Gabumon"); // UnitId(2), OnAllyLowHp
    let agumon = pilot(&roster, "Agumon"); // UnitId(1), decoy at low HP

    spawn_from_def(&mut app, &gabumon, 95, 48, 0);
    spawn_from_def(&mut app, &agumon, 40, 50, 0);
    spawn_custom_enemy(
        &mut app,
        UnitId(11),
        "DummyA",
        100,
        50,
        Attribute::Vaccine,
        "enemy_skill_fire",
        "tentomon_ult",
        vec![DamageTag::Fire],
    );

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);
    let mut trace_cursor = message_cursor::<FollowUpTrace>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(11),
        skill_id: SkillId("enemy_skill_fire".into()),
        target: agumon.id,
    });
    app.update();

    let events = drain_messages(&mut event_cursor, &app);
    let traces = drain_messages(&mut trace_cursor, &app);
    let log = action_log_entries(&app);
    let trace = trace_jsonl(&events, &traces, &log);

    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, CombatEventKind::OnAllyLowHp)
                && e.source == agumon.id
                && e.follow_up_depth == 0),
        "missing OnAllyLowHp event\n{trace}"
    );
    assert!(
        traces.iter().any(|t| {
            t.follower == gabumon.id
                && t.trigger == FollowUpTrigger::OnAllyLowHp
                && t.decision == FollowUpDecision::Scheduled
                && t.follow_up_target == Some(UnitId(11))
        }),
        "missing Gabumon follow-up scheduling evidence\n{trace}"
    );
    assert!(
        events.iter().any(|e| matches!(
            &e.kind,
            CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("gabumon_follow_up".into())
        ) && e.source == gabumon.id && e.follow_up_depth == 1),
        "missing Gabumon OnSkillCast at depth 1\n{trace}"
    );
    assert!(
        log.iter().any(|e| matches!(
            e,
            LogEntry::BasicHit { attacker, target, .. }
                if *attacker == gabumon.id && *target == UnitId(11)
        )),
        "missing Gabumon follow-up BasicHit\n{trace}"
    );
}

#[test]
fn patamon_triggers_follow_up_on_ally_low_hp() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let patamon = pilot(&roster, "Patamon"); // UnitId(9), OnAllyLowHp
    let agumon = pilot(&roster, "Agumon"); // UnitId(1), decoy at low HP

    spawn_from_def(&mut app, &patamon, 88, 44, 0);
    spawn_from_def(&mut app, &agumon, 40, 50, 0);
    spawn_custom_enemy(
        &mut app,
        UnitId(11),
        "DummyA",
        100,
        50,
        Attribute::Vaccine,
        "enemy_skill_fire",
        "tentomon_ult",
        vec![DamageTag::Fire],
    );

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);
    let mut trace_cursor = message_cursor::<FollowUpTrace>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(11),
        skill_id: SkillId("enemy_skill_fire".into()),
        target: agumon.id,
    });
    app.update();

    let events = drain_messages(&mut event_cursor, &app);
    let traces = drain_messages(&mut trace_cursor, &app);
    let log = action_log_entries(&app);
    let trace = trace_jsonl(&events, &traces, &log);

    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, CombatEventKind::OnAllyLowHp)
                && e.source == agumon.id
                && e.follow_up_depth == 0),
        "missing OnAllyLowHp event\n{trace}"
    );
    assert!(
        traces.iter().any(|t| {
            t.follower == patamon.id
                && t.trigger == FollowUpTrigger::OnAllyLowHp
                && t.decision == FollowUpDecision::Scheduled
                && t.follow_up_target == Some(UnitId(11))
        }),
        "missing Patamon follow-up scheduling evidence\n{trace}"
    );
    assert!(
        events.iter().any(|e| matches!(
            &e.kind,
            CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("patamon_follow_up".into())
        ) && e.source == patamon.id && e.follow_up_depth == 1),
        "missing Patamon OnSkillCast at depth 1\n{trace}"
    );
    assert!(
        log.iter().any(|e| matches!(
            e,
            LogEntry::BasicHit { attacker, target, .. }
                if *attacker == patamon.id && *target == UnitId(11)
        )),
        "missing Patamon follow-up BasicHit\n{trace}"
    );
}

// --- OnEnemyKill group (V-mon, Dorumon) ---

#[test]
fn dorumon_triggers_follow_up_on_enemy_kill() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let dorumon = pilot(&roster, "Dorumon"); // UnitId(5), OnEnemyKill
    let agumon = pilot(&roster, "Agumon"); // UnitId(1), fires the killing ultimate

    spawn_from_def(&mut app, &dorumon, 90, 47, 0);
    spawn_from_def(&mut app, &agumon, 100, 50, 100); // ult ready
    spawn_custom_enemy(
        &mut app,
        UnitId(11),
        "DummyA",
        40,
        50, // toughness 50 > ult ToughnessHit(30) — no break, only kill
        Attribute::Vaccine,
        "enemy_skill_fire",
        "tentomon_ult",
        vec![DamageTag::Fire],
    );
    spawn_custom_enemy(
        &mut app,
        UnitId(12),
        "DummyB",
        100,
        45,
        Attribute::Virus,
        "enemy_skill_fire",
        "biyomon_ult",
        vec![DamageTag::Fire],
    );

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);
    let mut trace_cursor = message_cursor::<FollowUpTrace>(&mut app);

    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: agumon.id,
        target: UnitId(11),
    });
    app.update();

    let events = drain_messages(&mut event_cursor, &app);
    let traces = drain_messages(&mut trace_cursor, &app);
    let log = action_log_entries(&app);
    let trace = trace_jsonl(&events, &traces, &log);

    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, CombatEventKind::OnEnemyKill)
                && e.source == agumon.id
                && e.target == UnitId(11)
                && e.follow_up_depth == 0),
        "missing root OnEnemyKill event\n{trace}"
    );
    assert!(
        traces.iter().any(|t| {
            t.follower == dorumon.id
                && t.trigger == FollowUpTrigger::OnEnemyKill
                && t.decision == FollowUpDecision::Scheduled
                && t.follow_up_target == Some(UnitId(12))
        }),
        "missing Dorumon follow-up scheduling evidence\n{trace}"
    );
    assert!(
        events.iter().any(|e| matches!(
            &e.kind,
            CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("dorumon_follow_up".into())
        ) && e.source == dorumon.id && e.target == UnitId(12) && e.follow_up_depth == 1),
        "missing Dorumon OnSkillCast at depth 1\n{trace}"
    );
    assert!(
        log.iter()
            .any(|e| matches!(e, LogEntry::Ko { target } if *target == UnitId(11))),
        "missing root KO log entry\n{trace}"
    );
    assert!(
        log.iter().any(|e| matches!(
            e,
            LogEntry::BasicHit { attacker, target, .. }
                if *attacker == dorumon.id && *target == UnitId(12)
        )),
        "missing Dorumon follow-up BasicHit\n{trace}"
    );
}

// --- OnEnemyBreak group (Hackmon, Guilmon) ---
