use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use crate::combat::counterplay::{
    EnemyCounterplayKind, ImplementationStatus as EnemyCounterplayStatus,
};
use crate::combat::kit::{FollowUpConfig, FormIdentityConfig};
use crate::combat::team::Team;
use crate::combat::toughness::ToughnessCategory;
use crate::combat::types::{Attribute, DamageTag, EvoLineId, EvoStage, SkillId, UnitId};
use crate::combat::ultimate::UltAccumulationTrigger;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwinCoreLine {
    Fire,
    Ice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwinCoreRole {
    Builder,
    Spender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwinCorePersonalLabel {
    Heat,
    HeatSink,
    Echo,
    PackVow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwinCoreRosterMetadata {
    #[serde(default)]
    pub line: Option<TwinCoreLine>,
    #[serde(default)]
    pub role: Option<TwinCoreRole>,
    #[serde(default)]
    pub personal_resource_label: Option<TwinCorePersonalLabel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolySupportLine {
    Hope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolySupportRole {
    GraceAccumulator,
    MartyrSpender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolySupportPersonalLabel {
    Grace,
    MartyrLight,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HolySupportRosterMetadata {
    #[serde(default)]
    pub line: Option<HolySupportLine>,
    #[serde(default)]
    pub role: Option<HolySupportRole>,
    #[serde(default)]
    pub personal_resource_label: Option<HolySupportPersonalLabel>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitDef {
    pub id: UnitId,
    pub name: String,
    pub role_tags: Vec<String>,
    pub signature_traits: Vec<String>,
    pub hp_max: i32,
    pub attribute: Attribute,
    // routing field: Ally for player units, Enemy for opponents
    pub team: Team,
    pub basic_damage_tag: DamageTag,
    pub basic_skill: SkillId,
    pub skill_ids: Vec<SkillId>,
    pub ultimate_skill: SkillId,
    pub follow_up: Option<FollowUpConfig>,
    #[serde(default)]
    pub enemy_traits: Vec<crate::combat::counterplay::EnemyTraitDeclaration>,
    #[serde(default)]
    pub charged_attack: Option<crate::combat::counterplay::ChargedAttackDeclaration>,
    /// Once-per-round conditional bonus; absent in RON for units without form identity.
    #[serde(default)]
    pub form_identity: Option<FormIdentityConfig>,
    /// Declarative Twin Core roster metadata used by contract tests and kernel-adjacent hooks.
    #[serde(default)]
    pub twin_core: TwinCoreRosterMetadata,
    /// Declarative Holy support roster metadata used by contract tests and kernel-adjacent hooks.
    #[serde(default)]
    pub holy_support: HolySupportRosterMetadata,
    pub resists: Vec<DamageTag>,
    pub toughness_max: i32,
    pub weaknesses: Vec<DamageTag>,
    pub ultimate_trigger: i32,
    pub ultimate_cap: i32,
    pub ultimate_accumulation_trigger: UltAccumulationTrigger,
    pub ultimate_charge_per_event: i32,
    pub speed: i32,
    pub evo_stage: EvoStage,
    pub evo_line: EvoLineId,
    pub evolves_to: Vec<UnitId>,
    /// Boss units with this flag get a `TempoResistance` component on spawn.
    #[serde(default)]
    pub tempo_resistant: bool,
    /// Toughness defensive archetype; defaults to Standard for backward compatibility.
    #[serde(default)]
    pub toughness_category: ToughnessCategory,
}

#[allow(dead_code)]
#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct UnitRoster(pub Vec<UnitDef>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::counterplay::{
        ChargedAttackDeclaration, EnemyCounterplayKind, EnemyTraitDeclaration, ImplementationStatus,
    };
    use crate::combat::kit::FollowUpTrigger;
    use crate::combat::types::DamageTag;
    use crate::data::skills_ron::{LegalityReasonCode, SkillBook};
    use std::collections::HashSet;

    fn canonical_roster() -> UnitRoster {
        ron::from_str(include_str!("../../assets/data/units.ron")).expect("parse units.ron")
    }

    fn canonical_skill_book() -> SkillBook {
        ron::from_str(include_str!("../../assets/data/skills.ron")).expect("parse skills.ron")
    }

    fn expected_unit_names() -> [&'static str; 15] {
        [
            // MVP v5.3 roster (D039) — 6 Child + 6 Adult
            "Agumon",
            "Gabumon",
            "Dorumon",
            "Renamon",
            "Patamon",
            "Tentomon",
            "Greymon",
            "Garurumon",
            "Kabuterimon",
            "Kyubimon",
            "DORUgamon",
            "Angemon",
            // Enemies
            "Devimon",
            "Goblimon",
            "Ogremon",
        ]
    }

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
            twin_core: TwinCoreRosterMetadata::default(),
            holy_support: HolySupportRosterMetadata::default(),
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
    fn parse_canonical_units_ron() {
        let roster = canonical_roster();
        assert_eq!(roster.0.len(), 15);

        let names: Vec<_> = roster.0.iter().map(|unit| unit.name.as_str()).collect();
        assert_eq!(names, expected_unit_names());

        let ids: HashSet<_> = roster.0.iter().map(|unit| unit.id).collect();
        assert_eq!(ids.len(), roster.0.len(), "duplicate unit ids in units.ron");

        // MVP v5.3 skill-count invariant — derived from EvoStage rather than
        // a hand-maintained name list:
        //   Adult forms (ally or enemy): 2 active skills.
        //   Child forms: 1, except Patamon which has 2 (basic + revive).
        for unit in &roster.0 {
            assert!(
                !unit.role_tags.is_empty(),
                "missing role_tags for {}",
                unit.name
            );
            assert!(
                !unit.signature_traits.is_empty(),
                "missing signature_traits for {}",
                unit.name
            );
            let expected_len = match unit.evo_stage {
                EvoStage::Adult => 2,
                EvoStage::Child if unit.name == "Patamon" => 2,
                _ => 1,
            };
            assert_eq!(
                unit.skill_ids.len(),
                expected_len,
                "unexpected active skill count for {}",
                unit.name
            );
        }

        let agumon = roster.0.iter().find(|unit| unit.name == "Agumon").unwrap();
        let renamon = roster.0.iter().find(|unit| unit.name == "Renamon").unwrap();
        let dorumon = roster.0.iter().find(|unit| unit.name == "Dorumon").unwrap();

        assert_eq!(
            agumon.follow_up,
            Some(FollowUpConfig {
                trigger: FollowUpTrigger::OnEnemyBreak,
                action: SkillId("agumon_follow_up".into()),
            })
        );
        assert_eq!(
            renamon.follow_up,
            Some(FollowUpConfig {
                trigger: FollowUpTrigger::OnAllyLowHp,
                action: SkillId("renamon_follow_up".into()),
            })
        );
        assert_eq!(
            dorumon.follow_up,
            Some(FollowUpConfig {
                trigger: FollowUpTrigger::OnEnemyKill,
                action: SkillId("dorumon_follow_up".into()),
            })
        );
        // All ally roster members have follow-ups; boss enemies may omit them.
        assert!(
            roster
                .0
                .iter()
                .filter(|unit| unit.team == Team::Ally)
                .all(|unit| unit.follow_up.is_some())
        );

        // Boss checks: enemy units must be tempo_resistant.
        let devimon = roster.0.iter().find(|unit| unit.name == "Devimon").unwrap();
        assert_eq!(devimon.team, Team::Enemy, "Devimon should be an Enemy");
        assert!(
            devimon.tempo_resistant,
            "Devimon should have tempo_resistant: true"
        );
        assert!(
            !devimon.role_tags.contains(&"vanguard".into()),
            "Devimon is a boss, not a vanguard"
        );
        assert_eq!(
            devimon.enemy_traits.len(),
            3,
            "Devimon should carry 3 typed counterplay declarations"
        );
        assert_eq!(
            devimon.enemy_traits[0].kind,
            EnemyCounterplayKind::TempoAnchor,
            "TempoAnchor should be the implemented declaration"
        );
        assert_eq!(
            devimon.enemy_traits[0].status,
            ImplementationStatus::Implemented,
            "TempoAnchor should be implemented"
        );
        assert!(matches!(
            devimon.enemy_traits[1].status,
            ImplementationStatus::Deferred {
                reason: LegalityReasonCode::EnemyTraitDeferred
            }
        ));
        assert!(matches!(
            devimon.enemy_traits[2].status,
            ImplementationStatus::Deferred {
                reason: LegalityReasonCode::EnemyTraitDeferred
            }
        ));
        let charged = devimon
            .charged_attack
            .as_ref()
            .expect("Devimon should carry a charged telegraph declaration");
        assert_eq!(charged.skill_id, SkillId("enemy_ult_fire".into()));
        assert_eq!(charged.lead_turns, 2);
        assert!(matches!(
            charged.status,
            ImplementationStatus::Deferred {
                reason: LegalityReasonCode::ChargedTelegraphDeferred
            }
        ));

        let goblimon = roster
            .0
            .iter()
            .find(|unit| unit.name == "Goblimon")
            .unwrap();
        assert!(
            goblimon.enemy_traits.is_empty(),
            "Goblimon should not declare counterplay traits"
        );
        assert!(
            goblimon.charged_attack.is_none(),
            "Goblimon should not declare a charged telegraph"
        );

        let ogremon = roster.0.iter().find(|unit| unit.name == "Ogremon").unwrap();
        assert!(
            ogremon.enemy_traits.is_empty(),
            "Ogremon does not need typed counterplay traits yet"
        );
        let ogre_charged = ogremon
            .charged_attack
            .as_ref()
            .expect("Ogremon should carry a charged telegraph declaration");
        assert_eq!(ogre_charged.skill_id, SkillId("ogremon_ult".into()));
        assert_eq!(ogre_charged.lead_turns, 1);
        assert!(matches!(
            ogre_charged.status,
            ImplementationStatus::Deferred {
                reason: LegalityReasonCode::ChargedTelegraphDeferred
            }
        ));

        // Diversification checks: at least 2 distinct ultimate_trigger thresholds and 2 distinct trigger types.
        let trigger_thresholds: std::collections::HashSet<_> =
            roster.0.iter().map(|u| u.ultimate_trigger).collect();
        assert!(
            trigger_thresholds.len() >= 2,
            "almeno 2 trigger threshold diversi: {:?}",
            trigger_thresholds
        );
        let trigger_types: std::collections::HashSet<_> = roster
            .0
            .iter()
            .map(|u| u.ultimate_accumulation_trigger)
            .collect();
        assert!(
            trigger_types.len() >= 2,
            "almeno 2 UltAccumulationTrigger diversi: {:?}",
            trigger_types
        );

        // EvoStage / evolves_to invariant — derived from `team` + `evo_stage`
        // rather than a hand-maintained name list. Only Child-stage allies
        // carry an evolution edge in the MVP v5.3 roster; everything else
        // (Adult forms and enemies) must have zero.
        for unit in &roster.0 {
            assert!(
                !unit.evo_line.0.is_empty(),
                "missing evo_line for {}",
                unit.name
            );

            let expected_evos = match (unit.team, unit.evo_stage) {
                (Team::Ally, EvoStage::Child) => 1,
                _ => 0,
            };
            assert_eq!(
                unit.evolves_to.len(),
                expected_evos,
                "unexpected evolves_to count for {} ({:?}/{:?})",
                unit.name,
                unit.team,
                unit.evo_stage,
            );
        }
    }

    #[test]
    fn rookie_skill_references_resolve_in_skill_book() {
        let roster = canonical_roster();
        let book = canonical_skill_book();
        let skill_ids: HashSet<_> = book.0.iter().map(|skill| skill.id.clone()).collect();

        for unit in &roster.0 {
            for skill_id in unit
                .skill_ids
                .iter()
                .chain(std::iter::once(&unit.basic_skill))
                .chain(std::iter::once(&unit.ultimate_skill))
            {
                assert!(
                    skill_ids.contains(skill_id),
                    "missing skill {:?} for {}",
                    skill_id,
                    unit.name
                );
            }

            if let Some(follow_up) = &unit.follow_up {
                assert!(
                    skill_ids.contains(&follow_up.action),
                    "missing follow-up action {:?} for {}",
                    follow_up.action,
                    unit.name
                );
            }
        }
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

        let err = ron::from_str::<UnitRoster>(missing_metadata)
            .expect_err("missing metadata should fail");
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

        let err =
            ron::from_str::<UnitRoster>(missing_evo).expect_err("missing evo_stage should fail");
        assert!(
            err.to_string().contains("missing field `evo_stage`"),
            "unexpected error: {}",
            err
        );
    }
}
