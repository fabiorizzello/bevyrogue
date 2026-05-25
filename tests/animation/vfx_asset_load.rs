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
    PlacementAnchor, VfxAsset, VfxValidationError,
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
    "agumon/baby_flame/turbulence",
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
fn agumon_spawn_plans_match_authored_atom_values() {
    // S06: the five Baby Flame effects were re-authored onto the composable atom
    // set (flame_orb/spark/comet/burst/streak) with rotation. These pin the new
    // authored spawn plans, verbs, sizes, and texture keys.
    let asset = agumon_vfx();

    let charge = resolve_effect(&asset, "baby_flame.charge").expect("charge present");
    assert_eq!(
        spawn_plan(charge),
        ImpactSpawnPlan { count: 1, spread_px: 0.0, ttl_ticks: 24 }
    );
    assert_eq!(charge.placement.verb, "agumon/baby_flame/static");
    assert_eq!(charge.appearance.size_px, 34.0);
    assert_eq!(charge.appearance.texture, "flame_orb");

    let ember = resolve_effect(&asset, "baby_flame.ember").expect("ember present");
    assert_eq!(
        spawn_plan(ember),
        ImpactSpawnPlan { count: 9, spread_px: 0.0, ttl_ticks: 24 }
    );
    assert_eq!(ember.placement.verb, "agumon/baby_flame/converge_inward");
    assert_eq!(ember.appearance.size_px, 12.0);
    assert_eq!(ember.appearance.texture, "flame_spark");

    let projectile = resolve_effect(&asset, "baby_flame.projectile").expect("projectile present");
    assert_eq!(
        spawn_plan(projectile),
        ImpactSpawnPlan { count: 1, spread_px: 0.0, ttl_ticks: 5 }
    );
    assert_eq!(projectile.placement.verb, "agumon/baby_flame/arc_launch");
    assert_eq!(projectile.appearance.size_px, 26.0);
    assert_eq!(projectile.appearance.texture, "flame_comet");

    // Impact is now the bright central burst (static, single particle).
    let impact = resolve_effect(&asset, "baby_flame.impact").expect("impact present");
    assert_eq!(
        spawn_plan(impact),
        ImpactSpawnPlan { count: 1, spread_px: 0.0, ttl_ticks: 4 }
    );
    assert_eq!(impact.placement.verb, "agumon/baby_flame/static");
    assert_eq!(impact.appearance.size_px, 56.0);
    assert_eq!(impact.appearance.texture, "flame_burst");

    // impact_flash is now the radiating streak fan-out follow-through.
    let shards = resolve_effect(&asset, "baby_flame.impact_flash").expect("impact_flash present");
    assert_eq!(
        spawn_plan(shards),
        ImpactSpawnPlan { count: 10, spread_px: 72.0, ttl_ticks: 7 }
    );
    assert_eq!(shards.placement.verb, "agumon/baby_flame/fan_out");
    assert_eq!(shards.appearance.texture, "flame_streak");
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

    // Growth maxes by 0.25 life, then holds.
    assert_eq!(eval_scale(&charge.appearance.scale, 0.0), 0.4);
    assert_eq!(eval_scale(&charge.appearance.scale, 0.25), 1.0);
    assert_eq!(eval_scale(&charge.appearance.scale, 1.0), 1.0);

    // Near-neutral warm HDR multiplier (>1.0 channels) so the flame_orb atom's own
    // hue shows through and blooms; alpha climbs 0.4 -> 0.95.
    assert_eq!(eval_color(&charge.appearance.color, 0.0), [1.5, 1.3, 1.0, 0.4]);
    assert_eq!(eval_color(&charge.appearance.color, 1.0), [1.9, 1.6, 1.2, 0.95]);
}

#[test]
fn agumon_ember_color_fades_from_bright() {
    let asset = agumon_vfx();
    let ember = resolve_effect(&asset, "baby_flame.ember").expect("ember present");
    let color = &ember.appearance.color;
    assert_eq!(eval_color(color, 0.0), [2.0, 1.6, 1.0, 0.95], "ember spawns bright");
    assert_eq!(eval_color(color, 1.0), [2.0, 1.6, 1.0, 0.0], "ember fades fully");
    assert_rgba_approx(eval_color(color, 0.5), [2.0, 1.6, 1.0, 0.475], "ember midpoint fade");
}

#[test]
fn agumon_impact_burst_scale_pops_then_settles() {
    let asset = agumon_vfx();
    let impact = resolve_effect(&asset, "baby_flame.impact").expect("impact present");
    let scale = &impact.appearance.scale;

    // Central burst pops past full size at 0.4 life, then settles back.
    assert_eq!(eval_scale(scale, 0.0), 0.5, "spawn: half size");
    assert_eq!(eval_scale(scale, 0.4), 1.15, "0.4 life: overshoot pop");
    assert_eq!(eval_scale(scale, 1.0), 1.0, "death: settled");

    // The eased outward spread now lives on the impact_flash shard fan.
    let shards = resolve_effect(&asset, "baby_flame.impact_flash").expect("shards present");
    let fan = &shards.appearance.scale;
    assert_eq!(eval_scale(fan, 0.0), 0.0, "shards start at center");
    assert_eq!(eval_scale(fan, 0.5), 0.75, "shards eased outward");
    assert_eq!(eval_scale(fan, 1.0), 1.0, "shards reach full spread");
}

#[test]
fn agumon_impact_color_curve_holds_hue_and_fades_alpha() {
    let asset = agumon_vfx();
    let impact = resolve_effect(&asset, "baby_flame.impact").expect("impact present");
    let color = &impact.appearance.color;

    // Bright neutral-warm burst multiplier, alpha fades to transparent.
    assert_eq!(eval_color(color, 0.0), [2.4, 2.0, 1.4, 0.98], "spawn: near-opaque");
    assert_eq!(eval_color(color, 1.0), [2.4, 2.0, 1.4, 0.0], "death: alpha 0.0");
    assert_rgba_approx(eval_color(color, 0.5), [2.4, 2.0, 1.4, 0.49], "midpoint fade");
}

#[test]
fn agumon_impact_flash_shards_fade_while_flying_out() {
    let asset = agumon_vfx();
    let shards = resolve_effect(&asset, "baby_flame.impact_flash").expect("shards present");
    let color = &shards.appearance.color;

    assert_eq!(eval_color(color, 0.0), [2.0, 1.4, 0.9, 0.95], "shards spawn bright");
    assert_eq!(eval_color(color, 1.0), [2.0, 1.4, 0.9, 0.0], "shards fade out");
    assert_rgba_approx(eval_color(color, 0.5), [2.0, 1.4, 0.9, 0.475], "shards midpoint");
    // The streak fan eases outward (scale curve drives fan_out distance), so the
    // midpoint scale is the eased 0.75, not a held constant.
    assert_eq!(eval_scale(&shards.appearance.scale, 0.5), 0.75);
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
fn agumon_vfx_contains_sharp_claws_slash() {
    let asset = agumon_vfx();
    let slash = resolve_effect(&asset, "sharp_claws.slash")
        .expect("authored asset must contain effect `sharp_claws.slash` (S05 data-driven path)");

    // Reuses an already-registered placement verb — no new verb / register change.
    assert_eq!(
        slash.placement.verb, "agumon/baby_flame/static",
        "Sharp Claws reuses the registered `static` verb (D037: no new placement verb)"
    );
    assert!(
        KNOWN_VERBS.contains(&slash.placement.verb.as_str()),
        "Sharp Claws verb `{}` must be one the windowed PlacementExt registry knows",
        slash.placement.verb
    );

    // Bounded, short-lived single-particle slash (windowed has no per-particle rotation,
    // so the claw orientation is baked into the texture).
    let plan = spawn_plan(slash);
    assert_eq!(
        plan,
        ImpactSpawnPlan { count: 1, spread_px: 0.0, ttl_ticks: 6 },
        "Sharp Claws is a single bounded-TTL streak"
    );
    assert_eq!(plan.count, 1, "exactly one slash particle");
    assert!(plan.ttl_ticks > 0 && plan.ttl_ticks <= 12, "TTL is bounded and short");
    assert_eq!(slash.placement.anchor, PlacementAnchor::TargetCenter);
    assert_eq!(slash.appearance.size_px, 34.0);
    assert_eq!(
        slash.appearance.texture, "sharp_claws_slash",
        "texture key must match the windowed `vfx_texture_handle` / asset_server path"
    );
    // Terminal effect: a slash does not chain another effect on expire.
    assert!(slash.on_expire.is_none(), "Sharp Claws slash is a terminal effect");
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

