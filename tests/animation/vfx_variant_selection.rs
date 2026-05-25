//! M004/S03 T01 — the deterministic variant-selection seam (D033 graft 5).
//!
//! Proves that a synthetic [`VfxContext`] (a stand-in for a future skill-tree
//! unlock) maps deterministically to a selected effect-tree variant over the
//! owned Agumon `vfx.ron`, reusing the same `resolve_effect`/`validate_effects`
//! path established in S01/S02. No new ExtRegistries axis, no RNG, no clock
//! (R004): the selection is a pure function of closed data (D035).
//!
//! Compile-time `include_str!` of the git-tracked asset (matching
//! `vfx_asset_load.rs`) — never reads a gitignored `.gsd/`/`.planning/` path.

use bevyrogue::animation::{
    resolve_effect, select_variant, validate_effects, VfxAsset, VfxContext, VfxValidationError,
};

/// The placement verb ids the windowed `PlacementExt` Registry registers (S02).
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

fn ctx(skill_id: &str, variant_key: &str) -> VfxContext {
    VfxContext { skill_id: skill_id.to_owned(), variant_key: variant_key.to_owned() }
}

#[test]
fn select_variant_maps_context_to_expected_effect() {
    let asset = agumon_vfx();

    // "base" -> baby_burner.detonate; the selected def must be that exact effect.
    let base = select_variant(&asset, &ctx("baby_burner", "base"))
        .expect("base variant must select an effect");
    assert!(
        std::ptr::eq(base, resolve_effect(&asset, "baby_burner.detonate").unwrap()),
        "base variant must select the baby_burner.detonate EffectDef"
    );

    // "empowered" -> baby_flame.impact.
    let empowered = select_variant(&asset, &ctx("baby_burner", "empowered"))
        .expect("empowered variant must select an effect");
    assert!(
        std::ptr::eq(empowered, resolve_effect(&asset, "baby_flame.impact").unwrap()),
        "empowered variant must select the baby_flame.impact EffectDef"
    );
}

#[test]
fn select_variant_is_deterministic_across_repeated_calls() {
    let asset = agumon_vfx();
    let context = ctx("baby_burner", "empowered");

    let first = select_variant(&asset, &context).expect("selection present");
    // Identical context must yield a byte-identical selected EffectDef every time
    // (R004): no RNG, no clock, no iteration-order nondeterminism.
    for _ in 0..1000 {
        let again = select_variant(&asset, &context).expect("selection present");
        assert_eq!(first, again, "variant selection must be deterministic");
    }
}

#[test]
fn select_variant_returns_none_for_unmapped_keys() {
    let asset = agumon_vfx();
    // Unknown skill_id and unknown variant_key both fall through to None so the
    // caller falls back to the base effect (None-not-panic discipline).
    assert!(select_variant(&asset, &ctx("nonexistent_skill", "base")).is_none());
    assert!(select_variant(&asset, &ctx("baby_burner", "nonexistent_key")).is_none());
}

#[test]
fn validate_effects_accepts_the_real_asset_with_variants() {
    let asset = agumon_vfx();
    assert_eq!(
        validate_effects(&asset, KNOWN_VERBS),
        Ok(()),
        "the authored asset's variant targets all resolve in `effects`"
    );
}

#[test]
fn validate_effects_names_a_dangling_variant_target() {
    // Q7 negative test: a synthetic asset whose variant target is absent from
    // `effects` must surface DanglingVariant naming skill_id/variant_key/missing.
    let bad: VfxAsset = ron::from_str(
        r#"(
            effects: {
                "x.ok": (
                    placement: (verb: "agumon/baby_flame/static", params: Static, anchor: Mouth),
                    appearance: (
                        count: 1, spread_px: 0.0, ttl_ticks: 1, size_px: 1.0,
                        texture: "t", scale: [], color: [],
                    ),
                ),
            },
            variants: {
                "skill_a": {
                    "v1": "x.ghost",
                },
            },
        )"#,
    )
    .expect("synthetic asset parses; the variant target is simply absent");

    assert_eq!(
        validate_effects(&bad, KNOWN_VERBS),
        Err(VfxValidationError::DanglingVariant {
            skill_id: "skill_a".to_owned(),
            variant_key: "v1".to_owned(),
            missing: "x.ghost".to_owned(),
        }),
        "validation must name the offending skill_id, variant_key, and missing target"
    );
}
