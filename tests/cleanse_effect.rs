/// Integration tests for Effect::Cleanse kernel primitive (M019/S03).
///
/// Cases:
///   1 — count=Some(2) on 4-debuff bag: removes the 2 longest-duration debuffs
///   2 — count=Some(2) with duration tie: lower insertion index removed first
///   3 — count=None removes all debuffs, Blessed survives
///   4 — count=Some(0) emits empty OnCleansed, no state change
///   5 — Blessed-only bag with count=Some(5): no removals, empty OnCleansed
///   6 — count exceeds debuff count: removes all without panic
///   7 — KO target: no state change, no event emitted
///   8 — empty StatusBag: emits empty OnCleansed
use bevyrogue::combat::{
    events::CombatEventKind,
    resolution::apply_cleanse_only,
    state::{ResolvedAction, UltEffect},
    status_effect::{StatusBag, StatusEffectKind},
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

fn cleanse_action(count: Option<u8>) -> ResolvedAction {
    ResolvedAction {
        source: UnitId(1),
        target: UnitId(2),
        skill_id: SkillId("cleanse".into()),
        damage_tag: DamageTag::Light,
        base_damage: 0,
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
        cleanse_count: Some(count),
    }
}

#[test]
fn cleanse_count_some_two_removes_two_longest_debuffs() {
    // 4 debuffs at durations 3,1,2,4; count=Some(2) → removes dur4 then dur3
    let action = cleanse_action(Some(2));
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 3);    // idx 0
    bag.apply(StatusEffectKind::Chilled, 1);   // idx 1
    bag.apply(StatusEffectKind::Paralyzed, 2); // idx 2
    bag.apply(StatusEffectKind::Slowed, 4);    // idx 3

    let (_outcome, events) = apply_cleanse_only(&action, &mut bag, true);

    assert_eq!(events.len(), 1);
    let CombatEventKind::OnCleansed { kinds } = &events[0] else {
        panic!("expected OnCleansed");
    };
    // Selection order: dur4 (Slowed) first, then dur3 (Heated)
    assert_eq!(kinds, &[StatusEffectKind::Slowed, StatusEffectKind::Heated]);
    // Remaining: Chilled(1) and Paralyzed(2)
    assert!(!bag.has(&StatusEffectKind::Slowed));
    assert!(!bag.has(&StatusEffectKind::Heated));
    assert!(bag.has(&StatusEffectKind::Chilled));
    assert!(bag.has(&StatusEffectKind::Paralyzed));
}

#[test]
fn cleanse_count_some_two_tie_break_lower_insertion_index_first() {
    // Two debuffs at the same duration: the one inserted first (lower idx) is selected first
    let action = cleanse_action(Some(2));
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 5);    // idx 0 — ties with Chilled
    bag.apply(StatusEffectKind::Chilled, 5);   // idx 1

    let (_outcome, events) = apply_cleanse_only(&action, &mut bag, true);

    assert_eq!(events.len(), 1);
    let CombatEventKind::OnCleansed { kinds } = &events[0] else {
        panic!("expected OnCleansed");
    };
    // Both removed; idx-0 (Heated) appears first in kinds vec
    assert_eq!(kinds, &[StatusEffectKind::Heated, StatusEffectKind::Chilled]);
    assert!(!bag.has(&StatusEffectKind::Heated));
    assert!(!bag.has(&StatusEffectKind::Chilled));
}

#[test]
fn cleanse_count_none_removes_all_debuffs_keeps_blessed() {
    // count=None → remove all debuffs; Blessed (buff) must survive
    let action = cleanse_action(None);
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Blessed, 3);
    bag.apply(StatusEffectKind::Heated, 2);
    bag.apply(StatusEffectKind::Slowed, 1);
    bag.apply(StatusEffectKind::Paralyzed, 4);

    let (_outcome, events) = apply_cleanse_only(&action, &mut bag, true);

    assert_eq!(events.len(), 1);
    let CombatEventKind::OnCleansed { kinds } = &events[0] else {
        panic!("expected OnCleansed");
    };
    assert_eq!(kinds.len(), 3);
    assert!(!kinds.contains(&StatusEffectKind::Blessed));
    assert!(bag.has(&StatusEffectKind::Blessed));
    assert!(!bag.has(&StatusEffectKind::Heated));
    assert!(!bag.has(&StatusEffectKind::Slowed));
    assert!(!bag.has(&StatusEffectKind::Paralyzed));
}

#[test]
fn cleanse_count_some_zero_emits_empty_event_no_state_change() {
    // count=Some(0) → no removals, OnCleansed { kinds: [] } still emitted
    let action = cleanse_action(Some(0));
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 2);
    bag.apply(StatusEffectKind::Chilled, 3);

    let (_outcome, events) = apply_cleanse_only(&action, &mut bag, true);

    assert_eq!(events.len(), 1);
    let CombatEventKind::OnCleansed { kinds } = &events[0] else {
        panic!("expected OnCleansed");
    };
    assert!(kinds.is_empty());
    assert!(bag.has(&StatusEffectKind::Heated));
    assert!(bag.has(&StatusEffectKind::Chilled));
}

#[test]
fn cleanse_blessed_only_no_op() {
    // Only Blessed (a buff) in the bag; count=Some(5) → no removals, empty event
    let action = cleanse_action(Some(5));
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Blessed, 3);

    let (_outcome, events) = apply_cleanse_only(&action, &mut bag, true);

    assert_eq!(events.len(), 1);
    let CombatEventKind::OnCleansed { kinds } = &events[0] else {
        panic!("expected OnCleansed");
    };
    assert!(kinds.is_empty());
    assert!(bag.has(&StatusEffectKind::Blessed));
}

#[test]
fn cleanse_count_exceeds_debuff_count_removes_all_no_panic() {
    // count=Some(10) with only 2 debuffs → removes both without panic
    let action = cleanse_action(Some(10));
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 2);
    bag.apply(StatusEffectKind::Slowed, 1);

    let (_outcome, events) = apply_cleanse_only(&action, &mut bag, true);

    assert_eq!(events.len(), 1);
    let CombatEventKind::OnCleansed { kinds } = &events[0] else {
        panic!("expected OnCleansed");
    };
    assert_eq!(kinds.len(), 2);
    assert!(!bag.has(&StatusEffectKind::Heated));
    assert!(!bag.has(&StatusEffectKind::Slowed));
}

#[test]
fn cleanse_on_ko_target_no_op_no_event() {
    // defender_alive=false (KO) → no state change, no event emitted
    let action = cleanse_action(Some(2));
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 3);
    bag.apply(StatusEffectKind::Chilled, 2);

    let (_outcome, events) = apply_cleanse_only(&action, &mut bag, false);

    assert!(events.is_empty());
    // Bag untouched
    assert!(bag.has(&StatusEffectKind::Heated));
    assert!(bag.has(&StatusEffectKind::Chilled));
}

#[test]
fn cleanse_on_empty_bag_emits_empty_event() {
    // Empty StatusBag, count=Some(3) → no removals, OnCleansed { kinds: [] }
    let action = cleanse_action(Some(3));
    let mut bag = StatusBag::default();

    let (_outcome, events) = apply_cleanse_only(&action, &mut bag, true);

    assert_eq!(events.len(), 1);
    let CombatEventKind::OnCleansed { kinds } = &events[0] else {
        panic!("expected OnCleansed");
    };
    assert!(kinds.is_empty());
}
