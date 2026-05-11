use std::collections::HashSet;

use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::ToughnessCategory;
use bevyrogue::combat::types::SkillId;
use bevyrogue::data::{skills_ron::SkillBook, units_ron::UnitRoster};

fn load_roster() -> UnitRoster {
    ron::from_str(include_str!("../assets/data/units.ron")).expect("parse units.ron")
}

fn load_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
}

#[test]
fn s11_roster_catalog_is_the_canonical_roster() {
    let roster = load_roster();
    let expected_names = [
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
        // Enemies — boss is tempo_resistant; minions/mini-bosses are not
        "Devimon",
        "Goblimon",
        "Ogremon",
    ];

    assert_eq!(
        roster.0.len(),
        expected_names.len(),
        "unexpected roster size"
    );

    let names: Vec<_> = roster.0.iter().map(|unit| unit.name.as_str()).collect();
    assert_eq!(names, expected_names, "roster order drifted");

    let ids: Vec<_> = roster.0.iter().map(|unit| unit.id.0).collect();
    assert_eq!(
        ids,
        vec![1, 2, 5, 7, 9, 11, 12, 13, 14, 15, 16, 17, 101, 102, 103],
        "unexpected unit ids"
    );

    let unique_ids: HashSet<_> = roster.0.iter().map(|unit| unit.id).collect();
    assert_eq!(
        unique_ids.len(),
        roster.0.len(),
        "duplicate unit ids detected"
    );

    // Boss enemies (role_tag "boss") must be tempo_resistant; minion/mini-boss enemies are not.
    assert!(
        roster
            .0
            .iter()
            .filter(|unit| unit.team == Team::Enemy && unit.role_tags.contains(&"boss".to_string()))
            .all(|unit| unit.tempo_resistant),
        "boss-tagged enemy units in units.ron must be tempo_resistant"
    );
    assert!(
        roster
            .0
            .iter()
            .all(|unit| !unit.role_tags.is_empty() && !unit.signature_traits.is_empty()),
        "every unit must carry catalog metadata for bootstrap selection"
    );
    assert!(
        roster.0.iter().all(|unit| !unit.skill_ids.is_empty()),
        "every unit must have at least one active skill"
    );

    let devimon = roster.0.iter().find(|unit| unit.name == "Devimon").unwrap();
    assert_eq!(devimon.toughness_category, ToughnessCategory::Armored);
    assert_eq!(
        devimon
            .enemy_traits
            .iter()
            .map(|decl| decl.kind)
            .collect::<Vec<_>>(),
        vec![
            bevyrogue::data::units_ron::EnemyCounterplayKind::TempoAnchor,
            bevyrogue::data::units_ron::EnemyCounterplayKind::TypeTrap,
            bevyrogue::data::units_ron::EnemyCounterplayKind::ReactiveArmor,
        ]
    );
    assert!(matches!(
        devimon.enemy_traits[0].status,
        bevyrogue::data::units_ron::EnemyCounterplayStatus::Implemented
    ));
    assert!(matches!(
        devimon.enemy_traits[1].status,
        bevyrogue::data::units_ron::EnemyCounterplayStatus::Deferred {
            reason: bevyrogue::data::skills_ron::LegalityReasonCode::EnemyTraitDeferred
        }
    ));
    assert!(matches!(
        devimon.enemy_traits[2].status,
        bevyrogue::data::units_ron::EnemyCounterplayStatus::Deferred {
            reason: bevyrogue::data::skills_ron::LegalityReasonCode::EnemyTraitDeferred
        }
    ));
    let charged_attack = devimon
        .charged_attack
        .as_ref()
        .expect("Devimon should declare a charged attack");
    assert_eq!(charged_attack.skill_id, SkillId("enemy_ult_fire".into()));
    assert_eq!(charged_attack.lead_turns, 2);
    assert!(matches!(
        charged_attack.status,
        bevyrogue::data::units_ron::EnemyCounterplayStatus::Deferred {
            reason: bevyrogue::data::skills_ron::LegalityReasonCode::ChargedTelegraphDeferred
        }
    ));

    let goblimon = roster
        .0
        .iter()
        .find(|unit| unit.name == "Goblimon")
        .unwrap();
    assert!(
        goblimon.enemy_traits.is_empty(),
        "Goblimon should not declare enemy traits"
    );
    assert!(
        goblimon.charged_attack.is_none(),
        "Goblimon should not declare a charged attack"
    );

    let ogremon = roster.0.iter().find(|unit| unit.name == "Ogremon").unwrap();
    assert!(
        ogremon.enemy_traits.is_empty(),
        "Ogremon should not declare enemy traits"
    );
    let ogre_charge = ogremon
        .charged_attack
        .as_ref()
        .expect("Ogremon should declare a charged attack");
    assert_eq!(ogre_charge.skill_id, SkillId("ogremon_ult".into()));
    assert_eq!(ogre_charge.lead_turns, 1);
    assert!(matches!(
        ogre_charge.status,
        bevyrogue::data::units_ron::EnemyCounterplayStatus::Deferred {
            reason: bevyrogue::data::skills_ron::LegalityReasonCode::ChargedTelegraphDeferred
        }
    ));
}

#[test]
fn s11_rookie_skill_refs_resolve_against_the_skill_book() {
    let roster = load_roster();
    let book = load_skill_book();
    let skills: HashSet<_> = book.0.iter().map(|skill| skill.id.clone()).collect();

    for unit in &roster.0 {
        for skill_id in unit
            .skill_ids
            .iter()
            .chain(std::iter::once(&unit.basic_skill))
            .chain(std::iter::once(&unit.ultimate_skill))
        {
            assert!(
                skills.contains(skill_id),
                "missing tracked skill {:?} for {}",
                skill_id,
                unit.name
            );
        }

        if let Some(follow_up) = &unit.follow_up {
            assert!(
                skills.contains(&follow_up.action),
                "missing follow-up skill {:?} for {}",
                follow_up.action,
                unit.name
            );
        }
    }

    assert!(
        skills.contains(&SkillId("agumon_follow_up".into()))
            && skills.contains(&SkillId("renamon_follow_up".into()))
            && skills.contains(&SkillId("dorumon_follow_up".into())),
        "pilot follow-up kits must remain tracked in the book"
    );
}
