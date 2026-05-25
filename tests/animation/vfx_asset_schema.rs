//! M004/S01 T01 — editor-ready `VfxAsset` schema (D033/D034).
//!
//! Asserts: (a) the typed schema round-trips through RON, (b) `deny_unknown_fields`
//! rejects unknown fields, and (c) the schema is reflectable (D034 editor-readiness).

use bevyrogue::animation::{Appearance, VfxAsset};

const SAMPLE: &str = r#"(
    effects: {
        "baby_flame.impact": (
            placement: (verb: "impact.fan_out"),
            appearance: (
                count: 8,
                spread_px: 24.0,
                ttl_ticks: 30,
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
fn unknown_field_is_rejected() {
    // Same shape as SAMPLE but with an unknown field inside an Appearance.
    let bad = r#"(
        effects: {
            "x": (
                placement: (verb: "v"),
                appearance: (
                    count: 1,
                    spread_px: 0.0,
                    ttl_ticks: 1,
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
fn appearance_is_reflectable_with_expected_fields() {
    use bevy::reflect::Struct;

    let appearance = Appearance {
        count: 1,
        spread_px: 0.0,
        ttl_ticks: 1,
        scale: bevyrogue::animation::ScaleCurve(vec![]),
        color: bevyrogue::animation::ColorCurve(vec![]),
    };

    let field_names: Vec<&str> = (0..appearance.field_len())
        .map(|i| appearance.name_at(i).expect("field name at index"))
        .collect();
    assert_eq!(
        field_names,
        vec!["count", "spread_px", "ttl_ticks", "scale", "color"],
        "Reflect must expose Appearance's typed fields for the future GUI editor"
    );
}
