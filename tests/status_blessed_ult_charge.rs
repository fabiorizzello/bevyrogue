/// Integration tests for §H.1 Blessed +1 Ult charge per action (S05/T03).
/// Three cases: baseline (no Blessed), Blessed Basic, Blessed Ultimate-cast (Reset → no leak).
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
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

fn defender() -> Unit {
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

fn fresh_ult() -> UltimateCharge {
    UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

fn basic_resolved() -> ResolvedAction {
    ResolvedAction {
        source: UnitId(1),
        target: UnitId(2),
        skill_id: SkillId("basic".into()),
        damage_tag: DamageTag::Physical,
        base_damage: 100,
        toughness_damage: 0,
        revive_pct: 0,
        sp_cost: 0,
        ult_effect: UltEffect::GainFromBasic,
        grant_free_skill_count: 0,
        status_to_apply: None,
        advance_pct: 0,
        delay_pct: 0,
        energy_grant: 0,
        self_advance_pct: 0,
        target_shape: TargetShape::Single,
        custom_signals: vec![],
        damage_curve: Default::default(),
    }
}

fn ult_resolved() -> ResolvedAction {
    ResolvedAction {
        ult_effect: UltEffect::Reset,
        sp_cost: 0,
        base_damage: 100,
        ..basic_resolved()
    }
}

fn run(resolved: &ResolvedAction, attacker_bag: Option<&StatusBag>) -> i32 {
    let atk = attacker();
    let mut def = defender();
    let mut tough = Toughness::new(1_000, vec![]);
    let mut ult = fresh_ult();
    let before = ult.current;
    let mut sp = SpPool { current: 5, max: 5 };

    apply_effects(
        resolved,
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
        attacker_bag,
        None,
    );

    ult.current - before
}

fn run_ult_action(attacker_bag: Option<&StatusBag>) -> i32 {
    let atk = attacker();
    let mut def = defender();
    let mut tough = Toughness::new(1_000, vec![]);
    let mut ult = UltimateCharge {
        current: 100, // ready
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let mut sp = SpPool { current: 5, max: 5 };
    let resolved = ult_resolved();

    apply_effects(
        &resolved,
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
        attacker_bag,
        None,
    );

    ult.current
}

/// Baseline: no Blessed → delta equals charge_per_event (25).
#[test]
fn baseline_no_blessed_basic_action() {
    let delta = run(&basic_resolved(), None);
    assert_eq!(delta, 25, "baseline delta must equal charge_per_event=25, got {delta}");
}

/// Blessed Basic action → delta = baseline + 1 = 26.
#[test]
fn blessed_basic_action_gains_extra_charge() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Blessed, 2);
    let delta = run(&basic_resolved(), Some(&bag));
    assert_eq!(delta, 26, "Blessed Basic must add 1 extra charge (25+1=26), got {delta}");
}

/// Blessed Ultimate-cast (Reset branch) → meter resets to 0, Blessed +1 does NOT leak.
#[test]
fn blessed_ult_cast_no_charge_leak() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Blessed, 2);
    let current = run_ult_action(Some(&bag));
    assert_eq!(current, 0, "Ult Reset must zero meter; Blessed must not leak +1, got {current}");
}
