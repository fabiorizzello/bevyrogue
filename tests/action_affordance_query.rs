use bevyrogue::combat::action_query::{
    ActionQueryKind, ActionStatus, CombatQuerySnapshot, ImplementationStatus,
    ResourceAffordanceDetail, ResourceKind, ResourceStatus, TargetStatus, ToughnessAffordance,
    UnitQuerySnapshot, query_action_affordance, query_all_target_affordances,
    query_energy_cap_affordance, query_intent_legality, query_target_affordance,
};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::state::CombatPhase;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::{Toughness, ToughnessCategory, ToughnessView};
use bevyrogue::combat::types::{DamageTag, SkillId, UnitId};
use bevyrogue::data::skills_ron::{
    Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
    SkillTargeting, TargetHpRule, TargetLife, TargetShape, TargetSide,
};

fn unit(
    id: u32,
    team: Team,
    hp_current: i32,
    hp_max: i32,
    is_ko: bool,
    is_commander: bool,
) -> UnitQuerySnapshot {
    UnitQuerySnapshot {
        id: UnitId(id),
        team,
        is_active: true,
        is_ko,
        is_stunned: false,
        is_commander,
        hp_current,
        hp_max,
        sp: 0,
        ultimate_current: 0,
        ultimate_trigger: 100,
        ultimate_ready: false,
        energy: 0,
        energy_secondary_gained: 0,
        energy_external_gained: 0,
        skills: None,
        toughness: None,
        ..Default::default()
    }
}

fn actor_with_skills(
    mut unit: UnitQuerySnapshot,
    basic: &str,
    skills: Vec<&str>,
    ultimate: &str,
) -> UnitQuerySnapshot {
    unit.skills = Some(UnitSkills {
        basic: SkillId(basic.into()),
        skills: skills
            .into_iter()
            .map(|skill| SkillId(skill.into()))
            .collect(),
        ultimate: SkillId(ultimate.into()),
        follow_up: None,
    });
    unit
}

fn unit_with_toughness(mut unit: UnitQuerySnapshot, toughness: Toughness) -> UnitQuerySnapshot {
    unit.toughness = Some(toughness);
    unit
}
fn snapshot_with(
    units: Vec<UnitQuerySnapshot>,
    acting_id: u32,
    phase: CombatPhase,
) -> CombatQuerySnapshot {
    let acting_unit = units
        .iter()
        .find(|unit| unit.id == UnitId(acting_id))
        .cloned()
        .expect("acting unit must exist in fixture");

    CombatQuerySnapshot {
        phase,
        acting_unit,
        target_unit: None,
        units,
    }
}

fn basic_attack_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: "Basic Attack".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Damage {
            amount: 10,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn offensive_skill(id: &str, sp_cost: i32) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: "Offensive Skill".into(),
        damage_tag: DamageTag::Fire,
        sp_cost,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Damage {
            amount: 10,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn any_target_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: "Any Target Skill".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Any,
            life: TargetLife::Any,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Damage {
            amount: 10,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn hidden_self_only_skill() -> SkillDef {
    SkillDef {
        id: SkillId("greymon_form_identity".into()),
        name: "Greymon Form Identity".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::SelfOnly,
            side: TargetSide::Any,
            life: TargetLife::Any,
            self_rule: SelfTargetRule::Allow,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Hidden {
            reason: LegalityReasonCode::UnimplementedEffect,
        },
        legacy_ops: vec![Effect::GrantEnergy(5)],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn revive_skill() -> SkillDef {
    SkillDef {
        id: SkillId("revive_skill".into()),
        name: "Revive Skill".into(),
        damage_tag: DamageTag::Light,
        sp_cost: 3,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Ally,
            life: TargetLife::Ko,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Revive(25)],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn damaged_ally_skill() -> SkillDef {
    SkillDef {
        id: SkillId("damaged_ally_skill".into()),
        name: "Damaged Ally Skill".into(),
        damage_tag: DamageTag::Light,
        sp_cost: 2,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Ally,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Damaged,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Damage {
            amount: 5,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn row_skill() -> SkillDef {
    SkillDef {
        id: SkillId("row_skill".into()),
        name: "Row Skill".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 4,
        targeting: SkillTargeting {
            shape: TargetShape::Row,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Damage {
            amount: 12,
            target: TargetShape::Row,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn deferred_row_skill() -> SkillDef {
    SkillDef {
        implementation: SkillImplementation::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape,
        },
        ..row_skill()
    }
}

fn hidden_row_skill() -> SkillDef {
    SkillDef {
        implementation: SkillImplementation::Hidden {
            reason: LegalityReasonCode::EnemyTraitDeferred,
        },
        ..row_skill()
    }
}

fn skill_book(skills: Vec<SkillDef>) -> SkillBook {
    SkillBook(skills)
}

fn resource_detail<'a>(
    details: &'a [ResourceAffordanceDetail],
    kind: ResourceKind,
) -> &'a ResourceAffordanceDetail {
    details
        .iter()
        .find(|detail| detail.kind == kind)
        .expect("expected resource detail in affordance")
}

#[test]
fn implemented_enemy_toughness_is_visible_while_ally_and_zero_bars_stay_hidden() {
    let skill = any_target_skill("any_target_skill");
    let mut acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["any_target_skill"],
        "ultimate_attack",
    );
    acting_unit.sp = 0;
    let enemy = unit_with_toughness(
        unit(2, Team::Enemy, 40, 40, false, false),
        Toughness::new(40, vec![DamageTag::Fire]),
    );
    let ally = unit_with_toughness(
        unit(3, Team::Ally, 30, 40, false, false),
        Toughness::new(20, vec![DamageTag::Ice]),
    );
    let zero_enemy = unit_with_toughness(
        unit(4, Team::Enemy, 40, 40, false, false),
        Toughness::with_category(0, vec![DamageTag::Fire], ToughnessCategory::Standard),
    );
    let snapshot = snapshot_with(
        vec![acting_unit, enemy, ally, zero_enemy],
        1,
        CombatPhase::WaitingAction,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Skill(&skill.id),
    );

    let enemy_aff = affordance
        .targets
        .iter()
        .find(|(id, _)| *id == UnitId(2))
        .expect("enemy target affordance");
    assert!(matches!(enemy_aff.1.status, TargetStatus::Enabled));
    assert!(matches!(
        enemy_aff.1.toughness,
        ToughnessAffordance::Visible
    ));
    assert_eq!(
        enemy_aff.1.toughness_view,
        Some(ToughnessView {
            current: 40,
            max: 40,
            weaknesses: vec![DamageTag::Fire],
            broken: false,
        })
    );
    assert!(enemy_aff.1.toughness_reason.is_none());

    let ally_aff = affordance
        .targets
        .iter()
        .find(|(id, _)| *id == UnitId(3))
        .expect("ally target affordance");
    assert!(matches!(ally_aff.1.status, TargetStatus::Enabled));
    assert!(matches!(ally_aff.1.toughness, ToughnessAffordance::Hidden));
    assert!(ally_aff.1.toughness_view.is_none());
    assert!(matches!(
        ally_aff.1.toughness_reason,
        Some(LegalityReasonCode::ToughnessEnemyOnly)
    ));

    let zero_aff = affordance
        .targets
        .iter()
        .find(|(id, _)| *id == UnitId(4))
        .expect("zero enemy target affordance");
    assert!(matches!(zero_aff.1.status, TargetStatus::Enabled));
    assert!(matches!(zero_aff.1.toughness, ToughnessAffordance::Hidden));
    assert!(zero_aff.1.toughness_view.is_none());
    assert!(matches!(
        zero_aff.1.toughness_reason,
        Some(LegalityReasonCode::ToughnessEnemyOnly)
    ));
    assert!(matches!(affordance.toughness, ToughnessAffordance::Visible));
}

#[test]
fn enabled_offensive_action_reports_target_and_resource_details() {
    let skill = basic_attack_skill("basic_attack");
    let mut acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["basic_attack"],
        "ultimate_attack",
    );
    acting_unit.sp = 5;
    acting_unit.ultimate_current = 120;
    acting_unit.ultimate_trigger = 100;
    acting_unit.ultimate_ready = true;
    let snapshot = snapshot_with(
        vec![
            acting_unit.clone(),
            unit_with_toughness(
                unit(2, Team::Enemy, 30, 60, false, false),
                Toughness::new(40, vec![DamageTag::Fire]),
            ),
        ],
        1,
        CombatPhase::WaitingAction,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Basic,
    );

    assert!(matches!(affordance.action, ActionStatus::Enabled));
    assert!(matches!(affordance.target, TargetStatus::Enabled));
    assert!(matches!(affordance.resource, ResourceStatus::Enabled));
    assert!(matches!(
        affordance.implementation,
        ImplementationStatus::Implemented
    ));
    assert!(matches!(affordance.toughness, ToughnessAffordance::Visible));
    assert_eq!(affordance.targets.len(), 2);
    assert!(matches!(
        affordance.targets[1].1.status,
        TargetStatus::Enabled
    ));
    assert_eq!(affordance.resource_details.len(), 2);
    assert_eq!(
        resource_detail(&affordance.resource_details, ResourceKind::Sp).current,
        Some(5)
    );
    assert_eq!(
        resource_detail(&affordance.resource_details, ResourceKind::Sp).required,
        Some(0)
    );
    assert_eq!(
        resource_detail(&affordance.resource_details, ResourceKind::Ultimate).current,
        Some(120)
    );
    assert_eq!(
        resource_detail(&affordance.resource_details, ResourceKind::Ultimate).required,
        Some(100)
    );
}

#[test]
fn non_active_actor_reports_not_active_without_losing_target_affordances() {
    let skill = offensive_skill("offensive_skill", 0);
    let mut acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["offensive_skill"],
        "ultimate_attack",
    );
    acting_unit.is_active = false;
    let snapshot = snapshot_with(
        vec![acting_unit, unit(2, Team::Enemy, 30, 60, false, false)],
        1,
        CombatPhase::WaitingAction,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Skill(&skill.id),
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::NotActiveUnit
        }
    ));
    assert!(matches!(affordance.target, TargetStatus::Enabled));
    assert!(
        affordance
            .targets
            .iter()
            .any(|(id, target)| *id == UnitId(2) && matches!(target.status, TargetStatus::Enabled))
    );
}

#[test]
fn wrong_phase_reports_wrong_phase_and_keeps_target_affordances() {
    let skill = offensive_skill("offensive_skill", 0);
    let acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["offensive_skill"],
        "ultimate_attack",
    );
    let snapshot = snapshot_with(
        vec![acting_unit, unit(2, Team::Enemy, 30, 60, false, false)],
        1,
        CombatPhase::Resolving,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Skill(&skill.id),
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::WrongPhase
        }
    ));
    assert!(matches!(affordance.target, TargetStatus::Enabled));
}

#[test]
fn attacker_ko_blocks_action_before_targets() {
    let skill = offensive_skill("offensive_skill", 0);
    let acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, true, false),
        "basic_attack",
        vec!["offensive_skill"],
        "ultimate_attack",
    );
    let snapshot = snapshot_with(
        vec![acting_unit, unit(2, Team::Enemy, 30, 60, false, false)],
        1,
        CombatPhase::WaitingAction,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Skill(&skill.id),
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::AttackerKo
        }
    ));
    assert!(matches!(affordance.target, TargetStatus::Enabled));
}

#[test]
fn attacker_stunned_blocks_action_before_targets() {
    let skill = offensive_skill("offensive_skill", 0);
    let mut acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["offensive_skill"],
        "ultimate_attack",
    );
    acting_unit.is_stunned = true;
    let snapshot = snapshot_with(
        vec![acting_unit, unit(2, Team::Enemy, 30, 60, false, false)],
        1,
        CombatPhase::WaitingAction,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Skill(&skill.id),
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::AttackerStunned
        }
    ));
    assert!(matches!(affordance.target, TargetStatus::Enabled));
}

#[test]
fn missing_skill_book_entry_returns_missing_skill() {
    let skill = offensive_skill("offensive_skill", 0);
    let acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["offensive_skill"],
        "ultimate_attack",
    );
    let snapshot = snapshot_with(
        vec![acting_unit, unit(2, Team::Enemy, 30, 60, false, false)],
        1,
        CombatPhase::WaitingAction,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![]),
        UnitId(1),
        ActionQueryKind::Skill(&skill.id),
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::MissingSkill
        }
    ));
    assert!(affordance.targets.is_empty());
}

#[test]
fn sp_shortfall_disables_action_and_reports_current_and_required_sp() {
    let skill = offensive_skill("offensive_skill", 3);
    let mut acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["offensive_skill"],
        "ultimate_attack",
    );
    acting_unit.sp = 1;
    acting_unit.ultimate_current = 120;
    acting_unit.ultimate_trigger = 100;
    acting_unit.ultimate_ready = true;
    let snapshot = snapshot_with(
        vec![acting_unit, unit(2, Team::Enemy, 30, 60, false, false)],
        1,
        CombatPhase::WaitingAction,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Skill(&skill.id),
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::SpShortfall
        }
    ));
    assert!(matches!(
        resource_detail(&affordance.resource_details, ResourceKind::Sp).status,
        ResourceStatus::Disabled {
            reason: LegalityReasonCode::SpShortfall
        }
    ));
    assert_eq!(
        resource_detail(&affordance.resource_details, ResourceKind::Sp).current,
        Some(1)
    );
    assert_eq!(
        resource_detail(&affordance.resource_details, ResourceKind::Sp).required,
        Some(3)
    );
}

#[test]
fn ultimate_not_ready_disables_action_and_reports_current_and_required_ult_charge() {
    let skill = offensive_skill("ultimate_attack", 0);
    let mut acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["ultimate_attack"],
        "ultimate_attack",
    );
    acting_unit.sp = 0;
    acting_unit.ultimate_current = 60;
    acting_unit.ultimate_trigger = 100;
    acting_unit.ultimate_ready = false;
    let snapshot = snapshot_with(
        vec![acting_unit, unit(2, Team::Enemy, 30, 60, false, false)],
        1,
        CombatPhase::WaitingAction,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Ultimate,
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::UltimateNotReady
        }
    ));
    assert!(matches!(
        resource_detail(&affordance.resource_details, ResourceKind::Ultimate).status,
        ResourceStatus::Disabled {
            reason: LegalityReasonCode::UltimateNotReady
        }
    ));
    assert_eq!(
        resource_detail(&affordance.resource_details, ResourceKind::Ultimate).current,
        Some(60)
    );
    assert_eq!(
        resource_detail(&affordance.resource_details, ResourceKind::Ultimate).required,
        Some(100)
    );
}

#[test]
fn no_valid_targets_disables_action_after_state_and_resource_checks() {
    let skill = offensive_skill("offensive_skill", 0);
    let acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["offensive_skill"],
        "ultimate_attack",
    );
    let snapshot = snapshot_with(vec![acting_unit], 1, CombatPhase::WaitingAction);

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Skill(&skill.id),
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::NoValidTargets
        }
    ));
    assert!(matches!(
        affordance.target,
        TargetStatus::Disabled {
            reason: LegalityReasonCode::NoValidTargets
        }
    ));
    assert!(
        affordance
            .targets
            .iter()
            .all(|(_, target)| matches!(target.status, TargetStatus::Disabled { .. }))
    );
}

#[test]
fn hidden_implementation_returns_hidden_action_and_targets() {
    let skill = hidden_self_only_skill();
    let mut acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["greymon_form_identity"],
        "ultimate_attack",
    );
    acting_unit.sp = 0;
    let snapshot = snapshot_with(
        vec![
            acting_unit,
            unit_with_toughness(
                unit(2, Team::Enemy, 30, 60, false, false),
                Toughness::new(40, vec![DamageTag::Fire]),
            ),
        ],
        1,
        CombatPhase::WaitingAction,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Skill(&skill.id),
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::NoValidTargets
        }
    ));
    assert!(matches!(
        affordance.implementation,
        ImplementationStatus::Hidden {
            reason: LegalityReasonCode::UnimplementedEffect
        }
    ));
    assert!(affordance.targets.iter().all(|(_, target)| matches!(
        target.status,
        TargetStatus::Hidden {
            reason: LegalityReasonCode::UnimplementedEffect
        }
    )));
    assert!(
        affordance
            .targets
            .iter()
            .all(|(_, target)| matches!(target.toughness, ToughnessAffordance::Hidden))
    );
    assert!(
        affordance
            .targets
            .iter()
            .all(|(_, target)| target.toughness_view.is_none())
    );
    assert!(affordance.targets.iter().all(|(_, target)| matches!(
        target.toughness_reason,
        Some(LegalityReasonCode::UnimplementedEffect)
    )));
    assert!(affordance.resource_details.iter().all(|detail| matches!(
        detail.status,
        ResourceStatus::Hidden {
            reason: LegalityReasonCode::UnimplementedEffect
        }
    )));
    assert!(matches!(affordance.toughness, ToughnessAffordance::Hidden));
}

#[test]
fn deferred_implementation_returns_deferred_action_and_targets() {
    let skill = deferred_row_skill();
    let mut acting_unit = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["row_skill"],
        "ultimate_attack",
    );
    acting_unit.sp = 4;
    let snapshot = snapshot_with(
        vec![acting_unit, unit(2, Team::Enemy, 30, 60, false, false)],
        1,
        CombatPhase::WaitingAction,
    );

    let affordance = query_action_affordance(
        &snapshot,
        &skill_book(vec![skill.clone()]),
        UnitId(1),
        ActionQueryKind::Skill(&skill.id),
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::NoValidTargets
        }
    ));
    assert!(matches!(
        affordance.implementation,
        ImplementationStatus::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape
        }
    ));
    assert!(affordance.targets.iter().all(|(_, target)| matches!(
        target.status,
        TargetStatus::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape
        }
    )));
    assert!(
        affordance
            .targets
            .iter()
            .all(|(_, target)| matches!(target.toughness, ToughnessAffordance::Hidden))
    );
    assert!(
        affordance
            .targets
            .iter()
            .all(|(_, target)| target.toughness_view.is_none())
    );
    assert!(affordance.targets.iter().all(|(_, target)| matches!(
        target.toughness_reason,
        Some(LegalityReasonCode::UnimplementedTargetShape)
    )));
    assert!(affordance.resource_details.iter().all(|detail| matches!(
        detail.status,
        ResourceStatus::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape
        }
    )));
    assert!(matches!(affordance.toughness, ToughnessAffordance::Hidden));
}

#[test]
fn target_affordance_surface_still_handles_enemy_ally_ko_commander_and_self() {
    let skill = offensive_skill("offensive_skill", 0);
    let snapshot = snapshot_with(
        vec![
            unit(1, Team::Ally, 50, 50, false, false),
            unit(2, Team::Enemy, 30, 60, false, false),
            unit(3, Team::Ally, 25, 25, false, false),
            unit(4, Team::Enemy, 0, 60, true, false),
            unit(5, Team::Enemy, 60, 60, false, true),
        ],
        1,
        CombatPhase::WaitingAction,
    );

    assert!(matches!(
        query_target_affordance(&snapshot, UnitId(1), &skill, UnitId(2)).status,
        TargetStatus::Enabled
    ));
    assert!(matches!(
        query_target_affordance(&snapshot, UnitId(1), &skill, UnitId(3)).status,
        TargetStatus::Disabled {
            reason: LegalityReasonCode::WrongSide
        }
    ));
    assert!(matches!(
        query_target_affordance(&snapshot, UnitId(1), &skill, UnitId(4)).status,
        TargetStatus::Disabled {
            reason: LegalityReasonCode::TargetKo
        }
    ));
    assert!(matches!(
        query_target_affordance(&snapshot, UnitId(1), &skill, UnitId(5)).status,
        TargetStatus::Disabled {
            reason: LegalityReasonCode::TargetIsCommander
        }
    ));
    assert!(matches!(
        query_target_affordance(&snapshot, UnitId(1), &skill, UnitId(1)).status,
        TargetStatus::Disabled {
            reason: LegalityReasonCode::TargetIsSelf
        }
    ));
}

#[test]
fn revive_affordance_requires_ko_ally_and_rejects_live_and_enemy_targets() {
    let skill = revive_skill();
    let snapshot = snapshot_with(
        vec![
            unit(1, Team::Ally, 50, 50, false, false),
            unit(2, Team::Ally, 0, 60, true, false),
            unit(3, Team::Ally, 40, 60, false, false),
            unit(4, Team::Enemy, 0, 60, true, false),
        ],
        1,
        CombatPhase::WaitingAction,
    );

    assert!(matches!(
        query_target_affordance(&snapshot, UnitId(1), &skill, UnitId(2)).status,
        TargetStatus::Enabled
    ));
    assert!(matches!(
        query_target_affordance(&snapshot, UnitId(1), &skill, UnitId(3)).status,
        TargetStatus::Disabled {
            reason: LegalityReasonCode::TargetNotKo
        }
    ));
    assert!(matches!(
        query_target_affordance(&snapshot, UnitId(1), &skill, UnitId(4)).status,
        TargetStatus::Disabled {
            reason: LegalityReasonCode::WrongSide
        }
    ));
}

#[test]
fn damaged_target_affordance_requires_missing_hp_for_ally_targets() {
    let skill = damaged_ally_skill();
    let snapshot = snapshot_with(
        vec![
            unit(1, Team::Ally, 50, 50, false, false),
            unit(2, Team::Ally, 30, 60, false, false),
            unit(3, Team::Ally, 60, 60, false, false),
        ],
        1,
        CombatPhase::WaitingAction,
    );

    assert!(matches!(
        query_target_affordance(&snapshot, UnitId(1), &skill, UnitId(2)).status,
        TargetStatus::Enabled
    ));
    assert!(matches!(
        query_target_affordance(&snapshot, UnitId(1), &skill, UnitId(3)).status,
        TargetStatus::Disabled {
            reason: LegalityReasonCode::TargetFullHp
        }
    ));
}

#[test]
fn deferred_row_shape_returns_deferred_for_every_target_in_snapshot() {
    let skill = deferred_row_skill();
    let snapshot = snapshot_with(
        vec![
            unit(1, Team::Ally, 50, 50, false, false),
            unit(2, Team::Enemy, 30, 60, false, false),
            unit(3, Team::Enemy, 0, 60, true, false),
        ],
        1,
        CombatPhase::WaitingAction,
    );

    let affordances = query_all_target_affordances(&snapshot, UnitId(1), &skill);
    assert!(affordances.iter().all(|(_, affordance)| matches!(
        affordance.status,
        TargetStatus::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape
        }
    )));
}

#[test]
fn energy_cap_affordance_reports_remaining_budget_and_true_cap_reason() {
    let unit = UnitQuerySnapshot {
        id: UnitId(1),
        team: Team::Ally,
        is_active: true,
        is_ko: false,
        is_stunned: false,
        is_commander: false,
        hp_current: 50,
        hp_max: 50,
        sp: 0,
        ultimate_current: 0,
        ultimate_trigger: 100,
        ultimate_ready: false,
        energy: 0,
        energy_secondary_gained: 3,
        energy_external_gained: 0,
        skills: None,
        toughness: None,
        ..Default::default()
    };

    let affordance = query_energy_cap_affordance(
        &unit,
        bevyrogue::combat::energy::EnergyGainSource::SecondaryAction,
        5,
    );

    assert_eq!(affordance.kind, ResourceKind::EnergyCap);
    assert!(matches!(affordance.status, ResourceStatus::Enabled));
    assert_eq!(affordance.current, Some(7));
    assert_eq!(affordance.required, Some(5));
}

#[test]
fn energy_cap_affordance_disables_when_requested_exceeds_remaining_or_budget_is_exhausted() {
    let exhausted = UnitQuerySnapshot {
        id: UnitId(1),
        team: Team::Ally,
        is_active: true,
        is_ko: false,
        is_stunned: false,
        is_commander: false,
        hp_current: 50,
        hp_max: 50,
        sp: 0,
        ultimate_current: 0,
        ultimate_trigger: 100,
        ultimate_ready: false,
        energy: 0,
        energy_secondary_gained: 10,
        energy_external_gained: 0,
        skills: None,
        toughness: None,
        ..Default::default()
    };
    let partial = UnitQuerySnapshot {
        energy_secondary_gained: 3,
        ..exhausted.clone()
    };

    let exhausted_affordance = query_energy_cap_affordance(
        &exhausted,
        bevyrogue::combat::energy::EnergyGainSource::SecondaryAction,
        1,
    );
    let partial_affordance = query_energy_cap_affordance(
        &partial,
        bevyrogue::combat::energy::EnergyGainSource::SecondaryAction,
        8,
    );

    assert!(matches!(
        exhausted_affordance.status,
        ResourceStatus::Disabled {
            reason: LegalityReasonCode::EnergyCapReached
        }
    ));
    assert_eq!(exhausted_affordance.current, Some(0));
    assert_eq!(exhausted_affordance.required, Some(1));

    assert!(matches!(
        partial_affordance.status,
        ResourceStatus::Disabled {
            reason: LegalityReasonCode::EnergyCapReached
        }
    ));
    assert_eq!(partial_affordance.current, Some(7));
    assert_eq!(partial_affordance.required, Some(8));
}

#[test]
fn target_hp_rule_distinguishes_any_and_damaged() {
    assert_ne!(TargetHpRule::Any, TargetHpRule::Damaged);
    assert!(matches!(TargetHpRule::Any, TargetHpRule::Any));
    assert!(matches!(TargetHpRule::Damaged, TargetHpRule::Damaged));
}

#[test]
fn legality_reason_codes_include_contract_values() {
    for reason in [
        LegalityReasonCode::NotActiveUnit,
        LegalityReasonCode::WrongPhase,
        LegalityReasonCode::AttackerKo,
        LegalityReasonCode::AttackerStunned,
        LegalityReasonCode::MissingSkill,
        LegalityReasonCode::SpShortfall,
        LegalityReasonCode::UltimateNotReady,
        LegalityReasonCode::TargetNotFound,
        LegalityReasonCode::TamerGaugeDeferred,
        LegalityReasonCode::TamerCommandDeferred,
        LegalityReasonCode::ChargedTelegraphDeferred,
        LegalityReasonCode::EnemyTraitDeferred,
        LegalityReasonCode::EnergyCapReached,
    ] {
        assert!(!format!("{reason:?}").is_empty());
    }
}

#[test]
fn intent_legality_respects_priority_and_specific_target_reasons() {
    let skill = basic_attack_skill("basic_attack");
    let revive = revive_skill();
    let deferred = deferred_row_skill();
    let book = skill_book(vec![skill.clone(), revive.clone(), deferred.clone()]);

    let mut actor = actor_with_skills(
        unit(1, Team::Ally, 50, 50, false, false),
        "basic_attack",
        vec!["revive_skill", "row_skill"],
        "ultimate_attack",
    );
    actor.sp = 5;

    let target_enemy = unit(2, Team::Enemy, 40, 40, false, false);
    let target_ally = unit(3, Team::Ally, 30, 40, false, false);
    let target_ko_enemy = unit(4, Team::Enemy, 0, 40, true, false);
    let target_ko_ally = unit(5, Team::Ally, 0, 40, true, false);
    let target_commander = unit(6, Team::Enemy, 100, 100, false, true);

    let snapshot = snapshot_with(
        vec![
            actor.clone(),
            target_enemy.clone(),
            target_ally.clone(),
            target_ko_enemy.clone(),
            target_ko_ally.clone(),
            target_commander.clone(),
        ],
        1,
        CombatPhase::WaitingAction,
    );

    // 1. Valid intent
    assert_eq!(
        query_intent_legality(
            &snapshot,
            &book,
            UnitId(1),
            &ActionQueryKind::Basic,
            UnitId(2)
        ),
        Ok(())
    );

    // 2. Specific target reasons
    // Offensive against ally -> WrongSide
    assert_eq!(
        query_intent_legality(
            &snapshot,
            &book,
            UnitId(1),
            &ActionQueryKind::Basic,
            UnitId(3)
        ),
        Err(LegalityReasonCode::WrongSide)
    );
    // Offensive against KO -> TargetKo
    assert_eq!(
        query_intent_legality(
            &snapshot,
            &book,
            UnitId(1),
            &ActionQueryKind::Basic,
            UnitId(4)
        ),
        Err(LegalityReasonCode::TargetKo)
    );
    // Offensive against Commander -> TargetIsCommander
    assert_eq!(
        query_intent_legality(
            &snapshot,
            &book,
            UnitId(1),
            &ActionQueryKind::Basic,
            UnitId(6)
        ),
        Err(LegalityReasonCode::TargetIsCommander)
    );
    // Revive against live -> TargetNotKo
    assert_eq!(
        query_intent_legality(
            &snapshot,
            &book,
            UnitId(1),
            &ActionQueryKind::Skill(&revive.id),
            UnitId(3)
        ),
        Err(LegalityReasonCode::TargetNotKo)
    );
    // Target not found
    assert_eq!(
        query_intent_legality(
            &snapshot,
            &book,
            UnitId(1),
            &ActionQueryKind::Basic,
            UnitId(99)
        ),
        Err(LegalityReasonCode::TargetNotFound)
    );

    // 3. Actor/Phase/Resource priority
    // SP shortfall
    let mut low_sp_actor = actor.clone();
    low_sp_actor.sp = 0; // revive costs 3
    let low_sp_snap = snapshot_with(
        vec![low_sp_actor, target_ko_ally.clone()],
        1,
        CombatPhase::WaitingAction,
    );
    assert_eq!(
        query_intent_legality(
            &low_sp_snap,
            &book,
            UnitId(1),
            &ActionQueryKind::Skill(&revive.id),
            UnitId(5)
        ),
        Err(LegalityReasonCode::SpShortfall)
    );

    // Stunned attacker
    let mut stunned_actor = actor.clone();
    stunned_actor.is_stunned = true;
    let stunned_snap = snapshot_with(
        vec![stunned_actor, target_enemy.clone()],
        1,
        CombatPhase::WaitingAction,
    );
    assert_eq!(
        query_intent_legality(
            &stunned_snap,
            &book,
            UnitId(1),
            &ActionQueryKind::Basic,
            UnitId(2)
        ),
        Err(LegalityReasonCode::AttackerStunned)
    );

    // KO attacker
    let mut ko_actor = actor.clone();
    ko_actor.is_ko = true;
    let ko_snap = snapshot_with(
        vec![ko_actor, target_enemy.clone()],
        1,
        CombatPhase::WaitingAction,
    );
    assert_eq!(
        query_intent_legality(
            &ko_snap,
            &book,
            UnitId(1),
            &ActionQueryKind::Basic,
            UnitId(2)
        ),
        Err(LegalityReasonCode::AttackerKo)
    );

    // Non-active actor
    let mut inactive_actor = actor.clone();
    inactive_actor.is_active = false;
    let inactive_snap = snapshot_with(
        vec![inactive_actor, target_enemy.clone()],
        1,
        CombatPhase::WaitingAction,
    );
    assert_eq!(
        query_intent_legality(
            &inactive_snap,
            &book,
            UnitId(1),
            &ActionQueryKind::Basic,
            UnitId(2)
        ),
        Err(LegalityReasonCode::NotActiveUnit)
    );

    // Wrong phase
    let wrong_phase_snap = snapshot_with(
        vec![actor.clone(), target_enemy.clone()],
        1,
        CombatPhase::Resolving,
    );
    assert_eq!(
        query_intent_legality(
            &wrong_phase_snap,
            &book,
            UnitId(1),
            &ActionQueryKind::Basic,
            UnitId(2)
        ),
        Err(LegalityReasonCode::WrongPhase)
    );

    // 4. Implementation priority
    assert_eq!(
        query_intent_legality(
            &snapshot,
            &book,
            UnitId(1),
            &ActionQueryKind::Skill(&deferred.id),
            UnitId(2)
        ),
        Err(LegalityReasonCode::UnimplementedTargetShape)
    );

    // 5. Missing skill
    assert_eq!(
        query_intent_legality(
            &snapshot,
            &book,
            UnitId(1),
            &ActionQueryKind::Skill(&SkillId("non_existent".into())),
            UnitId(2)
        ),
        Err(LegalityReasonCode::MissingSkill)
    );
}
