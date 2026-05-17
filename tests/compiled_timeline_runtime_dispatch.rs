use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use bevyrogue::combat::{
    api::timeline::{Beat, BeatEdge, BeatKind, BeatPayload, TimelineLibrary},
    api::{ExtRegistries, register_kernel_builtins},
    av::{ActionValue, ActionValueUpdated, MAX_AV},
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
    skill_timeline::{SkillTimeline, compile_skill_book_timelines},
    skills_ron::{
        SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting, TargetLife,
        TargetShape, TargetSide,
    },
};

fn timeline_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag: DamageTag::Physical,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![],
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
                    id: "damage".into(),
                    kind: BeatKind::Impact,
                    hook: Some("core/deal_damage".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(BeatPayload::DealDamage {
                        amount: 11,
                        tag: DamageTag::Physical,
                        target: TargetShape::Single,
                    }),
                },
                Beat {
                    id: "break".into(),
                    kind: BeatKind::Aftermath,
                    hook: Some("core/apply_effect".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(BeatPayload::BreakToughness {
                        amount: 25,
                        tag: DamageTag::Fire,
                        target: TargetShape::Single,
                    }),
                },
                Beat {
                    id: "status".into(),
                    kind: BeatKind::Aftermath,
                    hook: Some("core/apply_effect".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(BeatPayload::ApplyStatus {
                        kind: StatusEffectKind::Slowed,
                        duration: 3,
                        target: TargetShape::Single,
                    }),
                },
                Beat {
                    id: "delay".into(),
                    kind: BeatKind::Aftermath,
                    hook: Some("core/apply_effect".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(BeatPayload::DelayTurn {
                        amount_pct: 80,
                        target: TargetShape::Single,
                    }),
                },
                Beat {
                    id: "buff".into(),
                    kind: BeatKind::Aftermath,
                    hook: Some("core/apply_effect".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(BeatPayload::ApplyBuff {
                        kind: StatusEffectKind::Blessed,
                        duration: 2,
                        target: TargetShape::Single,
                    }),
                },
            ],
            edges: vec![
                BeatEdge {
                    from: "cast".into(),
                    to: "damage".into(),
                    gate: Some("core/always".into()),
                },
                BeatEdge {
                    from: "damage".into(),
                    to: "break".into(),
                    gate: Some("core/always".into()),
                },
                BeatEdge {
                    from: "break".into(),
                    to: "status".into(),
                    gate: Some("core/always".into()),
                },
                BeatEdge {
                    from: "status".into(),
                    to: "delay".into(),
                    gate: Some("core/always".into()),
                },
                BeatEdge {
                    from: "delay".into(),
                    to: "buff".into(),
                    gate: Some("core/always".into()),
                },
            ],
        }),
        ..Default::default()
    }
}

fn damage_timeline_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag: DamageTag::Physical,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![],
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
                    id: "damage".into(),
                    kind: BeatKind::Impact,
                    hook: Some("core/deal_damage".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(BeatPayload::DealDamage {
                        amount: 13,
                        tag: DamageTag::Physical,
                        target: TargetShape::Single,
                    }),
                },
                Beat {
                    id: "break".into(),
                    kind: BeatKind::Aftermath,
                    hook: Some("core/apply_effect".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(BeatPayload::BreakToughness {
                        amount: 5,
                        tag: DamageTag::Physical,
                        target: TargetShape::Single,
                    }),
                },
            ],
            edges: vec![
                BeatEdge {
                    from: "cast".into(),
                    to: "damage".into(),
                    gate: Some("core/always".into()),
                },
                BeatEdge {
                    from: "damage".into(),
                    to: "break".into(),
                    gate: Some("core/always".into()),
                },
            ],
        }),
        ..Default::default()
    }
}

fn build_app(book: SkillBook) -> App {
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
        .init_resource::<ExtRegistries>()
        .add_message::<ActionIntent>()
        .add_message::<TurnAdvanced>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, (resolve_action_system, apply_av_ops_system).chain());

    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        register_kernel_builtins(&mut regs);
        let compiled = compile_skill_book_timelines(&book, &regs)
            .expect("timeline-backed test book must compile");
        app.world_mut()
            .resource_mut::<TimelineLibrary<String>>()
            .timelines = compiled;
    }

    app
}

fn spawn_actor(app: &mut App, skill_ids: Vec<SkillId>) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id: UnitId(1),
                name: "Caster".into(),
                hp_max: 500,
                hp_current: 500,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            UnitSkills {
                basic: skill_ids[0].clone(),
                skills: skill_ids.clone(),
                ultimate: skill_ids[0].clone(),
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
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            ActionValue(MAX_AV),
            Toughness::new(20, vec![DamageTag::Fire]),
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
fn timeline_backed_action_runs_through_beat_runner_and_applier() {
    let book = SkillBook(vec![
        timeline_skill("timeline_kernel_demo"),
        damage_timeline_skill("damage_only_demo"),
    ]);
    let mut app = build_app(book);
    let _actor = spawn_actor(
        &mut app,
        vec![
            SkillId("timeline_kernel_demo".into()),
            SkillId("damage_only_demo".into()),
        ],
    );
    let target = spawn_target(&mut app);

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_skill(&mut app, "timeline_kernel_demo");

    let events = collect_events(&app, &mut cursor);
    let dump: Vec<_> = events.iter().map(|e| format!("{:?}", e.kind)).collect();

    let pos_declared = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionDeclared { .. })
    });
    let pos_preapp = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionPreApp)
    });
    let pos_damage = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::OnDamageDealt { .. })
    });
    let pos_break = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::OnBreak { .. })
    });
    let pos_status = event_pos(
        &events,
        |e| matches!(&e.kind, CombatEventKind::OnStatusApplied { kind } if *kind == StatusEffectKind::Slowed),
    );
    let pos_delay_status = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::DelayTurn { amount_pct: 30, .. })
    });
    let pos_delay_explicit = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::DelayTurn { amount_pct: 50, .. })
    });
    let pos_buff = event_pos(
        &events,
        |e| matches!(&e.kind, CombatEventKind::OnStatusApplied { kind } if *kind == StatusEffectKind::Blessed),
    );
    let pos_applied = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionApplied)
    });
    let pos_resolved = event_pos(&events, |e| {
        matches!(e.kind, CombatEventKind::OnActionResolved)
    });

    assert!(
        pos_declared < pos_preapp,
        "declared must precede preapp: {dump:?}"
    );
    assert!(
        pos_preapp < pos_damage,
        "preapp must precede damage: {dump:?}"
    );
    assert!(
        pos_damage < pos_break,
        "damage must precede break: {dump:?}"
    );
    assert!(
        pos_break < pos_status,
        "break must precede status: {dump:?}"
    );
    assert!(
        pos_status < pos_delay_status,
        "status must precede slowed delay: {dump:?}"
    );
    assert!(
        pos_delay_status < pos_delay_explicit,
        "slowed delay must precede explicit delay: {dump:?}"
    );
    assert!(
        pos_delay_explicit < pos_buff,
        "explicit delay must precede buff: {dump:?}"
    );
    assert!(
        pos_buff < pos_applied,
        "buff must precede applied: {dump:?}"
    );
    assert!(
        pos_applied < pos_resolved,
        "applied must precede resolved: {dump:?}"
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
        2000,
        "DelayTurn intents should flow through apply_av_ops_system"
    );

    let status_bag = app
        .world()
        .get::<StatusBag>(target)
        .expect("target status bag missing");
    assert!(
        status_bag.has(&StatusEffectKind::Slowed),
        "Slowed must be applied"
    );
    assert!(
        status_bag.has(&StatusEffectKind::Blessed),
        "Blessed must be applied"
    );
    assert!(
        app.world().get::<Stunned>(target).is_some(),
        "BreakToughness should stun the target"
    );
}

#[test]
fn damage_only_timeline_skill_still_uses_beat_runner() {
    let book = SkillBook(vec![
        timeline_skill("timeline_kernel_demo"),
        damage_timeline_skill("damage_only_demo"),
    ]);
    let mut app = build_app(book);
    let _actor = spawn_actor(
        &mut app,
        vec![
            SkillId("timeline_kernel_demo".into()),
            SkillId("damage_only_demo".into()),
        ],
    );
    let target = spawn_target(&mut app);

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_skill(&mut app, "damage_only_demo");

    let events = collect_events(&app, &mut cursor);

    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. })),
        "timeline damage skill must emit damage events"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, CombatEventKind::OnStatusApplied { .. })),
        "damage-only timeline skill should not emit status events"
    );
    assert!(
        app.world()
            .get::<Unit>(target)
            .expect("target missing")
            .hp_current
            < 200,
        "timeline damage skill should damage the target"
    );
}
