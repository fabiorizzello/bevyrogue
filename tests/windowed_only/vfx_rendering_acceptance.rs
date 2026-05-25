//! M004/S05 T01 — windowed rendering acceptance contract for HDR bloom policy.
//!
//! This is intentionally a static/contract test: `setup_camera` lives in the
//! binary crate's windowed module, so the integration harness verifies the
//! authored source and the real git-tracked VFX asset without launching a
//! window. That keeps failure localization precise for future agents: missing
//! HDR/bloom camera policy, missing tonemapping/dithering, or clamped VFX color
//! data all fail here before any manual visual pass.
#![cfg(feature = "windowed")]

use bevyrogue::animation::{eval_color, resolve_effect, VfxAsset};

const RENDER_SRC: &str = include_str!("../../src/windowed/render.rs");
const AGUMON_VFX_RON: &str = include_str!("../../assets/digimon/agumon/vfx.ron");

fn agumon_vfx() -> VfxAsset {
    ron::from_str::<VfxAsset>(AGUMON_VFX_RON)
        .expect("assets/digimon/agumon/vfx.ron should parse into VfxAsset")
}

fn setup_camera_block() -> &'static str {
    let start = RENDER_SRC
        .find("fn setup_camera")
        .expect("render.rs must define setup_camera");
    let rest = &RENDER_SRC[start..];
    let end = rest
        .find("fn load_vfx_visuals")
        .expect("setup_camera should remain adjacent to load_vfx_visuals for this contract test");
    &rest[..end]
}

#[test]
fn setup_camera_enables_hdr_bloom_tonemapping_and_deband_dither() {
    let setup = setup_camera_block();

    assert!(
        setup.contains("hdr: true") || setup.contains("Hdr"),
        "setup_camera must enable HDR output (`hdr: true` on older Bevy, `Hdr` component on Bevy 0.18)"
    );
    assert!(
        setup.contains("Bloom"),
        "setup_camera must attach a Bloom component so overbright VFX can bloom"
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
        "windowed VFX colors must be written as linear HDR values so >1.0 asset channels are preserved"
    );
}

#[test]
fn agumon_vfx_keeps_bloom_capable_overbright_color_channels() {
    let asset = agumon_vfx();
    let bloom_capable_ids = [
        "baby_flame.charge",
        "baby_flame.projectile",
        "baby_flame.impact",
        "baby_flame.impact_flash",
        "baby_burner.flash",
    ];

    let mut overbright_hits = Vec::new();
    for id in bloom_capable_ids {
        let effect = resolve_effect(&asset, id)
            .unwrap_or_else(|| panic!("authored bloom-capable effect `{id}` must exist"));
        for sample in [0.0_f32, 0.25, 0.5, 1.0] {
            let rgba = eval_color(&effect.appearance.color, sample);
            if rgba[..3].iter().any(|channel| *channel > 1.0) {
                overbright_hits.push(format!("{id}@{sample}: rgb={:?}", &rgba[..3]));
                break;
            }
        }
    }

    assert!(
        !overbright_hits.is_empty(),
        "at least one authored Agumon bloom-capable effect must keep an RGB channel above 1.0 so HDR bloom cannot regress back to clamped UI color data"
    );
}
