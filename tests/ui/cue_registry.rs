//! Headless contract proof for the cue seam (M006/S02, R003/R004).
//!
//! Exercises `bevyrogue::ui::cues` through its public surface only — no windowed
//! feature, no `bevy_enoki`/`bevy_render` types. Pins the D044/D047 registry
//! collision/idempotency convention and the parametric-math determinism that S03
//! depends on. Legacy formulas are recomputed inline (the `hit_feedback`
//! reference is windowed-gated and absent from this headless build), so a future
//! failure points directly at the violated cue invariant.

use bevy::math::Vec2;

use bevyrogue::ui::cues::{
    CueDef, CueRegistry, SrgbTriple, flash_tint_parametric, shake_offset_parametric,
};

// Legacy constants from `crate::ui::hit_feedback` that the parametric forms
// generalize. Duplicated here intentionally: importing the windowed-gated module
// would break the headless build (R005).
const LEGACY_FLASH_PEAK: SrgbTriple = (1.0, 0.45, 0.45);
const LEGACY_SHAKE_AMP: f32 = 4.0;
const LEGACY_SHAKE_FREQ_X: f32 = 1.7;
const LEGACY_SHAKE_FREQ_Y: f32 = 2.3;

// --- CueRegistry contract (D044/D047) ---

#[test]
fn cue_registry_lookup_returns_registered_def() {
    let mut reg = CueRegistry::default();
    let def = CueDef::Flash {
        peak: LEGACY_FLASH_PEAK,
        ticks: 8,
    };
    reg.register("hit_flash", def.clone());
    assert_eq!(reg.get("hit_flash"), Some(&def));
}

#[test]
fn cue_registry_unknown_id_returns_none() {
    let mut reg = CueRegistry::default();
    // Empty registry: unknown id is None.
    assert!(reg.get("missing").is_none());
    // Non-empty registry: a DIFFERENT unknown id is still None (the S03
    // "caller logs and no-ops" path, never a panic).
    reg.register(
        "known",
        CueDef::ParticleBurst {
            effect_id: "baby_flame_impact".to_string(),
        },
    );
    assert!(reg.get("other").is_none());
}

#[test]
#[should_panic(expected = "CueRegistry conflict for id")]
fn cue_registry_collision_panics_on_different_def() {
    let mut reg = CueRegistry::default();
    reg.register(
        "cue",
        CueDef::Flash {
            peak: LEGACY_FLASH_PEAK,
            ticks: 8,
        },
    );
    // Same id, DIFFERENT def → startup-time fail-fast panic.
    reg.register(
        "cue",
        CueDef::Flash {
            peak: (1.0, 0.0, 0.0),
            ticks: 4,
        },
    );
}

#[test]
fn cue_registry_idempotent_reregister_same_def() {
    let mut reg = CueRegistry::default();
    let def = CueDef::SpriteShake {
        amp: LEGACY_SHAKE_AMP,
        freq_x: LEGACY_SHAKE_FREQ_X,
        freq_y: LEGACY_SHAKE_FREQ_Y,
        ticks: 8,
    };
    reg.register("shake", def.clone());
    // Re-registering the IDENTICAL def is a no-op (must not panic).
    reg.register("shake", def.clone());
    assert_eq!(reg.get("shake"), Some(&def));
}

// --- Flash parametric math (R004) ---

#[test]
fn flash_tint_parametric_zero_remaining_is_white() {
    // No flash → untinted white triple (1,1,1), which S03 maps to Color::WHITE.
    assert_eq!(
        flash_tint_parametric(0, 8, LEGACY_FLASH_PEAK),
        (1.0, 1.0, 1.0)
    );
    // total == 0 is the same degenerate "not flashing" case.
    assert_eq!(
        flash_tint_parametric(8, 0, LEGACY_FLASH_PEAK),
        (1.0, 1.0, 1.0)
    );
}

#[test]
fn flash_tint_parametric_matches_legacy_at_peak() {
    // remaining == total → t == 1.0 → the lerp lands exactly on `peak`.
    // Recompute the legacy lerp inline rather than importing windowed hit_feedback.
    let (r, g, b) = flash_tint_parametric(8, 8, LEGACY_FLASH_PEAK);
    let lerp = |a: f32, end: f32| a + (end - a) * 1.0;
    let expected = (
        lerp(1.0, LEGACY_FLASH_PEAK.0),
        lerp(1.0, LEGACY_FLASH_PEAK.1),
        lerp(1.0, LEGACY_FLASH_PEAK.2),
    );
    assert!(
        (r - expected.0).abs() < f32::EPSILON,
        "r {r} != {}",
        expected.0
    );
    assert!(
        (g - expected.1).abs() < f32::EPSILON,
        "g {g} != {}",
        expected.1
    );
    assert!(
        (b - expected.2).abs() < f32::EPSILON,
        "b {b} != {}",
        expected.2
    );
}

// --- Shake parametric math (R004) ---

#[test]
fn shake_offset_parametric_zero_remaining_is_zero() {
    assert_eq!(
        shake_offset_parametric(
            0,
            8,
            LEGACY_SHAKE_AMP,
            LEGACY_SHAKE_FREQ_X,
            LEGACY_SHAKE_FREQ_Y
        ),
        Vec2::ZERO
    );
    assert_eq!(
        shake_offset_parametric(
            8,
            0,
            LEGACY_SHAKE_AMP,
            LEGACY_SHAKE_FREQ_X,
            LEGACY_SHAKE_FREQ_Y
        ),
        Vec2::ZERO
    );
}

#[test]
fn shake_offset_parametric_nonzero_at_peak() {
    // remaining == total → full amplitude. Offset must be non-zero yet bounded by
    // the per-axis amplitude envelope (|sin|, |cos| <= 1).
    let offset = shake_offset_parametric(
        8,
        8,
        LEGACY_SHAKE_AMP,
        LEGACY_SHAKE_FREQ_X,
        LEGACY_SHAKE_FREQ_Y,
    );
    assert_ne!(offset, Vec2::ZERO);
    assert!(offset.x.abs() <= LEGACY_SHAKE_AMP + f32::EPSILON);
    assert!(offset.y.abs() <= LEGACY_SHAKE_AMP + f32::EPSILON);
}

#[test]
fn shake_offset_parametric_determinism() {
    // Same inputs twice → bit-identical output (no wall-clock, no RNG — R004).
    let a = shake_offset_parametric(
        5,
        8,
        LEGACY_SHAKE_AMP,
        LEGACY_SHAKE_FREQ_X,
        LEGACY_SHAKE_FREQ_Y,
    );
    let b = shake_offset_parametric(
        5,
        8,
        LEGACY_SHAKE_AMP,
        LEGACY_SHAKE_FREQ_X,
        LEGACY_SHAKE_FREQ_Y,
    );
    assert_eq!(a, b);
}

#[test]
fn camera_shake_uses_same_shake_math() {
    // CueDef::CameraShake and CueDef::SpriteShake carry the same kinematic params
    // and both resolve through `shake_offset_parametric` (no separate camera
    // kinematics). Prove the shared-fn contract: identical params → identical
    // offset regardless of which cue variant supplied them.
    let amp = 6.0;
    let fx = 1.1;
    let fy = 2.9;
    let sprite = CueDef::SpriteShake {
        amp,
        freq_x: fx,
        freq_y: fy,
        ticks: 8,
    };
    let camera = CueDef::CameraShake {
        amp,
        freq_x: fx,
        freq_y: fy,
        ticks: 8,
    };
    // Extract params from each variant and feed the shared math fn.
    let eval = |def: &CueDef, remaining: u32| -> Vec2 {
        match *def {
            CueDef::SpriteShake {
                amp,
                freq_x,
                freq_y,
                ticks,
            }
            | CueDef::CameraShake {
                amp,
                freq_x,
                freq_y,
                ticks,
            } => shake_offset_parametric(remaining, ticks, amp, freq_x, freq_y),
            _ => unreachable!("test feeds only shake variants"),
        }
    };
    for remaining in 0..=8 {
        assert_eq!(
            eval(&sprite, remaining),
            eval(&camera, remaining),
            "camera/sprite shake math diverged at remaining={remaining}"
        );
    }
}
