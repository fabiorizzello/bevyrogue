//! M005/S04 T02 — proves Agumon's `baby_flame.impact` enoki effect asset parses
//! against bevy_enoki's real `Particle2dEffect` schema (the same type
//! `src/windowed/render.rs` loads into the `AgumonEnokiVfx` resource).
//!
//! `render.rs` lives in the binary crate so its private load/diagnostic systems
//! can't be called here. What this pins is the contract those systems depend on:
//! the authored `.particle.ron` deserializes into `Particle2dEffect` exactly as
//! enoki's `ParticleEffectLoader` would, so a future spawn seam attaching the
//! handle to a `ParticleSpawner` actually has a burst to render. Runs under the
//! `windowed` feature (matching the render path's gate) without opening a window.
//!
//! Compile-time `include_str!` of a git-tracked asset — never a gitignored path.
#![cfg(feature = "windowed")]

use bevy_enoki::{EmissionShape, Particle2dEffect};

/// Path mirrors `AGUMON_ENOKI_IMPACT_PATH` in `src/windowed/render.rs`.
fn agumon_enoki_impact() -> Particle2dEffect {
    ron::from_str::<Particle2dEffect>(include_str!(
        "../../assets/digimon/agumon/baby_flame_impact.particle.ron"
    ))
    .expect(
        "assets/digimon/agumon/baby_flame_impact.particle.ron should parse into \
         bevy_enoki::Particle2dEffect (the type render.rs loads)",
    )
}

#[test]
fn impact_effect_parses_into_enoki_schema() {
    let effect = agumon_enoki_impact();

    // One-shot burst: zero continuous rate, a non-trivial single-burst amount.
    assert_eq!(
        effect.spawn_rate, 0.0,
        "impact flash must be one-shot (spawn_rate 0), not a continuous emitter"
    );
    assert!(
        effect.spawn_amount > 0,
        "one-shot burst must emit at least one particle"
    );

    // Seeded from a small disc so embers scatter rather than stacking on a point.
    assert!(
        matches!(effect.emission_shape, EmissionShape::Circle(r) if r > 0.0),
        "impact burst should emit from a Circle emission shape with positive radius"
    );

    // A short positive lifetime keeps it a quick contact flash.
    assert!(
        effect.lifetime.0 > 0.0,
        "particles need a positive lifetime to render"
    );

    // The gradient + shrink curves the flash reads as flame are present.
    assert!(
        effect.color_curve.is_some(),
        "impact flash relies on a color_curve for its flame gradient + fade-out"
    );
    assert!(
        effect.scale_curve.is_some(),
        "impact flash relies on a scale_curve so embers collapse as they die"
    );
}
