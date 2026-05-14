/// Integration tests for Effect::Heal kernel primitive (M019/S02).
///
/// Cases:
///   1 — single heal on damaged ally: floor(hp_max * pct / 100), capped, OnHealed emitted
///   2 — single heal at full HP: amount=0, OnHealed still emitted
///   3 — single heal on KO target: no state change, no event, sp_ok=true
///   4 — AllAllies fan-out: KO slot skipped, 2 alive receive OnHealed ordered by slot_index asc
///   5 — cap test: ally at hp_max-3 with 50% heal → healed exactly 3, hp_after==hp_max
use bevyrogue::combat::{
    events::CombatEventKind,
    resolution::apply_heal_only,
    state::{ResolvedAction, UltEffect},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    unit::Unit,
};
use bevyrogue::data::skills_ron::TargetShape;

fn ally(id: u32, hp_current: i32, hp_max: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("ally{id}"),
        hp_max,
        hp_current,
        attribute: Attribute::Data,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn heal_action(pct: u32) -> ResolvedAction {
    ResolvedAction {
        source: UnitId(1),
        target: UnitId(1),
        skill_id: SkillId("heal".into()),
        damage_tag: DamageTag::Light,
        base_damage: 0,
        toughness_damage: 0,
        revive_pct: 0,
        heal_pct: pct,
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
    }
}

#[test]
fn single_heal_on_damaged_ally() {
    // hp_max=100, hp_current=60, pct=50 → raw=50, cap=40, healed=40, hp_after=100
    let action = heal_action(50);
    let mut unit = ally(2, 60, 100);
    let (_outcome, events) = apply_heal_only(&action, &mut unit);
    assert_eq!(unit.hp_current, 100);
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        CombatEventKind::OnHealed { amount: 40, hp_after: 100 }
    ));
}

#[test]
fn single_heal_at_full_hp_emits_zero_amount() {
    // hp_max=100, hp_current=100, pct=50 → raw=50, cap=0, healed=0, event still emitted
    let action = heal_action(50);
    let mut unit = ally(2, 100, 100);
    let (_outcome, events) = apply_heal_only(&action, &mut unit);
    assert_eq!(unit.hp_current, 100);
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        CombatEventKind::OnHealed { amount: 0, hp_after: 100 }
    ));
}

#[test]
fn single_heal_on_ko_is_no_op() {
    // KO target (hp_current=0): no state change, no event emitted, sp_ok=true
    let action = heal_action(50);
    let mut unit = ally(2, 0, 100);
    let (outcome, events) = apply_heal_only(&action, &mut unit);
    assert_eq!(unit.hp_current, 0);
    assert!(events.is_empty());
    assert!(outcome.sp_ok);
}

#[test]
fn all_allies_fan_out_ko_skipped_alive_healed_slot_order() {
    // 3 allies ordered by slot_index (simulating resolve_targets AllAllies output):
    //   slot=0: alive, hp=50/100
    //   slot=1: KO,   hp=0/100
    //   slot=2: alive, hp=70/100
    // 30% heal: slot0 → +30 (hp_after=80), slot1 → no-op, slot2 → +30 (hp_after=100)
    let action = heal_action(30);
    let mut slot0 = ally(1, 50, 100);
    let mut slot1_ko = ally(2, 0, 100);
    let mut slot2 = ally(3, 70, 100);

    let (_, events0) = apply_heal_only(&action, &mut slot0);
    let (_, events1) = apply_heal_only(&action, &mut slot1_ko);
    let (_, events2) = apply_heal_only(&action, &mut slot2);

    // KO unit: unchanged state, no events
    assert_eq!(slot1_ko.hp_current, 0);
    assert!(events1.is_empty());

    // slot0 healed by 30
    assert_eq!(slot0.hp_current, 80);
    assert_eq!(events0.len(), 1);
    assert!(matches!(
        events0[0],
        CombatEventKind::OnHealed { amount: 30, hp_after: 80 }
    ));

    // slot2 healed by 30, reaches hp_max
    assert_eq!(slot2.hp_current, 100);
    assert_eq!(events2.len(), 1);
    assert!(matches!(
        events2[0],
        CombatEventKind::OnHealed { amount: 30, hp_after: 100 }
    ));
}

#[test]
fn heal_cap_at_hp_max() {
    // hp_max=100, hp_current=97, pct=50 → raw=50, cap=3, healed exactly 3, hp_after=hp_max
    let action = heal_action(50);
    let mut unit = ally(2, 97, 100);
    let (_outcome, events) = apply_heal_only(&action, &mut unit);
    assert_eq!(unit.hp_current, 100);
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        CombatEventKind::OnHealed { amount: 3, hp_after: 100 }
    ));
}
