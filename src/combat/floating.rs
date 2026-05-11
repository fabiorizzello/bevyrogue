#![allow(dead_code)]

use bevy::prelude::*;

use crate::combat::toughness::DamageKind;
use crate::combat::types::UnitId;

pub const FLOATING_LIFETIME_SECS: f32 = 1.2;
pub const FLOATING_MAX_LIVE: usize = 8;

#[derive(Component, Debug, Clone)]
pub struct FloatingDamage {
    pub target: UnitId,
    pub amount: i32,
    pub kind: DamageKind,
    pub spawn_time: f32,
}

pub fn decay_floating_damage(
    time: Res<Time>,
    mut commands: Commands,
    q: Query<(Entity, &FloatingDamage)>,
) {
    let now = time.elapsed_secs();
    for (entity, fd) in &q {
        if now >= fd.spawn_time + FLOATING_LIFETIME_SECS {
            commands.entity(entity).despawn();
        }
    }
}
