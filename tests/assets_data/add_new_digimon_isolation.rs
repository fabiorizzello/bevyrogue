use bevyrogue::data::units_ron::UnitRoster;

fn canonical_roster() -> UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

#[test]
fn add_new_digimon_roster_metadata_stays_optional_for_existing_units() {
    let roster = canonical_roster();
    let agumon = roster
        .0
        .iter()
        .find(|unit| unit.name == "Agumon")
        .expect("Agumon in roster");

    assert!(
        agumon.blueprint_metadata.0.is_empty(),
        "adding a new digimon must not require shared roster edits for existing units"
    );
}
