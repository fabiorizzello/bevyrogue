use bevyrogue::combat::bootstrap::{
    EncounterPreset, SelectionError, SelectionRequest, bootstrap_encounter,
};
use bevyrogue::combat::types::UnitId;
use bevyrogue::data::units_ron::UnitRoster;

fn canonical_roster() -> UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

// MVP v5.3 roster (D039): valid IDs are 1, 2, 5, 7, 9, 11, 12, 13, 14, 15, 16, 17.
// IDs 3, 4, 6, 8, 10 were removed in the cleanup of deprecated lines.

#[test]
fn happy_path_select_four_rookies() {
    let roster = canonical_roster();
    let request = SelectionRequest {
        rookie_ids: vec![UnitId(1), UnitId(2), UnitId(11), UnitId(9)],
    };

    let result = bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter)
        .expect("bootstrap should succeed");

    // 4 rookies + 1 Taichi = 5 allies
    assert_eq!(result.allies.len(), 5);
    // BossEncounter spawns Devimon (UnitId 101)
    assert_eq!(result.enemies.len(), 1);
    assert_eq!(result.enemies[0].name, "Devimon");

    let names: Vec<_> = result.allies.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Agumon", "Gabumon", "Tentomon", "Patamon", "Taichi"]
    );

    // Verify Taichi was injected correctly
    let taichi = result.allies.last().unwrap();
    assert_eq!(taichi.id, UnitId(0));
    assert_eq!(taichi.name, "Taichi");
    assert_eq!(taichi.role_tags, vec!["commander"]);
}

#[test]
fn fail_on_wrong_count() {
    let roster = canonical_roster();

    // Too few
    let req_few = SelectionRequest {
        rookie_ids: vec![UnitId(1), UnitId(2), UnitId(5)],
    };
    let err = bootstrap_encounter(&roster, &req_few, EncounterPreset::BossEncounter).unwrap_err();
    assert!(matches!(
        err,
        SelectionError::WrongPickCount {
            expected: 4,
            actual: 3
        }
    ));

    // Too many
    let req_many = SelectionRequest {
        rookie_ids: vec![UnitId(1), UnitId(2), UnitId(5), UnitId(7), UnitId(9)],
    };
    let err = bootstrap_encounter(&roster, &req_many, EncounterPreset::BossEncounter).unwrap_err();
    assert!(matches!(
        err,
        SelectionError::WrongPickCount {
            expected: 4,
            actual: 5
        }
    ));
}

#[test]
fn fail_on_duplicates() {
    let roster = canonical_roster();
    let request = SelectionRequest {
        rookie_ids: vec![UnitId(1), UnitId(1), UnitId(2), UnitId(5)],
    };

    let err = bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter).unwrap_err();
    if let SelectionError::DuplicateRookies { duplicates } = err {
        assert_eq!(duplicates, vec![UnitId(1)]);
    } else {
        panic!("expected DuplicateRookies error, got {:?}", err);
    }
}

#[test]
fn fail_on_unknown_rookie() {
    let roster = canonical_roster();
    let request = SelectionRequest {
        rookie_ids: vec![UnitId(1), UnitId(2), UnitId(5), UnitId(999)],
    };

    let err = bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter).unwrap_err();
    assert_eq!(err, SelectionError::UnknownRookie { id: UnitId(999) });
}

#[test]
fn cannot_manually_select_taichi() {
    let roster = canonical_roster();
    // Taichi (UnitId(0)) is not in units.ron; bootstrap rejects him as UnknownRookie.
    let request = SelectionRequest {
        rookie_ids: vec![UnitId(1), UnitId(2), UnitId(5), UnitId(0)],
    };

    let err = bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter).unwrap_err();
    assert_eq!(err, SelectionError::UnknownRookie { id: UnitId(0) });
}

#[test]
fn bootstrap_is_deterministic() {
    let roster = canonical_roster();
    let request = SelectionRequest {
        rookie_ids: vec![UnitId(9), UnitId(1), UnitId(11), UnitId(5)],
    };

    let result1 = bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter).unwrap();
    let result2 = bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter).unwrap();

    let names1: Vec<_> = result1.allies.iter().map(|u| u.name.clone()).collect();
    let names2: Vec<_> = result2.allies.iter().map(|u| u.name.clone()).collect();

    assert_eq!(names1, names2);
    assert_eq!(
        names1,
        vec!["Patamon", "Agumon", "Tentomon", "Dorumon", "Taichi"]
    );
}
