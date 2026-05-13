/// Integration test for S03 DoD: status amplification pipeline + Heated DoT.
///
/// Cases:
///   A — Fire base=100 on non-Heated defender → final damage = 100
///   B — Fire base=100 on Heated defender      → final damage = 115
///   C — Ice base=100 on Chilled defender      → final damage = 115
///   D — Heated unit takes turn                → OnDamageDealt{amount:4, damage_tag:Fire}
use bevy::prelude::*;
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
    av::ActionValueUpdated,
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::{DamageKind, Toughness},
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
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

// ─── helpers ──────────────────────────────────────────────────────────────────

fn damage_skill(id: &str, tag: DamageTag, amount: i32) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag: tag,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![
            Effect::Damage { amount, target: TargetShape::Single , per_hop: Default::default()},
            Effect::ToughnessHit(0),
        ],
        ..Default::default()
    }
}

fn read_combat_events(app: &mut App) -> Vec<CombatEvent> {
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

/// Build minimal app with resolve_action_system + advance_turn_system.
/// Attacker: Vaccine, high HP. Defender: Vaccine, large HP pool so no KO noise.
fn setup_amp_app() -> (App, Entity, Entity) {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(SkillBook(vec![
        damage_skill("fire100", DamageTag::Fire, 100),
        damage_skill("ice100", DamageTag::Ice, 100),
    ]));
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool { current: 100, max: 100 })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(0))
        .add_message::<ActionIntent>()
        .add_message::<TurnAdvanced>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, resolve_action_system)
        .add_systems(Update, advance_turn_system);

    let attacker = app.world_mut().spawn((
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
            basic: SkillId("fire100".into()),
            skills: vec![SkillId("fire100".into()), SkillId("ice100".into())],
            ultimate: SkillId("fire100".into()),
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
    )).id();

    let defender = app.world_mut().spawn((
        Unit {
            id: UnitId(2),
            name: "Defender".into(),
            hp_max: 10_000,
            hp_current: 10_000,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        Toughness::new(1_000, vec![]),
        StatusBag::default(),
    )).id();

    (app, attacker, defender)
}

// ─── case A: Fire base=100 on non-Heated defender → 100 ───────────────────────

#[test]
fn fire_base100_non_heated_deals_100() {
    let (mut app, _, _) = setup_amp_app();

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("fire100".into()),
        target: UnitId(2),
    });
    app.update();

    let events = read_combat_events(&mut app);
    let damage_event = events.iter().find(|e| {
        matches!(
            &e.kind,
            CombatEventKind::OnDamageDealt { damage_tag: DamageTag::Fire, .. }
        )
    });
    let amount = match damage_event.map(|e| &e.kind) {
        Some(CombatEventKind::OnDamageDealt { amount, .. }) => *amount,
        _ => panic!("expected OnDamageDealt Fire event"),
    };
    assert_eq!(amount, 100, "non-Heated: fire100 base must deal 100");
}

// ─── case B: Fire base=100 on Heated defender → 115 ──────────────────────────

#[test]
fn fire_base100_heated_defender_deals_115() {
    let (mut app, _, defender) = setup_amp_app();

    // Pre-apply Heated to defender.
    app.world_mut()
        .get_mut::<StatusBag>(defender)
        .unwrap()
        .apply(StatusEffectKind::Heated, 3);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("fire100".into()),
        target: UnitId(2),
    });
    app.update();

    let events = read_combat_events(&mut app);
    let damage_event = events.iter().find(|e| {
        matches!(
            &e.kind,
            CombatEventKind::OnDamageDealt { damage_tag: DamageTag::Fire, .. }
        )
    });
    let amount = match damage_event.map(|e| &e.kind) {
        Some(CombatEventKind::OnDamageDealt { amount, .. }) => *amount,
        _ => panic!("expected OnDamageDealt Fire event"),
    };
    assert_eq!(amount, 115, "Heated defender: fire100 base must deal 115");
}

// ─── case C: Ice base=100 on Chilled defender → 115 ──────────────────────────

#[test]
fn ice_base100_chilled_defender_deals_115() {
    let (mut app, _, defender) = setup_amp_app();

    // Pre-apply Chilled to defender.
    app.world_mut()
        .get_mut::<StatusBag>(defender)
        .unwrap()
        .apply(StatusEffectKind::Chilled, 3);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("ice100".into()),
        target: UnitId(2),
    });
    app.update();

    let events = read_combat_events(&mut app);
    let damage_event = events.iter().find(|e| {
        matches!(
            &e.kind,
            CombatEventKind::OnDamageDealt { damage_tag: DamageTag::Ice, .. }
        )
    });
    let amount = match damage_event.map(|e| &e.kind) {
        Some(CombatEventKind::OnDamageDealt { amount, .. }) => *amount,
        _ => panic!("expected OnDamageDealt Ice event"),
    };
    assert_eq!(amount, 115, "Chilled defender: ice100 base must deal 115");
}

// ─── case D: Heated unit takes turn → DoT event amount=4, tag=Fire ───────────

#[test]
fn heated_unit_turn_emits_dot_4_fire() {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(0))
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, advance_turn_system);

    let unit_entity = app.world_mut().spawn((
        Unit {
            id: UnitId(1),
            name: "HotUnit".into(),
            hp_max: 500,
            hp_current: 500,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        Toughness::new(100, vec![]),
        {
            let mut bag = StatusBag::default();
            bag.apply(StatusEffectKind::Heated, 3);
            bag
        },
    )).id();

    // Trigger turn for unit 1.
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
    app.update();

    // Confirm HP decreased by 4.
    let hp = app.world().get::<Unit>(unit_entity).unwrap().hp_current;
    assert_eq!(hp, 496, "Heated DoT must reduce HP by 4");

    // Confirm OnDamageDealt Fire 4 event emitted.
    let events = read_combat_events(&mut app);
    let dot_event = events.iter().find(|e| {
        matches!(
            &e.kind,
            CombatEventKind::OnDamageDealt {
                amount: 4,
                damage_tag: DamageTag::Fire,
                kind: DamageKind::Normal,
                ..
            }
        )
    });
    assert!(
        dot_event.is_some(),
        "expected OnDamageDealt{{amount:4, damage_tag:Fire}} in event stream; got: {events:?}"
    );
    let evt = dot_event.unwrap();
    assert_eq!(evt.source, UnitId(1), "DoT source must be the Heated unit itself");
    assert_eq!(evt.target, UnitId(1), "DoT target must be the Heated unit itself");
}
