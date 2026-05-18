use super::*;
use crate::data::skills_ron::SkillBook;

mod canonical;
mod roundtrip;

fn canonical_roster() -> UnitRoster {
    crate::data::aggregate_unit_roster()
}

fn canonical_skill_book() -> SkillBook {
    crate::data::aggregate_skill_book()
}

fn expected_unit_names() -> [&'static str; 15] {
    [
        // Per-digimon file order: each evo line lists Child then Adult
        "Agumon", "Greymon",
        "Gabumon", "Garurumon",
        "Dorumon", "DORUgamon",
        "Renamon", "Kyubimon",
        "Patamon", "Angemon",
        "Tentomon", "Kabuterimon",
        // Enemies
        "Devimon", "Goblimon", "Ogremon",
    ]
}
