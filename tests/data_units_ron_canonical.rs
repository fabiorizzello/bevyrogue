use bevyrogue::combat::counterplay::ImplementationStatus;
use bevyrogue::combat::kit::{FollowUpConfig, FollowUpTrigger};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::{EvoStage, SkillId};
use bevyrogue::data::skills_ron::{LegalityReasonCode, SkillBook};
use bevyrogue::data::units_ron::{EnemyCounterplayKind, UnitRoster};
use bevyrogue::data::{aggregate_skill_book, aggregate_unit_roster};
use std::collections::HashSet;

fn canonical_roster() -> UnitRoster {
    aggregate_unit_roster()
}

fn canonical_skill_book() -> SkillBook {
    aggregate_skill_book()
}

fn expected_unit_names() -> [&'static str; 15] {
    [
        "Agumon", "Greymon",
        "Gabumon", "Garurumon",
        "Dorumon", "DORUgamon",
        "Renamon", "Kyubimon",
        "Patamon", "Angemon",
        "Tentomon", "Kabuterimon",
        "Devimon", "Goblimon", "Ogremon",
    ]
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
