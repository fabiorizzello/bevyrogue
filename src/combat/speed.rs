use bevy::prelude::*;

#[allow(dead_code)]
#[derive(Component, Debug, Clone, Copy)]
pub struct Speed(pub i32);

/// Delta applied to Speed when Chilled is active. Turn-order code reads (Speed - SpeedModifier)
/// without re-seeding the static VecDeque.
#[allow(dead_code)]
#[derive(Component, Debug, Clone, Copy)]
pub struct SpeedModifier(pub i32);
