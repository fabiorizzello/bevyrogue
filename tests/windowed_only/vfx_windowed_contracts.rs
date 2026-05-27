//! M004/S05 T01 — windowed render-path contract for VFX presentation wiring.
//!
//! This is intentionally a static/contract test: `setup_camera` lives in the
//! binary crate's windowed module, so the integration harness verifies the
//! authored source without launching a window. It pins technical preconditions
//! only — HDR output, Bloom/Tonemapping/DebandDither wiring, and linear color
//! writes — and does **not** claim human visual acceptance.
#![cfg(feature = "windowed")]

const RENDER_SRC: &str = include_str!("../../src/windowed/render.rs");
// M006/S10 decomposed render.rs into per-concern submodules; setup_camera now
// lives in spawn.rs and observe_camera_shake in feedback.rs.
const RENDER_SPAWN_SRC: &str = include_str!("../../src/windowed/render/spawn.rs");
const RENDER_FEEDBACK_SRC: &str = include_str!("../../src/windowed/render/feedback.rs");

fn setup_camera_block() -> &'static str {
    let src = RENDER_SPAWN_SRC;
    let start = src
        .find("fn setup_camera")
        .expect("render.rs (spawn submodule) must define setup_camera");
    let rest = &src[start..];
    // setup_camera is followed by init_soft_particle_material in spawn.rs.
    let end = rest.find("\npub(super) fn init_soft_particle_material").unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn setup_camera_configures_hdr_bloom_tonemapping_and_deband_dither() {
    let setup = setup_camera_block();

    assert!(
        setup.contains("hdr: true") || setup.contains("Hdr"),
        "setup_camera must enable HDR output (`hdr: true` on older Bevy, `Hdr` component on Bevy 0.18)"
    );
    assert!(
        setup.contains("Bloom"),
        "setup_camera must attach a Bloom component as part of the windowed render path"
    );
    assert!(
        setup.contains("Tonemapping::"),
        "setup_camera must attach an explicit Tonemapping policy for HDR->display output"
    );
    assert!(
        setup.contains("DebandDither::Enabled"),
        "setup_camera must enable DebandDither because bloom introduces gradients prone to banding"
    );
    // M006/S10: advance_death_fade (Color::linear_rgba) now lives in feedback.rs.
    assert!(
        RENDER_SRC.contains("Color::linear_rgba") || RENDER_FEEDBACK_SRC.contains("Color::linear_rgba"),
        "windowed VFX colors must be written as linear values so authored data is preserved through the render path"
    );
}
