use crate::combat::types::UnitId;
use bevy::prelude::Resource;
use std::collections::HashMap;

/// Global blueprint state store for counters, stacks, and other per-unit persistent values.
///
/// Write exclusively via `Intent::SetBlueprintState`; read via `SkillCtx::blueprint_state`.
/// (D034 / MEM001 canonical blueprint write-path).
#[derive(Resource, Default, Debug)]
pub struct BlueprintState {
    pub map: HashMap<(UnitId, String), i64>,
}
