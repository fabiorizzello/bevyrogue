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
const AGUMON_ANIM_GRAPH: &str = include_str!("../../assets/digimon/agumon/anim_graph.ron");

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

/// S08/T02 — prove that the cast→effect chain is fully wired for Agumon's Baby Flame skill.
///
/// The chain has three links:
///   1. The authored `baby_flame_cast` animation-graph node fires a `SpawnParticle`
///      named `"baby_flame_charge"` (locked by `AGUMON_ANIM_GRAPH`).
///   2. The Agumon module maps `"baby_flame_charge"` → effect ids
///      `"baby_flame.charge"` + `"baby_flame.ember"` via `on_enter_effect_specs`
///      (locked by `AGUMON_SRC`).
///   3. Both effect ids are registered with their `.particle.ron` asset paths in
///      `register_agumon_enoki_vfx` (locked by `AGUMON_SRC`).
///
/// This is a source-contract test: `src/windowed/` is binary-crate code
/// unreachable from `tests/`, so the chain is asserted by pinning the
/// co-occurrence of canonical tokens (K001 — do not launch the windowed binary).
#[test]
fn agumon_cast_cue_resolves_to_registered_enoki_effects() {
    // Link 1: the authored animation-graph node fires the SpawnParticle command
    // with name "baby_flame_charge" on entering "baby_flame_cast".
    assert!(
        AGUMON_ANIM_GRAPH.contains("baby_flame_cast"),
        "anim_graph.ron must define the baby_flame_cast node (cast FSM entry point)"
    );
    assert!(
        AGUMON_ANIM_GRAPH.contains(r#"SpawnParticle("#),
        "baby_flame_cast node must emit a SpawnParticle on_enter command"
    );
    assert!(
        AGUMON_ANIM_GRAPH.contains(r#"name: "baby_flame_charge""#),
        "the SpawnParticle on baby_flame_cast must be named \"baby_flame_charge\" \
         — this is the authored cue name that the OnEnterEffectRegistry key-matches"
    );

    // Link 2: the Agumon module fans the cue name out to the four owned fire-layer
    // effect ids (M006/S08 layering: flames + orb + core + ember). The vec is
    // multi-line in source, so each id const is pinned individually rather than as
    // one literal substring.
    assert!(
        AGUMON_SRC.contains(r#""baby_flame_charge","#),
        "on_enter_effect_specs must key the fan-out on the \"baby_flame_charge\" cue name"
    );
    for id_const in [
        "FLAMES_EFFECT_ID",
        "CHARGE_EFFECT_ID",
        "CORE_EFFECT_ID",
        "EMBER_EFFECT_ID",
    ] {
        assert!(
            AGUMON_SRC.contains(id_const),
            "the baby_flame_charge fan-out must co-spawn the {id_const} layer"
        );
    }
    assert!(
        AGUMON_SRC.contains(r#"const CHARGE_EFFECT_ID: &str = "baby_flame.charge""#),
        "CHARGE_EFFECT_ID must resolve to \"baby_flame.charge\""
    );
    assert!(
        AGUMON_SRC.contains(r#"const CORE_EFFECT_ID: &str = "baby_flame.core""#),
        "CORE_EFFECT_ID must resolve to \"baby_flame.core\""
    );
    assert!(
        AGUMON_SRC.contains(r#"const FLAMES_EFFECT_ID: &str = "baby_flame.flames""#),
        "FLAMES_EFFECT_ID must resolve to \"baby_flame.flames\""
    );
    assert!(
        AGUMON_SRC.contains(r#"const EMBER_EFFECT_ID: &str = "baby_flame.ember""#),
        "EMBER_EFFECT_ID must resolve to \"baby_flame.ember\""
    );

    // Link 3: each layer effect id is backed by a registered enoki asset path.
    for asset in [
        "baby_flame_charge.particle.ron",
        "baby_flame_core.particle.ron",
        "baby_flame_flames.particle.ron",
        "baby_flame_ember.particle.ron",
    ] {
        assert!(
            AGUMON_SRC.contains(asset),
            "the layered fire body must reference {asset} in EnokiVfxRegistry"
        );
    }

    // Structural guard: the skill start-node spec must wire baby_flame → baby_flame_cast,
    // ensuring the cast node is reachable when the player selects Baby Flame.
    assert!(
        AGUMON_SRC.contains(r#"(BABY_FLAME_SKILL_ID, BABY_FLAME_CAST_NODE)"#),
        "skill_start_node_specs must include (BABY_FLAME_SKILL_ID, BABY_FLAME_CAST_NODE), \
         connecting the skill id to the cast entry node"
    );
    assert!(
        AGUMON_SRC.contains(r#"const BABY_FLAME_SKILL_ID: &str = "baby_flame""#),
        "BABY_FLAME_SKILL_ID must equal \"baby_flame\""
    );
    assert!(
        AGUMON_SRC.contains(r#"const BABY_FLAME_CAST_NODE: &str = "baby_flame_cast""#),
        "BABY_FLAME_CAST_NODE must equal \"baby_flame_cast\""
    );
}
