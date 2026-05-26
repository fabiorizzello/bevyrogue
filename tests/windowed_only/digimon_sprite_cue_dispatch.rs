//! M006/S03 T04 — windowed source-contract for the generalized DigimonSprite +
//! CueRegistry dispatch seams.
//!
//! src/windowed/ is binary-crate code unreachable from tests/ (MEM030), so — like
//! the other windowed render contracts (enoki_impact_render.rs,
//! vfx_windowed_contracts.rs) — this is a static/source test: it `include_str!`s
//! the authored source and asserts structural tokens without launching a window
//! (K001 forbids running the windowed binary in auto-mode). It pins the S03 seams
//! that must outlive the S04 agumon extraction:
//!   1. The windowed sprite is the generalized, data-carrying `DigimonSprite` /
//!      `DigimonPlaybackMode` — the Agumon-named components are gone.
//!   2. `DigimonSprite` carries the stance/skill graph ids as data fields.
//!   3. The flash/shake feedback path reads the `CueRegistry` parametric math
//!      (`flash_tint_parametric` / `shake_offset_parametric`); the legacy
//!      `flash_tint(` / `shake_offset(` lib calls are gone.
//!   4. Camera-shake exists as a registered cue that writes the `Camera2d`
//!      transform as an absolute offset from a captured `CameraRest`.
//!   5. mod.rs registers the three Agumon cue ids in the `CueRegistry`.
//!
//! Token shape: assertions use code-shaped tokens (`struct DigimonSprite`,
//! `enum DigimonPlaybackMode`, `flash_tint(`) so surviving comments mentioning the
//! old names can't trip presence/absence checks (MEM101 / S01 pattern). Formulas
//! are never asserted — only the presence/absence of structural tokens.
#![cfg(feature = "windowed")]

const RENDER_SRC: &str = include_str!("../../src/windowed/render.rs");
const MOD_SRC: &str = include_str!("../../src/windowed/mod.rs");

#[test]
fn sprite_is_generalized_digimon_not_agumon() {
    assert!(
        RENDER_SRC.contains("struct DigimonSprite"),
        "render.rs must define the generalized struct DigimonSprite"
    );
    assert!(
        RENDER_SRC.contains("enum DigimonPlaybackMode"),
        "render.rs must define the generalized enum DigimonPlaybackMode"
    );
    // Code-shaped absence: the Agumon-named component/enum must be gone (S04 keeps
    // AGUMON_* consts, but the windowed sprite type itself is generalized in S03).
    assert!(
        !RENDER_SRC.contains("struct AgumonSprite"),
        "the Agumon-named struct AgumonSprite must be renamed to DigimonSprite (T01)"
    );
    assert!(
        !RENDER_SRC.contains("enum AgumonPlaybackMode"),
        "the Agumon-named enum AgumonPlaybackMode must be renamed to DigimonPlaybackMode (T01)"
    );
}

#[test]
fn digimon_sprite_carries_graph_ids_as_data() {
    assert!(
        RENDER_SRC.contains("stance_graph_id"),
        "DigimonSprite must carry the stance graph id as a data field (stance_graph_id) rather than a module const"
    );
    assert!(
        RENDER_SRC.contains("skill_graph_id"),
        "DigimonSprite must carry the skill graph id as a data field (skill_graph_id) rather than a module const"
    );
}

#[test]
fn flash_and_shake_read_the_cue_registry_parametric_math() {
    assert!(
        RENDER_SRC.contains("CueRegistry"),
        "the windowed presentation path must consult the CueRegistry for flash/shake cues"
    );
    assert!(
        RENDER_SRC.contains("flash_tint_parametric"),
        "the flash path must compute its tint via flash_tint_parametric from the registered cue params"
    );
    assert!(
        RENDER_SRC.contains("shake_offset_parametric"),
        "the shake path must compute its offset via shake_offset_parametric from the registered cue params"
    );
    // Code-shaped absence: the legacy hit_feedback lib calls (`flash_tint(` /
    // `shake_offset(`) must be gone. `flash_tint(` does NOT match
    // `flash_tint_parametric(` — the next char after the prefix is `_`, not `(`.
    assert!(
        !RENDER_SRC.contains("flash_tint("),
        "the legacy flash_tint() lib call must be replaced by flash_tint_parametric() (D048)"
    );
    assert!(
        !RENDER_SRC.contains("shake_offset("),
        "the legacy shake_offset() lib call must be replaced by shake_offset_parametric() (D048)"
    );
}

#[test]
fn camera_shake_exists_and_writes_the_camera_transform() {
    assert!(
        RENDER_SRC.contains("CameraRest"),
        "camera-shake must capture the camera's rest translation in a CameraRest so the offset is absolute, not additive (MEM094)"
    );
    assert!(
        RENDER_SRC.contains("CameraShakeState"),
        "camera-shake must track its decay in a CameraShakeState resource"
    );
    assert!(
        RENDER_SRC.contains("Camera2d"),
        "the camera-shake path must target the Camera2d entity"
    );
    assert!(
        RENDER_SRC.contains("&mut Transform"),
        "the camera-shake apply system must write the camera Transform (offset from CameraRest)"
    );
}

#[test]
fn mod_registers_the_three_agumon_cue_ids() {
    assert!(
        MOD_SRC.contains("CueRegistry"),
        "mod.rs must init and populate the CueRegistry resource"
    );
    for cue_id in ["hit_flash", "hit_shake", "camera_impact"] {
        assert!(
            MOD_SRC.contains(cue_id),
            "mod.rs must register the {cue_id:?} cue id in the CueRegistry"
        );
    }
}
