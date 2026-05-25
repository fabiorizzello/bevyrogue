//! M004/S01 T03 — load the real authored Agumon VFX asset and evaluate its
//! curves deterministically (R004).
//!
//! This proves content-as-data, not just the in-memory schema: the actual
//! `assets/digimon/agumon/vfx.ron` parses into a typed [`VfxAsset`], the Baby
//! Flame impact fan-out effect is present, and its scale/color curves evaluate
//! to the expected deterministic values at sampled progresses (0.0, 0.5, 1.0).
//!
//! Compile-time `include_str!` (matching `anim_validation.rs`) — never reads a
//! gitignored `.gsd/`/`.planning/` path.

use bevyrogue::animation::{
    eval_color, eval_scale, resolve_effect, spawn_plan, ImpactSpawnPlan, VfxAsset,
};

/// Tight epsilon for the linearly-interpolated midpoint samples; f32 lerp is
/// deterministic but not bit-identical to a decimal literal.
const EPS: f32 = 1e-6;

fn agumon_vfx() -> VfxAsset {
    ron::from_str::<VfxAsset>(include_str!("../../assets/digimon/agumon/vfx.ron"))
        .expect("assets/digimon/agumon/vfx.ron should parse into VfxAsset")
}

fn assert_rgba_approx(actual: [f32; 4], expected: [f32; 4], ctx: &str) {
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!(
            (a - e).abs() < EPS,
            "{ctx}: channel {i} expected ~{e}, got {a}"
        );
    }
}

#[test]
fn agumon_vfx_parses_and_contains_impact_fan_out() {
    let asset = agumon_vfx();
    let impact = resolve_effect(&asset, "baby_flame.impact")
        .expect("baby_flame.impact effect present in authored asset");

    // Spawn plan mirrors the hardcoded render.rs shard-burst constants.
    assert_eq!(
        spawn_plan(impact),
        ImpactSpawnPlan { count: 8, spread_px: 64.0, ttl_ticks: 5 },
        "impact fan-out reproduces render.rs BABY_FLAME_IMPACT_SHARD_* constants"
    );
    assert_eq!(impact.placement.verb, "impact.fan_out");

    // The central flash ships as a sibling effect.
    assert!(
        resolve_effect(&asset, "baby_flame.impact_flash").is_some(),
        "central impact flash present as sibling effect"
    );
    // A missing id resolves to None so the windowed layer can log + fall back.
    assert!(resolve_effect(&asset, "missing.effect").is_none());
}

#[test]
fn agumon_impact_scale_curve_evaluates_to_eased_spread() {
    let asset = agumon_vfx();
    let impact = resolve_effect(&asset, "baby_flame.impact").expect("impact present");
    let scale = &impact.appearance.scale;

    // Ease-out outward fraction sampled at the authored keyframes.
    assert_eq!(eval_scale(scale, 0.0), 0.0, "progress 0.0 -> no spread");
    assert_eq!(eval_scale(scale, 0.5), 0.75, "progress 0.5 -> eased 0.75");
    assert_eq!(eval_scale(scale, 1.0), 1.0, "progress 1.0 -> full spread");
}

#[test]
fn agumon_impact_color_curve_holds_hue_and_fades_alpha() {
    let asset = agumon_vfx();
    let impact = resolve_effect(&asset, "baby_flame.impact").expect("impact present");
    let color = &impact.appearance.color;

    // Endpoints are exact authored keyframe values.
    assert_eq!(eval_color(color, 0.0), [1.0, 0.55, 0.2, 0.9], "spawn: alpha 0.9");
    assert_eq!(eval_color(color, 1.0), [1.0, 0.55, 0.2, 0.0], "death: alpha 0.0");
    // Midpoint: hue constant, alpha linearly halved (interpolated -> approx).
    assert_rgba_approx(eval_color(color, 0.5), [1.0, 0.55, 0.2, 0.45], "midpoint fade");
}

#[test]
fn agumon_flash_color_curve_fades_from_bright_core() {
    let asset = agumon_vfx();
    let flash = resolve_effect(&asset, "baby_flame.impact_flash").expect("flash present");
    let color = &flash.appearance.color;

    assert_eq!(eval_color(color, 0.0), [1.0, 0.82, 0.45, 0.95], "flash spawns near-opaque");
    assert_eq!(eval_color(color, 1.0), [1.0, 0.82, 0.45, 0.0], "flash fades out");
    assert_rgba_approx(eval_color(color, 0.5), [1.0, 0.82, 0.45, 0.475], "flash midpoint");
    // Flash holds a constant scale across its (short) life.
    assert_eq!(eval_scale(&flash.appearance.scale, 0.5), 1.0);
}
