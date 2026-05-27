//! M006/S05 T03 — source-contract for Renamon's extension-only windowed
//! presentation seam.
//!
//! The windowed binary crate is unreachable from `tests/`, so this contract pins
//! the authored source and assets directly. It proves Renamon is added as an
//! extension module + assets rather than by species branches in engine files.
#![cfg(feature = "windowed")]

use bevyrogue::animation::anim_graph::{AnimGraph, NodeId, Predicate, TransitionTarget};
use bevyrogue::animation::reaction::StanceReaction;

const RENDER_SRC: &str = include_str!("../../src/windowed/render.rs");
const REGISTRIES_SRC: &str = include_str!("../../src/windowed/render/registries.rs");
const WINDOWED_MOD_SRC: &str = include_str!("../../src/windowed/mod.rs");
const DIGIMON_MOD_SRC: &str = include_str!("../../src/windowed/digimon/mod.rs");
const RENAMON_SRC: &str = include_str!("../../src/windowed/digimon/renamon/mod.rs");
const RENAMON_STANCE: &str = include_str!("../../assets/digimon/renamon/stance.ron");
const RENAMON_CLIP: &str = include_str!("../../assets/digimon/renamon/clip.ron");
const RENAMON_ANIM: &str = include_str!("../../assets/digimon/renamon/anim_graph.ron");
// M006/S10 decomposed render.rs into per-concern submodules. The presentation
// lookup call site is in spawn.rs; the cast-cue spawn-miss is in playback.rs.
const RENDER_SPAWN_SRC: &str = include_str!("../../src/windowed/render/spawn.rs");
const RENDER_PLAYBACK_SRC: &str = include_str!("../../src/windowed/render/playback.rs");

fn assert_contains_all(haystack: &str, label: &str, required: &[&str]) {
    for token in required {
        assert!(haystack.contains(token), "{label} must contain {token:?}");
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
    // M006/S09: the engine-generic registry/entry types moved to
    // `render/registries.rs`; the multi-presentation lookup seam (the
    // species-agnostic find + call sites) stays in `render.rs`.
    assert_contains_all(
        REGISTRIES_SRC,
        "src/windowed/render/registries.rs",
        &[
            "struct SpritePresentationRegistry",
            "entries: Vec<SpritePresentationEntry>",
        ],
    );
    // M006/S10: fn definition stays in render.rs; call site + log tokens in spawn.rs.
    let render_spawn = format!("{RENDER_SRC}{RENDER_SPAWN_SRC}");
    assert_contains_all(
        &render_spawn,
        "src/windowed/render.rs + spawn.rs",
        &[
            "fn presentation_entry_for_unit(",
            ".find(|entry| entry.matches_unit(unit_id))",
            "presentation_entry_for_unit(&presentation, unit.id)",
            "entry.presentation_id.as_str()",
            "entry.atlas_image_path.as_str()",
        ],
    );
    assert_contains_none(
        &render_spawn,
        "src/windowed/render.rs + spawn.rs",
        &["presentation.entries.first()", "presentation.entries[0]"],
    );
}

#[test]
fn digimon_aggregator_only_declares_and_registers_renamon() {
    assert_contains_all(
        DIGIMON_MOD_SRC,
        "src/windowed/digimon/mod.rs",
        &[
            "mod renamon",
            "renamon::register(app);",
            "fn register_all(app: &mut App)",
        ],
    );
}

#[test]
fn renamon_module_owns_the_extension_data_and_registration() {
    // S08 adds the diamond_storm_leaf VFX cue: these tokens prove the module
    // now populates the on-enter and enoki registries for its real authored cue.
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
            // S08 additions: on-enter cue + enoki VFX mapping for diamond_storm_leaf
            "diamond_storm_leaf",
            "diamond_storm.leaf",
            "OnEnterEffectRegistry",
            "EnokiVfxRegistry",
        ],
    );
}

/// S08 reverses the S05 idle-only contract: Renamon now legitimately populates
/// `OnEnterEffectRegistry` and `EnokiVfxRegistry` for its real authored
/// `diamond_storm_leaf` cue (anim_graph.ron line 9). The forbidden list is
/// narrowed to registries Renamon genuinely does NOT use: `SkillReleaseEffectRegistry`
/// (no release-boundary projectile in the current skill graph) and
/// `DetonateEffectRegistry` (no detonate burst). This keeps the contract tight
/// against unintended scope creep without blocking the legitimate S08 additions.
#[test]
fn renamon_module_does_not_use_unused_registries() {
    assert_contains_none(
        RENAMON_SRC,
        "src/windowed/digimon/renamon/mod.rs",
        &["SkillReleaseEffectRegistry", "DetonateEffectRegistry"],
    );
}

/// S08/T03 — the cast-cue spawn-miss is a warned-once diagnostic, not a silent
/// no-op. A `SpawnParticle` cue that resolves to zero spawned particles (unmapped
/// in `OnEnterEffectRegistry`, or its effect ids absent from `EnokiVfxRegistry`)
/// is logged at most once per cue id via the S06 `Local<HashSet>` warn-once
/// pattern, carrying the cue id so the unregistered cue is visible by name.
/// Registered cues that spawn stay silent (the warn is gated on `cue_spawned == 0`).
///
/// Source-contract test: `src/windowed/` is binary-crate code unreachable from
/// `tests/` and K001 forbids launching the windowed binary, so the seam is pinned
/// by co-occurring canonical tokens rather than live behaviour.
#[test]
fn cast_cue_spawn_miss_warns_once_with_cue_id() {
    // M006/S10: cast-cue spawn-miss logic moved to playback.rs.
    let render_playback = format!("{RENDER_SRC}{RENDER_PLAYBACK_SRC}");
    assert_contains_all(
        &render_playback,
        "src/windowed/render.rs + playback.rs",
        &[
            // S06 warn-once dedup state, keyed by cue id (String).
            "cast_cue_spawn_miss_warned: Local<HashSet<String>>",
            // Spawn count is accumulated across the cue's effect ids ...
            "cue_spawned += spawned;",
            // ... and the warn fires only when nothing spawned (registered cues silent),
            // deduped + keyed by the authored cue id.
            "if cue_spawned == 0",
            ".insert(descriptor.particle.0.clone())",
            // The log surfaces the cue id under the digimon playback target.
            "cue = descriptor.particle.0.as_str()",
            "target: \"windowed.digimon_playback\"",
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

/// S08/T04 — the idle-only-vs-hurt design call: Renamon's non-idle reactions
/// (hurt/death) use the **shared engine reaction defaults**, not species-specific
/// reaction data (decision recorded via gsd_save_decision).
///
/// The windowed reaction path (`drive_hurt_reactions` / `drive_death_reactions`
/// in render.rs) is species-agnostic: it drives the *target* sprite into the node
/// named by the pure lib mapping `StanceReaction::stance_node()` — the canonical
/// `"hurt"` / `"death"` vocabulary. A new Digimon gets working hurt/death purely by
/// authoring a stance graph whose node names + return transitions conform to that
/// vocabulary; no per-species reaction code exists. This test is the executable
/// half of that decision: it parses Renamon's authored stance graph (lib-reachable,
/// no windowed binary — K001) and proves it satisfies the shared contract.
#[test]
fn renamon_reactions_use_shared_engine_defaults() {
    let graph: AnimGraph = ron::from_str(RENAMON_STANCE)
        .expect("renamon stance.ron must parse as an AnimGraph");

    // Every shared StanceReaction resolves to a node Renamon actually authored:
    // the engine drives `stance_node()` against this graph with zero per-species
    // mapping, so a missing node would silently break Renamon's reaction path.
    for reaction in [StanceReaction::Hurt, StanceReaction::Death] {
        let node = reaction.stance_node();
        assert!(
            graph.nodes.contains_key(&node),
            "renamon stance graph must define the shared `{}` reaction node",
            node.0
        );
    }

    // Return transitions match the engine's degrade-to-idle / death-exit contract
    // (`drive_stance_reaction`): hurt is a transient detour back to idle on
    // TimeInNode; death exits the graph (the sprite rests on its final frame,
    // marked DeathExiting). A dropped/duplicated event degrades to "stays idle"
    // rather than a stuck frame because of these authored edges.
    let edge = |from: &str| {
        graph
            .transitions
            .iter()
            .find(|e| e.from == NodeId(from.to_string()))
            .unwrap_or_else(|| panic!("renamon stance graph must have a transition from `{from}`"))
    };
    let hurt_edge = edge("hurt");
    assert_eq!(hurt_edge.to, TransitionTarget::Node(NodeId("idle".to_string())));
    assert_eq!(hurt_edge.when, Predicate::TimeInNode);

    let death_edge = edge("death");
    assert_eq!(death_edge.to, TransitionTarget::Exit);
    assert_eq!(death_edge.when, Predicate::TimeInNode);
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
