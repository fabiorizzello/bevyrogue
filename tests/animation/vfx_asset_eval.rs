//! M004/S01 T02 — pure deterministic appearance-curve evaluator (R004).
//!
//! Asserts the curve math is pure and headless-testable: endpoint equality at
//! progress 0.0/1.0, the linear interpolant at a midpoint, clamping outside
//! `[0, 1]`, documented defaults for empty curves, single-keyframe constancy,
//! and bit-identical determinism across repeated calls (Q7 boundary/negative).

use bevyrogue::animation::{
    eval_color, eval_scale, resolve_effect, spawn_plan, Appearance, ColorCurve, ColorKeyframe,
    EffectDef, ImpactSpawnPlan, Placement, PlacementAnchor, PlacementParams, ScaleCurve,
    ScaleKeyframe, VfxAsset,
};

fn scale_curve() -> ScaleCurve {
    ScaleCurve(vec![
        ScaleKeyframe { t: 0.0, value: 0.2 },
        ScaleKeyframe { t: 1.0, value: 1.0 },
    ])
}

fn color_curve() -> ColorCurve {
    ColorCurve(vec![
        ColorKeyframe { t: 0.0, rgba: [1.0, 0.8, 0.2, 1.0] },
        ColorKeyframe { t: 1.0, rgba: [1.0, 0.2, 0.0, 0.0] },
    ])
}

#[test]
fn eval_scale_returns_endpoint_keyframes_at_0_and_1() {
    let curve = scale_curve();
    assert_eq!(eval_scale(&curve, 0.0), 0.2, "progress 0.0 -> first keyframe");
    assert_eq!(eval_scale(&curve, 1.0), 1.0, "progress 1.0 -> last keyframe");
}

#[test]
fn eval_scale_midpoint_is_linear_interpolant() {
    // Halfway between 0.2 and 1.0 is 0.6.
    assert_eq!(eval_scale(&scale_curve(), 0.5), 0.6);
}

#[test]
fn eval_scale_clamps_progress_outside_unit_interval() {
    let curve = scale_curve();
    assert_eq!(eval_scale(&curve, -5.0), 0.2, "below 0 clamps to first");
    assert_eq!(eval_scale(&curve, 5.0), 1.0, "above 1 clamps to last");
}

#[test]
fn eval_scale_empty_curve_returns_default() {
    assert_eq!(
        eval_scale(&ScaleCurve(vec![]), 0.5),
        1.0,
        "empty scale curve documents a 1.0 default"
    );
}

#[test]
fn eval_scale_single_keyframe_is_constant() {
    let curve = ScaleCurve(vec![ScaleKeyframe { t: 0.4, value: 0.75 }]);
    for p in [-1.0, 0.0, 0.4, 0.7, 1.0, 2.0] {
        assert_eq!(eval_scale(&curve, p), 0.75, "single keyframe holds at progress {p}");
    }
}

#[test]
fn eval_scale_is_deterministic_across_repeated_calls() {
    let curve = scale_curve();
    let baseline = eval_scale(&curve, 0.37);
    for _ in 0..1000 {
        assert_eq!(eval_scale(&curve, 0.37), baseline, "same input -> identical output");
    }
}

#[test]
fn eval_color_returns_endpoint_keyframes_at_0_and_1() {
    let curve = color_curve();
    assert_eq!(eval_color(&curve, 0.0), [1.0, 0.8, 0.2, 1.0]);
    assert_eq!(eval_color(&curve, 1.0), [1.0, 0.2, 0.0, 0.0]);
}

#[test]
fn eval_color_midpoint_is_per_channel_linear_interpolant() {
    // Per channel: r=1.0, g=0.5, b=0.1, a=0.5.
    assert_eq!(eval_color(&color_curve(), 0.5), [1.0, 0.5, 0.1, 0.5]);
}

#[test]
fn eval_color_clamps_and_defaults() {
    let curve = color_curve();
    assert_eq!(eval_color(&curve, -2.0), [1.0, 0.8, 0.2, 1.0], "below 0 clamps to first");
    assert_eq!(eval_color(&curve, 3.0), [1.0, 0.2, 0.0, 0.0], "above 1 clamps to last");
    assert_eq!(
        eval_color(&ColorCurve(vec![]), 0.5),
        [1.0, 1.0, 1.0, 1.0],
        "empty color curve documents opaque-white default"
    );
}

#[test]
fn eval_color_is_deterministic_across_repeated_calls() {
    let curve = color_curve();
    let baseline = eval_color(&curve, 0.62);
    for _ in 0..1000 {
        assert_eq!(eval_color(&curve, 0.62), baseline);
    }
}

#[test]
fn resolve_effect_and_spawn_plan_read_appearance() {
    let effect = EffectDef {
        placement: Placement {
            verb: "agumon/baby_flame/fan_out".to_owned(),
            params: PlacementParams::FanOut { spread_px: 64.0 },
            anchor: PlacementAnchor::TargetCenter,
        },
        appearance: Appearance {
            count: 8,
            spread_px: 24.0,
            ttl_ticks: 30,
            scale: scale_curve(),
            color: color_curve(),
            size_px: 14.0,
            texture: "baby_flame_impact".to_owned(),
        },
        on_expire: None,
    };
    let asset: VfxAsset = ron::from_str(
        r#"(
            effects: {
                "baby_flame.impact": (
                    placement: (
                        verb: "agumon/baby_flame/fan_out",
                        params: FanOut(spread_px: 64.0),
                        anchor: TargetCenter,
                    ),
                    appearance: (
                        count: 8,
                        spread_px: 24.0,
                        ttl_ticks: 30,
                        size_px: 14.0,
                        texture: "baby_flame_impact",
                        scale: [(t: 0.0, value: 0.2), (t: 1.0, value: 1.0)],
                        color: [
                            (t: 0.0, rgba: (1.0, 0.8, 0.2, 1.0)),
                            (t: 1.0, rgba: (1.0, 0.2, 0.0, 0.0)),
                        ],
                    ),
                ),
            },
        )"#,
    )
    .expect("sample asset parses");

    assert!(
        resolve_effect(&asset, "missing.effect").is_none(),
        "absent effect id resolves to None for windowed fallback"
    );
    let resolved = resolve_effect(&asset, "baby_flame.impact").expect("present effect resolves");
    assert_eq!(spawn_plan(resolved), spawn_plan(&effect));
    assert_eq!(
        spawn_plan(resolved),
        ImpactSpawnPlan { count: 8, spread_px: 24.0, ttl_ticks: 30 }
    );
}
