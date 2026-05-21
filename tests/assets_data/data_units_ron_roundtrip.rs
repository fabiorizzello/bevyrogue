use bevyrogue::combat::counterplay::{ChargedAttackDeclaration, EnemyTraitDeclaration};
use bevyrogue::combat::kit::{FollowUpConfig, FollowUpTrigger};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::ToughnessCategory;
use bevyrogue::combat::types::{Attribute, DamageTag, EvoLineId, EvoStage, SkillId, UnitId};
use bevyrogue::combat::ultimate::UltAccumulationTrigger;
use bevyrogue::data::skills_ron::LegalityReasonCode;
use bevyrogue::data::units_ron::{
    BlueprintRoster, BlueprintRosterPayload, EnemyCounterplayKind, EnemyCounterplayStatus, UnitDef,
    UnitRoster,
};
use std::collections::BTreeMap;

#[test]
fn round_trip_unit_def() {
    let def = UnitDef {
        id: UnitId(1),
        name: "Agumon".into(),
        role_tags: vec!["vanguard".into(), "breaker".into()],
        signature_traits: vec!["courage".into(), "fire".into()],
        hp_max: 100,
        attribute: Attribute::Vaccine,
        team: Team::Ally,
        basic_damage_tag: DamageTag::Fire,
        basic_skill: SkillId("baby_flame".into()),
        skill_ids: vec![SkillId("baby_flame".into())],
        ultimate_skill: SkillId("agumon_ult".into()),
        follow_up: Some(FollowUpConfig {
            trigger: FollowUpTrigger::OnEnemyBreak,
            action: SkillId("agumon_follow_up".into()),
        }),
        enemy_traits: vec![EnemyTraitDeclaration {
            kind: EnemyCounterplayKind::TempoAnchor,
            status: EnemyCounterplayStatus::Implemented,
        }],
        charged_attack: Some(ChargedAttackDeclaration {
            skill_id: SkillId("agumon_charged".into()),
            lead_turns: 2,
            status: EnemyCounterplayStatus::Deferred {
                reason: LegalityReasonCode::ChargedTelegraphDeferred,
            },
        }),
        form_identity: None,
        blueprint_metadata: BlueprintRoster::default(),
        resists: vec![],
        toughness_max: 50,
        weaknesses: vec![DamageTag::Ice],
        ultimate_trigger: 100,
        ultimate_cap: 150,
        ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
        ultimate_charge_per_event: 25,
        speed: 100,
        evo_stage: EvoStage::Child,
        evo_line: EvoLineId("agumon_line".into()),
        evolves_to: vec![UnitId(12)],
        tempo_resistant: false,
        toughness_category: ToughnessCategory::Standard,
    };
    let s = ron::to_string(&def).expect("serialize");
    let back: UnitDef = ron::from_str(&s).expect("deserialize");
    assert_eq!(def, back);
}

#[test]
fn missing_enemy_counterplay_fields_default_on_parse() {
    let roster = ron::from_str::<UnitRoster>(
        r#"[
        (
            id: UnitId(9),
            name: "Fallbackmon",
            role_tags: ["test"],
            signature_traits: ["test"],
            hp_max: 100,
            attribute: Vaccine,
            team: Ally,
            basic_damage_tag: Fire,
            basic_skill: SkillId("baby_flame"),
            skill_ids: [SkillId("baby_flame")],
            ultimate_skill: SkillId("agumon_ult"),
            follow_up: None,
            resists: [],
            toughness_max: 50,
            weaknesses: [Ice],
            ultimate_trigger: 100,
            ultimate_cap: 150,
            ultimate_accumulation_trigger: OnBasicAttack,
            ultimate_charge_per_event: 25,
            speed: 100,
            evo_stage: Child,
            evo_line: EvoLineId("test"),
            evolves_to: [],
        ),
    ]"#,
    )
    .expect("parse roster without enemy counterplay fields");

    let unit = &roster.0[0];
    assert!(
        unit.enemy_traits.is_empty(),
        "enemy_traits should default to empty"
    );
    assert!(
        unit.charged_attack.is_none(),
        "charged_attack should default to None"
    );
}

#[test]
fn missing_blueprint_metadata_defaults_to_empty() {
    let roster = ron::from_str::<UnitRoster>(
        r#"[
        (
            id: UnitId(9),
            name: "Fallbackmon",
            role_tags: ["test"],
            signature_traits: ["test"],
            hp_max: 100,
            attribute: Vaccine,
            team: Ally,
            basic_damage_tag: Fire,
            basic_skill: SkillId("baby_flame"),
            skill_ids: [SkillId("baby_flame")],
            ultimate_skill: SkillId("agumon_ult"),
            follow_up: None,
            resists: [],
            toughness_max: 50,
            weaknesses: [Ice],
            ultimate_trigger: 100,
            ultimate_cap: 150,
            ultimate_accumulation_trigger: OnBasicAttack,
            ultimate_charge_per_event: 25,
            speed: 100,
            evo_stage: Child,
            evo_line: EvoLineId("test"),
            evolves_to: [],
        ),
    ]"#,
    )
    .expect("parse roster without blueprint metadata");

    let unit = &roster.0[0];
    assert!(
        unit.blueprint_metadata.0.is_empty(),
        "blueprint_metadata should default to empty"
    );
}

#[test]
fn blueprint_metadata_round_trips_in_owner_sorted_order() {
    let mut blueprint_metadata = BlueprintRoster::default();
    blueprint_metadata.0.insert(
        "twin_core".into(),
        BlueprintRosterPayload(BTreeMap::from([
            ("role".into(), "spender".into()),
            ("line".into(), "ice".into()),
        ])),
    );
    blueprint_metadata.0.insert(
        "holy_support".into(),
        BlueprintRosterPayload(BTreeMap::from([
            ("role".into(), "accumulator".into()),
            ("line".into(), "hope".into()),
        ])),
    );

    let def = UnitDef {
        id: UnitId(1),
        name: "Agumon".into(),
        role_tags: vec!["vanguard".into(), "breaker".into()],
        signature_traits: vec!["courage".into(), "fire".into()],
        hp_max: 100,
        attribute: Attribute::Vaccine,
        team: Team::Ally,
        basic_damage_tag: DamageTag::Fire,
        basic_skill: SkillId("baby_flame".into()),
        skill_ids: vec![SkillId("baby_flame".into())],
        ultimate_skill: SkillId("agumon_ult".into()),
        follow_up: Some(FollowUpConfig {
            trigger: FollowUpTrigger::OnEnemyBreak,
            action: SkillId("agumon_follow_up".into()),
        }),
        enemy_traits: vec![EnemyTraitDeclaration {
            kind: EnemyCounterplayKind::TempoAnchor,
            status: EnemyCounterplayStatus::Implemented,
        }],
        charged_attack: Some(ChargedAttackDeclaration {
            skill_id: SkillId("agumon_charged".into()),
            lead_turns: 2,
            status: EnemyCounterplayStatus::Deferred {
                reason: LegalityReasonCode::ChargedTelegraphDeferred,
            },
        }),
        form_identity: None,
        blueprint_metadata,
        resists: vec![],
        toughness_max: 50,
        weaknesses: vec![DamageTag::Ice],
        ultimate_trigger: 100,
        ultimate_cap: 150,
        ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
        ultimate_charge_per_event: 25,
        speed: 100,
        evo_stage: EvoStage::Child,
        evo_line: EvoLineId("agumon_line".into()),
        evolves_to: vec![UnitId(12)],
        tempo_resistant: false,
        toughness_category: ToughnessCategory::Standard,
    };

    let s = ron::to_string(&def).expect("serialize");
    assert!(
        s.find("holy_support").unwrap() < s.find("twin_core").unwrap(),
        "owner keys should serialize deterministically in sorted order: {s}"
    );
    let back: UnitDef = ron::from_str(&s).expect("deserialize");
    assert_eq!(def, back);
}

#[test]
fn missing_identity_metadata_fails_to_parse() {
    let missing_metadata = r#"[
        (
            id: UnitId(1),
            name: "Brokenmon",
            hp_max: 100,
            attribute: Vaccine,
            team: Ally,
            basic_damage_tag: Fire,
            basic_skill: SkillId("baby_flame"),
            skill_ids: [SkillId("baby_flame")],
            ultimate_skill: SkillId("agumon_ult"),
            follow_up: None,
            resists: [],
            toughness_max: 50,
            weaknesses: [Ice],
            ultimate_trigger: 100,
            ultimate_cap: 150,
            ultimate_accumulation_trigger: OnBasicAttack,
            ultimate_charge_per_event: 25,
            speed: 100,
            evo_stage: Child,
            evo_line: EvoLineId("test"),
            evolves_to: [],
        ),
    ]"#;

    let err =
        ron::from_str::<UnitRoster>(missing_metadata).expect_err("missing metadata should fail");
    let message = err.to_string();
    assert!(
        message.contains("role_tags") || message.contains("signature_traits"),
        "unexpected parse error: {message}"
    );
}

#[test]
fn parsing_units_with_invalid_follow_up_trigger_fails() {
    let invalid_trigger = r#"[
        (
            id: UnitId(1),
            name: "Brokenmon",
            role_tags: ["test"],
            signature_traits: ["test"],
            hp_max: 100,
            attribute: Vaccine,
            team: Ally,
            basic_damage_tag: Fire,
            basic_skill: SkillId("baby_flame"),
            skill_ids: [SkillId("baby_flame")],
            ultimate_skill: SkillId("agumon_ult"),
            follow_up: Some((
                trigger: OnCriticalHit,
                action: SkillId("agumon_follow_up"),
            )),
            resists: [],
            toughness_max: 50,
            weaknesses: [Ice],
            ultimate_trigger: 100,
            ultimate_cap: 150,
            ultimate_accumulation_trigger: OnBasicAttack,
            ultimate_charge_per_event: 25,
            speed: 100,
            evo_stage: Child,
            evo_line: EvoLineId("test"),
            evolves_to: [],
        ),
    ]"#;

    let err =
        ron::from_str::<UnitRoster>(invalid_trigger).expect_err("invalid trigger should fail");
    let message = err.to_string();
    assert!(
        message.contains("OnCriticalHit")
            || message.contains("No such enum variant")
            || message.contains("Expected identifier")
            || message.contains("variant"),
        "unexpected error: {message}"
    );
}

#[test]
fn missing_evo_stage_fails_to_parse() {
    let missing_evo = r#"[
        (
            id: UnitId(1),
            name: "Brokenmon",
            role_tags: ["test"],
            signature_traits: ["test"],
            hp_max: 100,
            attribute: Vaccine,
            team: Ally,
            basic_damage_tag: Fire,
            basic_skill: SkillId("baby_flame"),
            skill_ids: [SkillId("baby_flame")],
            ultimate_skill: SkillId("agumon_ult"),
            follow_up: None,
            resists: [],
            toughness_max: 50,
            weaknesses: [Ice],
            ultimate_trigger: 100,
            ultimate_cap: 150,
            ultimate_accumulation_trigger: OnBasicAttack,
            ultimate_charge_per_event: 25,
            speed: 100,
            // missing evo_stage
            evo_line: EvoLineId("test"),
            evolves_to: [],
        ),
    ]"#;

    let err = ron::from_str::<UnitRoster>(missing_evo).expect_err("missing evo_stage should fail");
    assert!(
        err.to_string().contains("missing field `evo_stage`"),
        "unexpected error: {}",
        err
    );
}
