//! M005/S05 T02 — proves Agumon's `sharp_claws.slash` and `baby_burner.detonate`
//! enoki contact-burst assets parse against bevy_enoki's real `Particle2dEffect`
//! schema (the same type `src/windowed/render.rs` loads into the per-effect-id
//! `AgumonEnokiVfx` handle map).
//!
//! Mirrors `enoki_impact_effect_parses.rs`: `render.rs` lives in the binary crate
//! so its private load/diagnostic systems can't be called here. What this pins is
//! the contract those systems depend on — each authored `.particle.ron`
//! deserializes into `Particle2dEffect` exactly as enoki's `ParticleEffectLoader`
//! would, so the generalized spawn seam attaching the handle to a `ParticleSpawner`
//! actually has a one-shot burst to render. Runs under the `windowed` feature
//! (matching the render path's gate) without opening a window.
//!
//! Compile-time `include_str!` of git-tracked assets — never a gitignored path.
#![cfg(feature = "windowed")]

use bevy_enoki::Particle2dEffect;

/// Parse an authored enoki effect asset, panicking with a path-named message on
/// failure so a malformed asset surfaces by name.
fn parse_effect(label: &str, ron: &str) -> Particle2dEffect {
    ron::from_str::<Particle2dEffect>(ron)
        .unwrap_or_else(|e| panic!("{label} should parse into bevy_enoki::Particle2dEffect: {e}"))
}

/// Assert the one-shot burst invariants shared by every Agumon contact burst.
fn assert_one_shot_burst(label: &str, effect: &Particle2dEffect) {
    assert_eq!(
        effect.spawn_rate, 0.0,
        "{label} must be one-shot (spawn_rate 0), not a continuous emitter"
    );
    assert!(
        effect.spawn_amount > 0,
        "{label} one-shot burst must emit at least one particle"
    );
    assert!(
        effect.lifetime.0 > 0.0,
        "{label} particles need a positive lifetime to render"
    );
    assert!(
        effect.color_curve.is_some(),
        "{label} relies on a color_curve for its gradient + fade-out"
    );
    assert!(
        effect.scale_curve.is_some(),
        "{label} relies on a scale_curve so particles pop/collapse over life"
    );
}

/// Assert the invariants shared by every continuous Baby Flame sequence emitter
/// (charge orb, ember swirl, projectile core): a positive spawn_rate keeps the
/// effect alive across the charge/travel window, and the gradient + scale curves
/// drive the warm HDR look and per-particle fade.
fn assert_continuous_emitter(label: &str, effect: &Particle2dEffect) {
    assert!(
        effect.spawn_rate > 0.0,
        "{label} is a continuous emitter — spawn_rate must be > 0 to sustain it"
    );
    assert!(
        effect.spawn_amount > 0,
        "{label} must emit at least one particle per spawn"
    );
    assert!(
        effect.lifetime.0 > 0.0,
        "{label} particles need a positive lifetime to render"
    );
    assert!(
        effect.color_curve.is_some(),
        "{label} relies on a color_curve for its warm gradient + fade-out"
    );
    assert!(
        effect.scale_curve.is_some(),
        "{label} relies on a scale_curve so particles pop/collapse over life"
    );
}

#[test]
fn baby_flame_charge_parses_into_enoki_schema() {
    // Path mirrors `AGUMON_ENOKI_CHARGE_PATH` wired into render.rs in this slice.
    let effect = parse_effect(
        "assets/digimon/agumon/baby_flame_charge.particle.ron",
        include_str!("../../assets/digimon/agumon/baby_flame_charge.particle.ron"),
    );
    assert_continuous_emitter("baby_flame.charge", &effect);
}

#[test]
fn baby_flame_core_parses_into_enoki_schema() {
    // M006/S08 layering: the white-hot core layer co-spawned with the charge orb.
    let effect = parse_effect(
        "assets/digimon/agumon/baby_flame_core.particle.ron",
        include_str!("../../assets/digimon/agumon/baby_flame_core.particle.ron"),
    );
    assert_continuous_emitter("baby_flame.core", &effect);
    // Anime-cel HDR check: the core's t=0 color must be white-hot — all three
    // channels well above 1.0 so `Hdr + Bloom` blooms it into the glowing heart.
    let curve = effect
        .color_curve
        .as_ref()
        .expect("baby_flame.core needs a color_curve for the white-hot heart");
    let first = &curve.points[0].0;
    assert!(
        first.red > 1.0 && first.green > 1.0 && first.blue > 1.0,
        "baby_flame.core t=0 must be white-hot (all channels > 1.0) for bloom (got r={} g={} b={})",
        first.red,
        first.green,
        first.blue
    );
}

#[test]
fn baby_flame_flames_parses_into_enoki_schema() {
    // M006/S08 layering: the rising-tongues layer that gives the body its fire
    // silhouette. Upward `direction` with low randomness = leaning tongues.
    let effect = parse_effect(
        "assets/digimon/agumon/baby_flame_flames.particle.ron",
        include_str!("../../assets/digimon/agumon/baby_flame_flames.particle.ron"),
    );
    assert_continuous_emitter("baby_flame.flames", &effect);
    // The tongues launch upward; pin the direction survived the parse (the
    // silhouette depends on it, not on wide scatter).
    let dir = effect
        .direction
        .expect("baby_flame.flames needs an upward direction for the rising silhouette");
    assert!(
        dir.0.y > 0.0,
        "baby_flame.flames must launch upward (direction.y > 0, got {})",
        dir.0.y
    );
    assert!(
        dir.1 <= 0.25,
        "baby_flame.flames needs LOW directional randomness for a coherent column, not a spray (got {})",
        dir.1
    );
}

#[test]
fn baby_flame_ember_parses_into_enoki_schema() {
    // Path mirrors `AGUMON_ENOKI_EMBER_PATH` wired into render.rs in this slice.
    let effect = parse_effect(
        "assets/digimon/agumon/baby_flame_ember.particle.ron",
        include_str!("../../assets/digimon/agumon/baby_flame_ember.particle.ron"),
    );
    assert_continuous_emitter("baby_flame.ember", &effect);
    // The ember swirl converges via an attractor; pin that it survived the parse.
    assert!(
        effect.attractors.as_ref().is_some_and(|a| !a.is_empty()),
        "baby_flame.ember needs an attractor to pull the ring inward (converge)"
    );
}

#[test]
fn baby_flame_projectile_parses_into_enoki_schema() {
    // Path mirrors `AGUMON_ENOKI_PROJECTILE_PATH` wired into render.rs in this slice.
    let effect = parse_effect(
        "assets/digimon/agumon/baby_flame_projectile.particle.ron",
        include_str!("../../assets/digimon/agumon/baby_flame_projectile.particle.ron"),
    );
    assert_continuous_emitter("baby_flame.projectile", &effect);
}

#[test]
fn sharp_claws_slash_parses_into_enoki_schema() {
    // Path mirrors `AGUMON_ENOKI_SHARP_CLAWS_PATH` in `src/windowed/render.rs`.
    let effect = parse_effect(
        "assets/digimon/agumon/sharp_claws_slash.particle.ron",
        include_str!("../../assets/digimon/agumon/sharp_claws_slash.particle.ron"),
    );
    assert_one_shot_burst("sharp_claws.slash", &effect);
}

#[test]
fn baby_burner_detonate_parses_into_enoki_schema() {
    // Path mirrors `AGUMON_ENOKI_DETONATE_PATH` in `src/windowed/render.rs`.
    let effect = parse_effect(
        "assets/digimon/agumon/baby_burner_detonate.particle.ron",
        include_str!("../../assets/digimon/agumon/baby_burner_detonate.particle.ron"),
    );
    assert_one_shot_burst("baby_burner.detonate", &effect);
}

/// M006/S08/T01 — Renamon's `diamond_storm.leaf` traveling projectile asset
/// must parse into `Particle2dEffect`. It is a continuous emitter (spawn_rate
/// > 0) driven by `advance_enoki_projectiles` across its caster→target flight.
/// Pins that the authored `.particle.ron` survives the enoki deserializer before
/// any windowed run, catching RON syntax or field errors early.
#[test]
fn diamond_storm_leaf_parses_into_enoki_schema() {
    let effect = parse_effect(
        "assets/digimon/renamon/diamond_storm_leaf.particle.ron",
        include_str!("../../assets/digimon/renamon/diamond_storm_leaf.particle.ron"),
    );
    assert_continuous_emitter("diamond_storm.leaf", &effect);
    // World-space trail: relative_positioning must be Some(false) so shards
    // stay in world space as the spawner moves caster→target.
    assert_eq!(
        effect.relative_positioning,
        Some(false),
        "diamond_storm.leaf must use world-space particles (relative_positioning: false)"
    );
    // Anime-cel HDR check: color_curve must include at least one channel > 1.0
    // at t=0 (the white-hot diamond glint on emission).
    let curve = effect
        .color_curve
        .as_ref()
        .expect("diamond_storm.leaf needs a color_curve for the HDR glint");
    let first_color = &curve.points[0].0;
    assert!(
        first_color.red > 1.0 || first_color.green > 1.0 || first_color.blue > 1.0,
        "diamond_storm.leaf color_curve t=0 must have at least one HDR channel > 1.0 for bloom (got r={} g={} b={})",
        first_color.red,
        first_color.green,
        first_color.blue
    );
}
