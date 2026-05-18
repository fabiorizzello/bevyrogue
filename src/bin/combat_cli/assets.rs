use std::path::PathBuf;

use bevyrogue::data::units_ron::UnitDef;

pub fn manifest_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

pub fn manifest_asset_path(relative_path: &str) -> PathBuf {
    manifest_assets_dir().join(relative_path)
}

pub fn verify_required_data_assets() -> Result<(), String> {
    // Check that per-digimon asset directories exist instead of monolithic files.
    for relative_path in [
        "data/digimon/agumon/unit.ron",
        "data/digimon/gabumon/unit.ron",
        "data/party.ron",
    ] {
        let path = manifest_asset_path(relative_path);
        if !path.is_file() {
            return Err(format!("required data asset missing: {}", path.display()));
        }
    }
    Ok(())
}

pub fn load_ally_roster() -> Result<Vec<UnitDef>, String> {
    let roster =
        bevyrogue::data::try_aggregate_unit_roster().map_err(|e| e.to_string())?;
    Ok(roster
        .0
        .into_iter()
        .filter(|u| u.team == bevyrogue::combat::team::Team::Ally)
        .collect())
}
