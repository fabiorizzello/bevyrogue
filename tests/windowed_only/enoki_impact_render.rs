//! M005/S04 T03 — windowed source-contract for the enoki one-shot spawn seam.
//!
//! Like the other windowed render contracts, this is a static/source test: the
//! spawn wiring lives in the binary crate's windowed module, so the integration
//! harness verifies the authored source without launching a window (K001 forbids
//! running the windowed binary in auto-mode). It pins three preconditions:
//!   1. `EnokiPlugin` is registered on the windowed app.
//!   2. `spawn_effect_by_id` routes the single `baby_flame.impact` id through an
//!      enoki one-shot bundle (`ParticleSpawner` + `ParticleEffectHandle` +
//!      `OneShot`) while leaving the quad loop for every other id.
//!   3. The kernel/FSM control flow (`fire_kernel_cue` + `request_release`) is
//!      untouched (D031/D032) — only what gets spawned for one id changed.
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
fn spawn_effect_by_id_routes_baby_flame_impact_through_an_enoki_one_shot() {
    let block = spawn_effect_by_id_block();

    assert!(
        block.contains("effect_id == AGUMON_IMPACT_EFFECT_ID"),
        "spawn_effect_by_id must branch on AGUMON_IMPACT_EFFECT_ID so only that one id is routed to enoki"
    );
    assert!(
        block.contains("ParticleSpawner"),
        "the baby_flame.impact branch must spawn a ParticleSpawner (the enoki spawner component)"
    );
    assert!(
        block.contains("ParticleEffectHandle"),
        "the baby_flame.impact branch must attach the loaded ParticleEffectHandle from AgumonEnokiVfx"
    );
    assert!(
        block.contains("OneShot"),
        "the baby_flame.impact branch must mark the spawner OneShot so it self-despawns rather than entering the kernel timeline"
    );
    // The quad loop must still exist for every non-impact id.
    assert!(
        block.contains("for i in 0..count"),
        "the quad spawn loop must remain for all non-impact effect ids — only baby_flame.impact is intercepted"
    );
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
