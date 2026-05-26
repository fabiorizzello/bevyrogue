//! M006/S05 T03 — source-contract for Renamon's extension-only windowed
//! presentation seam.
//!
//! The windowed binary crate is unreachable from `tests/`, so this contract pins
//! the authored source and assets directly. It proves Renamon is added as an
//! extension module + assets rather than by species branches in engine files.
#![cfg(feature = "windowed")]

const RENDER_SRC: &str = include_str!("../../src/windowed/render.rs");
const WINDOWED_MOD_SRC: &str = include_str!("../../src/windowed/mod.rs");
const DIGIMON_MOD_SRC: &str = include_str!("../../src/windowed/digimon/mod.rs");
const RENAMON_SRC: &str = include_str!("../../src/windowed/digimon/renamon/mod.rs");
const RENAMON_STANCE: &str = include_str!("../../assets/digimon/renamon/stance.ron");
const RENAMON_CLIP: &str = include_str!("../../assets/digimon/renamon/clip.ron");
const RENAMON_ANIM: &str = include_str!("../../assets/digimon/renamon/anim_graph.ron");

fn assert_contains_all(haystack: &str, label: &str, required: &[&str]) {
    for token in required {
        assert!(
            haystack.contains(token),
            "{label} must contain {token:?}"
        );
    }
}

fn assert_contains_none(haystack: &str, label: &str, forbidden: &[&str]) {
    for token in forbidden {
        assert!(
            !haystack.contains(token),
            "{label} must not contain {token:?}"
        );
    }
}

#[test]
fn engine_files_stay_species_agnostic() {
    for (label, src) in [
        ("src/windowed/render.rs", RENDER_SRC),
        ("src/windowed/mod.rs", WINDOWED_MOD_SRC),
    ] {
        assert_contains_none(
            src,
            label,
            &[
                "renamon",
                "RENAMON_",
                "diamond_storm",
                "digimon/renamon",
                "digimon/renamon_atlas.png",
            ],
        );
    }
}

#[test]
fn render_keeps_the_multi_presentation_lookup_seam() {
    assert_contains_all(
        RENDER_SRC,
        "src/windowed/render.rs",
        &[
            "struct SpritePresentationRegistry",
            "entries: Vec<SpritePresentationEntry>",
            "fn presentation_entry_for_unit(",
            ".find(|entry| entry.matches_unit(unit_id))",
            "presentation_entry_for_unit(&presentation, unit.id)",
            "entry.presentation_id.as_str()",
            "entry.atlas_image_path.as_str()",
        ],
    );
    assert_contains_none(
        RENDER_SRC,
        "src/windowed/render.rs",
        &["presentation.entries.first()", "presentation.entries[0]"],
    );
}

#[test]
fn digimon_aggregator_only_declares_and_registers_renamon() {
    assert_contains_all(
        DIGIMON_MOD_SRC,
        "src/windowed/digimon/mod.rs",
        &["mod renamon", "renamon::register(app);", "fn register_all(app: &mut App)"],
    );
}

#[test]
fn renamon_module_owns_the_extension_data_and_registration() {
    assert_contains_all(
        RENAMON_SRC,
        "src/windowed/digimon/renamon/mod.rs",
        &[
            "AnimationStancePaths",
            "resource_mut::<AnimationStancePaths>()",
            "WindowedDemoRegistry",
            "SkillStartNodeRegistry",
            "SpritePresentationRegistry",
            "presentation_id: RENAMON_PRESENTATION_ID.to_string()",
            "unit_ids: vec![RENAMON_UNIT_ID]",
            "digimon/renamon/stance.ron",
            "digimon/renamon_atlas.png",
            "renamon_stance",
            "renamon_skill",
            "diamond_storm",
            "diamond_storm_cast",
        ],
    );
}

#[test]
fn renamon_module_does_not_invent_fake_particle_or_engine_branches() {
    assert_contains_none(
        RENAMON_SRC,
        "src/windowed/digimon/renamon/mod.rs",
        &[
            "EnokiVfxRegistry",
            "OnEnterEffectRegistry",
            "SkillReleaseEffectRegistry",
            "DetonateEffectRegistry",
        ],
    );
}

#[test]
fn renamon_stance_asset_defines_the_expected_stance_contract() {
    assert_contains_all(
        RENAMON_STANCE,
        "assets/digimon/renamon/stance.ron",
        &[
            "id: \"renamon_stance\"",
            "clip: \"all\"",
            "entry: \"idle\"",
            "\"idle\"",
            "Loop(count: 0)",
            "\"hurt\"",
            "\"death\"",
            "\"victory\"",
            "to: Node(\"idle\")",
            "to: Exit",
        ],
    );
}

#[test]
fn renamon_clip_asset_exposes_the_all_range() {
    assert_contains_all(
        RENAMON_CLIP,
        "assets/digimon/renamon/clip.ron",
        &["ranges:", "\"all\":", "start:", "end:"],
    );
}

#[test]
fn renamon_skill_graph_releases_the_kernel_on_impact() {
    assert_contains_all(
        RENAMON_ANIM,
        "assets/digimon/renamon/anim_graph.ron",
        &[
            "id: \"renamon_skill\"",
            "clip: \"skill\"",
            "\"diamond_storm_cast\"",
            "\"diamond_storm_impact\"",
            "\"diamond_storm_recover\"",
            "cues:",
            "ReleaseKernel(())",
        ],
    );
}
