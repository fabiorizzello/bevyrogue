//! Canonical enemy-counterplay catalog assertions over the aggregated roster.
//!
//! Roster size / name order / id uniqueness / skill-ref resolution are already
//! covered by `src/data/units_ron/tests.rs` (`parse_canonical_units_ron` and
//! `rookie_skill_references_resolve_in_skill_book`); this file deliberately
//! asserts only what those do not: the exact id mapping and the per-enemy
//! counterplay / charged-attack declarations.

use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::ToughnessCategory;
use bevyrogue::combat::types::SkillId;
use bevyrogue::data::units_ron::UnitRoster;

fn load_roster() -> UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

#[test]
fn canonical_unit_ids_keep_their_fixed_mapping() {
    let roster = load_roster();
    let ids: Vec<_> = roster.0.iter().map(|unit| unit.id.0).collect();
    assert_eq!(
        ids,
        vec![1, 12, 2, 13, 5, 16, 7, 15, 9, 17, 11, 14, 101, 102, 103],
        "unit id mapping drifted — downstream encounters pin these ids"
    );
}

#[test]
fn boss_tagged_enemies_are_tempo_resistant() {
    let roster = load_roster();
    assert!(
        roster
            .0
            .iter()
            .filter(|unit| unit.team == Team::Enemy && unit.role_tags.contains(&"boss".to_string()))
            .all(|unit| unit.tempo_resistant),
        "boss-tagged enemy units in units.ron must be tempo_resistant"
    );
}

#[test]
fn enemy_counterplay_declarations_match_canonical_units_ron() {
    let roster = load_roster();

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
