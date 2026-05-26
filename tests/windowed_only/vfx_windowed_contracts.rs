//! M004/S05 T01 — windowed render-path contract for VFX presentation wiring.
//!
//! This is intentionally a static/contract test: `setup_camera` lives in the
//! binary crate's windowed module, so the integration harness verifies the
//! authored source without launching a window. It pins technical preconditions
//! only — HDR output, Bloom/Tonemapping/DebandDither wiring, and linear color
//! writes — and does **not** claim human visual acceptance.
#![cfg(feature = "windowed")]

const RENDER_SRC: &str = include_str!("../../src/windowed/render.rs");

fn setup_camera_block() -> &'static str {
    let start = RENDER_SRC
        .find("fn setup_camera")
        .expect("render.rs must define setup_camera");
    let rest = &RENDER_SRC[start..];
    // M006/S01 T04 deleted load_vfx_visuals (the quad VFX loader); setup_camera is
    // now followed by load_agumon_enoki_vfx. Slice to that boundary.
    let end = rest
        .find("fn load_agumon_enoki_vfx")
        .expect("setup_camera should remain adjacent to load_agumon_enoki_vfx for this contract test");
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
    assert!(
        RENDER_SRC.contains("Color::linear_rgba"),
        "windowed VFX colors must be written as linear values so authored data is preserved through the render path"
    );
}
