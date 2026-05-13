use super::*;
use crate::combat::{
    sp::{RoundSpTracker, SpPool},
    toughness::Toughness,
    turn_system::ActionIntent,
    types::{Attribute, DamageTag, EvoStage, UnitId},
    ultimate::UltAccumulationTrigger,
    unit::BasicStreak,
};
use crate::data::skills_ron::{
    Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
    SkillTargeting, TargetLife, TargetShape, TargetSide,
};

fn grant_free_skill_def(id: &str, grant_count: usize) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag: DamageTag::Light,
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
                amount: 30,
                target: TargetShape::Single,
            },
            Effect::ToughnessHit(15),
            Effect::GrantFreeSkill { count: grant_count },
        ],

        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        ..Default::default()
    }
}

fn unit(id: u32, attribute: Attribute, hp_current: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max: 100,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn child_unit(id: u32, attribute: Attribute, hp_current: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("ChildUnit{id}"),
        hp_max: 100,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Child,
    }
}

fn skill(
    id: &str,
    damage_tag: DamageTag,
    damage: i32,
    sp_cost: i32,
    toughness_damage: i32,
) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag,
        sp_cost,
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
                amount: damage,
                target: TargetShape::Single,
            },
            Effect::ToughnessHit(toughness_damage),
        ],

        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        ..Default::default()
    }
}

fn revive_skill(id: &str, pct: i32, sp_cost: i32) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag: DamageTag::Light,
        sp_cost,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Ally,
            life: TargetLife::Ko,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![Effect::Revive(pct)],

        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        ..Default::default()
    }
}

fn resolved(intent: &ActionIntent, skill: SkillDef) -> ResolvedAction {
    let book = SkillBook(vec![skill.clone()]);
    let kit = UnitSkills {
        basic: skill.id.clone(),
        skills: vec![skill.id.clone()],
        ultimate: skill.id,
        follow_up: None,
    };
    resolve_action(intent, &kit, Some(&book)).expect("skill should resolve")
}

fn basic_intent() -> ActionIntent {
    ActionIntent::Basic {
        attacker: UnitId(1),
        target: UnitId(2),
    }
}

#[test]
fn resolve_action_uses_targeting_shape_over_damage_effect_shape() {
    let intent = ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("row".into()),
        target: UnitId(2),
    };
    let skill = SkillDef {
        id: SkillId("row".into()),
        name: "Row".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 3,
        targeting: SkillTargeting {
            shape: TargetShape::Row,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape,
        },
        effects: vec![Effect::Damage {
            amount: 12,
            target: TargetShape::Single,
        }],

        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        ..Default::default()
    };

    let resolved = resolved(&intent, skill);

    assert_eq!(resolved.target_shape, TargetShape::Row);
}

#[test]
fn resolve_action_uses_explicit_targeting_shape_for_revive_skills() {
    let intent = ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("revive".into()),
        target: UnitId(2),
    };
    let skill = revive_skill("revive", 25, 6);

    let expected_shape = skill.targeting.shape;
    let resolved = resolved(&intent, skill);

    assert_eq!(resolved.target_shape, expected_shape);
}

#[test]
fn resolve_apply_basic_adds_sp_and_not_on_skill_cast() {
    let attacker = unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
    let mut ult = UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let mut sp = SpPool { current: 3, max: 5 };
    let resolved = resolved(&basic_intent(), skill("basic", DamageTag::Fire, 10, 0, 5));

    let (outcome, events) = apply_effects(
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
    );

    assert!(outcome.sp_ok);
    assert_eq!(sp.current, 4);
    assert_eq!(ult.current, 25); // charge_per_event for this UltimateCharge
    assert!(defender.hp_current < 100);
    // Basic attacks now emit both OnDamageDealt and OnSkillCast (same as Skill/Ultimate).
    assert!(matches!(
        events.as_slice(),
        [
            CombatEventKind::OnDamageDealt { .. },
            CombatEventKind::OnSkillCast { .. }
        ]
    ));
}

#[test]
fn resolve_apply_skill_spends_sp_and_emits_on_skill_cast() {
    let attacker = unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
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
        skill_id: SkillId("skill".into()),
        target: UnitId(2),
    };
    let resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 4, 5));

    let (outcome, events) = apply_effects(
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
    );

    assert!(outcome.sp_ok);
    assert_eq!(sp.current, 1);
    assert!(events.iter().any(|event| matches!(
        event,
        CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("skill".into())
    )));
}

#[test]
fn resolve_apply_skill_fails_when_pool_too_low() {
    let attacker = unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
    let mut ult = UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let mut sp = SpPool { current: 1, max: 5 };
    let intent = ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("skill".into()),
        target: UnitId(2),
    };
    let resolved = resolved(&intent, skill("skill", DamageTag::Fire, 10, 4, 5));

    let (outcome, events) = apply_effects(
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
    );

    assert!(!outcome.sp_ok);
    assert_eq!(sp.current, 1);
    assert_eq!(defender.hp_current, 100);
    assert!(events.is_empty());
}

#[test]
fn resolve_apply_break_sets_broke_flag_and_on_break_event() {
    let attacker = unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(10, vec![DamageTag::Fire]);
    let mut ult = UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let mut sp = SpPool { current: 3, max: 5 };
    let resolved = resolved(&basic_intent(), skill("basic", DamageTag::Fire, 10, 0, 10));

    let (outcome, events) = apply_effects(
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
    );

    assert!(outcome.broke);
    assert_eq!(outcome.kind, DamageKind::Break);
    assert!(tough.broken);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, CombatEventKind::OnBreak { damage_tag } if *damage_tag == DamageTag::Fire))
    );
}

#[test]
fn resolve_apply_no_break_no_on_break_event() {
    let attacker = unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Fire]);
    let mut ult = UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let mut sp = SpPool { current: 3, max: 5 };
    let resolved = resolved(&basic_intent(), skill("basic", DamageTag::Fire, 10, 0, 5));

    let (outcome, events) = apply_effects(
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
    );

    assert!(!outcome.broke);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, CombatEventKind::OnBreak { .. }))
    );
}

#[test]
fn resolve_apply_ko_flag_when_hp_drops_below_zero_and_emits_on_ko() {
    let attacker = unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 5);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
    let mut ult = UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let mut sp = SpPool { current: 3, max: 5 };
    let resolved = resolved(&basic_intent(), skill("basic", DamageTag::Fire, 10, 0, 5));

    let (outcome, events) = apply_effects(
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
    );

    assert!(outcome.ko);
    assert!(defender.hp_current <= 0);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, CombatEventKind::OnKO))
    );
}

#[test]
fn resolve_apply_no_ko_no_on_ko_event() {
    let attacker = unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
    let mut ult = UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let mut sp = SpPool { current: 3, max: 5 };
    let resolved = resolved(&basic_intent(), skill("basic", DamageTag::Fire, 10, 0, 5));

    let (outcome, events) = apply_effects(
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
    );

    assert!(!outcome.ko);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, CombatEventKind::OnKO))
    );
}

#[test]
fn resolve_apply_ultimate_resets_charge_and_emits_on_skill_cast() {
    let attacker = unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Ice]);
    let mut ult = UltimateCharge {
        current: 100,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let mut sp = SpPool { current: 3, max: 5 };
    let intent = ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    };
    let resolved = resolved(&intent, skill("ultimate", DamageTag::Fire, 30, 0, 20));

    let (outcome, events) = apply_effects(
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
    );

    assert!(outcome.sp_ok);
    assert_eq!(ult.current, 0);
    assert!(events.iter().any(|event| matches!(
        event,
        CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("ultimate".into())
    )));
}

#[test]
fn test_apply_revive_success() {
    let attacker = unit(1, Attribute::Vaccine, 100);
    let mut defender = unit(2, Attribute::Virus, 0); // KO
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

    let (outcome, events) = apply_effects(
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
    );

    assert!(outcome.sp_ok);
    assert_eq!(defender.hp_current, 25); // 25% of 100
    assert!(
        events
            .iter()
            .any(|e| matches!(e, CombatEventKind::OnRevive { hp_after: 25 }))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, CombatEventKind::OnSkillCast { .. }))
    );
}

#[test]
fn grant_free_skill_resolve_sets_grant_count() {
    let intent = ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    };
    let skill = grant_free_skill_def("brave_tri_strike", 4);
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
            matches!(event, CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId(format!("basic_{}", i + 1))),
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

    let (_outcome, events) = apply_effects(
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
    );

    assert_eq!(defender.hp_current, 50); // No change
    assert!(
        events
            .iter()
            .any(|e| matches!(e, CombatEventKind::OnActionFailed { .. }))
    );
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
    apply_effects(
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
    );
    apply_effects(
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
    let (outcome, _) = apply_effects(
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
    let _ = apply_effects(
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
    let (outcome, _) = apply_effects(
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
    apply_effects(
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
    let (outcome, _) = apply_effects(
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
    );

    assert!(outcome.sp_ok);
    assert_eq!(sp.current, 2, "Adult paid full 3 SP even with 5 basics");
    assert_eq!(streak.count, 5, "Adult streak unchanged");
}
