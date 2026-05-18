mod types;
mod skill_extract;
mod apply;

// ── Re-exports ──────────────────────────────────────────────────────────────
// All public items are re-exported so external consumers keep using
// `crate::combat::resolution::Foo` unchanged.

#[allow(unused_imports)]
pub use types::{
    ResolutionOutcome, TargetEntry, TargetableSnapshot,
    resolve_targets, select_bounce_hop,
    target_shape_is_executable_now, target_shape_rejection_reason,
};

#[allow(unused_imports)]
pub use skill_extract::{
    compute_hop_damage, resolve_action, skill_damage_curve,
};

pub use apply::{
    apply_cleanse_only, apply_damage_only, apply_heal_only, apply_legacy_ops,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use crate::combat::{
        sp::{RoundSpTracker, SpPool},
        team::Team,
        toughness::Toughness,
        turn_system::ActionIntent,
        types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
        ultimate::UltAccumulationTrigger,
        unit::BasicStreak,
    };
    use crate::combat::events::CombatEventKind;
    use crate::combat::kit::UnitSkills;
    use crate::combat::state::UltEffect;
    use crate::combat::toughness::DamageKind;
    use crate::combat::ultimate::UltimateCharge;
    use crate::combat::unit::Unit;
    use crate::data::skills_ron::{
        Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    };
    use crate::data::skills_ron::{BounceSelector, RepeatPolicy};

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
            legacy_ops: vec![
                Effect::Damage {
                    amount: 30,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
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

    fn grant_free_skill_events(count: usize, ally_basics: &[SkillId]) -> Vec<CombatEventKind> {
        ally_basics
            .iter()
            .take(count)
            .cloned()
            .map(|skill_id| CombatEventKind::OnSkillCast { skill_id })
            .collect()
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
            legacy_ops: vec![
                Effect::Damage {
                    amount: damage,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
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
            legacy_ops: vec![Effect::Revive(pct)],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            ..Default::default()
        }
    }

    fn resolved(intent: &ActionIntent, skill: SkillDef) -> crate::combat::state::ResolvedAction {
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
            legacy_ops: vec![Effect::Damage {
                amount: 12,
                target: TargetShape::Single,
                per_hop: Default::default(),
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

        let (outcome, events) = apply_legacy_ops(
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

        let (outcome, events) = apply_legacy_ops(
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

        let (outcome, events) = apply_legacy_ops(
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

        let (outcome, events) = apply_legacy_ops(
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

        let (outcome, events) = apply_legacy_ops(
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

        let (outcome, events) = apply_legacy_ops(
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

        assert!(outcome.ko);
        assert!(defender.hp_current <= 0);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, CombatEventKind::UnitDied { .. }))
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

        let (outcome, events) = apply_legacy_ops(
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

        assert!(!outcome.ko);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, CombatEventKind::UnitDied { .. }))
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

        let (outcome, events) = apply_legacy_ops(
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

        let (outcome, events) = apply_legacy_ops(
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

    // ── resolve_targets table-driven tests ──────────────────────────────────

    fn snap(entries: Vec<(UnitId, Team, u8, bool)>) -> TargetableSnapshot {
        TargetableSnapshot {
            entries: entries
                .into_iter()
                .map(|(id, team, slot_index, alive)| TargetEntry {
                    id,
                    team,
                    slot_index,
                    alive,
                    hp_per_mille: 1000, // full HP default for shape tests that don't use HP selector
                })
                .collect(),
        }
    }

    /// Build a snapshot with explicit HP percentages (per-mille: 0–1000).
    fn snap_hp(entries: Vec<(UnitId, Team, u8, bool, u32)>) -> TargetableSnapshot {
        TargetableSnapshot {
            entries: entries
                .into_iter()
                .map(|(id, team, slot_index, alive, hp_per_mille)| TargetEntry {
                    id,
                    team,
                    slot_index,
                    alive,
                    hp_per_mille,
                })
                .collect(),
        }
    }

    #[test]
    fn resolve_targets_single_returns_primary() {
        let s = snap(vec![
            (UnitId(1), Team::Ally, 0, true),
            (UnitId(2), Team::Enemy, 0, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::Single, UnitId(2), &s),
            vec![UnitId(2)]
        );
    }

    #[test]
    fn resolve_targets_blast_edge_slot_zero_returns_only_0_and_1() {
        // primary at slot 0 → slot -1 absent → only slots 0 and 1
        let s = snap(vec![
            (UnitId(10), Team::Enemy, 0, true),
            (UnitId(11), Team::Enemy, 1, true),
            (UnitId(12), Team::Enemy, 2, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::Blast, UnitId(10), &s),
            vec![UnitId(10), UnitId(11)],
        );
    }

    #[test]
    fn resolve_targets_blast_ko_adjacent_omitted() {
        // primary at slot 1, slot 0 KO'd → only [slot1, slot2]
        let s = snap(vec![
            (UnitId(10), Team::Enemy, 0, false),
            (UnitId(11), Team::Enemy, 1, true),
            (UnitId(12), Team::Enemy, 2, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::Blast, UnitId(11), &s),
            vec![UnitId(11), UnitId(12)],
        );
    }

    #[test]
    fn resolve_targets_blast_all_three_alive_sorted_asc() {
        // Inserted out of order → sorted by slot_index
        let s = snap(vec![
            (UnitId(12), Team::Enemy, 2, true),
            (UnitId(10), Team::Enemy, 0, true),
            (UnitId(11), Team::Enemy, 1, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::Blast, UnitId(11), &s),
            vec![UnitId(10), UnitId(11), UnitId(12)],
        );
    }

    #[test]
    fn resolve_targets_all_enemies_omits_dead() {
        let s = snap(vec![
            (UnitId(1), Team::Ally, 0, true),
            (UnitId(10), Team::Enemy, 0, true),
            (UnitId(11), Team::Enemy, 1, false),
            (UnitId(12), Team::Enemy, 2, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::AllEnemies, UnitId(10), &s),
            vec![UnitId(10), UnitId(12)],
        );
    }

    #[test]
    fn resolve_targets_all_enemies_sorted_slot_asc() {
        let s = snap(vec![
            (UnitId(12), Team::Enemy, 2, true),
            (UnitId(10), Team::Enemy, 0, true),
            (UnitId(11), Team::Enemy, 1, true),
        ]);
        assert_eq!(
            resolve_targets(&TargetShape::AllEnemies, UnitId(12), &s),
            vec![UnitId(10), UnitId(11), UnitId(12)],
        );
    }

    // ── select_bounce_hop dispatcher tests ──────────────────────────────────

    #[test]
    fn bounce_lowest_hp_pct_picks_lowest_pct() {
        // Three enemies: slot 0 @ 500‰, slot 1 @ 300‰, slot 2 @ 800‰
        // LowestHpPctAlive should pick slot 1 (300‰)
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 500),
            (UnitId(11), Team::Enemy, 1, true, 300),
            (UnitId(12), Team::Enemy, 2, true, 800),
        ]);
        let already_hit = HashSet::new();
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, Some(UnitId(11)));
    }

    #[test]
    fn bounce_lowest_hp_pct_tiebreak_slot_asc() {
        // Three enemies all at 500‰; lowest slot_index should win
        let s = snap_hp(vec![
            (UnitId(12), Team::Enemy, 2, true, 500),
            (UnitId(10), Team::Enemy, 0, true, 500),
            (UnitId(11), Team::Enemy, 1, true, 500),
        ]);
        let already_hit = HashSet::new();
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, Some(UnitId(10))); // slot 0 wins tie
    }

    #[test]
    fn bounce_lowest_hp_pct_excludes_already_hit_no_repeat() {
        // slot 0 @ 100‰ (lowest), slot 1 @ 400‰ — slot 0 already hit → slot 1 wins
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 100),
            (UnitId(11), Team::Enemy, 1, true, 400),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(10));
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, Some(UnitId(11)));
    }

    #[test]
    fn bounce_lowest_hp_pct_allow_repeat_can_repick_same() {
        // Only one alive enemy; with NoRepeat it would return None (already in set),
        // but AllowRepeat allows re-selecting it.
        let s = snap_hp(vec![(UnitId(10), Team::Enemy, 0, true, 100)]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(10));
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::AllowRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(
            result,
            Some(UnitId(10)),
            "AllowRepeat should re-pick the only target"
        );

        // Confirm NoRepeat returns None in same scenario
        let result_no_repeat = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(
            result_no_repeat, None,
            "NoRepeat should return None when only target already hit"
        );
    }

    #[test]
    fn bounce_lowest_hp_pct_empty_pool_returns_none() {
        // No alive enemies at all
        let s = snap_hp(vec![(UnitId(10), Team::Enemy, 0, false, 0)]);
        let already_hit = HashSet::new();
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn bounce_next_slot_picks_next_above_last() {
        // Last hit slot = 0; candidates: slot 0 (already hit → excluded), slot 1, slot 2
        // NextSlotAlive should pick slot 1 (first slot > 0)
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 500),
            (UnitId(11), Team::Enemy, 1, true, 800),
            (UnitId(12), Team::Enemy, 2, true, 300),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(10));
        let result = select_bounce_hop(
            BounceSelector::NextSlotAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            Some(0),
        );
        assert_eq!(result, Some(UnitId(11)));
    }

    #[test]
    fn bounce_next_slot_no_slot_above_last_returns_none() {
        // Last hit = slot 2 (highest); no slot > 2 exists → None
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 500),
            (UnitId(11), Team::Enemy, 1, true, 500),
            (UnitId(12), Team::Enemy, 2, true, 500),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(12));
        let result = select_bounce_hop(
            BounceSelector::NextSlotAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            Some(2),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn bounce_next_slot_no_last_picks_lowest_slot() {
        // No last_slot → pick the alive enemy with the lowest slot_index
        let s = snap_hp(vec![
            (UnitId(12), Team::Enemy, 2, true, 300),
            (UnitId(10), Team::Enemy, 0, true, 800),
            (UnitId(11), Team::Enemy, 1, true, 500),
        ]);
        let already_hit = HashSet::new();
        let result = select_bounce_hop(
            BounceSelector::NextSlotAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, Some(UnitId(10))); // slot 0
    }

    #[test]
    fn bounce_adj_lowest_picks_adjacent_with_lowest_hp() {
        // Last hit slot = 1; adjacent = slots 0 and 2.
        // slot 0 @ 600‰, slot 2 @ 200‰ → slot 2 wins
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 600),
            (UnitId(11), Team::Enemy, 1, true, 500), // last hit, excluded by already_hit
            (UnitId(12), Team::Enemy, 2, true, 200),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(11));
        let result = select_bounce_hop(
            BounceSelector::AdjLowest,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            Some(1),
        );
        assert_eq!(result, Some(UnitId(12)));
    }

    #[test]
    fn bounce_adj_lowest_tiebreak_slot_asc() {
        // Last hit slot = 1; both slot 0 and slot 2 at same HP% → slot 0 wins (lower index)
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 400),
            (UnitId(11), Team::Enemy, 1, true, 800), // last hit, excluded
            (UnitId(12), Team::Enemy, 2, true, 400),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(11));
        let result = select_bounce_hop(
            BounceSelector::AdjLowest,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            Some(1),
        );
        assert_eq!(result, Some(UnitId(10))); // slot 0 wins tie
    }

    #[test]
    fn bounce_adj_lowest_no_adjacent_alive_returns_none() {
        // Last hit slot = 1, but slots 0 and 2 are dead → None
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, false, 0),
            (UnitId(11), Team::Enemy, 1, true, 500),
            (UnitId(12), Team::Enemy, 2, false, 0),
        ]);
        let mut already_hit = HashSet::new();
        already_hit.insert(UnitId(11));
        let result = select_bounce_hop(
            BounceSelector::AdjLowest,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            Some(1),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn bounce_ignores_ally_team() {
        // Ally team entries should never be returned regardless of HP
        let s = snap_hp(vec![
            (UnitId(1), Team::Ally, 0, true, 50), // ally, very low HP
            (UnitId(10), Team::Enemy, 0, true, 900),
        ]);
        let already_hit = HashSet::new();
        let result = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::NoRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(result, Some(UnitId(10)), "ally must never be selected");
    }

    #[test]
    fn bounce_allow_repeat_picks_same_target_twice() {
        // Two enemies; AllowRepeat + LowestHpPct: slot 1 @ 200‰ wins both picks even when it's
        // already in the hit set (simulated by calling the dispatcher twice with it inserted).
        let s = snap_hp(vec![
            (UnitId(10), Team::Enemy, 0, true, 700),
            (UnitId(11), Team::Enemy, 1, true, 200),
        ]);
        let mut already_hit = HashSet::new();

        // First pick
        let first = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::AllowRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(first, Some(UnitId(11)));
        already_hit.insert(UnitId(11));

        // Second pick — AllowRepeat ignores already_hit, so slot 1 wins again
        let second = select_bounce_hop(
            BounceSelector::LowestHpPctAlive,
            &s,
            &already_hit,
            RepeatPolicy::AllowRepeat,
            Team::Enemy,
            None,
        );
        assert_eq!(
            second,
            Some(UnitId(11)),
            "AllowRepeat: same lowest-HP target can be picked again"
        );
    }
}
