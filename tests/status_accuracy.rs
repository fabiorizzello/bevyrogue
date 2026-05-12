/// Deterministic tests for the status accuracy roll introduced in M011/S02/T03.
///
/// Vaccine > Virus > Data > Vaccine cycle.  When the attacker *loses* the
/// triangle (e.g. Vaccine attacks Data), `status_acc_modifier = 0.9` so the
/// effective threshold is 90 — a 10% chance to miss.  These tests verify:
///
/// 1. (miss)    Vaccine → Data,  seed chosen so roll ≥ 90  → `OnStatusResisted`, no `StatusEffect`.
/// 2. (hit)     Vaccine → Data,  seed chosen so roll < 90  → `OnStatusApplied`,  `StatusEffect` present.
/// 3. (neutral) Vaccine → Vaccine, threshold = 100           → always passes (R076).
use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    StatusEffect, StatusEffectKind,
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
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
    skills_ron::{
        Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn shock_skill() -> SkillDef {
    SkillDef {
        id: SkillId("shock_strike".into()),
        name: "Shock Strike".into(),
        damage_tag: DamageTag::Electric,
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
            Effect::Damage {
                amount: 1,
                target: TargetShape::Single,
            },
            Effect::ToughnessHit(0),
            Effect::ApplyStatus {
                kind: StatusEffectKind::Paralyzed,
                duration: 1,
            },
        ],
        ..Default::default()
    }
}

/// Returns `(app, defender_entity)`.
fn setup_app(seed: u64, defender_attribute: Attribute) -> (App, Entity) {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(SkillBook(vec![shock_skill()]));
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
        .insert_resource(CombatRng::from_seed(seed))
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

    // Attacker: Vaccine
    app.world_mut().spawn((
        Unit {
            id: UnitId(1),
            name: "Attacker".into(),
            hp_max: 200,
            hp_current: 200,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        UnitSkills {
            basic: SkillId("shock_strike".into()),
            skills: vec![],
            ultimate: SkillId("shock_strike".into()),
            follow_up: None,
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 10,
        },
        Toughness::new(100, vec![]),
    ));

    // Defender: attribute chosen by caller (Data or Vaccine); high HP so it survives the hit.
    let defender = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(2),
                name: "Defender".into(),
                hp_max: 10_000,
                hp_current: 10_000,
                attribute: defender_attribute,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Toughness::new(1_000, vec![]),
        ))
        .id();

    (app, defender)
}

fn drain_combat_events(app: &mut App) -> Vec<CombatEvent> {
    let mut cursor: MessageCursor<CombatEvent> = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

/// Find the lowest seed in [0, 10_000) where `CombatRng::roll_pct(threshold)` returns false.
fn miss_seed(threshold: i32) -> u64 {
    (0u64..10_000)
        .find(|&s| !CombatRng::from_seed(s).roll_pct(threshold))
        .expect("miss seed must exist for threshold < 100")
}

/// Find the lowest seed in [0, 10_000) where `CombatRng::roll_pct(threshold)` returns true.
fn hit_seed(threshold: i32) -> u64 {
    (0u64..10_000)
        .find(|&s| CombatRng::from_seed(s).roll_pct(threshold))
        .expect("hit seed must exist for threshold > 0")
}

// ---------------------------------------------------------------------------
// Test 1: Vaccine → Data, miss  (status_acc = 0.90, threshold = 90)
// ---------------------------------------------------------------------------

#[test]
fn vaccine_vs_data_status_miss_emits_on_status_resisted() {
    // Vaccine loses the triangle against Data → status_acc_modifier = 0.9 → threshold = 90
    let threshold = 90;
    let seed = miss_seed(threshold);

    let (mut app, defender) = setup_app(seed, Attribute::Data);

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    let events = drain_combat_events(&mut app);
    let kinds: Vec<_> = events.iter().map(|e| &e.kind).collect();

    assert!(
        kinds.iter().any(|k| matches!(
            k,
            CombatEventKind::OnStatusResisted {
                kind: StatusEffectKind::Paralyzed
            }
        )),
        "expected OnStatusResisted(Shock) for miss — seed={seed}, threshold={threshold}\nevents: {kinds:?}"
    );
    assert!(
        !kinds
            .iter()
            .any(|k| matches!(k, CombatEventKind::OnStatusApplied { .. })),
        "OnStatusApplied must not appear on a miss — seed={seed}"
    );
    // StatusEffect component must NOT be present
    assert!(
        app.world().get::<StatusEffect>(defender).is_none(),
        "StatusEffect must not be inserted on a miss — seed={seed}"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Vaccine → Data, hit   (same threshold = 90, different seed)
// ---------------------------------------------------------------------------

#[test]
fn vaccine_vs_data_status_hit_emits_on_status_applied() {
    let threshold = 90;
    let seed = hit_seed(threshold);

    let (mut app, defender) = setup_app(seed, Attribute::Data);

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    let events = drain_combat_events(&mut app);
    let kinds: Vec<_> = events.iter().map(|e| &e.kind).collect();

    assert!(
        kinds.iter().any(|k| matches!(
            k,
            CombatEventKind::OnStatusApplied {
                kind: StatusEffectKind::Paralyzed
            }
        )),
        "expected OnStatusApplied(Shock) for hit — seed={seed}, threshold={threshold}\nevents: {kinds:?}"
    );
    assert!(
        !kinds
            .iter()
            .any(|k| matches!(k, CombatEventKind::OnStatusResisted { .. })),
        "OnStatusResisted must not appear on a hit — seed={seed}"
    );
    // StatusEffect component MUST be present
    assert!(
        app.world().get::<StatusEffect>(defender).is_some(),
        "StatusEffect must be inserted on a hit — seed={seed}"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Vaccine → Vaccine, neutral (threshold = 100 → always passes, R076)
// ---------------------------------------------------------------------------

#[test]
fn vaccine_vs_vaccine_neutral_status_always_applies() {
    // Neutral matchup: status_acc_modifier = 1.0 → threshold = 100 → roll_pct always true
    // Use seed 0 — the result must be deterministic regardless of seed.
    let (mut app, defender) = setup_app(0, Attribute::Vaccine);

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    let events = drain_combat_events(&mut app);
    let kinds: Vec<_> = events.iter().map(|e| &e.kind).collect();

    assert!(
        kinds
            .iter()
            .any(|k| matches!(k, CombatEventKind::OnStatusApplied { .. })),
        "neutral matchup must always apply status (threshold=100)\nevents: {kinds:?}"
    );
    assert!(
        !kinds
            .iter()
            .any(|k| matches!(k, CombatEventKind::OnStatusResisted { .. })),
        "OnStatusResisted must not appear on neutral matchup"
    );
    assert!(
        app.world().get::<StatusEffect>(defender).is_some(),
        "StatusEffect must be present after neutral matchup"
    );
}
