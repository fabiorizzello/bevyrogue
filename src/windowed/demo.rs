use std::collections::HashSet;

use bevy::prelude::Resource;

use bevyrogue::combat::bootstrap::EncounterComposition;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::UnitId;
use bevyrogue::data::units_ron::{UnitDef, UnitRoster};

/// Windowed-demo composition entries contributed by per-Digimon modules.
///
/// Each entry clones a source `UnitDef` from the merged roster, then applies a
/// stable spawned `UnitId`, target `Team`, and optional display-name override.
/// The engine combines all registered entries into one windowed demo without
/// hardcoding a single species/preset in `src/windowed/mod.rs`.
#[derive(Resource, Debug, Clone, Default)]
pub(in crate::windowed) struct WindowedDemoRegistry {
    pub(in crate::windowed) entries: Vec<WindowedDemoEntry>,
}

#[derive(Debug, Clone)]
pub(in crate::windowed) struct WindowedDemoEntry {
    pub(in crate::windowed) demo_id: String,
    pub(in crate::windowed) source_unit_id: UnitId,
    pub(in crate::windowed) spawned_unit_id: UnitId,
    pub(in crate::windowed) team: Team,
    pub(in crate::windowed) name_override: Option<String>,
}

pub(in crate::windowed) fn build_demo_composition(
    roster: &UnitRoster,
    registry: &WindowedDemoRegistry,
) -> Result<Option<EncounterComposition>, String> {
    if registry.entries.is_empty() {
        return Ok(None);
    }

    let mut seen_spawned_ids = HashSet::new();
    let mut allies = Vec::new();
    let mut enemies = Vec::new();

    for entry in &registry.entries {
        if !seen_spawned_ids.insert(entry.spawned_unit_id) {
            return Err(format!(
                "windowed demo entry {:?} reuses spawned unit id {:?}",
                entry.demo_id, entry.spawned_unit_id
            ));
        }

        let Some(source) = roster.0.iter().find(|unit| unit.id == entry.source_unit_id) else {
            return Err(format!(
                "windowed demo entry {:?} references missing source unit {:?}",
                entry.demo_id, entry.source_unit_id
            ));
        };

        let mut unit = clone_for_demo(source, entry);
        if let Some(name) = &entry.name_override {
            unit.name = name.clone();
        }

        match entry.team {
            Team::Ally => allies.push(unit),
            Team::Enemy => enemies.push(unit),
        }
    }

    Ok(Some(EncounterComposition { allies, enemies }))
}

fn clone_for_demo(source: &UnitDef, entry: &WindowedDemoEntry) -> UnitDef {
    let mut unit = source.clone();
    unit.id = entry.spawned_unit_id;
    unit.team = entry.team;
    unit
}
