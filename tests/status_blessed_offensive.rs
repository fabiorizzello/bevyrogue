/// Integration test for §H.1 Blessed offensive bonus (S05/T02):
/// attacker with Blessed deals ×1.15 damage; without Blessed, baseline.
///
/// Uses `apply_effects` directly (no full Bevy world needed) to isolate the
/// Blessed dmg-mult computation path in resolution.rs → calculate_damage.
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
    events::CombatEventKind,
    resolution::apply_effects,
    sp::SpPool,
    state::{ResolvedAction, UltEffect},
    team::Team,
    toughness::Toughness,
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{BasicStreak, Unit},
};
use bevyrogue::combat::sp::RoundSpTracker;

fn attacker_unit() -> Unit {
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

fn defender_unit() -> Unit {
    Unit {
        id: UnitId(2),
        name: "defender".into(),
        hp_max: 1_000,
        hp_current: 1_000,
        attribute: Attribute::Data,
        resists: vec![],
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

/// Minimal ResolvedAction for a Basic attack with given base_damage.
fn basic_action(base_damage: i32) -> ResolvedAction {
    ResolvedAction {
        source: UnitId(1),
        target: UnitId(2),
        skill_id: SkillId("basic".into()),
        damage_tag: DamageTag::Physical,
        base_damage,
        toughness_damage: 0,
        revive_pct: 0,
        heal_pct: 0,
        sp_cost: 0,
        ult_effect: UltEffect::GainFromBasic,
        grant_free_skill_count: 0,
        status_to_apply: None,
        advance_pct: 0,
        delay_pct: 0,
        energy_grant: 0,
        self_advance_pct: 0,
        target_shape: bevyrogue::data::skills_ron::TargetShape::Single,
        custom_signals: vec![],
        damage_curve: Default::default(),
    }
}

fn run_apply(base_damage: i32, attacker_bag: Option<&StatusBag>) -> i32 {
    let attacker = attacker_unit();
    let mut defender = defender_unit();
    let mut tough = Toughness::new(1_000, vec![]);
    let mut ult = default_ult();
    let mut sp = SpPool { current: 5, max: 5 };
    let action = basic_action(base_damage);

    let (_, events) = apply_effects(
        &action,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut BasicStreak::default(),
        false,
        false,
        None,
        attacker_bag,
        None,
    );

    events
        .iter()
        .find_map(|e| match e {
            CombatEventKind::OnDamageDealt { amount, .. } => Some(*amount),
            _ => None,
        })
        .expect("OnDamageDealt must be emitted")
}

/// Blessed attacker: base=100, tag=neutral(1.0), tri=tie(1.0), break=no → 100×1.15 = 115.
#[test]
fn blessed_attacker_deals_115_pct_damage() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Blessed, 2);

    let dmg = run_apply(100, Some(&bag));
    assert_eq!(dmg, 115, "Blessed must apply ×1.15 mult: expected 115, got {dmg}");
}

/// No-Blessed attacker: base=100, all modifiers 1.0 → 100.
#[test]
fn no_blessed_attacker_deals_baseline_damage() {
    let dmg = run_apply(100, None);
    assert_eq!(dmg, 100, "No Blessed: expected baseline 100, got {dmg}");
}

/// Empty-bag (Blessed absent): same as no-bag path.
#[test]
fn empty_bag_attacker_deals_baseline_damage() {
    let bag = StatusBag::default();
    let dmg = run_apply(100, Some(&bag));
    assert_eq!(dmg, 100, "Empty bag must not activate Blessed mult: expected 100, got {dmg}");
}

/// Verify that non-Blessed statuses on attacker do not activate the ×1.15 bonus.
#[test]
fn heated_attacker_does_not_get_blessed_bonus() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 2);

    let dmg = run_apply(100, Some(&bag));
    assert_eq!(dmg, 100, "Heated ≠ Blessed: expected 100, got {dmg}");
}
