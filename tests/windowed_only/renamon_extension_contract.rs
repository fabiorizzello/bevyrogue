//! M006/S05 T01 — source-contract for the species-agnostic multi-presentation and
//! windowed-demo composition seam.
//!
//! The windowed engine files are binary-crate code unreachable from `tests/`, so
//! this contract reads the authored source directly and pins the extension-first
//! boundary before Renamon is added.
#![cfg(feature = "windowed")]

const RENDER_SRC: &str = include_str!("../../src/windowed/render.rs");
const WINDOWED_MOD_SRC: &str = include_str!("../../src/windowed/mod.rs");
const DEMO_SRC: &str = include_str!("../../src/windowed/demo.rs");
const AGUMON_SRC: &str = include_str!("../../src/windowed/digimon/agumon/mod.rs");

#[test]
fn render_engine_uses_generic_multi_presentation_tokens() {
    for forbidden in [
        "presentation.entries.first()",
        "AgumonAtlas",
        "advance_agumon_presentation",
        "Renamon",
    ] {
        assert!(
            !RENDER_SRC.contains(forbidden),
            "src/windowed/render.rs must not contain {forbidden:?}; the engine seam should stay species-agnostic"
        );
    }

    for required in [
        "PresentationAtlasRegistry",
        "presentation_id",
        "unit_ids",
        "presentation_entry_for_unit",
    ] {
        assert!(
            RENDER_SRC.contains(required),
            "src/windowed/render.rs must contain {required:?} so presentations are keyed by owned ids/selectors rather than engine-side species matches"
        );
    }
}

#[test]
fn windowed_bootstrap_builds_from_demo_registry_not_a_hardcoded_preset() {
    for forbidden in ["EncounterPreset::AgumonTrainingDummy", "bootstrap_encounter(", "Renamon"] {
        assert!(
            !WINDOWED_MOD_SRC.contains(forbidden),
            "src/windowed/mod.rs must not contain {forbidden:?}; windowed bootstrap should no longer hardcode a single demo species"
        );
    }

    for required in ["WindowedDemoRegistry", "build_demo_composition("] {
        assert!(
            WINDOWED_MOD_SRC.contains(required),
            "src/windowed/mod.rs must contain {required:?} so the demo is composed from registry data"
        );
    }
}

#[test]
fn demo_and_agumon_modules_own_the_extension_data() {
    for required in ["WindowedDemoRegistry", "WindowedDemoEntry", "build_demo_composition"] {
        assert!(
            DEMO_SRC.contains(required),
            "src/windowed/demo.rs must contain {required:?} so windowed demo composition remains a dedicated registry seam"
        );
    }

    for required in [
        "AGUMON_DUMMY_ID",
        "WindowedDemoRegistry",
        "WindowedDemoEntry",
        "presentation_id",
        "unit_ids",
    ] {
        assert!(
            AGUMON_SRC.contains(required),
            "src/windowed/digimon/agumon/mod.rs must contain {required:?} so Agumon owns its presentation selectors and demo composition wiring"
        );
    }
}
