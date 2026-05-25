//! M004/S02 T02 — load the real authored Agumon VFX asset, evaluate its curves
//! deterministically (R004), and exercise the headless load-time validation path.
//!
//! This proves content-as-data, not just the in-memory schema: the actual
//! `assets/digimon/agumon/vfx.ron` parses into a typed [`VfxAsset`], all five
//! Baby Flame effects are present, their spawn plans / curves evaluate to the
//! expected deterministic values, the projectile->impact chain is wired via
//! `on_expire`, and [`validate_effects`] accepts the real asset while naming the
//! offending id for synthetic broken assets (Q7 negative tests).
//!
//! Compile-time `include_str!` (matching `anim_validation.rs`) — never reads a
//! gitignored `.gsd/`/`.planning/` path.

use bevyrogue::animation::{
    eval_color, eval_scale, resolve_effect, spawn_plan, validate_effects, ImpactSpawnPlan,
    VfxAsset, VfxValidationError,
};

/// Tight epsilon for the linearly-interpolated midpoint samples; f32 lerp is
/// deterministic but not bit-identical to a decimal literal.
const EPS: f32 = 1e-6;

/// The placement verb ids the windowed `PlacementExt` Registry registers in S02
/// (one per pure verb in `src/animation/placement.rs`).
const KNOWN_VERBS: &[&str] = &[
    "agumon/baby_flame/static",
    "agumon/baby_flame/converge_inward",
    "agumon/baby_flame/arc_launch",
    "agumon/baby_flame/fan_out",
];

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
fn agumon_vfx_contains_all_five_effects() {
    let asset = agumon_vfx();
    for id in [
        "baby_flame.charge",
        "baby_flame.ember",
        "baby_flame.projectile",
        "baby_flame.impact",
        "baby_flame.impact_flash",
    ] {
        assert!(
            resolve_effect(&asset, id).is_some(),
            "authored asset must contain effect `{id}`"
        );
    }
    // A missing id resolves to None so the windowed layer can log + fall back.
    assert!(resolve_effect(&asset, "missing.effect").is_none());
}

#[test]
fn agumon_spawn_plans_reproduce_hardcoded_constants() {
    let asset = agumon_vfx();

    let charge = resolve_effect(&asset, "baby_flame.charge").expect("charge present");
    assert_eq!(
        spawn_plan(charge),
        ImpactSpawnPlan { count: 1, spread_px: 0.0, ttl_ticks: 24 }
    );
    assert_eq!(charge.placement.verb, "agumon/baby_flame/static");
    assert_eq!(charge.appearance.size_px, 22.0);
    assert_eq!(charge.appearance.texture, "baby_flame_charge");

    let ember = resolve_effect(&asset, "baby_flame.ember").expect("ember present");
    assert_eq!(
        spawn_plan(ember),
        ImpactSpawnPlan { count: 7, spread_px: 0.0, ttl_ticks: 24 },
        "ember reproduces BABY_FLAME_EMBER_COUNT / TTL"
    );
    assert_eq!(ember.placement.verb, "agumon/baby_flame/converge_inward");
    assert_eq!(ember.appearance.size_px, 11.0);

    let projectile = resolve_effect(&asset, "baby_flame.projectile").expect("projectile present");
    assert_eq!(
        spawn_plan(projectile),
        ImpactSpawnPlan { count: 1, spread_px: 0.0, ttl_ticks: 4 }
    );
    assert_eq!(projectile.placement.verb, "agumon/baby_flame/arc_launch");
    assert_eq!(projectile.appearance.size_px, 16.0);
    assert_eq!(projectile.appearance.texture, "baby_flame_projectile");

    let impact = resolve_effect(&asset, "baby_flame.impact").expect("impact present");
    assert_eq!(
        spawn_plan(impact),
        ImpactSpawnPlan { count: 8, spread_px: 64.0, ttl_ticks: 5 },
        "impact fan-out reproduces render.rs BABY_FLAME_IMPACT_SHARD_* constants"
    );
    assert_eq!(impact.placement.verb, "agumon/baby_flame/fan_out");
    assert_eq!(impact.appearance.size_px, 14.0);
}

#[test]
fn projectile_on_expire_chains_the_impact_burst() {
    let asset = agumon_vfx();
    let projectile = resolve_effect(&asset, "baby_flame.projectile").expect("projectile present");
    let chained = projectile
        .on_expire
        .as_ref()
        .expect("projectile chains an effect on expire");
    assert_eq!(
        chained.0, "baby_flame.impact",
        "projectile->impact burst is now data, not a hardcoded spawn"
    );
}

#[test]
fn agumon_charge_curves_grow_and_brighten() {
    let asset = agumon_vfx();
    let charge = resolve_effect(&asset, "baby_flame.charge").expect("charge present");

    // Growth maxes by 0.25 life (age 6 of 24), then holds (micro-pulse dropped).
    assert_eq!(eval_scale(&charge.appearance.scale, 0.0), 0.42);
    assert_eq!(eval_scale(&charge.appearance.scale, 0.25), 0.9);
    assert_eq!(eval_scale(&charge.appearance.scale, 1.0), 0.9);

    // Overbright linear RGB (>1.0) so the windowed HDR+bloom camera blooms the
    // charge orb (M004/S05 color policy); alpha still climbs 0.35 -> 0.88.
    assert_eq!(eval_color(&charge.appearance.color, 0.0), [1.8, 1.2, 0.35, 0.35]);
    assert_eq!(eval_color(&charge.appearance.color, 1.0), [2.8, 1.7, 0.45, 0.88]);
}

#[test]
fn agumon_ember_color_fades_from_bright() {
    let asset = agumon_vfx();
    let ember = resolve_effect(&asset, "baby_flame.ember").expect("ember present");
    let color = &ember.appearance.color;
    assert_eq!(eval_color(color, 0.0), [1.0, 0.85, 0.4, 0.9], "ember spawns bright");
    assert_eq!(eval_color(color, 1.0), [1.0, 0.85, 0.4, 0.0], "ember fades fully");
    assert_rgba_approx(eval_color(color, 0.5), [1.0, 0.85, 0.4, 0.45], "ember midpoint fade");
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
    assert_eq!(eval_color(color, 0.0), [2.2, 1.0, 0.3, 0.9], "spawn: alpha 0.9");
    assert_eq!(eval_color(color, 1.0), [2.2, 1.0, 0.3, 0.0], "death: alpha 0.0");
    // Midpoint: hue constant, alpha linearly halved (interpolated -> approx).
    assert_rgba_approx(eval_color(color, 0.5), [2.2, 1.0, 0.3, 0.45], "midpoint fade");
}

#[test]
fn agumon_flash_color_curve_fades_from_bright_core() {
    let asset = agumon_vfx();
    let flash = resolve_effect(&asset, "baby_flame.impact_flash").expect("flash present");
    let color = &flash.appearance.color;

    assert_eq!(eval_color(color, 0.0), [3.6, 2.2, 0.8, 0.95], "flash spawns near-opaque");
    assert_eq!(eval_color(color, 1.0), [3.6, 2.2, 0.8, 0.0], "flash fades out");
    assert_rgba_approx(eval_color(color, 0.5), [3.6, 2.2, 0.8, 0.475], "flash midpoint");
    // Flash holds a constant scale across its (short) life.
    assert_eq!(eval_scale(&flash.appearance.scale, 0.5), 1.0);
}

#[test]
fn baby_burner_detonate_is_fan_out_burst_chaining_flash() {
    let asset = agumon_vfx();
    let det = resolve_effect(&asset, "baby_burner.detonate").expect("detonate present");

    assert_eq!(det.placement.verb, "agumon/baby_flame/fan_out");
    assert_eq!(
        spawn_plan(det),
        ImpactSpawnPlan { count: 8, spread_px: 64.0, ttl_ticks: 5 },
        "detonate fan-out mirrors baby_flame.impact spawn plan"
    );

    let chained = det.on_expire.as_ref().expect("detonate must chain on_expire");
    assert_eq!(chained.0, "baby_burner.flash", "detonate chains baby_burner.flash");
}

#[test]
fn baby_burner_flash_is_static_and_fades() {
    let asset = agumon_vfx();
    let flash = resolve_effect(&asset, "baby_burner.flash").expect("flash present");

    assert_eq!(flash.placement.verb, "agumon/baby_flame/static");
    assert_eq!(
        spawn_plan(flash),
        ImpactSpawnPlan { count: 1, spread_px: 0.0, ttl_ticks: 2 }
    );
    assert_eq!(flash.appearance.size_px, 26.0);
    // No on_expire — flash is the terminal effect.
    assert!(flash.on_expire.is_none());
}

#[test]
fn baby_burner_detonate_curves_match_authored_values() {
    let asset = agumon_vfx();
    let det = resolve_effect(&asset, "baby_burner.detonate").expect("detonate present");

    // Scale: ease-out, mirrors baby_flame.impact exactly.
    assert_eq!(eval_scale(&det.appearance.scale, 0.0), 0.0);
    assert_eq!(eval_scale(&det.appearance.scale, 0.5), 0.75);
    assert_eq!(eval_scale(&det.appearance.scale, 1.0), 1.0);

    // Color: holds hue, alpha linear-fades to transparent.
    assert_eq!(eval_color(&det.appearance.color, 0.0), [2.2, 1.0, 0.3, 0.9]);
    assert_eq!(eval_color(&det.appearance.color, 1.0), [2.2, 1.0, 0.3, 0.0]);
    assert_rgba_approx(
        eval_color(&det.appearance.color, 0.5),
        [2.2, 1.0, 0.3, 0.45],
        "detonate midpoint alpha fade",
    );
}

#[test]
fn baby_burner_flash_curves_match_authored_values() {
    let asset = agumon_vfx();
    let flash = resolve_effect(&asset, "baby_burner.flash").expect("flash present");

    assert_eq!(eval_color(&flash.appearance.color, 0.0), [3.6, 2.2, 0.8, 0.95]);
    assert_eq!(eval_color(&flash.appearance.color, 1.0), [3.6, 2.2, 0.8, 0.0]);
    assert_rgba_approx(
        eval_color(&flash.appearance.color, 0.5),
        [3.6, 2.2, 0.8, 0.475],
        "flash midpoint alpha",
    );
    // Scale holds constant across life.
    assert_eq!(eval_scale(&flash.appearance.scale, 0.5), 1.0);
}

#[test]
fn validate_effects_accepts_the_real_asset() {
    let asset = agumon_vfx();
    assert_eq!(
        validate_effects(&asset, KNOWN_VERBS),
        Ok(()),
        "the authored asset's verbs are all registered and on_expire resolves"
    );
}

#[test]
fn validate_effects_names_an_unregistered_verb() {
    // Synthetic asset whose single effect references a verb not in KNOWN_VERBS.
    let bad: VfxAsset = ron::from_str(
        r#"(
            effects: {
                "x.bad": (
                    placement: (verb: "agumon/baby_flame/nope", params: Static, anchor: Mouth),
                    appearance: (
                        count: 1, spread_px: 0.0, ttl_ticks: 1, size_px: 1.0,
                        texture: "t", scale: [], color: [],
                    ),
                ),
            },
        )"#,
    )
    .expect("synthetic asset parses (the shape is valid; the verb id is not)");

    assert_eq!(
        validate_effects(&bad, KNOWN_VERBS),
        Err(VfxValidationError::UnknownVerb {
            effect_id: "x.bad".to_owned(),
            verb: "agumon/baby_flame/nope".to_owned(),
        }),
        "validation must name the offending effect and unregistered verb"
    );
}

#[test]
fn validate_effects_names_a_dangling_on_expire() {
    let bad: VfxAsset = ron::from_str(
        r#"(
            effects: {
                "x.src": (
                    placement: (verb: "agumon/baby_flame/static", params: Static, anchor: Mouth),
                    appearance: (
                        count: 1, spread_px: 0.0, ttl_ticks: 1, size_px: 1.0,
                        texture: "t", scale: [], color: [],
                    ),
                    on_expire: Some("x.ghost"),
                ),
            },
        )"#,
    )
    .expect("synthetic asset parses; the on_expire target is simply absent");

    assert_eq!(
        validate_effects(&bad, KNOWN_VERBS),
        Err(VfxValidationError::DanglingOnExpire {
            effect_id: "x.src".to_owned(),
            missing: "x.ghost".to_owned(),
        }),
        "validation must name the offending effect and missing on_expire target"
    );
}

