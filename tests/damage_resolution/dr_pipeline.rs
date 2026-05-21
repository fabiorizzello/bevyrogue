use bevyrogue::combat::sp::RoundSpTracker;
/// Integration tests for the DR pipeline (M019/S01 T04).
///
/// Cases:
///   A — single DR=30%: base=100, neutral → 70
///   B — stacked DR=20%+30%=50%: base=100, neutral → 50
///   C — DR=30% + Fire resist (0.75): 100×0.75×0.70 = 52.5 → 53
///   D — DR=30% when toughness already Broken: DR still applies → 70
///   E — DR=100%: damage clamped to 0; OnDamageDealt emitted with amount=0
///   F — DR=120% (unclamped sum > 1): no panic; damage=0
use bevyrogue::combat::{
    buffs::DrBag,
    events::CombatEventKind,
    resolution::apply_legacy_ops,
    sp::SpPool,
    state::{ResolvedAction, UltEffect},
    team::Team,
    toughness::Toughness,
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{BasicStreak, Unit},
};
use bevyrogue::data::skills_ron::TargetShape;

fn attacker() -> Unit {
    Unit {
        id: UnitId(1),
        name: "attacker".into(),
        hp_max: 1_000,
        hp_current: 1_000,
        attribute: Attribute::Data,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn defender(resists: Vec<DamageTag>) -> Unit {
    Unit {
        id: UnitId(2),
        name: "defender".into(),
        hp_max: 100_000,
        hp_current: 100_000,
        attribute: Attribute::Data,
        resists,
        evo_stage: EvoStage::Adult,
    }
}

fn default_ult() -> UltimateCharge {
    UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

fn damage_action(tag: DamageTag, base: i32) -> ResolvedAction {
    ResolvedAction {
        source: UnitId(1),
        target: UnitId(2),
        skill_id: SkillId("dmg".into()),
        damage_tag: tag,
        base_damage: base,
        toughness_damage: 0,
        revive_pct: 0,
        heal_pct: 0,
        sp_cost: 0,
        ult_effect: UltEffect::None,
        grant_free_skill_count: 0,
        status_to_apply: None,
        advance_pct: 0,
        delay_pct: 0,
        energy_grant: 0,
        self_advance_pct: 0,
        target_shape: TargetShape::Single,
        custom_signals: vec![],
        damage_curve: Default::default(),
        cleanse_count: None,
    }
}

/// Run apply_legacy_ops with the given parameters; return (OnDamageDealt amount, all events).
fn run(
    base: i32,
    tag: DamageTag,
    resists: Vec<DamageTag>,
    dr_bag: Option<&DrBag>,
    toughness_broken: bool,
) -> (i32, Vec<CombatEventKind>) {
    let atk = attacker();
    let mut def = defender(resists);
    let mut tough = {
        let mut t = Toughness::new(1_000, vec![]);
        t.broken = toughness_broken;
        t
    };
    let mut ult = default_ult();
    let mut sp = SpPool {
        current: 99,
        max: 99,
    };
    let action = damage_action(tag, base);

    let (_outcome, events) = apply_legacy_ops(
        &action,
        &atk,
        &mut def,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut BasicStreak::default(),
        false,
        false,
        None,
        None,
        dr_bag,
            None,
            None,
        );

    let amount = events
        .iter()
        .find_map(|e| {
            if let CombatEventKind::OnDamageDealt { amount, .. } = e {
                Some(*amount)
            } else {
                None
            }
        })
        .unwrap_or(0);

    (amount, events)
}

// ─── Case A: single DR=30% ────────────────────────────────────────────────────

#[test]
fn dr_single_30pct_reduces_damage() {
    // base=100, neutral tag+tri, dr=0.30 → factor=0.70 → 70
    let mut bag = DrBag::default();
    bag.apply(0.30, 2);

    let (amount, events) = run(100, DamageTag::Physical, vec![], Some(&bag), false);

    assert_eq!(amount, 70, "30% DR should yield 70; got {amount}");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, CombatEventKind::OnDamageDealt { amount: 70, .. })),
        "OnDamageDealt with amount=70 must be present"
    );
}

// ─── Case B: stacked DR=20%+30%=50% ──────────────────────────────────────────

#[test]
fn dr_stacked_sums_unclamped() {
    // Two instances: 0.20 + 0.30 = 0.50 → factor=0.50 → 50
    let mut bag = DrBag::default();
    bag.apply(0.20, 3);
    bag.apply(0.30, 3);

    let (amount, _) = run(100, DamageTag::Physical, vec![], Some(&bag), false);

    assert_eq!(
        amount, 50,
        "Stacked 20%+30% DR should yield 50; got {amount}"
    );
}

// ─── Case C: DR=30% + Fire resist (0.75 tag mod) ─────────────────────────────

#[test]
fn dr_combined_with_resist_stacks_multiplicatively() {
    // Fire base=100, defender resists Fire (tag_mod=0.75), DR=30% (factor=0.70)
    // → 100 × 0.75 × 0.70 = 52.5 → 53
    let mut bag = DrBag::default();
    bag.apply(0.30, 2);

    let (amount, _) = run(
        100,
        DamageTag::Fire,
        vec![DamageTag::Fire],
        Some(&bag),
        false,
    );

    assert_eq!(
        amount, 53,
        "DR + resist should stack multiplicatively to 53; got {amount}"
    );
}

// ─── Case D: DR during Break (toughness already broken) ───────────────────────

#[test]
fn dr_applies_when_toughness_already_broken() {
    // Toughness already in broken state; DR should still reduce HP damage normally.
    let mut bag = DrBag::default();
    bag.apply(0.30, 2);

    let (amount, _) = run(100, DamageTag::Physical, vec![], Some(&bag), true);

    assert_eq!(
        amount, 70,
        "DR must reduce HP damage even when toughness is already Broken; got {amount}"
    );
}

// ─── Case E: DR=100% clamps damage to 0; event still emitted ─────────────────

#[test]
fn dr_100pct_clamps_to_zero_and_event_emitted() {
    // DR=1.0 → factor=0.0 → damage=0; OnDamageDealt must still be emitted.
    let mut bag = DrBag::default();
    bag.apply(1.0, 2);

    let (amount, events) = run(100, DamageTag::Physical, vec![], Some(&bag), false);

    assert_eq!(amount, 0, "100% DR must clamp damage to 0; got {amount}");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, CombatEventKind::OnDamageDealt { amount: 0, .. })),
        "OnDamageDealt with amount=0 must be emitted even when DR absorbs all damage"
    );
}

// ─── Case F: DR>100% (unclamped sum > 1.0): no panic, damage stays 0 ─────────

#[test]
fn dr_over_100pct_no_panic_damage_zero() {
    // sum_dr = 0.60 + 0.60 = 1.20; (1.0 - 1.20).max(0.0) = 0.0 → damage=0, no panic.
    let mut bag = DrBag::default();
    bag.apply(0.60, 2);
    bag.apply(0.60, 2);

    let (amount, _) = run(100, DamageTag::Physical, vec![], Some(&bag), false);

    assert_eq!(
        amount, 0,
        "DR > 100% must not panic and must clamp damage to 0; got {amount}"
    );
}
