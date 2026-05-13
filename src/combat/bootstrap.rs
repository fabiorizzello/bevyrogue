use crate::combat::team::Team;
use crate::combat::types::UnitId;
use crate::data::units_ron::{UnitDef, UnitRoster};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::av::ActionValue;
use super::enemy_counterplay::EnemyCounterplayKit;
use super::energy::{Energy, RoundEnergyTracker};
use super::kit::{FormIdentityKit, UnitSkills};
use super::resistance::TempoResistance;
use super::round_flags::RoundFlags;
use super::speed::{Speed, SpeedModifier};
use super::status_effect::StatusBag;
use super::toughness::{Toughness, ToughnessCategory};
use super::turn_order::TurnOrder;
use super::ultimate::{UltAccumulationTrigger, UltimateCharge};
use super::unit::{BasicStreak, Commander, Unit};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRequest {
    pub rookie_ids: Vec<UnitId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterComposition {
    pub allies: Vec<UnitDef>,
    pub enemies: Vec<UnitDef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionError {
    WrongPickCount { expected: usize, actual: usize },
    DuplicateRookies { duplicates: Vec<UnitId> },
    UnknownRookie { id: UnitId },
    UnselectableEntry { id: UnitId, reason: String },
}

impl std::fmt::Display for SelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionError::WrongPickCount { expected, actual } => {
                write!(f, "expected exactly {} picks, got {}", expected, actual)
            }
            SelectionError::DuplicateRookies { duplicates } => {
                write!(f, "duplicate rookies selected: {:?}", duplicates)
            }
            SelectionError::UnknownRookie { id } => {
                write!(f, "unknown rookie id: {:?}", id)
            }
            SelectionError::UnselectableEntry { id, reason } => {
                write!(f, "unselectable entry {:?}: {}", id, reason)
            }
        }
    }
}

impl std::error::Error for SelectionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterPreset {
    /// Three Goblimon minions (UnitId 102 ×3).
    MinionWave,
    /// One Ogremon mini-boss (UnitId 103) flanked by two Goblimon (UnitId 102 ×2).
    MiniBossEncounter,
    /// Devimon boss solo (UnitId 101).
    BossEncounter,
}

impl std::fmt::Display for EncounterPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncounterPreset::MinionWave => write!(f, "Minion Wave (3× Goblimon)"),
            EncounterPreset::MiniBossEncounter => {
                write!(f, "Mini-Boss Encounter (Ogremon + 2× Goblimon)")
            }
            EncounterPreset::BossEncounter => write!(f, "Boss Encounter (Devimon)"),
        }
    }
}

pub fn bootstrap_encounter(
    roster: &UnitRoster,
    request: &SelectionRequest,
    preset: EncounterPreset,
) -> Result<EncounterComposition, SelectionError> {
    if request.rookie_ids.len() != 4 {
        return Err(SelectionError::WrongPickCount {
            expected: 4,
            actual: request.rookie_ids.len(),
        });
    }

    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();
    for &id in &request.rookie_ids {
        if !seen.insert(id) {
            duplicates.push(id);
        }
    }
    if !duplicates.is_empty() {
        return Err(SelectionError::DuplicateRookies { duplicates });
    }

    let mut allies = Vec::new();
    for &id in &request.rookie_ids {
        let Some(def) = roster.0.iter().find(|d| d.id == id) else {
            return Err(SelectionError::UnknownRookie { id });
        };

        if def.team != Team::Ally {
            return Err(SelectionError::UnselectableEntry {
                id,
                reason: "unit is not on the Ally team".into(),
            });
        }
        allies.push(def.clone());
    }

    // Always inject Taichi
    allies.push(taichi_def());

    let enemy_ids: Vec<UnitId> = match preset {
        EncounterPreset::MinionWave => vec![UnitId(102), UnitId(102), UnitId(102)],
        EncounterPreset::MiniBossEncounter => vec![UnitId(103), UnitId(102), UnitId(102)],
        EncounterPreset::BossEncounter => vec![UnitId(101)],
    };

    let mut enemies = Vec::new();
    for &id in &enemy_ids {
        let Some(def) = roster.0.iter().find(|d| d.id == id) else {
            return Err(SelectionError::UnknownRookie { id });
        };
        enemies.push(def.clone());
    }

    Ok(EncounterComposition { allies, enemies })
}

pub fn spawn_unit_from_def(commands: &mut Commands, def: &UnitDef) -> Entity {
    let mut entity = commands.spawn((
        Unit {
            id: def.id,
            name: def.name.clone(),
            hp_max: def.hp_max,
            hp_current: def.hp_max,
            attribute: def.attribute,
            resists: def.resists.clone(),
            evo_stage: def.evo_stage,
        },
        ActionValue::default(), // Add ActionValue component
        Speed(def.speed),
        SpeedModifier(0),
        def.team,
        Toughness::with_category(
            def.toughness_max,
            def.weaknesses.clone(),
            def.toughness_category,
        ),
        RoundFlags::default(),
        StatusBag::default(),
        UltimateCharge::new(
            def.ultimate_trigger,
            def.ultimate_cap,
            def.ultimate_accumulation_trigger,
            def.ultimate_charge_per_event,
        ),
        UnitSkills {
            basic: def.basic_skill.clone(),
            skills: def.skill_ids.clone(),
            ultimate: def.ultimate_skill.clone(),
            follow_up: def.follow_up.clone(),
        },
        Energy::default(),
        RoundEnergyTracker::default(),
        BasicStreak::default(),
    ));

    if def.role_tags.contains(&"commander".to_string()) {
        entity.insert(Commander);
    }

    if def.tempo_resistant {
        entity.insert(TempoResistance::default());
    }

    if let Some(fi_config) = def.form_identity.clone() {
        entity.insert(FormIdentityKit { config: fi_config });
    }

    if let Some(counterplay) = EnemyCounterplayKit::from_def(def) {
        entity.insert(counterplay);
    }

    entity.id()
}

pub fn apply_composition(
    commands: &mut Commands,
    composition: &EncounterComposition,
    _order: &mut TurnOrder, // TurnOrder is now managed by AV system
) {
    for def in &composition.allies {
        spawn_unit_from_def(commands, def);
    }

    for def in &composition.enemies {
        spawn_unit_from_def(commands, def);
    }
}

pub fn taichi_def() -> UnitDef {
    UnitDef {
        id: UnitId(0),
        name: "Taichi".into(),
        role_tags: vec!["commander".into()],
        signature_traits: vec!["courage".into()],
        hp_max: 1,
        attribute: crate::combat::types::Attribute::Free,
        team: Team::Ally,
        basic_damage_tag: crate::combat::types::DamageTag::Fire,
        basic_skill: crate::combat::types::SkillId("rally".into()),
        skill_ids: vec![
            crate::combat::types::SkillId("rally".into()),
            crate::combat::types::SkillId("first_aid".into()),
            crate::combat::types::SkillId("taunt".into()),
        ],
        ultimate_skill: crate::combat::types::SkillId("brave_tri_strike".into()),
        follow_up: None,
        enemy_traits: vec![],
        charged_attack: None,
        form_identity: None,
        twin_core: crate::data::units_ron::TwinCoreRosterMetadata::default(),
        holy_support: crate::data::units_ron::HolySupportRosterMetadata::default(),
        resists: vec![],
        toughness_max: 1,
        weaknesses: vec![],
        ultimate_trigger: 100,
        ultimate_cap: 100,
        ultimate_accumulation_trigger: UltAccumulationTrigger::OnOffensivePartyEvent,
        ultimate_charge_per_event: 10,
        speed: 100,
        evo_stage: crate::combat::types::EvoStage::Child,
        evo_line: crate::combat::types::EvoLineId("tamer".into()),
        evolves_to: vec![],
        tempo_resistant: false,
        toughness_category: ToughnessCategory::Standard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_commander() {
        let mut app = App::new();
        let def = taichi_def();

        let entity = spawn_unit_from_def(&mut app.world_mut().commands(), &def);
        app.update();

        assert!(app.world().get::<Commander>(entity).is_some());
    }

    #[test]
    fn test_spawn_non_commander() {
        let mut app = App::new();
        let mut def = taichi_def();
        def.role_tags = vec!["damage".into()];

        let entity = spawn_unit_from_def(&mut app.world_mut().commands(), &def);
        app.update();

        assert!(app.world().get::<Commander>(entity).is_none());
    }
}
