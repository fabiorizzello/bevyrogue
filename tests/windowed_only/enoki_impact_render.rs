//! M005/S04 T03 — windowed source-contract for the enoki one-shot spawn seam.
//!
//! Like the other windowed render contracts, this is a static/source test: the
//! spawn wiring lives in the binary crate's windowed module, so the integration
//! harness verifies the authored source without launching a window (K001 forbids
//! running the windowed binary in auto-mode). It pins three preconditions:
//!   1. `EnokiPlugin` is registered on the windowed app.
//!   2. `spawn_effect_by_id` routes any effect id present in the enoki handle map
//!      (`enoki.handles.get(effect_id)`) through an enoki one-shot bundle
//!      (`ParticleSpawner` + `ParticleEffectHandle` + `OneShot`); enoki is the sole
//!      particle renderer (M006/S01 T04 deleted the quad system), and the map is
//!      keyed by all six Agumon ids (baby_flame.charge/ember/projectile/impact,
//!      sharp_claws.slash, baby_burner.detonate). The enoki-native lifecycle layer
//!      (ChargeEmberEnokiMarker, ProjectileFlight, advance_enoki_projectiles) that
//!      replaced the quad per-frame work is present.
//!   3. The kernel/FSM control flow (`fire_kernel_cue` + `request_release`) is
//!      untouched (D031/D032) — only what gets spawned for a matched id changed.
#![cfg(feature = "windowed")]

const RENDER_SRC: &str = include_str!("../../src/windowed/render.rs");

/// Slice `spawn_effect_by_id` from its `fn` keyword to the next top-level `fn`,
/// so assertions about the enoki branch can't accidentally match unrelated code.
fn spawn_effect_by_id_block() -> &'static str {
    let start = RENDER_SRC
        .find("fn spawn_effect_by_id")
        .expect("render.rs must define spawn_effect_by_id");
    let rest = &RENDER_SRC[start..];
    let end = rest
        .find("\nfn should_spawn_node_vfx")
        .expect("spawn_effect_by_id should remain adjacent to should_spawn_node_vfx for this contract test");
    &rest[..end]
}

#[test]
fn enoki_plugin_is_registered_on_the_windowed_app() {
    assert!(
        RENDER_SRC.contains("add_plugins(EnokiPlugin)"),
        "RenderPlugin::build must register EnokiPlugin so the GPU 2D particle backend runs in the windowed app"
    );
}

#[test]
fn spawn_effect_by_id_routes_mapped_ids_through_an_enoki_one_shot() {
    let block = spawn_effect_by_id_block();

    assert!(
        block.contains("enoki.handles.get(effect_id)"),
        "spawn_effect_by_id must look the effect id up in the per-effect handle map so any mapped id is routed to enoki"
    );
    assert!(
        block.contains("ParticleSpawner"),
        "the enoki branch must spawn a ParticleSpawner (the enoki spawner component)"
    );
    assert!(
        block.contains("ParticleEffectHandle"),
        "the enoki branch must attach the mapped ParticleEffectHandle from AgumonEnokiVfx"
    );
    assert!(
        block.contains("OneShot"),
        "the enoki branch must mark the spawner OneShot so it self-despawns rather than entering the kernel timeline"
    );
    // M006/S01 T04 (D043): enoki is now the sole particle renderer — an unmapped
    // id returns 0 (no quad fallback). Pin that the function early-returns instead
    // of carrying the deleted quad spawn loop.
    assert!(
        !block.contains("for i in 0..count"),
        "the quad spawn loop must be gone — enoki is the sole particle renderer (D043)"
    );
    assert!(
        block.contains("return 0"),
        "spawn_effect_by_id must early-return 0 for an unmapped id now that the quad fallback is deleted"
    );
}

#[test]
fn quad_vfx_system_is_fully_deleted_from_render_src() {
    // M006/S01 T04 deleted the custom quad VFX system, leaving enoki as the sole
    // particle renderer (D043). Pin that its defining code symbols are gone from
    // render.rs. Historical mentions survive only inside comments, so match on
    // code-shaped tokens (`fn ...`, struct literal `VfxParticle {`, the spawn
    // loop) rather than bare substrings that the surviving comments would trip.
    assert!(
        !RENDER_SRC.contains("fn advance_vfx_particles"),
        "the quad advance_vfx_particles system must be deleted — enoki advances particles now"
    );
    assert!(
        !RENDER_SRC.contains("VfxParticle {"),
        "the quad VfxParticle component bundle must no longer be spawned"
    );
    assert!(
        !RENDER_SRC.contains("for i in 0..count"),
        "the quad spawn loop must be gone — enoki is the sole particle renderer (D043)"
    );
}

#[test]
fn enoki_lifecycle_layer_is_present() {
    // M006/S01 T03 added the enoki-native lifecycle layer that replaced the quad
    // system's per-frame work: persistent charge/ember emitters (cleared on launch)
    // and a traveling projectile that chains the impact burst on arrival.
    assert!(
        RENDER_SRC.contains("ChargeEmberEnokiMarker"),
        "the persistent charge/ember emitter marker must remain so launch can clear them"
    );
    assert!(
        RENDER_SRC.contains("ProjectileFlight"),
        "the traveling-projectile flight component must remain so the projectile can travel caster->target"
    );
    assert!(
        RENDER_SRC.contains("fn advance_enoki_projectiles"),
        "the enoki projectile advance system must remain — it lerps flight and chains the impact burst"
    );
}

#[test]
fn enoki_handle_map_is_keyed_by_all_six_agumon_ids() {
    // The map is built in load_agumon_enoki_vfx; slice that fn and assert each
    // Agumon effect id is inserted so the full Baby Flame sequence (charge, ember,
    // projectile, impact) plus Sharp Claws and Baby Burner all route through enoki.
    // M006/S01 made enoki the sole particle renderer, so the map must cover all six.
    let start = RENDER_SRC
        .find("fn load_agumon_enoki_vfx")
        .expect("render.rs must define load_agumon_enoki_vfx");
    let rest = &RENDER_SRC[start..];
    let end = rest
        .find("\nfn ")
        .expect("load_agumon_enoki_vfx should be followed by another top-level fn");
    let block = &rest[..end];

    assert!(
        block.contains("handles.insert("),
        "load_agumon_enoki_vfx must build the per-effect handle map via handles.insert(...)"
    );
    for id_const in [
        "AGUMON_CHARGE_EFFECT_ID",
        "AGUMON_EMBER_EFFECT_ID",
        "AGUMON_PROJECTILE_EFFECT_ID",
        "AGUMON_IMPACT_EFFECT_ID",
        "AGUMON_DETONATE_EFFECT_ID",
        "AGUMON_SHARP_CLAWS_EFFECT_ID",
    ] {
        assert!(
            block.contains(id_const),
            "load_agumon_enoki_vfx must insert {id_const} into the enoki handle map"
        );
    }
}

#[test]
fn kernel_and_fsm_control_flow_remains_untouched() {
    assert!(
        RENDER_SRC.contains("fire_kernel_cue()"),
        "the FSM kernel cue (fire_kernel_cue) must remain — the enoki seam changes only what spawns, not control flow (D031/D032)"
    );
    assert!(
        RENDER_SRC.contains("request_release("),
        "the cue barrier release (request_release) must remain — the enoki seam changes only what spawns, not control flow (D031/D032)"
    );
}
