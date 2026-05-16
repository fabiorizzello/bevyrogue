use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
    events::{CombatEvent, CombatEventKind},
    follow_up::{
        FollowUpIntent, FollowUpTrace, follow_up_listener_system, resolve_follow_up_action_system,
    },
    kit::UnitSkills,
    log::{ActionLog, LogEntry},
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system, check_victory_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{
        UltAccumulationTrigger, UltGainQueue, UltimateCharge, flush_ult_gain_system,
        ult_accumulation_system,
    },
    unit::{Ko, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
    units_ron::{UnitDef, UnitRoster},
};

fn canonical_roster() -> UnitRoster {
    ron::from_str(include_str!("../assets/data/units.ron")).expect("parse units.ron")
}

fn canonical_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
}

fn skill_book_with_shock_fixture() -> SkillBook {
    let mut book = canonical_skill_book();
    book.0.push(SkillDef {
        id: SkillId("para_bolt".into()),
        name: "Para Bolt".into(),
        damage_tag: DamageTag::Dark,
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
                amount: 15,
                target: TargetShape::Single,
            per_hop: Default::default(),
            },
            Effect::ToughnessHit(8),
            Effect::ApplyStatus {
                kind: StatusEffectKind::Paralyzed,
                duration: 1,
            },
        ],

        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    });
    book
}

fn sp_fixture_skill_book() -> SkillBook {
    SkillBook(vec![
        SkillDef {
            id: SkillId("ally_basic".into()),
            name: "Ally Basic".into(),
            damage_tag: DamageTag::Fire,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![Effect::Damage {
                amount: 8,
                target: TargetShape::Single,
            per_hop: Default::default(),
            }],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
        timeline: None,
        },
        SkillDef {
            id: SkillId("ally_skill_3".into()),
            name: "Ally Skill 3".into(),
            damage_tag: DamageTag::Ice,
            sp_cost: 3,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![Effect::Damage {
                amount: 16,
                target: TargetShape::Single,
            per_hop: Default::default(),
            }],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
        timeline: None,
        },
        SkillDef {
            id: SkillId("ally_skill_4".into()),
            name: "Ally Skill 4".into(),
            damage_tag: DamageTag::Light,
            sp_cost: 4,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![Effect::Damage {
                amount: 18,
                target: TargetShape::Single,
            per_hop: Default::default(),
            }],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
        timeline: None,
        },
        SkillDef {
            id: SkillId("holy_revive".into()),
            name: "Holy Revive".into(),
            damage_tag: DamageTag::Light,
            sp_cost: 5,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Ally,
                life: TargetLife::Ko,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![Effect::Revive(25)],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
        timeline: None,
        },
        SkillDef {
            id: SkillId("enemy_smash".into()),
            name: "Enemy Smash".into(),
            damage_tag: DamageTag::Dark,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![Effect::Damage {
                amount: 9999,
                target: TargetShape::Single,
            per_hop: Default::default(),
            }],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
        timeline: None,
        },
    ])
}

fn pilot(roster: &UnitRoster, name: &str) -> UnitDef {
    roster
        .0
        .iter()
        .find(|unit| unit.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("missing pilot {name}"))
}

fn insert_skill_book(app: &mut App, skill_book: SkillBook) {
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(skill_book);
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
}

fn build_follow_up_app(skill_book: SkillBook) -> App {
    let mut app = App::new();
    insert_skill_book(&mut app, skill_book);
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<ActionLog>()
        .init_resource::<UltGainQueue>()
        .insert_resource(SpPool {
            current: 999,
            max: 999,
        })
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
                ult_accumulation_system,
                flush_ult_gain_system,
                check_victory_system,
            )
                .chain(),
        );
    app
}

fn build_sp_app(skill_book: SkillBook) -> App {
    let mut app = App::new();
    insert_skill_book(&mut app, skill_book);
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<ActionLog>()
        .insert_resource(SpPool { current: 5, max: 5 })
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<FollowUpTrace>()
        .add_systems(Update, resolve_action_system);
    app
}

fn build_status_app(skill_book: SkillBook) -> App {
    let mut app = App::new();
    insert_skill_book(&mut app, skill_book);
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<ActionLog>()
        .init_resource::<UltGainQueue>()
        .insert_resource(SpPool {
            current: 999,
            max: 999,
        })
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<FollowUpIntent>()
        .add_message::<FollowUpTrace>()
        .add_message::<TurnAdvanced>()
        .add_message::<bevyrogue::combat::av::ActionValueUpdated>()
        .add_systems(
            Update,
            (
                resolve_action_system,
                follow_up_listener_system,
                resolve_follow_up_action_system,
                ult_accumulation_system,
                flush_ult_gain_system,
                advance_turn_system,
                check_victory_system,
            )
                .chain(),
        );
    app
}

fn spawn_from_def(
    app: &mut App,
    def: &UnitDef,
    hp_current: i32,
    toughness_current: i32,
    ultimate_current: i32,
) -> Entity {
    let mut toughness = Toughness::new(def.toughness_max, def.weaknesses.clone());
    toughness.current = toughness_current;

    app.world_mut()
        .spawn((
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
            toughness,
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
            StatusBag::default(),
        ))
        .id()
}

fn spawn_custom_enemy(
    app: &mut App,
    id: UnitId,
    name: &str,
    hp_max: i32,
    hp_current: i32,
    toughness_max: i32,
    toughness_current: i32,
    attribute: Attribute,
    basic_skill: &str,
    skill_ids: Vec<&str>,
    ultimate_skill: &str,
    weaknesses: Vec<DamageTag>,
) -> Entity {
    let mut toughness = Toughness::new(toughness_max, weaknesses);
    toughness.current = toughness_current;

    app.world_mut()
        .spawn((
            Unit {
                id,
                name: name.into(),
                hp_max,
                hp_current,
                attribute,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            toughness,
            UltimateCharge {
                current: 0,
                trigger: 100,
                cap: 150,
                trigger_type: UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 25,
            },
            UnitSkills {
                basic: SkillId(basic_skill.into()),
                skills: skill_ids.into_iter().map(|id| SkillId(id.into())).collect(),
                ultimate: SkillId(ultimate_skill.into()),
                follow_up: None,
            },
            StatusBag::default(),
        ))
        .id()
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

fn unit_hp(app: &mut App, unit_id: UnitId) -> i32 {
    let world = app.world_mut();
    let mut query = world.query::<&Unit>();
    query
        .iter(world)
        .find(|unit| unit.id == unit_id)
        .unwrap_or_else(|| panic!("missing unit {:?}", unit_id))
        .hp_current
}

fn ult_current(app: &mut App, unit_id: UnitId) -> i32 {
    let world = app.world_mut();
    let mut query = world.query::<(&Unit, &UltimateCharge)>();
    query
        .iter(world)
        .find(|(unit, _)| unit.id == unit_id)
        .unwrap_or_else(|| panic!("missing ult charge for {:?}", unit_id))
        .1
        .current
}

fn status_effect_kind(app: &mut App, unit_id: UnitId) -> Option<StatusEffectKind> {
    let world = app.world_mut();
    let mut query = world.query::<(&Unit, Option<&StatusBag>)>();
    query
        .iter(world)
        .find(|(unit, _)| unit.id == unit_id)
        .and_then(|(_, bag_opt)| {
            bag_opt.and_then(|bag| bag.iter().next().map(|inst| inst.kind.clone()))
        })
}

fn is_ko(app: &mut App, unit_id: UnitId) -> bool {
    let world = app.world_mut();
    let mut query = world.query::<(&Unit, Option<&Ko>)>();
    query
        .iter(world)
        .any(|(unit, ko)| unit.id == unit_id && ko.is_some())
}

fn trace_kind_json(kind: &CombatEventKind) -> String {
    match kind {
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
        CombatEventKind::UnitDied { status_remaining, heated_remaining } => {
            format!("{{\"kind\":\"UnitDied\",\"status_remaining\":{:?},\"heated_remaining\":{}}}", status_remaining, heated_remaining)
        }
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
        CombatEventKind::OnAllyLowHp => "{\"kind\":\"OnAllyLowHp\"}".to_string(),
        CombatEventKind::OnEnemyKill => "{\"kind\":\"OnEnemyKill\"}".to_string(),
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

fn audit_trace(
    events: &[CombatEvent],
    traces: &[FollowUpTrace],
    log: &[LogEntry],
    sp_history: &[i32],
) -> String {
    let mut lines = vec![format!(
        "{{\"type\":\"summary\",\"combat_events\":{},\"follow_up_traces\":{},\"action_log_entries\":{},\"sp_samples\":{}}}",
        events.len(),
        traces.len(),
        log.len(),
        sp_history.len()
    )];

    for event in events {
        lines.push(format!(
            "{{\"type\":\"combat_event\",\"source\":{},\"target\":{},\"follow_up_depth\":{},\"event\":{}}}",
            event.source.0,
            event.target.0,
            event.follow_up_depth,
            trace_kind_json(&event.kind)
        ));
    }

    for trace in traces {
        let decision = match &trace.decision {
            bevyrogue::combat::follow_up::FollowUpDecision::Scheduled => {
                "{\"decision\":\"scheduled\"}".to_string()
            }
            bevyrogue::combat::follow_up::FollowUpDecision::Suppressed { reason } => format!(
                "{{\"decision\":\"suppressed\",\"reason\":\"{:?}\"}}",
                reason
            ),
        };
        let follow_up_target = trace
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
            trace_kind_json(&trace.origin_kind),
            follow_up_target,
            decision
        ));
    }

    for (index, sp) in sp_history.iter().enumerate() {
        lines.push(format!(
            "{{\"type\":\"sp_snapshot\",\"index\":{},\"current\":{}}}",
            index, sp
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

fn cast(app: &mut App, intent: ActionIntent, sp_history: &mut Vec<i32>) {
    app.world_mut().write_message(intent);
    app.update();
    sp_history.push(app.world().resource::<SpPool>().current);
}

#[test]
fn s_m008_s06_break_follow_up_and_ult_timing_trace() {
    let roster = canonical_roster();
    let mut app = build_follow_up_app(canonical_skill_book());

    let agumon = pilot(&roster, "Agumon");
    let greymon = pilot(&roster, "Greymon");
    let renamon = pilot(&roster, "Renamon");

    spawn_from_def(&mut app, &agumon, 100, 50, 0);
    spawn_from_def(&mut app, &greymon, 130, 60, 0);
    spawn_from_def(&mut app, &renamon, 90, 45, 0);
    spawn_custom_enemy(
        &mut app,
        UnitId(101),
        "Training Dummy",
        220,
        220,
        10,
        10,
        Attribute::Virus,
        "enemy_skill_fire",
        vec!["enemy_skill_fire"],
        "enemy_ult_fire",
        vec![DamageTag::Fire],
    );

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);
    let mut trace_cursor = message_cursor::<FollowUpTrace>(&mut app);

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: agumon.id,
        target: UnitId(101),
    });
    app.update();

    // Drain before update 2 so the message ring buffer doesn't prune update-1 entries.
    // FIFO follow-up queue processes one intent per update:
    // Agumon's follow-up fires in update 1, Hackmon's in update 2.
    let mut events = drain_messages(&mut event_cursor, &app);
    let mut traces = drain_messages(&mut trace_cursor, &app);

    app.update();

    assert_eq!(
        ult_current(&mut app, renamon.id),
        100,
        "timeline-backed root breaks now open the allied follow-up ultimate window"
    );

    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: renamon.id,
        target: UnitId(101),
    });
    app.update();

    events.extend(drain_messages(&mut event_cursor, &app));
    traces.extend(drain_messages(&mut trace_cursor, &app));
    let log = action_log_entries(&app);
    let trace = audit_trace(&events, &traces, &log, &[]);

    assert!(
        traces.iter().any(|entry| {
            entry.trigger == bevyrogue::combat::kit::FollowUpTrigger::OnEnemyBreak
                && entry.decision == bevyrogue::combat::follow_up::FollowUpDecision::Scheduled
                && entry.origin_source == agumon.id
                && entry.origin_target == UnitId(101)
        }),
        "timeline-backed root breaks should schedule allied break follow-ups\n{trace}"
    );
    assert!(
        events.iter().any(|event| {
            matches!(
                &event.kind,
                CombatEventKind::UltGain { unit_id, .. }
                    if *unit_id == renamon.id
            )
        }),
        "expected Renamon ult gain once allied follow-up triggers are wired\n{trace}"
    );
    assert!(
        events.iter().any(|event| {
            event.source == renamon.id
                && event.target == UnitId(101)
                && event.follow_up_depth == 0
                && matches!(
                    &event.kind,
                    CombatEventKind::OnSkillCast { skill_id }
                        if *skill_id == SkillId("renamon_ult".into())
                )
        }),
        "Renamon ultimate should unlock once the allied follow-up charge window exists\n{trace}"
    );
    assert!(
        trace.contains("\"type\":\"summary\"")
            && trace.contains("\"type\":\"follow_up_trace\"")
            && trace.contains("\"type\":\"action_log\""),
        "trace should be readable enough to diagnose the deferred follow-up surface\n{trace}"
    );
}

#[test]
fn s_m008_s06_shared_sp_history_blocks_and_then_unlocks_a_revive() {
    let mut app = build_sp_app(sp_fixture_skill_book());

    app.world_mut().spawn((
        Unit {
            id: UnitId(9),
            name: "Calibrator".into(),
            hp_max: 120,
            hp_current: 120,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        Toughness::new(1000, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("ally_basic".into()),
            skills: vec![
                SkillId("ally_skill_3".into()),
                SkillId("ally_skill_4".into()),
                SkillId("holy_revive".into()),
            ],
            ultimate: SkillId("ally_basic".into()),
            follow_up: None,
        },
    ));
    let victim_id = UnitId(102);
    app.world_mut().spawn((
        Unit {
            id: victim_id,
            name: "Downed Ally".into(),
            hp_max: 100,
            hp_current: 0,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        Ko,
        Toughness::new(1000, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 80,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 20,
        },
    ));
    spawn_custom_enemy(
        &mut app,
        UnitId(103),
        "Training Dummy",
        1_000,
        1_000,
        1_000,
        1_000,
        Attribute::Virus,
        "enemy_smash",
        vec!["enemy_smash"],
        "enemy_smash",
        vec![],
    );

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);
    let mut trace_cursor = message_cursor::<FollowUpTrace>(&mut app);
    let mut sp_history = vec![app.world().resource::<SpPool>().current];

    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(9),
            skill_id: SkillId("ally_skill_3".into()),
            target: UnitId(103),
        },
        &mut sp_history,
    );
    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(9),
            skill_id: SkillId("ally_skill_4".into()),
            target: UnitId(103),
        },
        &mut sp_history,
    );

    let failed_log_len = action_log_entries(&app).len();
    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(9),
            skill_id: SkillId("holy_revive".into()),
            target: victim_id,
        },
        &mut sp_history,
    );

    assert_eq!(
        sp_history[sp_history.len() - 1],
        2,
        "the blocked revive must leave SP unchanged at the gate"
    );
    assert!(
        is_ko(&mut app, victim_id),
        "the victim should remain KO while the revive is unaffordable"
    );
    assert_eq!(
        action_log_entries(&app).len(),
        failed_log_len,
        "the blocked revive should not emit a successful action log entry"
    );

    cast(
        &mut app,
        ActionIntent::Basic {
            attacker: UnitId(9),
            target: UnitId(103),
        },
        &mut sp_history,
    );
    cast(
        &mut app,
        ActionIntent::Basic {
            attacker: UnitId(9),
            target: UnitId(103),
        },
        &mut sp_history,
    );
    cast(
        &mut app,
        ActionIntent::Basic {
            attacker: UnitId(9),
            target: UnitId(103),
        },
        &mut sp_history,
    );

    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(9),
            skill_id: SkillId("holy_revive".into()),
            target: victim_id,
        },
        &mut sp_history,
    );

    let events = drain_messages(&mut event_cursor, &app);
    let traces = drain_messages(&mut trace_cursor, &app);
    let log = action_log_entries(&app);
    let trace = audit_trace(&events, &traces, &log, &sp_history);

    assert!(
        sp_history.windows(3).any(|window| window == [5, 2, 2]),
        "the trace should show the initial spend that made the revive unavailable
{trace}"
    );
    assert!(
        sp_history.windows(3).any(|window| window == [3, 4, 5])
            || sp_history.windows(3).any(|window| window == [4, 5, 6]),
        "the trace should show SP rebuilding before the revive becomes available again
{trace}"
    );
    assert!(
        !is_ko(&mut app, victim_id),
        "the victim should be revived once SP is rebuilt
{trace}"
    );
    assert_eq!(
        unit_hp(&mut app, UnitId(102)),
        25,
        "the revived ally should return at the canonical revive HP
{trace}"
    );
    assert!(events.iter().any(|event| {
        event.source == UnitId(9)
            && event.target == victim_id
            && event.follow_up_depth == 0
            && matches!(&event.kind, CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("holy_revive".into()))
    }), "missing the successful revive event after SP rebuild
{trace}");
    assert!(log.iter().any(|entry| matches!(entry, LogEntry::Revive { target, hp_after } if *target == victim_id && *hp_after == 25)), "missing the revive log entry
{trace}");
}

#[test]
fn s_m008_s06_status_pressure_turns_low_hp_into_a_failed_action_and_a_follow_up_window() {
    let roster = canonical_roster();
    let mut app = build_status_app(skill_book_with_shock_fixture());

    let agumon = pilot(&roster, "Agumon");
    let gabumon = pilot(&roster, "Gabumon");
    let renamon = pilot(&roster, "Renamon");

    spawn_from_def(&mut app, &agumon, 38, 50, 0);
    spawn_from_def(&mut app, &gabumon, 95, 48, 0);
    spawn_from_def(&mut app, &renamon, 90, 45, 0);
    spawn_custom_enemy(
        &mut app,
        UnitId(201),
        "Para Tester",
        220,
        220,
        12,
        12,
        Attribute::Virus,
        "para_bolt",
        vec!["para_bolt"],
        "enemy_ult_fire",
        vec![DamageTag::Dark],
    );

    let mut event_cursor = message_cursor::<CombatEvent>(&mut app);
    let mut trace_cursor = message_cursor::<FollowUpTrace>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(201),
        skill_id: SkillId("para_bolt".into()),
        target: agumon.id,
    });
    app.update();

    assert_eq!(
        status_effect_kind(&mut app, agumon.id),
        Some(StatusEffectKind::Paralyzed),
        "paralyzed should be present immediately after application"
    );

    app.world_mut().write_message(TurnAdvanced::of(agumon.id));
    app.update();

    let events = drain_messages(&mut event_cursor, &app);
    let traces = drain_messages(&mut trace_cursor, &app);
    let log = action_log_entries(&app);
    let trace = audit_trace(&events, &traces, &log, &[]);

    assert!(
        events.iter().any(|event| {
            event.source == UnitId(201)
                && event.target == agumon.id
                && event.follow_up_depth == 0
                && matches!(
                    &event.kind,
                    CombatEventKind::OnStatusApplied {
                        kind: StatusEffectKind::Paralyzed
                    }
                )
        }),
        "missing the shock application event\n{trace}"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(&event.kind, CombatEventKind::OnAllyLowHp)
                && event.target == agumon.id),
        "missing the low-HP tactical pressure event\n{trace}"
    );
    // NOTE: Paralyzed action-cancel semantics deferred to S03-S05; not asserted in S01.
    assert!(
        events.iter().any(|event| {
            event.source == agumon.id
                && event.target == agumon.id
                && matches!(
                    &event.kind,
                    CombatEventKind::OnStatusTick {
                        kind: StatusEffectKind::Paralyzed,
                        turns_left: 0,
                    }
                )
        }),
        "missing the shock tick event\n{trace}"
    );
    assert!(
        events.iter().any(|event| {
            event.source == agumon.id
                && event.target == agumon.id
                && matches!(
                    &event.kind,
                    CombatEventKind::OnStatusExpired {
                        kind: StatusEffectKind::Paralyzed,
                    }
                )
        }),
        "missing the shock expiration event\n{trace}"
    );
    assert!(
        traces
            .iter()
            .filter(|entry| {
                entry.origin_target == agumon.id
                    && entry.decision == bevyrogue::combat::follow_up::FollowUpDecision::Scheduled
            })
            .count()
            >= 2,
        "missing the low-HP follow-up pressure from the status window\n{trace}"
    );
    assert!(
        trace.contains("\"type\":\"summary\"")
            && trace.contains("\"type\":\"follow_up_trace\"")
            && trace.contains("\"type\":\"action_log\""),
        "trace should stay readable on failure\n{trace}"
    );
}
