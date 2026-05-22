
use std::collections::BTreeMap;

use bevyrogue::combat::energy::Energy;
use bevyrogue::combat::events::CombatEventKind;
use bevyrogue::combat::resolution::apply_legacy_ops;
use bevyrogue::combat::toughness::DamageKind;
use bevyrogue::combat::ult_gauge::UltGaugeMetadata;
use bevyrogue::combat::ultimate::UltimateCharge;
use bevyrogue::combat::{
    sp::{RoundSpTracker, SpPool},
    team::Team,
    toughness::Toughness,
    turn_system::ActionIntent,
    types::{Attribute, DamageTag, SkillId, UnitId},
    ultimate::UltAccumulationTrigger,
};
use bevyrogue::data::{
    skills_ron::{
        Effect, LegalityReasonCode, SelfTargetRule, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
    units_ron::{BlueprintRoster, BlueprintRosterPayload},
};
use crate::common::resolution_helpers::{basic_intent, resolved, revive_skill, skill, unit};

fn energy_backed_metadata() -> UltGaugeMetadata {
    let mut owner = BTreeMap::new();
    owner.insert("ult_gauge".to_string(), "energy".to_string());

    let mut roster = BTreeMap::new();
    roster.insert("agumon".to_string(), BlueprintRosterPayload(owner));

    UltGaugeMetadata(BlueprintRoster(roster))
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
        false,
        false,
        None,
        None,
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
        false,
        false,
        None,
        None,
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
        false,
        false,
        None,
        None,
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
        false,
        false,
        None,
        None,
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
        false,
        false,
        None,
        None,
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
        false,
        false,
        None,
        None,
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
        false,
        false,
        None,
        None,
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
        false,
        false,
        None,
        None,
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
fn energy_backed_ultimate_reset_drains_energy_and_legacy_charge() {
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
    let mut energy = Energy {
        current: 100,
        max: 100,
    };
    let gauge_meta = energy_backed_metadata();
    let mut sp = SpPool { current: 3, max: 5 };
    let intent = ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    };
    let resolved = resolved(&intent, skill("ultimate", DamageTag::Fire, 30, 0, 20));

    let (outcome, _events) = apply_legacy_ops(
        &resolved,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        false,
        false,
        None,
        None,
        None,
        Some(&mut energy),
        Some(&gauge_meta),
    );

    assert!(outcome.sp_ok);
    assert_eq!(ult.current, 0, "legacy gauge stays zeroed for back-compat");
    assert_eq!(energy.current, 0, "energy-backed ult spend must drain Energy.current");
}

#[test]
fn legacy_ultimate_reset_leaves_energy_untouched() {
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
    let mut energy = Energy {
        current: 73,
        max: 100,
    };
    let mut sp = SpPool { current: 3, max: 5 };
    let intent = ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    };
    let resolved = resolved(&intent, skill("ultimate", DamageTag::Fire, 30, 0, 20));

    let (outcome, _events) = apply_legacy_ops(
        &resolved,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        false,
        false,
        None,
        None,
        None,
        Some(&mut energy),
        None,
    );

    assert!(outcome.sp_ok);
    assert_eq!(ult.current, 0);
    assert_eq!(energy.current, 73, "legacy ult users must not drain Energy.current");
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
        false,
        false,
        None,
        None,
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
