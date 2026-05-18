use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    action_query::{
        ActionQueryKind, CombatQuerySnapshot, UnitQuerySnapshot, query_action_affordance,
    },
    blueprints::patamon::{HolySupportState, HolySupportTransition},
    events::{CombatEvent, CombatEventKind},
    kernel::{CombatBeatId, CombatKernelTransition, register_combat_kernel_runtime},
    kit::UnitSkills,
    log::ActionLog,
    observability::{capture_validation_snapshot, format_validation_snapshot},
    resolution::resolve_action,
    sp::SpPool,
    state::{CombatPhase, CombatState, ResolvedAction, UltEffect},
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
    skills_ron::{
        CustomSignalPayload, Effect, SelfTargetRule, SkillBook, SkillCustomSignal, SkillDef,
        SkillImplementation, SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

const ACTOR: UnitId = UnitId(1);
const TARGET: UnitId = UnitId(2);

fn canonical_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse tracked skills.ron")
}

fn find_skill<'a>(book: &'a SkillBook, id: &str) -> &'a SkillDef {
    book.0
        .iter()
        .find(|skill| skill.id == SkillId(id.into()))
        .unwrap_or_else(|| panic!("missing tracked skill fixture {id}"))
}

fn boundary_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: format!("Boundary Skill {id}"),
        damage_tag: DamageTag::Light,
        sp_cost: 2,
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
                amount: 23,
                target: TargetShape::Single,
                per_hop: Default::default(),
            },
            Effect::ToughnessHit(11),
            Effect::GainSP(1),
        ],
        timeline: None,
        ..Default::default()
    }
}

fn with_misleading_presentation_metadata(mut skill: SkillDef) -> SkillDef {
    skill.animation_sequence = Some(vec![
        "OnKernelTransition(HolySupport(BuildGrace(amount: 3)))".to_string(),
        "Damage(amount: 999, target: AllEnemies)".to_string(),
        "SetPhase(Victory)".to_string(),
    ]);
    skill.qte = Some(
        "PERFECT: grant Grace=3, refund SP, stun target, dispatch HolySupport::BuildGrace"
            .to_string(),
    );
    skill
}

fn with_patamon_signal(mut skill: SkillDef) -> SkillDef {
    skill.custom_signals = vec![SkillCustomSignal::blueprint(
        "patamon",
        "build_holy_support_grace",
        CustomSignalPayload::Amount { amount: 1 },
    )];
    skill
}

fn unit(id: UnitId, hp_current: i32, attribute: Attribute) -> Unit {
    Unit {
        id,
        name: format!("Unit{}", id.0),
        hp_max: 100,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Child,
    }
}

fn ultimate_charge(current: i32) -> UltimateCharge {
    UltimateCharge {
        current,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 20,
    }
}

fn kit_for(skill_id: &SkillId) -> UnitSkills {
    UnitSkills {
        basic: SkillId("metadata_boundary_basic".into()),
        skills: vec![skill_id.clone()],
        ultimate: SkillId("metadata_boundary_ultimate".into()),
        follow_up: None,
    }
}

fn query_snapshot(skill_id: &SkillId) -> CombatQuerySnapshot {
    let actor = UnitQuerySnapshot {
        id: ACTOR,
        team: Team::Ally,
        is_active: true,
        is_ko: false,
        is_stunned: false,
        is_commander: false,
        hp_current: 90,
        hp_max: 100,
        sp: 5,
        ultimate_current: 100,
        ultimate_trigger: 100,
        ultimate_ready: true,
        skills: Some(kit_for(skill_id)),
        ..Default::default()
    };
    let target = UnitQuerySnapshot {
        id: TARGET,
        team: Team::Enemy,
        is_active: false,
        is_ko: false,
        is_stunned: false,
        is_commander: false,
        hp_current: 80,
        hp_max: 100,
        toughness: Some(Toughness::new(30, vec![DamageTag::Light])),
        ..Default::default()
    };

    CombatQuerySnapshot {
        phase: CombatPhase::WaitingAction,
        acting_unit: actor.clone(),
        target_unit: Some(target.clone()),
        units: vec![actor, target],
    }
}

fn cursor(app: &mut App) -> MessageCursor<CombatEvent> {
    app.world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor()
}

fn drain(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    let messages = app.world().resource::<Messages<CombatEvent>>();
    cursor.read(messages).cloned().collect()
}

fn app_with_skill_book(book: SkillBook, skill_id: &SkillId) -> App {
    let mut app = App::new();
    register_combat_kernel_runtime(&mut app);
    bevyrogue::combat::blueprints::add_runtime_plugins(&mut app);
    {
        let mut regs = app
            .world_mut()
            .resource_mut::<bevyrogue::combat::api::ExtRegistries>();
        bevyrogue::combat::blueprints::register_all_blueprint_validation_exts(&mut regs);
    }
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book);
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    app.world_mut().resource_mut::<SpPool>().current = 5;

    app.world_mut().spawn((
        unit(ACTOR, 90, Attribute::Vaccine),
        Team::Ally,
        Toughness::new(30, vec![]),
        ultimate_charge(100),
        kit_for(skill_id),
    ));
    app.world_mut().spawn((
        unit(TARGET, 80, Attribute::Virus),
        Team::Enemy,
        Toughness::new(30, vec![DamageTag::Light]),
        ultimate_charge(0),
        UnitSkills {
            basic: SkillId("enemy_basic".into()),
            skills: vec![],
            ultimate: SkillId("enemy_ultimate".into()),
            follow_up: None,
        },
    ));

    app
}

fn run_skill_action(book: SkillBook, skill_id: &SkillId) -> (Vec<CombatEvent>, String) {
    let mut app = app_with_skill_book(book, skill_id);
    let mut reader = cursor(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: ACTOR,
        skill_id: skill_id.clone(),
        target: TARGET,
    });
    app.update();
    app.update();
    let events = drain(&mut reader, &app);
    let snapshot = capture_validation_snapshot(app.world_mut()).expect("validation snapshot");
    let formatted = format_validation_snapshot(&snapshot);

    (events, formatted)
}

fn event_kinds(events: &[CombatEvent]) -> Vec<CombatEventKind> {
    events.iter().map(|event| event.kind.clone()).collect()
}

fn combat_beats(events: &[CombatEvent]) -> Vec<CombatBeatId> {
    events
        .iter()
        .filter_map(|event| match event.kind {
            CombatEventKind::OnCombatBeat { beat } => Some(beat),
            _ => None,
        })
        .collect()
}

fn kernel_beats(events: &[CombatEvent]) -> Vec<CombatBeatId> {
    events
        .iter()
        .filter_map(|event| match event.kind {
            CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::Beat(beat),
            } => Some(beat),
            _ => None,
        })
        .collect()
}

fn holy_support_transitions(events: &[CombatEvent]) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| match &event.kind {
            CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::Blueprint { owner, name, .. },
            } if owner == "patamon" => Some(name.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn tracked_ron_exposes_custom_signals_separately_from_presentation_metadata() {
    let book = canonical_skill_book();
    let patamon_ult = find_skill(&book, "patamon_ult");

    assert_eq!(
        patamon_ult.custom_signals,
        vec![SkillCustomSignal::blueprint(
            "patamon",
            "build_holy_support_grace",
            CustomSignalPayload::Amount { amount: 1 },
        )],
        "RON contrast leak: patamon_ult must keep its gameplay custom signal typed"
    );
    assert!(
        patamon_ult.animation_sequence.is_some(),
        "RON contrast drift: patamon_ult should still expose presentation animation metadata"
    );
    assert!(
        patamon_ult.qte.is_some(),
        "RON contrast drift: patamon_ult should still expose presentation QTE metadata"
    );

    let metadata_free_control = find_skill(&book, "holy_breeze");
    assert!(
        metadata_free_control.animation_sequence.is_none()
            && metadata_free_control.qte.is_none()
            && metadata_free_control.custom_signals.is_empty(),
        "RON contrast drift: holy_breeze should remain a metadata-free control skill"
    );
}

#[test]
fn presentation_metadata_does_not_change_action_query_or_resolved_action() {
    let skill_id = SkillId("metadata_boundary_skill".into());
    let plain_skill = with_patamon_signal(boundary_skill(&skill_id.0));
    let dramatic_skill = with_misleading_presentation_metadata(plain_skill.clone());
    let plain_book = SkillBook(vec![plain_skill.clone()]);
    let dramatic_book = SkillBook(vec![dramatic_skill]);
    let snapshot = query_snapshot(&skill_id);
    let action_kind = ActionQueryKind::Skill(&skill_id);

    let plain_affordance = query_action_affordance(&snapshot, &plain_book, ACTOR, action_kind);
    let dramatic_affordance = query_action_affordance(
        &snapshot,
        &dramatic_book,
        ACTOR,
        ActionQueryKind::Skill(&skill_id),
    );

    assert_eq!(
        plain_affordance.action, dramatic_affordance.action,
        "action query leak: presentation metadata changed action legality"
    );
    assert_eq!(
        plain_affordance.target, dramatic_affordance.target,
        "action query leak: presentation metadata changed aggregate target legality"
    );
    assert_eq!(
        plain_affordance.targets, dramatic_affordance.targets,
        "action query leak: presentation metadata changed per-target details"
    );
    assert_eq!(
        plain_affordance.resource, dramatic_affordance.resource,
        "action query leak: presentation metadata changed resource legality"
    );
    assert_eq!(
        plain_affordance.resource_details, dramatic_affordance.resource_details,
        "action query leak: presentation metadata changed SP/ultimate details"
    );
    assert_eq!(
        plain_affordance.implementation, dramatic_affordance.implementation,
        "action query leak: presentation metadata changed implementation status"
    );
    assert_eq!(
        plain_affordance.toughness, dramatic_affordance.toughness,
        "action query leak: presentation metadata changed toughness affordance"
    );

    let kit = kit_for(&skill_id);
    let intent = ActionIntent::Skill {
        attacker: ACTOR,
        skill_id: skill_id.clone(),
        target: TARGET,
    };
    let plain_resolved = resolve_action(&intent, &kit, Some(&plain_book)).expect("plain resolves");
    let dramatic_resolved =
        resolve_action(&intent, &kit, Some(&dramatic_book)).expect("dramatic resolves");

    assert_eq!(
        plain_resolved, dramatic_resolved,
        "resolved action leak: presentation metadata entered gameplay payload"
    );
    assert_eq!(
        plain_resolved,
        ResolvedAction {
            source: ACTOR,
            target: TARGET,
            skill_id: skill_id.clone(),
            damage_tag: DamageTag::Light,
            base_damage: 23,
            toughness_damage: 11,
            revive_pct: 0,
            heal_pct: 0,
            sp_cost: 2,
            ult_effect: UltEffect::None,
            grant_free_skill_count: 0,
            status_to_apply: None,
            advance_pct: 0,
            delay_pct: 0,
            energy_grant: 0,
            self_advance_pct: 0,
            target_shape: TargetShape::Single,
            custom_signals: plain_skill.custom_signals,
            damage_curve: Default::default(),
            cleanse_count: None,
        },
        "resolved action contract drift: gameplay fields should come only from canonical fields and custom signals"
    );
}

#[test]
fn malformed_presentation_strings_are_inert_without_custom_signals() {
    let skill_id = SkillId("metadata_only_command_text".into());
    let skill = with_misleading_presentation_metadata(boundary_skill(&skill_id.0));
    let book = SkillBook(vec![skill.clone()]);
    let kit = kit_for(&skill_id);
    let intent = ActionIntent::Skill {
        attacker: ACTOR,
        skill_id: skill_id.clone(),
        target: TARGET,
    };

    assert!(
        skill.custom_signals.is_empty(),
        "metadata-only negative fixture must not declare typed custom signals"
    );
    assert!(
        bevyrogue::combat::blueprints::transitions_for_action(
            &resolve_action(&intent, &kit, Some(&book)).expect("metadata-only skill resolves")
        )
        .is_empty(),
        "blueprint dispatch leak: presentation text produced kernel transitions without custom signals"
    );
}

#[test]
fn runtime_events_and_snapshots_ignore_misleading_presentation_metadata() {
    let skill_id = SkillId("metadata_runtime_skill".into());
    let plain_book = SkillBook(vec![boundary_skill(&skill_id.0)]);
    let dramatic_book = SkillBook(vec![with_misleading_presentation_metadata(boundary_skill(
        &skill_id.0,
    ))]);

    let (plain_events, plain_snapshot) = run_skill_action(plain_book, &skill_id);
    let (dramatic_events, dramatic_snapshot) = run_skill_action(dramatic_book, &skill_id);

    assert_eq!(
        combat_beats(&plain_events),
        vec![
            CombatBeatId::Declared,
            CombatBeatId::PreApp,
            CombatBeatId::Impact,
            CombatBeatId::Damage,
            CombatBeatId::Applied,
            CombatBeatId::Resolved,
        ],
        "event stream drift: canonical lifecycle beats changed"
    );
    assert_eq!(
        combat_beats(&plain_events),
        kernel_beats(&plain_events),
        "event stream leak: OnCombatBeat and mirrored kernel Beat transitions diverged"
    );
    assert_eq!(
        event_kinds(&plain_events),
        event_kinds(&dramatic_events),
        "event stream leak: presentation metadata changed runtime combat output"
    );
    assert!(
        event_kinds(&dramatic_events).iter().any(
            |kind| matches!(kind, CombatEventKind::OnSkillCast { skill_id: id } if id == &skill_id)
        ),
        "event stream drift: skill-cast event missing from metadata boundary proof"
    );
    assert!(
        event_kinds(&dramatic_events)
            .iter()
            .any(|kind| matches!(kind, CombatEventKind::OnDamageDealt { .. })),
        "event stream drift: damage event missing from metadata boundary proof"
    );
    assert!(
        holy_support_transitions(&dramatic_events).is_empty(),
        "kernel transition leak: metadata-only skill emitted HolySupport transitions"
    );

    let mut dramatic_state_app = app_with_skill_book(
        SkillBook(vec![with_misleading_presentation_metadata(boundary_skill(
            &skill_id.0,
        ))]),
        &skill_id,
    );
    dramatic_state_app
        .world_mut()
        .write_message(ActionIntent::Skill {
            attacker: ACTOR,
            skill_id: skill_id.clone(),
            target: TARGET,
        });
    dramatic_state_app.update();
    dramatic_state_app.update();
    let holy_support = dramatic_state_app.world().resource::<HolySupportState>();
    assert_eq!(
        holy_support,
        &HolySupportState::default(),
        "HolySupport state leak: presentation metadata changed canonical HolySupport state"
    );

    assert_eq!(
        plain_snapshot, dramatic_snapshot,
        "snapshot leak: presentation metadata changed validation snapshot output"
    );
    assert!(
        dramatic_snapshot.contains("grace=0"),
        "snapshot drift: HolySupport grace should remain zero for metadata-only skill: {dramatic_snapshot}"
    );
    assert!(
        dramatic_snapshot.contains("last=none"),
        "snapshot drift: HolySupport last signal should remain none for metadata-only skill: {dramatic_snapshot}"
    );
}

#[test]
fn optional_blueprint_sections_render_stable_none_tokens_when_missing() {
    let skill_id = SkillId("metadata_optional_sections".into());
    let mut app = app_with_skill_book(SkillBook(vec![boundary_skill(&skill_id.0)]), &skill_id);
    app.world_mut().remove_resource::<HolySupportState>();

    let snapshot = capture_validation_snapshot(app.world_mut()).expect("validation snapshot");
    let formatted = format_validation_snapshot(&snapshot);

    assert!(snapshot.section("support").is_none());
    assert!(snapshot.section("twin_core").is_some());
    assert!(!formatted.contains("support="), "{formatted}");
    assert!(!formatted.contains("holy_support="), "{formatted}");
}
