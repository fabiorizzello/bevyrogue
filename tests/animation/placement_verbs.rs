//! M004/S02 T01 — PlacementExt axis + pure placement verbs (R004/R016).
//!
//! Proves the open-dispatch + typed-params seam end-to-end in the headless lib:
//! every verb resolves through a freshly-built `ExtRegistries` (no `App`), each
//! is bit-identical across 1000 calls (determinism), and the closed-form
//! endpoints match the ported render.rs math.

use bevyrogue::animation::placement::{
    arc_launch, converge_inward, fan_out, static_placement,
};
use bevyrogue::animation::vfx_asset::{PlacementCtx, PlacementParams};
use bevyrogue::combat::blueprints::agumon::register_agumon_ext;
use bevyrogue::combat::runtime::ExtRegistries;

fn ctx(progress: f32) -> PlacementCtx {
    PlacementCtx {
        age_ticks: 7,
        ttl_ticks: 20,
        progress,
        phase: 0.0,
        caster_xy: [10.0, 20.0],
        target_xy: [110.0, 220.0],
    }
}

#[test]
fn all_four_verbs_resolve_via_freshly_built_registries() {
    let mut regs = ExtRegistries::default();
    register_agumon_ext(&mut regs);

    for id in [
        "agumon/baby_flame/converge_inward",
        "agumon/baby_flame/fan_out",
        "agumon/baby_flame/arc_launch",
        "agumon/baby_flame/static",
    ] {
        assert!(
            regs.placements.get(id).is_some(),
            "verb `{id}` must resolve after register_agumon_ext"
        );
    }
}

#[test]
fn default_registries_have_no_placements() {
    assert!(ExtRegistries::default().placements.is_empty());
}

type PlacementVerb = fn(&PlacementCtx, &PlacementParams) -> [f32; 2];

#[test]
fn verbs_are_bit_identical_across_1000_calls() {
    let c = ctx(0.37);
    let cases: [(PlacementVerb, PlacementParams); 4] = [
        (
            converge_inward,
            PlacementParams::ConvergeInward { radius_px: 58.0, omega: 0.9 },
        ),
        (fan_out, PlacementParams::FanOut { spread_px: 64.0 }),
        (arc_launch, PlacementParams::ArcLaunch {}),
        (static_placement, PlacementParams::Static),
    ];
    for (verb, params) in &cases {
        let first = verb(&c, params);
        for _ in 0..1000 {
            assert_eq!(verb(&c, params), first, "verb output must be deterministic");
        }
    }
}

#[test]
fn converge_inward_radius_collapses_from_radius_px_to_origin() {
    let params = PlacementParams::ConvergeInward { radius_px: 58.0, omega: 0.9 };

    // progress 0 with phase 0 and age 0 -> full radius along +x.
    let start = converge_inward(
        &PlacementCtx { age_ticks: 0, progress: 0.0, ..ctx(0.0) },
        &params,
    );
    let start_radius = (start[0] * start[0] + start[1] * start[1]).sqrt();
    assert!((start_radius - 58.0).abs() < 1e-4, "radius at progress 0 == radius_px");

    // progress 1 -> collapsed to the anchor.
    let end = converge_inward(&ctx(1.0), &params);
    assert!(
        end[0].abs() < 1e-6 && end[1].abs() < 1e-6,
        "radius collapses to origin at progress 1, got {end:?}"
    );
}

#[test]
fn arc_launch_endpoints_match_target_minus_caster() {
    let params = PlacementParams::ArcLaunch {};
    assert_eq!(arc_launch(&ctx(0.0), &params), [0.0, 0.0]);
    // target_xy - caster_xy = [100, 200].
    assert_eq!(arc_launch(&ctx(1.0), &params), [100.0, 200.0]);
}

#[test]
fn fan_out_reaches_spread_px_along_phase_at_progress_1() {
    let params = PlacementParams::FanOut { spread_px: 64.0 };
    // phase 0 -> full spread along +x.
    let end = fan_out(&PlacementCtx { phase: 0.0, ..ctx(1.0) }, &params);
    assert!((end[0] - 64.0).abs() < 1e-4, "x reaches spread_px");
    assert!(end[1].abs() < 1e-6, "y stays zero along phase 0");
}

#[test]
fn verbs_return_origin_for_mismatched_params() {
    let c = ctx(0.5);
    assert_eq!(converge_inward(&c, &PlacementParams::Static), [0.0, 0.0]);
    assert_eq!(fan_out(&c, &PlacementParams::ArcLaunch {}), [0.0, 0.0]);
    assert_eq!(
        arc_launch(&c, &PlacementParams::FanOut { spread_px: 1.0 }),
        [0.0, 0.0]
    );
}
