use bevy::ecs::message::Messages;
/// Integration tests for the AdvanceTurn/DelayTurn split primitives (M018/S01).
///
/// Boundary coverage:
///   (a) DelayTurn(80) → cap 50 → AV MAX_AV → 5000
///   (b) AdvanceTurn(80) → cap 50 → AV 0 → 5000
///   (c)+(d) double/triple AdvanceTurn(50) from AV=10000 → ceiling 20000, third stuck
///   (e) DelayTurn(50) on AV=2000 → floor 0 (no negative)
///   (f) DelayTurn(50) + TempoResistance(0.25 multiplier) → reduced delay
///   (g) event already carries amount_pct ≤ 50 when skill specified 80
use bevy::prelude::*;
use bevyrogue::combat::{
    StatusBag,
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    resistance::TempoResistance,
    rng::CombatRng,
    sp::SpPool,
    state::CombatState,
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
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn delay_skill(id: &str, pct: u32) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::DelayTurn(pct)],
        ..Default::default()
    }
}

fn advance_skill(id: &str, pct: u32) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::AdvanceTurn(pct)],
        ..Default::default()
    }
}

struct Setup {
    app: App,
    target: Entity,
}

fn build_app(
    skills: Vec<SkillDef>,
    target_av: i32,
    target_resistance: Option<TempoResistance>,
) -> Setup {
    let mut app = App::new();

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(SkillBook(skills.clone()));

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool {
            current: 100,
            max: 100,
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(0))
        .add_message::<ActionIntent>()
        .add_message::<TurnAdvanced>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, (resolve_action_system, apply_av_ops_system).chain());

    let skill_id = skills
        .first()
        .map(|s| s.id.clone())
        .unwrap_or_else(|| SkillId("noop".into()));

    app.world_mut()
        .spawn((
            Unit {
                id: UnitId(1),
                name: "Attacker".into(),
                hp_max: 10_000,
                hp_current: 10_000,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            UnitSkills {
                basic: skill_id.clone(),
                skills: vec![skill_id.clone()],
                ultimate: skill_id.clone(),
                follow_up: None,
            },
            UltimateCharge {
                current: 0,
                trigger: 100,
                cap: 150,
                trigger_type: UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 10,
            },
            Toughness::new(1_000, vec![]),
            StatusBag::default(),
        ));

    let mut target_builder = app.world_mut().spawn((
        Unit {
            id: UnitId(2),
            name: "Target".into(),
            hp_max: 10_000,
            hp_current: 10_000,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        ActionValue(target_av),
        Toughness::new(1_000, vec![]),
        StatusBag::default(),
    ));
    if let Some(res) = target_resistance {
        target_builder.insert(res);
    }
    let target = target_builder.id();

    Setup {
        app,
        target,
    }
}

fn fire_skill(app: &mut App, skill_id: &str) {
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId(skill_id.into()),
        target: UnitId(2),
    });
    app.update();
}

fn collect_events(
    app: &App,
    cursor: &mut bevy::ecs::message::MessageCursor<CombatEvent>,
) -> Vec<CombatEvent> {
    let msgs = app.world().resource::<Messages<CombatEvent>>();
    cursor.read(msgs).cloned().collect()
}

// ── (a) DelayTurn(80) capped to 50 ────────────────────────────────────────────

#[test]
fn delay_turn_80_capped_to_50_reduces_av_by_5000() {
    let mut s = build_app(vec![delay_skill("delay80", 80)], MAX_AV, None);

    let mut cursor = s
        .app
        .world()
        .resource::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_skill(&mut s.app, "delay80");

    let events = collect_events(&s.app, &mut cursor);

    let delay_event = events.iter().find(
        |e| matches!(&e.kind, CombatEventKind::DelayTurn { target: t, .. } if *t == UnitId(2)),
    );
    assert!(
        delay_event.is_some(),
        "expected DelayTurn event; got: {events:?}"
    );
    if let Some(e) = delay_event {
        if let CombatEventKind::DelayTurn { amount_pct, .. } = &e.kind {
            assert_eq!(
                *amount_pct, 50,
                "event amount_pct must be capped to 50 even though skill specified 80"
            );
        }
    }

    let av = s.app.world().get::<ActionValue>(s.target).unwrap().0;
    assert_eq!(
        av, 5000,
        "AV must drop from 10000 to 5000 (50% cap applied); got {av}"
    );
}

// ── (b) AdvanceTurn(80) capped to 50 ──────────────────────────────────────────

#[test]
fn advance_turn_80_capped_to_50_increases_av_by_5000() {
    let mut s = build_app(vec![advance_skill("advance80", 80)], 0, None);

    let mut cursor = s
        .app
        .world()
        .resource::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_skill(&mut s.app, "advance80");

    let events = collect_events(&s.app, &mut cursor);

    let advance_event = events.iter().find(
        |e| matches!(&e.kind, CombatEventKind::AdvanceTurn { target: t, .. } if *t == UnitId(2)),
    );
    assert!(
        advance_event.is_some(),
        "expected AdvanceTurn event; got: {events:?}"
    );
    if let Some(e) = advance_event {
        if let CombatEventKind::AdvanceTurn { amount_pct, .. } = &e.kind {
            assert_eq!(
                *amount_pct, 50,
                "event amount_pct must be capped to 50; got {amount_pct}"
            );
        }
    }

    let av = s.app.world().get::<ActionValue>(s.target).unwrap().0;
    assert_eq!(
        av, 5000,
        "AV must rise from 0 to 5000 (50% cap applied); got {av}"
    );
}

// ── (c)+(d) double/triple AdvanceTurn ceiling at 2*MAX_AV ─────────────────────

#[test]
fn advance_turn_ceiling_at_2x_max_av() {
    let mut s = build_app(vec![advance_skill("advance50", 50)], MAX_AV, None);

    // First advance: 10000 + 5000 = 15000
    fire_skill(&mut s.app, "advance50");
    let av1 = s.app.world().get::<ActionValue>(s.target).unwrap().0;
    assert_eq!(av1, 15000, "after 1st advance: expected 15000, got {av1}");

    // Second advance: 15000 + 5000 = 20000
    fire_skill(&mut s.app, "advance50");
    let av2 = s.app.world().get::<ActionValue>(s.target).unwrap().0;
    assert_eq!(
        av2,
        2 * MAX_AV,
        "after 2nd advance: expected ceiling {}, got {av2}",
        2 * MAX_AV
    );

    // Third advance: already at ceiling → stays at 20000
    fire_skill(&mut s.app, "advance50");
    let av3 = s.app.world().get::<ActionValue>(s.target).unwrap().0;
    assert_eq!(
        av3,
        2 * MAX_AV,
        "3rd advance must not exceed 2*MAX_AV ceiling; got {av3}"
    );
}

// ── (e) DelayTurn floor 0 ────────────────────────────────────────────────────

#[test]
fn delay_turn_floor_no_negative_av() {
    let mut s = build_app(vec![delay_skill("delay50", 50)], 2000, None);

    fire_skill(&mut s.app, "delay50");

    let av = s.app.world().get::<ActionValue>(s.target).unwrap().0;
    assert_eq!(
        av, 0,
        "AV must clamp to 0 (floor), not go negative; got {av}"
    );
}

// ── (f) DelayTurn + TempoResistance 0.25 multiplier ──────────────────────────

#[test]
fn delay_turn_tempo_resistance_quarter_multiplier() {
    // hit_count=2 → multiplier=0.25
    let resistance = TempoResistance { hit_count: 2 };
    let mut s = build_app(vec![delay_skill("delay50r", 50)], MAX_AV, Some(resistance));

    fire_skill(&mut s.app, "delay50r");

    // raw = 50 * 10000 / 100 = 5000; effective = 5000 * 0.25 = 1250
    // AV: 10000 - 1250 = 8750
    let av = s.app.world().get::<ActionValue>(s.target).unwrap().0;
    assert_eq!(
        av, 8750,
        "TempoResistance(0.25) must reduce delay from 5000 to 1250; got {av}"
    );
}

// ── (g) Event amount_pct already capped at emission site ─────────────────────

#[test]
fn advance_turn_event_pct_already_capped_before_apply() {
    let mut s = build_app(vec![advance_skill("advance_over", 80)], 0, None);

    let mut cursor = s
        .app
        .world()
        .resource::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_skill(&mut s.app, "advance_over");

    let events = collect_events(&s.app, &mut cursor);

    for e in &events {
        if let CombatEventKind::AdvanceTurn { amount_pct, .. } = &e.kind {
            assert!(
                *amount_pct <= 50,
                "AdvanceTurn event amount_pct must be ≤ 50 (cap applied at emission); got {amount_pct}"
            );
        }
        if let CombatEventKind::DelayTurn { amount_pct, .. } = &e.kind {
            assert!(
                *amount_pct <= 50,
                "DelayTurn event amount_pct must be ≤ 50 (cap applied at emission); got {amount_pct}"
            );
        }
    }
}
