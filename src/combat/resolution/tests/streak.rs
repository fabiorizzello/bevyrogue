use super::super::*;
use super::*;
use crate::combat::events::CombatEventKind;
use crate::combat::kit::UnitSkills;
use crate::combat::{
    sp::{RoundSpTracker, SpPool},
    team::Team,
    toughness::Toughness,
    turn_system::ActionIntent,
    types::{Attribute, DamageTag, SkillId, UnitId},
    unit::BasicStreak,
};
use crate::data::skills_ron::SkillBook;

#[test]
fn grant_free_skill_resolve_sets_grant_count() {
    let intent = ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    };
    let skill = grant_free_skill_def("test_grant_skill", 4);
    let book = SkillBook(vec![skill.clone()]);
    let kit = UnitSkills {
        basic: skill.id.clone(),
        skills: vec![skill.id.clone()],
        ultimate: skill.id,
        follow_up: None,
    };
    let resolved = resolve_action(&intent, &kit, Some(&book)).expect("should resolve");
    assert_eq!(resolved.grant_free_skill_count, 4);
}

#[test]
fn grant_free_skill_events_emits_four_on_skill_cast() {
    let ally_basics: Vec<SkillId> = (1u32..=5).map(|i| SkillId(format!("basic_{i}"))).collect();
    let events = grant_free_skill_events(4, &ally_basics);
    assert_eq!(events.len(), 4, "expected exactly 4 OnSkillCast events");
    for (i, event) in events.iter().enumerate() {
        assert!(
            matches!(event, CombatEventKind::OnSkillCast { skill_id } if skill_id == &SkillId(format!("basic_{}", i + 1))),
            "event {i} should be OnSkillCast for basic_{}",
            i + 1
        );
    }
}

#[test]
fn grant_free_skill_events_caps_at_available_allies() {
    let ally_basics: Vec<SkillId> = vec![SkillId("basic_1".into()), SkillId("basic_2".into())];
    let events = grant_free_skill_events(4, &ally_basics);
    assert_eq!(events.len(), 2, "should not exceed available allies");
}

#[test]
fn test_apply_revive_fails_on_active() {
    let attacker = unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 50); // Not KO
    let mut tough = Toughness::new(50, vec![DamageTag::Light]);
    let mut ult = UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let mut sp = SpPool { current: 5, max: 5 };
    let intent = ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("revive".into()),
        target: UnitId(2),
    };
    let resolved = resolved(&intent, revive_skill("revive", 25, 4));

    let (_outcome, events) = apply_legacy_ops(
        &resolved,
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
        None,
        None,
    );

    assert_eq!(defender.hp_current, 50); // No change
    assert!(
        events
            .iter()
            .any(|e| matches!(e, CombatEventKind::OnActionFailed { .. }))
    );
}

#[test]
fn child_gets_minus1_sp_after_2_consecutive_basics() {
    let attacker = child_unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
    let mut ult = default_ult();
    let mut sp = SpPool { current: 5, max: 5 };

    // Two basics build up streak
    let basic = basic_intent();
    let basic_resolved = resolved(&basic, skill("basic", DamageTag::Fire, 5, 0, 0));
    let mut streak = BasicStreak::default();
    apply_legacy_ops(
        &basic_resolved,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut streak,
        false,
        false,
        None,
        None,
        None,
    );
    apply_legacy_ops(
        &basic_resolved,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut streak,
        false,
        false,
        None,
        None,
        None,
    );
    assert_eq!(streak.count, 2);
    assert!(streak.qualifies_for_discount());

    // Skill with sp_cost 3 should cost only 2 due to Child discount
    let intent = ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("skill".into()),
        target: UnitId(2),
    };
    let skill_resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 3, 0));
    sp.current = 3;
    let (outcome, _) = apply_legacy_ops(
        &skill_resolved,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut streak,
        false,
        false,
        None,
        None,
        None,
    );

    assert!(outcome.sp_ok, "skill should succeed with discounted cost");
    assert_eq!(sp.current, 1, "paid 2 SP not 3 (discount applied)");
    assert_eq!(streak.count, 0, "streak reset after discount");
}

#[test]
fn adult_gets_no_discount_after_consecutive_basics() {
    let attacker = unit(1, Attribute::Vaccine, 100); // Adult
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
    let mut ult = default_ult();
    let mut sp = SpPool { current: 5, max: 5 };

    let mut streak = BasicStreak::default();
    // Adult can still track streak internally but never gets discount
    streak.increment();
    streak.increment();
    assert!(streak.qualifies_for_discount());

    let intent = ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("skill".into()),
        target: UnitId(2),
    };
    let skill_resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 3, 0));
    sp.current = 3;
    let _ = apply_legacy_ops(
        &skill_resolved,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut streak,
        false,
        false,
        None,
        None,
        None,
    );

    assert_eq!(sp.current, 0, "Adult paid full 3 SP, no discount");
    assert_eq!(
        streak.count, 2,
        "Adult streak not reset (no discount applied)"
    );
}

#[test]
fn child_1_basic_not_enough_for_discount() {
    let attacker = child_unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
    let mut ult = default_ult();
    let mut sp = SpPool { current: 5, max: 5 };

    let mut streak = BasicStreak::default();
    streak.increment(); // Only 1 basic
    assert!(!streak.qualifies_for_discount());

    let intent = ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("skill".into()),
        target: UnitId(2),
    };
    let skill_resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 2, 0));
    let (outcome, _) = apply_legacy_ops(
        &skill_resolved,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut streak,
        false,
        false,
        None,
        None,
        None,
    );

    assert!(outcome.sp_ok);
    assert_eq!(sp.current, 3, "paid full 2 SP, no discount for 1 basic");
    assert_eq!(streak.count, 1, "streak unchanged");
}

#[test]
fn child_discount_resets_streak_needs_2_more_basics() {
    let attacker = child_unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
    let mut ult = default_ult();
    let mut sp = SpPool { current: 5, max: 5 };

    let mut streak = BasicStreak::default();
    streak.increment();
    streak.increment();

    // Use the discount
    let intent = ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("skill".into()),
        target: UnitId(2),
    };
    let skill_resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 3, 0));
    apply_legacy_ops(
        &skill_resolved,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut streak,
        false,
        false,
        None,
        None,
        None,
    );
    assert_eq!(streak.count, 0, "streak reset after discount use");

    // 1 more basic → still not enough
    streak.increment();
    assert!(
        !streak.qualifies_for_discount(),
        "needs 2 basics after reset"
    );

    // 2nd basic → qualifies again
    streak.increment();
    assert!(
        streak.qualifies_for_discount(),
        "2 basics after reset → qualifies again"
    );
}

#[test]
fn adult_5_consecutive_basics_no_discount() {
    let attacker = unit(1, Attribute::Vaccine, 100); // Adult
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
    let mut ult = default_ult();
    let mut sp = SpPool { current: 5, max: 5 };

    let mut streak = BasicStreak::default();
    for _ in 0..5 {
        streak.increment();
    }
    assert!(streak.qualifies_for_discount(), "streak counts up");

    let intent = ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("skill".into()),
        target: UnitId(2),
    };
    let skill_resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 3, 0));
    let (outcome, _) = apply_legacy_ops(
        &skill_resolved,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut streak,
        false,
        false,
        None,
        None,
        None,
    );

    assert!(outcome.sp_ok);
    assert_eq!(sp.current, 2, "Adult paid full 3 SP even with 5 basics");
    assert_eq!(streak.count, 5, "Adult streak unchanged");
}
