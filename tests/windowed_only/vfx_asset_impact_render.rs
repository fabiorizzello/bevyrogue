//! M004/S01 T04 — windowed integration of the Baby Flame impact fan-out data
//! path.
//!
//! `src/windowed/render.rs` lives in the binary crate, so its private spawn/tick
//! helpers can't be called here. What this test pins is the *contract render.rs
//! consumes*: the authored `assets/digimon/agumon/vfx.ron` resolves to the
//! impact effect, [`spawn_plan`] yields the shard count/spread/ttl the burst
//! loop reads, and [`eval_scale`]/[`eval_color`] produce the per-tick outward
//! distance + rgba the shard branch writes onto each particle. It runs under the
//! `windowed` feature (matching the render path's gate) without opening a window.
//!
//! Compile-time `include_str!` of a git-tracked asset — never a gitignored path.
#![cfg(feature = "windowed")]

use bevyrogue::animation::{
    eval_color, eval_scale, resolve_effect, spawn_plan, ImpactSpawnPlan, VfxAsset,
};

/// The effect id render.rs resolves (`AGUMON_IMPACT_EFFECT_ID`).
const IMPACT_EFFECT_ID: &str = "baby_flame.impact";
/// Tight epsilon for interpolated f32 samples.
const EPS: f32 = 1e-6;

fn agumon_vfx() -> VfxAsset {
    ron::from_str::<VfxAsset>(include_str!("../../assets/digimon/agumon/vfx.ron"))
        .expect("assets/digimon/agumon/vfx.ron should parse into VfxAsset")
}

/// Reproduces the render.rs data-path shard offset: `spread_px * eval_scale`
/// projected along the shard's fan-out angle.
fn data_shard_offset(plan: &ImpactSpawnPlan, frac: f32, phase: f32) -> [f32; 2] {
    let dist = plan.spread_px * frac;
    [dist * phase.cos(), dist * phase.sin()]
}

#[test]
fn impact_effect_resolves_to_the_spawn_plan_the_burst_loop_consumes() {
    let asset = agumon_vfx();
    let impact = resolve_effect(&asset, IMPACT_EFFECT_ID)
        .expect("render.rs resolves baby_flame.impact from the data path");

    // The burst loop sources count + lifetime from here; the per-tick branch
    // sources the outward distance from spread_px.
    assert_eq!(
        spawn_plan(impact),
        ImpactSpawnPlan {
            count: 8,
            spread_px: 64.0,
            ttl_ticks: 5,
        },
        "data path must reproduce the hardcoded BABY_FLAME_IMPACT_SHARD_* burst constants"
    );

    // A missing id resolves to None so render.rs logs + falls back rather than panicking.
    assert!(
        resolve_effect(&asset, "missing.effect").is_none(),
        "absent effect id must resolve to None for the windowed fallback path"
    );
}

#[test]
fn shard_offset_curve_matches_what_the_per_tick_branch_writes() {
    let asset = agumon_vfx();
    let impact = resolve_effect(&asset, IMPACT_EFFECT_ID).expect("impact present");
    let plan = spawn_plan(impact);
    let scale = &impact.appearance.scale;

    // age 0 of ttl 5 -> progress 0.0 -> no outward travel.
    let frac0 = eval_scale(scale, 0.0);
    assert_eq!(frac0, 0.0);
    let off0 = data_shard_offset(&plan, frac0, 0.0);
    assert!(off0[0].abs() < EPS && off0[1].abs() < EPS, "spawn sits on the impact origin");

    // Eased midpoint fraction 0.75 along the +x fan direction (phase 0).
    let frac_mid = eval_scale(scale, 0.5);
    assert_eq!(frac_mid, 0.75);
    let off_mid = data_shard_offset(&plan, frac_mid, 0.0);
    assert!((off_mid[0] - 48.0).abs() < EPS, "0.75 * 64.0 spread = 48px outward");
    assert!(off_mid[1].abs() < EPS);

    // End of life -> full spread along a quarter-turn fan direction (+y).
    let frac_end = eval_scale(scale, 1.0);
    assert_eq!(frac_end, 1.0);
    let off_end = data_shard_offset(&plan, frac_end, std::f32::consts::FRAC_PI_2);
    assert!(off_end[0].abs() < 1e-4, "quarter-turn shard has ~0 x travel");
    assert!(
        (off_end[1] - plan.spread_px).abs() < 1e-4,
        "quarter-turn shard reaches full spread on +y"
    );
}

#[test]
fn shard_color_curve_matches_what_the_per_tick_branch_writes() {
    let asset = agumon_vfx();
    let impact = resolve_effect(&asset, IMPACT_EFFECT_ID).expect("impact present");
    let color = &impact.appearance.color;

    // Constant ember hue, alpha linear-fades to transparent over life.
    assert_eq!(eval_color(color, 0.0), [1.0, 0.55, 0.2, 0.9], "spawn rgba");
    let mid = eval_color(color, 0.5);
    for (i, (a, e)) in mid.iter().zip([1.0, 0.55, 0.2, 0.45].iter()).enumerate() {
        assert!((a - e).abs() < EPS, "midpoint channel {i}: expected ~{e}, got {a}");
    }
    assert_eq!(eval_color(color, 1.0), [1.0, 0.55, 0.2, 0.0], "death rgba");
}
