use crate::combat::kit::UnitSkills;
use crate::combat::types::{SkillId, UnitId};
use crate::data::skills_ron::{LegalityReasonCode, SkillBook, SkillDef, SkillImplementation};

use super::super::types::{
    ActionQueryKind, CombatQuerySnapshot, ImplementationStatus, UnitQuerySnapshot,
};

pub(super) fn snapshot_units(snapshot: &CombatQuerySnapshot) -> Vec<UnitQuerySnapshot> {
    if !snapshot.units.is_empty() {
        return snapshot.units.clone();
    }

    let mut units = vec![snapshot.acting_unit.clone()];
    if let Some(target_unit) = &snapshot.target_unit {
        if target_unit.id != snapshot.acting_unit.id {
            units.push(target_unit.clone());
        }
    }
    units
}

pub(super) fn resolve_unit(
    snapshot: &CombatQuerySnapshot,
    unit_id: UnitId,
) -> Option<UnitQuerySnapshot> {
    snapshot_units(snapshot)
        .into_iter()
        .find(|unit| unit.id == unit_id)
}

pub(super) fn implementation_status(skill_def: &SkillDef) -> ImplementationStatus {
    match &skill_def.implementation {
        SkillImplementation::Implemented => ImplementationStatus::Implemented,
        SkillImplementation::Deferred { reason } => ImplementationStatus::Deferred {
            reason: reason.clone(),
        },
        SkillImplementation::Hidden { reason } => ImplementationStatus::Hidden {
            reason: reason.clone(),
        },
    }
}

fn kit_has_skill(kit: &UnitSkills, skill_id: &SkillId) -> bool {
    kit.basic == *skill_id
        || kit.ultimate == *skill_id
        || kit.skills.iter().any(|candidate| candidate == skill_id)
}

pub(super) fn resolve_action_skill<'a>(
    snapshot: &'a CombatQuerySnapshot,
    skill_book: &'a SkillBook,
    actor_id: UnitId,
    kind: &ActionQueryKind<'_>,
) -> Result<(UnitQuerySnapshot, &'a SkillDef), LegalityReasonCode> {
    let actor = resolve_unit(snapshot, actor_id).ok_or(LegalityReasonCode::MissingSkill)?;
    let kit = actor
        .skills
        .as_ref()
        .ok_or(LegalityReasonCode::MissingSkill)?;

    let skill_id = match kind {
        ActionQueryKind::Basic => &kit.basic,
        ActionQueryKind::Skill(skill_id) => {
            if !kit_has_skill(kit, skill_id) {
                return Err(LegalityReasonCode::MissingSkill);
            }
            skill_id
        }
        ActionQueryKind::Ultimate => &kit.ultimate,
    };

    if !kit_has_skill(kit, skill_id) {
        return Err(LegalityReasonCode::MissingSkill);
    }

    let skill_def = skill_book
        .0
        .iter()
        .find(|skill| &skill.id == skill_id)
        .ok_or(LegalityReasonCode::MissingSkill)?;

    Ok((actor, skill_def))
}
