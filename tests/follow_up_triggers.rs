use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    blueprints::register_all_blueprint_exts,
    events::{CombatEvent, CombatEventKind},
    follow_up::{
        FollowUpDecision, FollowUpIntent, FollowUpTrace, follow_up_listener_system,
        resolve_follow_up_action_system,
    },
    kit::{FollowUpConfig, FollowUpTrigger, UnitSkills},
    log::{ActionLog, LogEntry},
    rng::CombatRng,
    runtime::{
        ExtRegistries, SignalBus, SignalTaxonomy, register_kernel_builtins,
        timeline::{Beat, BeatEdge, BeatKind, BeatPayload, TimelineLibrary},
    },
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
    skill_timeline::{SkillTimeline, compile_skill_book_timelines},
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
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
            .expect("follow_up_triggers test book must compile");
        app.world_mut()
            .resource_mut::<TimelineLibrary<String>>()
            .timelines = compiled;
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
        CombatEventKind::UnitDied {
            status_remaining,
            heated_remaining,
        } => {
            format!(
                "{{\"kind\":\"UnitDied\",\"status_remaining\":{:?},\"heated_remaining\":{}}}",
                status_remaining, heated_remaining
            )
        }
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
        CombatEventKind::AdvanceTurn { target, amount_pct } => format!(
            "{{\"kind\":\"AdvanceTurn\",\"target\":{},\"amount_pct\":{}}}",
            target.0, amount_pct
        ),
        CombatEventKind::DelayTurn { target, amount_pct } => format!(
            "{{\"kind\":\"DelayTurn\",\"target\":{},\"amount_pct\":{}}}",
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
            LogEntry::AdvanceTurn { target, amount_pct } => format!(
                "{{\"kind\":\"AdvanceTurn\",\"target\":{},\"amount_pct\":{}}}",
                target.0, amount_pct
            ),
            LogEntry::DelayTurn { target, amount_pct } => format!(
                "{{\"kind\":\"DelayTurn\",\"target\":{},\"amount_pct\":{}}}",
                target.0, amount_pct
            ),
        };
        lines.push(format!("{{\"type\":\"action_log\",\"entry\":{payload}}}"));
    }

    lines.join("\n")
}

fn agumon_follow_up_snapshot(
    events: &[CombatEvent],
    traces: &[FollowUpTrace],
    log: &[LogEntry],
    enemy_hp: i32,
) -> String {
    let salient_events = events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                CombatEventKind::OnDamageDealt { .. }
                    | CombatEventKind::OnBreak { .. }
                    | CombatEventKind::OnSkillCast { .. }
                    | CombatEventKind::OnHitTaken { .. }
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let salient_traces = traces
        .iter()
        .filter(|trace| {
            trace.decision == FollowUpDecision::Scheduled
                || matches!(trace.origin_kind, CombatEventKind::OnBreak { .. })
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut snapshot = trace_jsonl(&salient_events, &salient_traces, log);
    snapshot.push_str(&format!(
        "\n{{\"type\":\"unit_hp\",\"unit_id\":4,\"hp\":{enemy_hp}}}"
    ));
    snapshot
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
fn agumon_break_follow_up_uses_real_pilot_config() {
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
    let enemy_hp = unit_hp(&mut app, UnitId(4));
    let snapshot = agumon_follow_up_snapshot(&events, &traces, &log, enemy_hp);

    insta::assert_snapshot!("agumon_break_follow_up_uses_real_pilot_config", snapshot);

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
        enemy_hp, 49,
        "follow-up should leave the enemy at the expected deterministic HP\n{trace}"
    );
}

#[test]
fn renamon_low_hp_follow_up_targets_the_attacker() {
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
fn control_scenario_without_matching_trigger_has_no_follow_up_action() {
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
        "enemy_ult_fire",
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

// --- OnEnemyBreak engine path (synthetic, no blueprint exts) ---
//
// The real-pilot OnEnemyBreak coverage lives in
// `agumon_break_follow_up_uses_real_pilot_config`. This test deliberately runs
// the listener/resolver chain on a minimal synthetic skill book registered with
// `register_kernel_builtins` ONLY — no `register_all_blueprint_exts`. It guards
// the architectural invariant that follow-up resolution is decoupled from
// blueprint extensions: a regression that coupled the resolver to a blueprint
// ext would still pass every real-pilot test but fail here.

fn synthetic_unit(id: u32, attribute: Attribute, hp_max: i32, hp_current: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn synthetic_skill(
    id: &str,
    damage_tag: DamageTag,
    damage: i32,
    toughness_damage: i32,
) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![
            Effect::Damage {
                amount: damage,
                target: TargetShape::Single,
                per_hop: Default::default(),
            },
            Effect::ToughnessHit(toughness_damage),
        ],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
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
                    id: "impact_damage".into(),
                    kind: BeatKind::Impact,
                    hook: Some("core/deal_damage".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(BeatPayload::DealDamage {
                        amount: damage,
                        tag: damage_tag,
                        target: TargetShape::Single,
                    }),
                },
                Beat {
                    id: "impact_break".into(),
                    kind: BeatKind::Impact,
                    hook: Some("core/apply_effect".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(BeatPayload::BreakToughness {
                        amount: toughness_damage,
                        tag: damage_tag,
                        target: TargetShape::Single,
                    }),
                },
            ],
            edges: vec![
                BeatEdge {
                    from: "cast".into(),
                    to: "impact_damage".into(),
                    gate: Some("core/always".into()),
                },
                BeatEdge {
                    from: "impact_damage".into(),
                    to: "impact_break".into(),
                    gate: Some("core/always".into()),
                },
            ],
        }),
    }
}

fn setup_engine_app(book: SkillBook) -> App {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(42))
        .insert_resource(TimelineLibrary::<String>::default())
        .init_resource::<SignalBus>()
        .init_resource::<ExtRegistries>()
        .init_resource::<SignalTaxonomy>()
        .add_message::<ActionIntent>()
        .add_message::<FollowUpIntent>()
        .add_message::<FollowUpTrace>()
        .add_message::<CombatEvent>()
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
    let handle = assets.add(book.clone());
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        register_kernel_builtins(&mut regs);
        let compiled = compile_skill_book_timelines(&book, &regs)
            .expect("synthetic engine test book must compile");
        app.world_mut()
            .resource_mut::<TimelineLibrary<String>>()
            .timelines = compiled;
    }
    app
}

fn spawn_engine_combatant(
    app: &mut App,
    unit: Unit,
    team: Team,
    toughness_max: i32,
    weaknesses: Vec<DamageTag>,
    skills: UnitSkills,
) {
    app.world_mut().spawn((
        unit,
        team,
        Toughness::new(toughness_max, weaknesses),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        skills,
    ));
}

#[test]
fn follow_up_resolves_in_same_update_without_blueprint_exts() {
    let mut app = setup_engine_app(SkillBook(vec![
        synthetic_skill("breaker", DamageTag::Fire, 8, 10),
        synthetic_skill("ally_follow_up", DamageTag::Light, 6, 3),
        synthetic_skill("enemy_basic", DamageTag::Ice, 4, 0),
    ]));

    spawn_engine_combatant(
        &mut app,
        synthetic_unit(1, Attribute::Vaccine, 100, 100),
        Team::Ally,
        40,
        vec![],
        UnitSkills {
            basic: SkillId("breaker".into()),
            skills: vec![SkillId("breaker".into())],
            ultimate: SkillId("breaker".into()),
            follow_up: None,
        },
    );
    spawn_engine_combatant(
        &mut app,
        synthetic_unit(2, Attribute::Data, 90, 90),
        Team::Ally,
        35,
        vec![],
        UnitSkills {
            basic: SkillId("ally_follow_up".into()),
            skills: vec![SkillId("ally_follow_up".into())],
            ultimate: SkillId("ally_follow_up".into()),
            follow_up: Some(FollowUpConfig {
                trigger: FollowUpTrigger::OnEnemyBreak,
                action: SkillId("ally_follow_up".into()),
            }),
        },
    );
    spawn_engine_combatant(
        &mut app,
        synthetic_unit(4, Attribute::Virus, 100, 100),
        Team::Enemy,
        5,
        vec![DamageTag::Fire],
        UnitSkills {
            basic: SkillId("enemy_basic".into()),
            skills: vec![SkillId("enemy_basic".into())],
            ultimate: SkillId("enemy_basic".into()),
            follow_up: None,
        },
    );

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("breaker".into()),
        target: UnitId(4),
    });

    app.update();

    let events = drain_messages(&mut event_cursor, &app);
    assert!(
        events.iter().any(|event| {
            event.follow_up_depth == 1 && event.source == UnitId(2) && event.target == UnitId(4)
        }),
        "follow-up should resolve at depth 1 within the same update"
    );

    let hits: Vec<(UnitId, UnitId)> = action_log_entries(&app)
        .iter()
        .filter_map(|entry| match entry {
            LogEntry::BasicHit {
                attacker, target, ..
            } => Some((*attacker, *target)),
            _ => None,
        })
        .collect();
    assert!(
        hits.contains(&(UnitId(1), UnitId(4))),
        "missing root hit 1 -> 4"
    );
    assert!(
        hits.contains(&(UnitId(2), UnitId(4))),
        "missing follow-up hit 2 -> 4 in the same update"
    );
}
