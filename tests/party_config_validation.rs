use bevyrogue::combat::bootstrap::{
    EncounterPreset, SelectionError, SelectionRequest, bootstrap_encounter,
};
use bevyrogue::combat::types::UnitId;
use bevyrogue::data::party_ron::PartyConfig;
use bevyrogue::data::units_ron::UnitRoster;
use bevyrogue::party_validation::{PartyConfigError, validate_party_config};

fn canonical_roster() -> UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

#[test]
fn party_config_deserializes_and_validates() {
    let raw = include_str!("../assets/data/party.ron");
    let p: PartyConfig = ron::from_str(raw).expect("party.ron must parse");
    assert!(validate_party_config(&p).is_ok());
    // MVP v5.3 party: Agumon, Gabumon, Tentomon, Patamon (D039)
    assert_eq!(p.ally_ids, [UnitId(1), UnitId(2), UnitId(11), UnitId(9)]);
    assert_eq!(p.tamer_id, UnitId(0));
}

#[test]
fn happy_path_bootstrap_succeeds() {
    let roster = canonical_roster();
    // MVP v5.3 valid IDs: 1, 2, 5, 7, 9, 11, 12, 13, 14, 15, 16, 17.
    let request = SelectionRequest {
        rookie_ids: vec![UnitId(1), UnitId(2), UnitId(11), UnitId(9)],
    };
    let composition = bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter)
        .expect("bootstrap should succeed");
    assert_eq!(composition.allies.len(), 5);
}

#[test]
fn unknown_rookie_is_rejected() {
    let roster = canonical_roster();
    let request = SelectionRequest {
        rookie_ids: vec![UnitId(1), UnitId(2), UnitId(3), UnitId(99)],
    };
    let err = bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter).unwrap_err();
    assert!(matches!(err, SelectionError::UnknownRookie { .. }));
}

#[test]
fn wrong_pick_count_is_rejected() {
    let roster = canonical_roster();
    let request = SelectionRequest {
        rookie_ids: vec![UnitId(1), UnitId(2), UnitId(3)],
    };
    let err = bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter).unwrap_err();
    assert!(matches!(err, SelectionError::WrongPickCount { .. }));
}

#[test]
fn wrong_tamer_is_rejected() {
    let cfg = PartyConfig {
        ally_ids: [UnitId(1), UnitId(2), UnitId(3), UnitId(4)],
        tamer_id: UnitId(1),
    };
    let err = validate_party_config(&cfg).unwrap_err();
    assert_eq!(err, PartyConfigError::WrongTamer { got: UnitId(1) });
}
