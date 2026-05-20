use bevy::prelude::*;
use bevyrogue::combat::encounter::bootstrap::{spawn_unit_from_def, taichi_def};
use bevyrogue::combat::unit::Commander;

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
