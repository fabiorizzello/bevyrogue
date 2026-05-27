//! Proves the two generic test VFX (`assets/vfx/fire_test` and `water_test`)
//! deserialize into bevy_enoki's real `Particle2dEffect` schema — the same type
//! the combat renderer loads via `asset_server.load::<Particle2dEffect>`.
//!
//! This is the K001-safe guard for "the effect doesn't show": an enoki asset
//! that fails to parse loads as a silent error (no panic, no particles), so it
//! would render an empty scene. Pinning the parse here catches that without
//! opening a window — relevant because these `.ron` are authored externally
//! (the bevy_enoki web editor / `enoki2d_editor`), so a save can drift from the
//! `ron 0.8` reader. Both effects exercise the `Rval<Vec2>` `direction` form
//! (`Some(((x, y), randomness))`) that the baby_flame assets never used.
//!
//! Compile-time `include_str!` of git-tracked assets.
#![cfg(feature = "windowed")]

use bevy_enoki::Particle2dEffect;

#[test]
fn fire_preset_parses_into_enoki_schema() {
    let effect =
        ron::from_str::<Particle2dEffect>(include_str!("../../assets/vfx/fire_test.particle.ron"))
            .expect(
                "assets/vfx/fire_test.particle.ron must parse into bevy_enoki::Particle2dEffect",
            );

    // Continuous emitter (no OneShot attached), so it must keep feeding.
    assert!(
        effect.spawn_rate > 0.0,
        "fire is a continuous emitter and needs a positive spawn_rate to ever show particles"
    );
    assert!(
        effect.lifetime.0 > 0.0,
        "particles need a positive lifetime to render"
    );
    // The upward-direction Rval<Vec2> form is the never-loaded-before construct.
    assert!(
        effect.direction.is_some(),
        "fire's rising look depends on a fixed upward direction Rval<Vec2>"
    );
}

#[test]
fn water_preset_parses_into_enoki_schema() {
    let effect =
        ron::from_str::<Particle2dEffect>(include_str!("../../assets/vfx/water_test.particle.ron"))
            .expect(
                "assets/vfx/water_test.particle.ron must parse into bevy_enoki::Particle2dEffect",
            );

    assert!(
        effect.spawn_rate > 0.0,
        "water is a continuous emitter and needs a positive spawn_rate"
    );
    assert!(
        effect.lifetime.0 > 0.0,
        "particles need a positive lifetime to render"
    );
    // Water arcs under gravity — both the gravity Rval<Vec2> and its speed.
    assert!(
        effect.gravity_direction.is_some() && effect.gravity_speed.is_some(),
        "water's parabola depends on a gravity_direction Rval<Vec2> + gravity_speed"
    );
}
