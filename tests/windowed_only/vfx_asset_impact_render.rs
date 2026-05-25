//! M004/S01 T04 — windowed integration of the Baby Flame impact / follow-through
//! data path.
//!
//! `src/windowed/render.rs` lives in the binary crate, so its private spawn/tick
//! helpers can't be called here. What this test pins is the *contract render.rs
//! consumes*: the authored `assets/digimon/agumon/vfx.ron` resolves the central
//! impact burst and its chained fan-out follow-through, [`spawn_plan`] yields the
//! count/spread/ttl values the dispatcher reads, and [`eval_scale`]/[`eval_color`]
//! produce the per-tick outward distance + rgba the shard branch writes onto each
//! particle. It runs under the `windowed` feature (matching the render path's
//! gate) without opening a window.
//!
//! Compile-time `include_str!` of a git-tracked asset — never a gitignored path.
#![cfg(feature = "windowed")]

use bevyrogue::animation::{
    eval_color, eval_scale, resolve_effect, spawn_plan, ImpactSpawnPlan, PlacementCtx,
    PlacementParams, VfxAsset,
};
use bevyrogue::combat::blueprints::agumon::register_agumon_ext;
use bevyrogue::combat::runtime::ExtRegistries;

/// The effect ids render.rs resolves for Baby Flame's impact chain.
const IMPACT_EFFECT_ID: &str = "baby_flame.impact";
const IMPACT_FLASH_EFFECT_ID: &str = "baby_flame.impact_flash";
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
fn impact_effect_resolves_to_the_central_burst_the_dispatcher_consumes() {
    let asset = agumon_vfx();
    let impact = resolve_effect(&asset, IMPACT_EFFECT_ID)
        .expect("render.rs resolves baby_flame.impact from the data path");

    assert_eq!(
        spawn_plan(impact),
        ImpactSpawnPlan {
            count: 1,
            spread_px: 0.0,
            ttl_ticks: 4,
        },
        "data path must reproduce the authored central-burst impact contract"
    );

    // A missing id resolves to None so render.rs logs + falls back rather than panicking.
    assert!(
        resolve_effect(&asset, "missing.effect").is_none(),
        "absent effect id must resolve to None for the windowed fallback path"
    );
}

#[test]
fn impact_flash_shard_offset_curve_matches_what_the_per_tick_branch_writes() {
    let asset = agumon_vfx();
    let impact_flash = resolve_effect(&asset, IMPACT_FLASH_EFFECT_ID).expect("impact_flash present");
    let plan = spawn_plan(impact_flash);
    let scale = &impact_flash.appearance.scale;

    // age 0 -> progress 0.0 -> no outward travel.
    let frac0 = eval_scale(scale, 0.0);
    assert_eq!(frac0, 0.0);
    let off0 = data_shard_offset(&plan, frac0, 0.0);
    assert!(off0[0].abs() < EPS && off0[1].abs() < EPS, "spawn sits on the impact origin");

    // Eased midpoint fraction 0.75 along the +x fan direction (phase 0).
    let frac_mid = eval_scale(scale, 0.5);
    assert_eq!(frac_mid, 0.75);
    let off_mid = data_shard_offset(&plan, frac_mid, 0.0);
    assert!((off_mid[0] - 54.0).abs() < EPS, "0.75 * 72.0 spread = 54px outward");
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
fn impact_flash_shard_color_curve_matches_what_the_per_tick_branch_writes() {
    let asset = agumon_vfx();
    let impact_flash = resolve_effect(&asset, IMPACT_FLASH_EFFECT_ID).expect("impact_flash present");
    let color = &impact_flash.appearance.color;

    // Constant overbright ember hue (>1.0 RGB for HDR bloom), alpha linear-fades to transparent.
    assert_eq!(eval_color(color, 0.0), [2.0, 1.4, 0.9, 0.95], "spawn rgba");
    let mid = eval_color(color, 0.5);
    for (i, (a, e)) in mid.iter().zip([2.0, 1.4, 0.9, 0.475].iter()).enumerate() {
        assert!((a - e).abs() < EPS, "midpoint channel {i}: expected ~{e}, got {a}");
    }
    assert_eq!(eval_color(color, 1.0), [2.0, 1.4, 0.9, 0.0], "death rgba");
}

/// The generalized dispatcher (`advance_vfx_particles`) resolves every effect's
/// placement verb against a built `ExtRegistries`. Pin that all four authored
/// verb ids resolve so a missing registration is caught here rather than via a
/// runtime warn-and-skip.
#[test]
fn built_registry_resolves_all_authored_placement_verbs() {
    let mut regs = ExtRegistries::default();
    register_agumon_ext(&mut regs);
    for verb in [
        "agumon/baby_flame/static",
        "agumon/baby_flame/converge_inward",
        "agumon/baby_flame/fan_out",
        "agumon/baby_flame/arc_launch",
    ] {
        assert!(
            regs.placements.get(verb).is_some(),
            "dispatcher must resolve placement verb `{verb}`"
        );
    }
    assert!(
        regs.placements.get("agumon/baby_flame/missing").is_none(),
        "an unregistered verb id resolves to None so the dispatcher warns + skips"
    );
}

/// Every effect the dispatcher spawns must resolve from the loaded asset, and
/// each one's placement verb must be registered — the exact pairing the per-tick
/// loop performs (`resolve_effect` -> `regs.placements.get`).
#[test]
fn every_effect_resolves_and_its_verb_is_registered() {
    let asset = agumon_vfx();
    let mut regs = ExtRegistries::default();
    register_agumon_ext(&mut regs);
    for id in [
        "baby_flame.charge",
        "baby_flame.ember",
        "baby_flame.projectile",
        "baby_flame.impact",
        "baby_flame.impact_flash",
        "baby_burner.detonate",
    ] {
        let effect = resolve_effect(&asset, id).unwrap_or_else(|| panic!("effect `{id}` present"));
        assert!(
            regs.placements.get(&effect.placement.verb).is_some(),
            "effect `{id}` names registered verb `{}`",
            effect.placement.verb
        );
    }
}

/// Feed the resolved `converge_inward` / `arc_launch` verbs the same
/// `PlacementCtx` the dispatcher builds and pin their anchored offsets.
#[test]
fn resolved_verbs_yield_the_expected_anchored_offsets() {
    let asset = agumon_vfx();
    let mut regs = ExtRegistries::default();
    register_agumon_ext(&mut regs);

    let ember = resolve_effect(&asset, "baby_flame.ember").expect("ember present");
    let converge = regs
        .placements
        .get(&ember.placement.verb)
        .expect("converge_inward registered");
    // At progress 0 the swirl sits on the full radius (58px) along phase 0 (+x).
    let ctx0 = PlacementCtx {
        age_ticks: 0,
        ttl_ticks: 24,
        progress: 0.0,
        phase: 0.0,
        caster_xy: [0.0, 0.0],
        target_xy: [0.0, 0.0],
    };
    let off0 = converge(&ctx0, &ember.placement.params);
    assert!((off0[0] - 58.0).abs() < 1e-4 && off0[1].abs() < 1e-4, "ember starts on the rim");
    // At end of life it has collapsed onto the anchor.
    let ctx1 = PlacementCtx { progress: 1.0, ..ctx0 };
    let off1 = converge(&ctx1, &ember.placement.params);
    assert!(off1[0].abs() < 1e-4 && off1[1].abs() < 1e-4, "ember merges into the mouth");

    let projectile = resolve_effect(&asset, "baby_flame.projectile").expect("projectile present");
    let arc = regs
        .placements
        .get(&projectile.placement.verb)
        .expect("arc_launch registered");
    // arc_launch lerps caster->target by progress; halfway is the midpoint.
    let ctx_mid = PlacementCtx {
        age_ticks: 2,
        ttl_ticks: 5,
        progress: 0.5,
        phase: 0.0,
        caster_xy: [0.0, 0.0],
        target_xy: [100.0, 40.0],
    };
    let mid = arc(&ctx_mid, &projectile.placement.params);
    assert!((mid[0] - 50.0).abs() < 1e-4 && (mid[1] - 20.0).abs() < 1e-4);
}

/// The hardcoded projectile->impact burst is now data: the projectile's
/// `on_expire` chains the central impact burst, which then chains the radiating
/// fan-out follow-through.
#[test]
fn projectile_on_expire_chains_the_impact_then_flash_fan() {
    let asset = agumon_vfx();
    let projectile = resolve_effect(&asset, "baby_flame.projectile").expect("projectile present");
    let chained = projectile
        .on_expire
        .as_ref()
        .expect("projectile chains an on_expire effect");
    assert_eq!(chained.0, IMPACT_EFFECT_ID);

    let impact = resolve_effect(&asset, &chained.0).expect("chained impact present");
    assert!(matches!(impact.placement.params, PlacementParams::Static));

    let impact_flash = resolve_effect(&asset, IMPACT_FLASH_EFFECT_ID).expect("impact_flash present");
    assert!(matches!(impact_flash.placement.params, PlacementParams::FanOut { .. }));
    assert_eq!(
        impact.on_expire.as_ref().map(|next| next.0.as_str()),
        Some(IMPACT_FLASH_EFFECT_ID),
        "central impact must chain the radiating follow-through"
    );
}
