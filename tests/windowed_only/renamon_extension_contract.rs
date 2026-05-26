//! M006/S05 T02 — source-contract for Renamon's extension-only windowed
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

#[test]
fn engine_files_stay_species_agnostic() {
    for src in [RENDER_SRC, WINDOWED_MOD_SRC] {
        for forbidden in [
            "renamon_stance",
            "renamon_skill",
            "diamond_storm",
            "digimon/renamon",
            "digimon/renamon_atlas.png",
        ] {
            assert!(
                !src.contains(forbidden),
                "engine files must not contain {forbidden:?}; Renamon-specific ids/paths belong in src/windowed/digimon/renamon/"
            );
        }
    }
}

#[test]
fn digimon_aggregator_only_declares_and_registers_renamon() {
    assert!(
        DIGIMON_MOD_SRC.contains("mod renamon"),
        "src/windowed/digimon/mod.rs must declare the renamon submodule"
    );
    assert!(
        DIGIMON_MOD_SRC.contains("renamon::register(app);"),
        "src/windowed/digimon/mod.rs must register Renamon through the aggregator seam"
    );
}

#[test]
fn renamon_module_owns_presentation_ids_and_build_time_stance_registration() {
    for required in [
        "AnimationStancePaths",
        "resource_mut::<AnimationStancePaths>()",
        "digimon/renamon/stance.ron",
        "WindowedDemoRegistry",
        "SkillStartNodeRegistry",
        "SpritePresentationRegistry",
        "presentation_id: RENAMON_PRESENTATION_ID.to_string()",
        "unit_ids: vec![RENAMON_UNIT_ID]",
        "UnitId(7)",
        "renamon_stance",
        "renamon_skill",
        "diamond_storm",
        "diamond_storm_cast",
        "digimon/renamon_atlas.png",
        "clip_index: RENAMON_CLIP_INDEX",
    ] {
        assert!(
            RENAMON_SRC.contains(required),
            "src/windowed/digimon/renamon/mod.rs must contain {required:?} so Renamon owns its stance path, registry data, and demo selection"
        );
    }
}

#[test]
fn renamon_module_does_not_invent_fake_particle_or_engine_branches() {
    for forbidden in [
        "EnokiVfxRegistry",
        "OnEnterEffectRegistry",
        "SkillReleaseEffectRegistry",
        "DetonateEffectRegistry",
        "diamond_storm_leaf",
    ] {
        assert!(
            !RENAMON_SRC.contains(forbidden),
            "Renamon module must not contain {forbidden:?}; missing authored VFX mappings should stay a no-op instead of adding fake effect plumbing"
        );
    }
}

#[test]
fn renamon_stance_asset_owns_idle_hurt_death_and_victory_ranges() {
    for required in [
        "id: \"renamon_stance\"",
        "clip: \"all\"",
        "\"idle\"",
        "frames: (35, 42)",
        "Loop(count: 0)",
        "\"hurt\"",
        "frames: (28, 34)",
        "\"death\"",
        "frames: (15, 18)",
        "\"victory\"",
        "frames: (55, 67)",
        "to: Node(\"idle\")",
        "to: Exit",
    ] {
        assert!(
            RENAMON_STANCE.contains(required),
            "assets/digimon/renamon/stance.ron must contain {required:?}"
        );
    }
}

#[test]
fn renamon_clip_asset_exposes_the_all_range() {
    assert!(
        RENAMON_CLIP.contains("\"all\": (start: 0, end: 67)"),
        "assets/digimon/renamon/clip.ron must define the all range covering frames 0-67"
    );
}

#[test]
fn renamon_skill_graph_releases_the_kernel_on_impact() {
    for required in [
        "id: \"renamon_skill\"",
        "clip: \"skill\"",
        "\"diamond_storm_impact\"",
        "cues:",
        "(at: 1, command: ReleaseKernel(()))",
    ] {
        assert!(
            RENAMON_ANIM.contains(required),
            "assets/digimon/renamon/anim_graph.ron must contain {required:?} so Diamond Storm releases the barrier on impact"
        );
    }
}
