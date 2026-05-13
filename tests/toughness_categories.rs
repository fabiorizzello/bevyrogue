use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    av::ActionValueUpdated,
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    round_flags::RoundFlags,
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::{Toughness, ToughnessCategory},
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{BasicStreak, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

const FIRE_BASIC: &str = "fire_basic";
// Exactly one full toughness bar worth of damage; enables precise per-category assertions.
const TOUGHNESS_HIT: i32 = 20;

fn skill_book() -> SkillBook {
    SkillBook(vec![SkillDef {
        id: SkillId(FIRE_BASIC.into()),
        name: "Fire Basic".into(),
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
        effects: vec![
            Effect::Damage {
                amount: 5,
                target: TargetShape::Single,
            },
            Effect::ToughnessHit(TOUGHNESS_HIT),
        ],
        animation_sequence: None,
        qte: None,

        custom_signals: vec![],
    }])
}

fn build_app() -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(skill_book());
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
        .insert_resource(CombatRng::from_seed(42))
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, (resolve_action_system, advance_turn_system).chain());
    app
}

fn make_ult() -> UltimateCharge {
    UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

fn spawn_attacker(app: &mut App) {
    app.world_mut().spawn((
        Unit {
            id: UnitId(1),
            name: "Agumon".into(),
            hp_max: 200,
            hp_current: 200,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Child,
        },
        Team::Ally,
        Toughness::new(100, vec![]),
        make_ult(),
        UnitSkills {
            basic: SkillId(FIRE_BASIC.into()),
            skills: vec![],
            ultimate: SkillId(FIRE_BASIC.into()),
            follow_up: None,
        },
        BasicStreak::default(),
    ));
}

/// Spawn a defender with toughness_max=20 and a Fire weakness.
/// High HP (10_000) ensures the unit never dies during any test.
fn spawn_defender(app: &mut App, id: u32, category: ToughnessCategory) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id: UnitId(id),
                name: format!("Defender{id}"),
                hp_max: 10_000,
                hp_current: 10_000,
                attribute: Attribute::Virus,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Toughness::with_category(20, vec![DamageTag::Fire], category),
            RoundFlags::default(),
            make_ult(),
        ))
        .id()
}

fn event_cursor(app: &mut App) -> MessageCursor<CombatEvent> {
    app.world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor()
}

/// Drains new CombatEvents since the cursor was last read and returns the OnBreak count.
fn new_break_count(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> usize {
    let messages = app.world().resource::<Messages<CombatEvent>>();
    cursor
        .read(messages)
        .filter(|e| matches!(e.kind, CombatEventKind::OnBreak { .. }))
        .count()
}

fn fire_basic(app: &mut App, attacker: UnitId, target: UnitId) {
    app.world_mut()
        .write_message(ActionIntent::Basic { attacker, target });
    app.update();
}

// ── Test 1 ─────────────────────────────────────────────────────────────────

/// Standard enemies break on the first full ToughnessHit (20 damage, bar = 20).
#[test]
fn standard_breaks_in_one_full_hit() {
    let mut app = build_app();
    spawn_attacker(&mut app);
    let defender = spawn_defender(&mut app, 2, ToughnessCategory::Standard);

    let mut cursor = event_cursor(&mut app);

    fire_basic(&mut app, UnitId(1), UnitId(2));

    let breaks = new_break_count(&mut cursor, &app);
    let tough = app.world().get::<Toughness>(defender).unwrap();

    assert_eq!(
        breaks, 1,
        "Standard: expected 1 OnBreak, got {breaks}; toughness.current={}",
        tough.current
    );
    assert!(
        tough.broken,
        "Standard: expected broken=true; OnBreak count={breaks}, current={}",
        tough.current
    );
}

// ── Test 2 ─────────────────────────────────────────────────────────────────

/// Armored halves incoming toughness damage (round-up), requiring ~2 full hits to break.
#[test]
fn armored_requires_two_full_hits() {
    let mut app = build_app();
    spawn_attacker(&mut app);
    let defender = spawn_defender(&mut app, 2, ToughnessCategory::Armored);

    let mut cursor = event_cursor(&mut app);

    // Hit 1: effective = (20 + 1) / 2 = 10; current 20 → 10; no break.
    fire_basic(&mut app, UnitId(1), UnitId(2));
    let breaks_1 = new_break_count(&mut cursor, &app);
    let current_1 = app.world().get::<Toughness>(defender).unwrap().current;

    assert_eq!(
        breaks_1, 0,
        "Armored: first hit must NOT break; OnBreak={breaks_1}, current={current_1}"
    );
    assert_eq!(
        current_1, 10,
        "Armored: first hit should reduce current to 10 (halved); OnBreak={breaks_1}"
    );

    // Hit 2: effective = 10; current 10 → 0; Fire is a weakness → breaks.
    fire_basic(&mut app, UnitId(1), UnitId(2));
    let breaks_2 = new_break_count(&mut cursor, &app);
    let tough_2 = app.world().get::<Toughness>(defender).unwrap();

    assert_eq!(
        breaks_2, 1,
        "Armored: second hit should break; OnBreak={breaks_2}, current={}",
        tough_2.current
    );
    assert!(
        tough_2.broken,
        "Armored: broken should be true after second hit; OnBreak={breaks_2}"
    );
}

// ── Test 3 ─────────────────────────────────────────────────────────────────

/// Shielded units never break; the toughness bar clamps at 0 but broken stays false.
#[test]
fn shielded_never_breaks() {
    let mut app = build_app();
    spawn_attacker(&mut app);
    let defender = spawn_defender(&mut app, 2, ToughnessCategory::Shielded);

    let mut cursor = event_cursor(&mut app);

    for _ in 0..3 {
        fire_basic(&mut app, UnitId(1), UnitId(2));
    }

    let breaks = new_break_count(&mut cursor, &app);
    let tough = app.world().get::<Toughness>(defender).unwrap();

    assert_eq!(
        breaks, 0,
        "Shielded: expected 0 OnBreak across 3 hits, got {breaks}; current={}",
        tough.current
    );
    assert!(
        !tough.broken,
        "Shielded: broken must remain false; OnBreak={breaks}, current={}",
        tough.current
    );
    // Shielded uses saturating_sub(...).max(0): bar drains to 0 but never negative.
    assert_eq!(
        tough.current, 0,
        "Shielded: current should be floor-clamped to 0; OnBreak={breaks}, broken={}",
        tough.broken
    );
}

// ── Test 4 ─────────────────────────────────────────────────────────────────

/// Break Seal prevents a second break in the same round; the seal resets when the
/// defender's TurnAdvanced fires, restoring breakability the next round.
#[test]
fn break_seal_blocks_repeat_break_in_same_round_then_lifts_on_next_turn() {
    let mut app = build_app();
    spawn_attacker(&mut app);
    let defender = spawn_defender(&mut app, 2, ToughnessCategory::Standard);

    let mut cursor = event_cursor(&mut app);

    // ── Step 1: first break ───────────────────────────────────────────────
    fire_basic(&mut app, UnitId(1), UnitId(2));
    let breaks_1 = new_break_count(&mut cursor, &app);
    assert_eq!(
        breaks_1, 1,
        "Step 1: expected first OnBreak; got {breaks_1}"
    );

    let sealed_after_break = app
        .world()
        .get::<RoundFlags>(defender)
        .unwrap()
        .break_sealed;
    assert!(
        sealed_after_break,
        "Step 1: break_sealed must be true immediately after first break"
    );

    // ── Step 2: restore toughness to simulate same-round second attempt ──
    {
        let mut tough = app.world_mut().get_mut::<Toughness>(defender).unwrap();
        tough.current = 20;
        tough.broken = false;
    }

    // ── Step 3: second hit — seal must block the break ────────────────────
    fire_basic(&mut app, UnitId(1), UnitId(2));
    let breaks_2 = new_break_count(&mut cursor, &app);
    let sealed_after_blocked = app
        .world()
        .get::<RoundFlags>(defender)
        .unwrap()
        .break_sealed;

    assert_eq!(
        breaks_2, 0,
        "Step 3: seal should block second break; new OnBreak={breaks_2}, sealed={sealed_after_blocked}"
    );
    assert!(
        sealed_after_blocked,
        "Step 3: break_sealed must remain true after blocked attempt; new OnBreak={breaks_2}"
    );

    // ── Step 4: defender's turn advances — seal must lift ─────────────────
    // advance_turn_system resets break_sealed=false at the start of the unit's turn.
    // The unit was Stunned (from step 1's break), so enemy AI dispatch is skipped —
    // no spurious ActionIntent is injected.
    app.world_mut().write_message(TurnAdvanced::of(UnitId(2)));
    app.update();

    // ── Step 5: assert seal is gone ───────────────────────────────────────
    let sealed_after_turn = app
        .world()
        .get::<RoundFlags>(defender)
        .unwrap()
        .break_sealed;
    assert!(
        !sealed_after_turn,
        "Step 5: break_sealed must be false after TurnAdvanced; got sealed={sealed_after_turn}"
    );

    // ── Step 6: third hit — seal lifted, should break again ───────────────
    // toughness.current was restored to 20 in step 2 and was NOT reduced by the sealed
    // hit in step 3 (apply_hit short-circuits without mutating when break_sealed=true).
    fire_basic(&mut app, UnitId(1), UnitId(2));
    let breaks_3 = new_break_count(&mut cursor, &app);
    let tough_final = app.world().get::<Toughness>(defender).unwrap();

    assert_eq!(
        breaks_3, 1,
        "Step 6: after seal lift, should break again; OnBreak={breaks_3}, current={}",
        tough_final.current
    );
    assert!(
        tough_final.broken,
        "Step 6: broken must be true after third break; OnBreak={breaks_3}"
    );
}
