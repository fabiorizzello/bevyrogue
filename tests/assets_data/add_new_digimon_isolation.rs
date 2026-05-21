use bevyrogue::data::units_ron::UnitRoster;

fn canonical_roster() -> UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

#[test]
fn add_new_digimon_roster_metadata_stays_optional_for_non_opted_in_units() {
    let roster = canonical_roster();
    let gabumon = roster
        .0
        .iter()
        .find(|unit| unit.name == "Gabumon")
        .expect("Gabumon in roster");

    assert!(
        gabumon.blueprint_metadata.0.is_empty(),
        "adding owner-keyed ult metadata for one Digimon must not force shared roster edits for non-opted-in units"
    );
}
