//! M004/S02 T02 — editor-ready `VfxAsset` schema (D033/D034/D035).
//!
//! Asserts: (a) the typed schema (now with typed placement params + anchor and
//! per-particle size/texture) round-trips through RON, (b) `deny_unknown_fields`
//! rejects unknown fields, and (c) the schema is reflectable (D034
//! editor-readiness) — including the new `Placement.params`/`anchor` and
//! `Appearance.size_px`/`texture` fields.

use bevyrogue::animation::{
    Appearance, ColorCurve, Placement, PlacementAnchor, PlacementParams, ScaleCurve, VfxAsset,
};

const SAMPLE: &str = r#"(
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
            on_expire: Some("baby_flame.embers"),
        ),
    },
)"#;

#[test]
fn vfx_asset_round_trips_through_ron() {
    let parsed: VfxAsset = ron::from_str(SAMPLE).expect("sample VfxAsset should deserialize");
    let serialized = ron::to_string(&parsed).expect("VfxAsset should serialize");
    let reparsed: VfxAsset =
        ron::from_str(&serialized).expect("serialized VfxAsset should re-deserialize");
    assert_eq!(parsed, reparsed, "VfxAsset must round-trip losslessly");
}

#[test]
fn all_authored_effects_round_trip() {
    let asset: VfxAsset =
        ron::from_str(include_str!("../../assets/digimon/agumon/vfx.ron")).expect("asset parses");
    let serialized = ron::to_string(&asset).expect("asset serializes");
    let reparsed: VfxAsset = ron::from_str(&serialized).expect("asset re-deserializes");
    assert_eq!(asset, reparsed, "all effects must round-trip losslessly");
    // Five Baby Flame effects (T02) plus the Baby Burner detonate flash routed
    // through the same owned path (T03) so it renders without a hardcoded kind.
    assert_eq!(asset.effects.len(), 6, "asset ships the five Baby Flame effects + detonate");
}

#[test]
fn unknown_field_is_rejected() {
    // Same shape as SAMPLE but with an unknown field inside an Appearance.
    let bad = r#"(
        effects: {
            "x": (
                placement: (verb: "v", params: Static, anchor: Mouth),
                appearance: (
                    count: 1,
                    spread_px: 0.0,
                    ttl_ticks: 1,
                    size_px: 1.0,
                    texture: "t",
                    scale: [],
                    color: [],
                    bogus_field: 3,
                ),
            ),
        },
    )"#;
    assert!(
        ron::from_str::<VfxAsset>(bad).is_err(),
        "deny_unknown_fields must reject the unknown `bogus_field`"
    );
}

#[test]
fn unknown_placement_field_is_rejected() {
    let bad = r#"(
        effects: {
            "x": (
                placement: (verb: "v", params: Static, anchor: Mouth, bogus: 1),
                appearance: (
                    count: 1, spread_px: 0.0, ttl_ticks: 1, size_px: 1.0,
                    texture: "t", scale: [], color: [],
                ),
            ),
        },
    )"#;
    assert!(
        ron::from_str::<VfxAsset>(bad).is_err(),
        "deny_unknown_fields must reject an unknown Placement field"
    );
}

#[test]
fn appearance_is_reflectable_with_expected_fields() {
    use bevy::reflect::Struct;

    let appearance = Appearance {
        count: 1,
        spread_px: 0.0,
        ttl_ticks: 1,
        scale: ScaleCurve(vec![]),
        color: ColorCurve(vec![]),
        size_px: 1.0,
        texture: "t".to_owned(),
    };

    let field_names: Vec<&str> = (0..appearance.field_len())
        .map(|i| appearance.name_at(i).expect("field name at index"))
        .collect();
    assert_eq!(
        field_names,
        vec!["count", "spread_px", "ttl_ticks", "scale", "color", "size_px", "texture"],
        "Reflect must expose Appearance's typed fields (incl. new size_px/texture) for the GUI editor"
    );
}

#[test]
fn placement_is_reflectable_with_typed_params_and_anchor() {
    use bevy::reflect::Struct;

    let placement = Placement {
        verb: "agumon/baby_flame/fan_out".to_owned(),
        params: PlacementParams::FanOut { spread_px: 64.0 },
        anchor: PlacementAnchor::TargetCenter,
    };

    let field_names: Vec<&str> = (0..placement.field_len())
        .map(|i| placement.name_at(i).expect("field name at index"))
        .collect();
    assert_eq!(
        field_names,
        vec!["verb", "params", "anchor"],
        "Reflect must expose Placement's verb + typed params + anchor for the GUI editor"
    );
}
