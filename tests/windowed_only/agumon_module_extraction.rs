//! M006/S04 T04 — source-contract for the Agumon extraction seam.
//!
//! `src/windowed/` is binary-crate code unreachable from `tests/`, so this is a
//! static/source contract test: it `include_str!`s the authored source and pins
//! the structural S04 claim without launching the windowed binary (K001).
//! The contract is intentionally grep-shaped rather than behaviour-shaped:
//!   1. `src/windowed/render.rs` and `src/windowed/mod.rs` no longer carry
//!      Agumon-specific constants/helpers/paths.
//!   2. `src/windowed/digimon/agumon/mod.rs` now owns the registry population
//!      for Agumon effect, cue, skill-start-node, and sprite-presentation data.
//!   3. `src/windowed/digimon/mod.rs` remains the per-Digimon registration seam
//!      via `mod agumon` + `fn register_all`.
//!
//! Assertions use code-shaped tokens only. They do not pin numeric values or
//! formulas, so behaviour-preserving refactors can still move freely.
#![cfg(feature = "windowed")]

const RENDER_SRC: &str = include_str!("../../src/windowed/render.rs");
const WINDOWED_MOD_SRC: &str = include_str!("../../src/windowed/mod.rs");
const DIGIMON_MOD_SRC: &str = include_str!("../../src/windowed/digimon/mod.rs");
const AGUMON_SRC: &str = include_str!("../../src/windowed/digimon/agumon/mod.rs");

#[test]
fn engine_files_no_longer_embed_agumon_specific_tokens() {
    for src in [RENDER_SRC, WINDOWED_MOD_SRC] {
        for forbidden in [
            "AGUMON_",
            "fn on_enter_effect_ids",
            "fn skill_start_node",
            "fn load_agumon_enoki_vfx",
            "enoki_effect_path",
            "digimon/agumon_atlas.png",
        ] {
            assert!(
                !src.contains(forbidden),
                "engine source must not contain {forbidden:?}; Agumon-specific data/helpers belong in src/windowed/digimon/agumon/"
            );
        }
    }
}

#[test]
fn agumon_module_owns_the_registry_population_tokens() {
    for required in [
        "EnokiVfxRegistry",
        "OnEnterEffectRegistry",
        "SkillStartNodeRegistry",
        "SkillReleaseEffectRegistry",
        "SpritePresentationRegistry",
        "hit_flash",
        "hit_shake",
        "camera_impact",
        "digimon/agumon_atlas.png",
    ] {
        assert!(
            AGUMON_SRC.contains(required),
            "agumon module must contain {required:?} so the S04 data/registration seam stays owned by src/windowed/digimon/agumon/"
        );
    }
}

#[test]
fn digimon_module_exposes_the_register_all_seam() {
    assert!(
        DIGIMON_MOD_SRC.contains("mod agumon"),
        "src/windowed/digimon/mod.rs must declare the agumon submodule"
    );
    assert!(
        DIGIMON_MOD_SRC.contains("fn register_all"),
        "src/windowed/digimon/mod.rs must expose fn register_all as the per-Digimon registration seam"
    );
}
