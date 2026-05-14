//! Integration tests for `TargetShape::Bounce` — generic kernel hop loop.
//!
//! All 4 test cases use Vaccine-vs-Vaccine attribute matching (neutral triangle,
//! no toughness weaknesses, no status) so `final_damage == base_damage` exactly.
//! Per-hop damage deltas are asserted from `OnDamageDealt` events.

use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    sp::SpPool,
    state::CombatState,
    team::Team,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{SlotIndex, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        BounceSelector, DamageCurve, Effect, RepeatPolicy, SelfTargetRule, SkillBook, SkillDef,
        SkillImplementation, SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

// ── App builder ──────────────────────────────────────────────────────────────

fn build_app(book: SkillBook, sp_start: i32) -> App {
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book);
    let mut app = App::new();
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool {
            current: sp_start,
            max: sp_start.max(5),
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);
    app
}

fn message_cursor<T: Message>(app: &mut App) -> MessageCursor<T> {
    app.world_mut().resource_mut::<Messages<T>>().get_cursor()
}

fn drain_events(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

// ── Spawn helpers ─────────────────────────────────────────────────────────────

fn spawn_attacker(app: &mut App, id: u32, slot: u8, skill_id: &str) {
    let skill_id = SkillId(skill_id.into());
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Attacker{id}"),
            hp_max: 200,
            hp_current: 200,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        SlotIndex(slot),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: skill_id.clone(),
            skills: vec![skill_id.clone()],
            ultimate: skill_id,
            follow_up: None,
        },
    ));
}

/// Spawn an enemy with full HP (no toughness) at `slot`.
/// Vaccine+Vaccine → neutral triangle, no weakness, so damage == base_damage.
fn spawn_enemy(app: &mut App, id: u32, slot: u8, hp_max: i32) {
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Enemy{id}"),
            hp_max,
            hp_current: hp_max,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        SlotIndex(slot),
    ));
}

/// Spawn a pre-damaged enemy (hp_current < hp_max) so selectors can distinguish it.
fn spawn_enemy_with_hp(app: &mut App, id: u32, slot: u8, hp_max: i32, hp_current: i32) {
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Enemy{id}"),
            hp_max,
            hp_current,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        SlotIndex(slot),
    ));
}

// ── Skill book fixtures ───────────────────────────────────────────────────────

fn bounce_skill(
    skill_id: &str,
    sp_cost: i32,
    hops: u8,
    selector: BounceSelector,
    repeat: RepeatPolicy,
    base_damage: i32,
    curve: DamageCurve,
) -> SkillBook {
    let shape = TargetShape::Bounce { hops, selector, repeat };
    SkillBook(vec![SkillDef {
        id: SkillId(skill_id.into()),
        name: skill_id.into(),
        damage_tag: DamageTag::Fire,
        sp_cost,
        targeting: SkillTargeting {
            shape,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![Effect::Damage {
            amount: base_damage,
            target: shape,
            per_hop: curve,
        }],
        ..Default::default()
    }])
}

/// Collect the amounts from all OnDamageDealt events in order.
fn damage_amounts(events: &[CombatEvent]) -> Vec<i32> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            CombatEventKind::OnDamageDealt { amount, .. } => Some(amount),
            _ => None,
        })
        .collect()
}

/// Collect the targets from all OnDamageDealt events in order.
fn damage_targets(events: &[CombatEvent]) -> Vec<UnitId> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            CombatEventKind::OnDamageDealt { .. } => Some(e.target),
            _ => None,
        })
        .collect()
}

// ── Case 1: LowestHpPct + NoRepeat + Constant, 3 hops, no KO ─────────────────

/// Case 1: LowestHpPctAlive + NoRepeat + Constant damage.
///
/// 3 enemies with different HP percentages:
///   - E10 slot 0: 80/100 HP → 800‰
///   - E11 slot 1: 50/100 HP → 500‰
///   - E12 slot 2: 30/100 HP → 300‰ (lowest first)
///
/// Expected hop order: E12 (300‰) → E11 (500‰) → E10 (800‰).
/// Each hop deals exactly base_damage=20 (Constant curve, neutral matchup).
/// SP consumed once (cost=2).
#[test]
fn bounce_lowest_hp_no_repeat_constant_full_chain() {
    let book = bounce_skill(
        "chain_bolt",
        2,
        3,
        BounceSelector::LowestHpPctAlive,
        RepeatPolicy::NoRepeat,
        20,
        DamageCurve::Constant,
    );
    let mut app = build_app(book, 5);

    spawn_attacker(&mut app, 1, 0, "chain_bolt");
    spawn_enemy_with_hp(&mut app, 10, 0, 100, 80); // 800‰
    spawn_enemy_with_hp(&mut app, 11, 1, 100, 50); // 500‰
    spawn_enemy_with_hp(&mut app, 12, 2, 100, 30); // 300‰ — lowest

    let sp_before = app.world().resource::<SpPool>().current;

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("chain_bolt".into()),
        target: UnitId(12), // primary target = lowest HP
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let amounts = damage_amounts(&events);
    let targets = damage_targets(&events);

    // All 3 hops executed.
    assert_eq!(
        amounts.len(),
        3,
        "Case 1: expected 3 hops, got {}",
        amounts.len()
    );

    // Constant curve: every hop deals 20.
    assert_eq!(amounts, vec![20, 20, 20], "Case 1: Constant curve must be 20 each hop");

    // Order: E12 (lowest %) → E11 → E10 (highest %).
    assert_eq!(
        targets,
        vec![UnitId(12), UnitId(11), UnitId(10)],
        "Case 1: bounce order must follow LowestHpPct ascending"
    );

    // SP consumed exactly once.
    let sp_after = app.world().resource::<SpPool>().current;
    assert_eq!(
        sp_after,
        sp_before - 2,
        "Case 1: SP consumed once (cost=2)"
    );
}

// ── Case 2: NextSlotAlive + NoRepeat + Falloff, KO mid-chain ─────────────────

/// Case 2: NextSlotAlive + NoRepeat + Falloff(80%).
///
/// 3 enemies in slots 0, 1, 2. Enemy at slot 1 (E11) starts at 1 HP so
/// the first hit KOs it. The chain should truncate or skip to next slot.
///
/// Hop 0: E10 (slot 0, no last slot → pick lowest slot = slot 0)  → dealt 20
/// Hop 1: next slot > 0 among alive = slot 1 (E11, 1 HP → KO)    → dealt 16 (20 * 0.8)
/// Hop 2: next slot > 1 among alive = slot 2 (E12)               → dealt 13 (20 * 0.8^2 = 12.8 → floor → 12, but floor_at_1 → 12)
///   (Note: 20 * 0.8^2 = 12.8 → floor = 12)
///
/// Actually NoRepeat means already-hit set is populated, so:
///   hop 0: already_hit={}, last_slot=None → NextSlotAlive picks lowest = slot 0 = E10
///   hop 1: already_hit={E10}, last_slot=Some(0) → next slot > 0 = slot 1 = E11 (1 HP, will KO)
///   hop 2: already_hit={E10, E11}, last_slot=Some(1) → next slot > 1 = slot 2 = E12
///
/// The snapshot is rebuilt each hop, so after E11 KOs, it is excluded from alive pool
/// at hop 2, but E12 is still at slot 2 > 1, so the chain continues.
///
/// Damage curve Falloff{pct: 80}: hop k = base * (80/100)^k
///   hop 0: 20 * 1.0 = 20
///   hop 1: 20 * 0.8 = 16.0 → 16
///   hop 2: 20 * 0.64 = 12.8 → floor = 12
#[test]
fn bounce_next_slot_no_repeat_falloff_ko_mid_chain() {
    let book = bounce_skill(
        "bolt_chain",
        2,
        3,
        BounceSelector::NextSlotAlive,
        RepeatPolicy::NoRepeat,
        20,
        DamageCurve::Falloff { pct: 80 },
    );
    let mut app = build_app(book, 5);

    spawn_attacker(&mut app, 1, 0, "bolt_chain");
    spawn_enemy(&mut app, 10, 0, 100);
    spawn_enemy_with_hp(&mut app, 11, 1, 100, 1); // 1 HP → will KO on hop 1
    spawn_enemy(&mut app, 12, 2, 100);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("bolt_chain".into()),
        target: UnitId(10), // primary = slot 0 (NextSlotAlive: no last slot → picks lowest = slot 0)
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let amounts = damage_amounts(&events);
    let targets = damage_targets(&events);

    // All 3 hops should complete (E11 KOs but E12 is still available at slot 2 > 1).
    assert_eq!(
        amounts.len(),
        3,
        "Case 2: expected 3 hops (KO mid-chain doesn't stop the chain), got {}",
        amounts.len()
    );

    // Falloff 80%: hop 0 = 20, hop 1 = 16, hop 2 = 12.
    assert_eq!(
        amounts,
        vec![20, 16, 12],
        "Case 2: Falloff(80) curve must be [20, 16, 12]"
    );

    // NextSlotAlive order: slot 0 → slot 1 → slot 2.
    assert_eq!(
        targets,
        vec![UnitId(10), UnitId(11), UnitId(12)],
        "Case 2: bounce order must follow NextSlotAlive slot order"
    );

    // E11 must be KO'd.
    let mut unit_q = app.world_mut().query::<(&Unit, &Team)>();
    let world = app.world();
    let e11_hp = unit_q
        .iter(world)
        .find(|(u, t)| u.id == UnitId(11) && **t == Team::Enemy)
        .map(|(u, _)| u.hp_current)
        .expect("E11 not found");
    assert!(e11_hp <= 0, "Case 2: E11 should be KO'd, hp={}", e11_hp);
}

// ── Case 3: LowestHpPct + AllowRepeat + PerHop ────────────────────────────────

/// Case 3: LowestHpPctAlive + AllowRepeat + PerHop[30, 15, 5].
///
/// 3 enemies; E12 starts at lowest HP%. AllowRepeat means already_hit is ignored,
/// so E12 can be selected multiple times if it remains lowest HP%.
///
/// E10 slot 0: 90/100 HP → 900‰
/// E11 slot 1: 70/100 HP → 700‰
/// E12 slot 2: 20/100 HP → 200‰  ← lowest
///
/// Hop 0 (k=0): picks E12 (200‰), damage = 30. E12 hp: 20 - 30 = -10 → KO.
/// Hop 1 (k=1): E12 KO'd (alive=false), next lowest = E11 (700‰). Damage = 15.
/// Hop 2 (k=2): E12 KO'd, E11 now at 55/100 (550‰) vs E10 at 900‰ → E11 wins. Damage = 5.
///
/// Note: AllowRepeat only matters when the previously-hit target is still alive
/// and would otherwise be excluded. In this case E12 KOs on hop 0, so repeat
/// doesn't matter for hops 1+. The per-hop damage curve is still verified.
#[test]
fn bounce_lowest_hp_allow_repeat_per_hop_curve() {
    let book = bounce_skill(
        "scatter_bolt",
        2,
        3,
        BounceSelector::LowestHpPctAlive,
        RepeatPolicy::AllowRepeat,
        0,
        DamageCurve::PerHop(vec![30, 15, 5]),
    );
    let mut app = build_app(book, 5);

    spawn_attacker(&mut app, 1, 0, "scatter_bolt");
    spawn_enemy_with_hp(&mut app, 10, 0, 100, 90); // 900‰
    spawn_enemy_with_hp(&mut app, 11, 1, 100, 70); // 700‰
    spawn_enemy_with_hp(&mut app, 12, 2, 100, 20); // 200‰ — lowest

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("scatter_bolt".into()),
        target: UnitId(12), // primary = lowest HP
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let amounts = damage_amounts(&events);

    // PerHop curve must produce exactly [30, 15, 5].
    assert_eq!(
        amounts.len(),
        3,
        "Case 3: expected 3 hops, got {}",
        amounts.len()
    );
    assert_eq!(
        amounts,
        vec![30, 15, 5],
        "Case 3: PerHop curve must be [30, 15, 5]"
    );

    // Hop 0 should hit E12 (lowest HP%).
    let targets = damage_targets(&events);
    assert_eq!(
        targets[0],
        UnitId(12),
        "Case 3: first hop must hit E12 (lowest HP%)"
    );
}

// ── Case 4: Pool exhaustion truncates silently ────────────────────────────────

/// Case 4: Requesting 4 hops but only 2 alive enemies available with NoRepeat.
///
/// After hitting both enemies, the pool is exhausted; remaining hops are silently
/// dropped (no error event, chain just stops at 2).
///
/// 2 enemies at slots 0 and 1.
/// Bounce with hops=4, LowestHpPct + NoRepeat + Constant(15).
///
/// Hop 0: E10 (100‰) picked. Already_hit={E10}.
/// Hop 1: E11 (100‰) picked. Already_hit={E10, E11}.
/// Hop 2: No alive enemy not in already_hit → None → break.
/// Hop 3: unreachable.
///
/// Exactly 2 OnDamageDealt events emitted; no OnActionFailed.
#[test]
fn bounce_pool_exhaustion_truncates_silently() {
    let shape = TargetShape::Bounce {
        hops: 4,
        selector: BounceSelector::LowestHpPctAlive,
        repeat: RepeatPolicy::NoRepeat,
    };
    let book = SkillBook(vec![SkillDef {
        id: SkillId("arc_bolt".into()),
        name: "Arc Bolt".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 2,
        targeting: SkillTargeting {
            shape,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![Effect::Damage {
            amount: 15,
            target: shape,
            per_hop: DamageCurve::Constant,
        }],
        ..Default::default()
    }]);

    let mut app = build_app(book, 5);

    spawn_attacker(&mut app, 1, 0, "arc_bolt");
    spawn_enemy(&mut app, 10, 0, 100);
    spawn_enemy(&mut app, 11, 1, 100);
    // Only 2 enemies, but hops=4.

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("arc_bolt".into()),
        target: UnitId(10),
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let amounts = damage_amounts(&events);

    // Exactly 2 hops (one per enemy), chain truncated silently.
    assert_eq!(
        amounts.len(),
        2,
        "Case 4: pool exhaustion should truncate to 2 hops, got {}",
        amounts.len()
    );
    assert_eq!(
        amounts,
        vec![15, 15],
        "Case 4: Constant curve must deal 15 each hop"
    );

    // No OnActionFailed emitted.
    let failed = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnActionFailed { .. }))
        .count();
    assert_eq!(
        failed, 0,
        "Case 4: pool exhaustion must not emit OnActionFailed"
    );
}
