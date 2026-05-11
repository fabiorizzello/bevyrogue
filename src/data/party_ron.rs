use bevy::prelude::*;
use serde::Deserialize;

use crate::combat::types::UnitId;

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct PartyConfig {
    pub ally_ids: [UnitId; 4],
    pub tamer_id: UnitId,
}
