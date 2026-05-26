//! Data-driven cosmetic cue seam (M006/S02, D044/D047/MEM108).
//!
//! Replaces the ad-hoc, Agumon-coupled flash/shake consts in
//! `crate::ui::hit_feedback` with a generic registry of cosmetic primitives and
//! the pure, deterministic parametric math behind them. This module lives in the
//! LIB crate with NO `#[cfg(feature = "windowed")]` gate, so the headless build
//! and the `dependency_gating` test stay green (R002/R005).
//!
//! Enoki isolation is structural: [`CueDef::ParticleBurst`] stores an opaque
//! `effect_id: String`, never a `bevy_enoki`/`bevy_render` handle. The id is
//! resolved to a real handle only in the windowed binary (S03) — never here.
//!
//! Render-type isolation is likewise structural: `bevy_color` is part of the
//! render stack and is ABSENT from the headless dependency graph (R005), so the
//! flash primitive returns an sRGB `(f32, f32, f32)` triple — NOT a `bevy::Color`
//! — which S03's windowed binary maps verbatim to `Color::srgb(r, g, b)`. Only
//! `Vec2` (from the always-present `bevy_math`) crosses the seam.
//!
//! No `std::time`/`rand`: every cue is a pure function of tick counters (R004).

use std::collections::HashMap;

use bevy::math::Vec2;
use bevy::prelude::Resource;

/// An sRGB colour triple `(r, g, b)` with channels nominally in `0.0..=1.0`.
/// The lib seam stays render-type-free (R005); S03's windowed binary maps this
/// verbatim to `bevy::prelude::Color::srgb(r, g, b)`.
pub type SrgbTriple = (f32, f32, f32);

/// A single cosmetic cue primitive. Pure data — carries no Bevy render or
/// `bevy_enoki` types so it stays headless-buildable.
#[derive(Debug, Clone, PartialEq)]
pub enum CueDef {
    /// Colour flash lerping from white toward `peak` (sRGB triple) over `ticks`.
    Flash { peak: SrgbTriple, ticks: u32 },
    /// Positional sprite jitter: decaying sinusoid with per-axis frequencies.
    SpriteShake {
        amp: f32,
        freq_x: f32,
        freq_y: f32,
        ticks: u32,
    },
    /// Camera jitter; reuses the exact sprite-shake math (no separate kinematics).
    CameraShake {
        amp: f32,
        freq_x: f32,
        freq_y: f32,
        ticks: u32,
    },
    /// Particle burst keyed by an opaque effect id. The id is resolved to a
    /// `bevy_enoki` handle only in the windowed binary (S03) — the `String`
    /// here is the explicit enoki-isolation seam, never a handle.
    ParticleBurst { effect_id: String },
}

/// String id → [`CueDef`] lookup table. Populated at startup; fail-fast on
/// conflicting registrations (D047/D044).
#[derive(Resource, Default, Debug, Clone)]
pub struct CueRegistry {
    entries: HashMap<String, CueDef>,
}

impl CueRegistry {
    /// Register `def` under `id`.
    ///
    /// Idempotent and order-independent: re-registering the same id with an
    /// EQUAL def is a no-op. Registering a DIFFERENT def under an existing id is
    /// a startup-time programming error and panics, naming the conflicting id
    /// and both defs (fail-fast per D047/D044 — never a runtime path).
    pub fn register(&mut self, id: impl Into<String>, def: CueDef) {
        let id = id.into();
        match self.entries.get(&id) {
            Some(existing) if *existing == def => {} // idempotent no-op
            Some(existing) => panic!(
                "CueRegistry conflict for id {id:?}: already registered as \
                 {existing:?}, cannot re-register as {def:?}"
            ),
            None => {
                self.entries.insert(id, def);
            }
        }
    }

    /// Look up the cue registered under `id`. Returns `None` for unknown ids;
    /// the S03 caller logs and no-ops on `None` (this never panics).
    pub fn get(&self, id: &str) -> Option<&CueDef> {
        self.entries.get(id)
    }
}

/// sRGB tint applied to a struck sprite during a flash window. Lerps from white
/// `(1.0, 1.0, 1.0)` (no tint) at `remaining == 0` toward `peak` at
/// `remaining == total`. Returns `(1.0, 1.0, 1.0)` when not flashing.
///
/// Generalizes `hit_feedback::flash_tint`: with `peak == (1.0, 0.45, 0.45)`,
/// feeding this triple into `Color::srgb(r, g, b)` is bit-for-bit identical to
/// the legacy function (which builds `Color::srgb` from the same lerp).
pub fn flash_tint_parametric(remaining: u32, total: u32, peak: SrgbTriple) -> SrgbTriple {
    if remaining == 0 || total == 0 {
        return (1.0, 1.0, 1.0);
    }
    let t = (remaining as f32 / total as f32).clamp(0.0, 1.0);
    let lerp = |a: f32, b: f32| a + (b - a) * t;
    (lerp(1.0, peak.0), lerp(1.0, peak.1), lerp(1.0, peak.2))
}

/// Deterministic decaying jitter offset (R004 — no RNG): a fixed sinusoid scaled
/// by `remaining / total`, so amplitude decays to zero as the window drains.
/// Returns `Vec2::ZERO` when not shaking. `CameraShake` reuses this exact fn.
///
/// Generalizes `hit_feedback::shake_offset`: with `amp == 4.0, freq_x == 1.7,
/// freq_y == 2.3` it is bit-for-bit identical to the legacy function.
pub fn shake_offset_parametric(
    remaining: u32,
    total: u32,
    amp: f32,
    freq_x: f32,
    freq_y: f32,
) -> Vec2 {
    if remaining == 0 || total == 0 {
        return Vec2::ZERO;
    }
    let decay = (remaining as f32 / total as f32).clamp(0.0, 1.0);
    let amplitude = amp * decay;
    let phase = remaining as f32;
    Vec2::new(
        amplitude * (phase * freq_x).sin(),
        amplitude * (phase * freq_y).cos(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Legacy params from `hit_feedback` that the parametric forms must match.
    const LEGACY_FLASH_PEAK: SrgbTriple = (1.0, 0.45, 0.45);
    const LEGACY_SHAKE: (f32, f32, f32) = (4.0, 1.7, 2.3);

    #[test]
    fn flash_white_when_not_flashing() {
        assert_eq!(
            flash_tint_parametric(0, 8, LEGACY_FLASH_PEAK),
            (1.0, 1.0, 1.0)
        );
        assert_eq!(
            flash_tint_parametric(8, 0, LEGACY_FLASH_PEAK),
            (1.0, 1.0, 1.0)
        );
    }

    #[test]
    fn flash_peak_tint_at_full_remaining() {
        assert_eq!(
            flash_tint_parametric(8, 8, LEGACY_FLASH_PEAK),
            (1.0, 0.45, 0.45)
        );
    }

    #[test]
    fn flash_matches_legacy_across_window() {
        // Reproduce legacy `hit_feedback::flash_tint` math inline (returning the
        // pre-`Color::srgb` triple) and compare for every tick.
        let legacy = |remaining: u32, total: u32| -> SrgbTriple {
            if remaining == 0 || total == 0 {
                return (1.0, 1.0, 1.0);
            }
            let t = (remaining as f32 / total as f32).clamp(0.0, 1.0);
            let tint = (1.0_f32, 0.45_f32, 0.45_f32);
            let lerp = |a: f32, b: f32| a + (b - a) * t;
            (lerp(1.0, tint.0), lerp(1.0, tint.1), lerp(1.0, tint.2))
        };
        for remaining in 0..=8 {
            assert_eq!(
                flash_tint_parametric(remaining, 8, LEGACY_FLASH_PEAK),
                legacy(remaining, 8),
                "flash mismatch at remaining={remaining}"
            );
        }
    }

    #[test]
    fn shake_zero_when_not_shaking() {
        let (amp, fx, fy) = LEGACY_SHAKE;
        assert_eq!(shake_offset_parametric(0, 8, amp, fx, fy), Vec2::ZERO);
        assert_eq!(shake_offset_parametric(8, 0, amp, fx, fy), Vec2::ZERO);
    }

    #[test]
    fn shake_matches_legacy_across_window() {
        let (amp, fx, fy) = LEGACY_SHAKE;
        let legacy = |remaining: u32, total: u32| -> Vec2 {
            if remaining == 0 || total == 0 {
                return Vec2::ZERO;
            }
            let decay = (remaining as f32 / total as f32).clamp(0.0, 1.0);
            let amplitude = 4.0 * decay;
            let phase = remaining as f32;
            Vec2::new(
                amplitude * (phase * 1.7).sin(),
                amplitude * (phase * 2.3).cos(),
            )
        };
        for remaining in 0..=8 {
            assert_eq!(
                shake_offset_parametric(remaining, 8, amp, fx, fy),
                legacy(remaining, 8),
                "shake mismatch at remaining={remaining}"
            );
        }
    }

    #[test]
    fn shake_magnitude_within_decaying_envelope() {
        // Axes use distinct frequencies, so sin^2 + cos^2 != 1; the magnitude is
        // bounded by amplitude * sqrt(2), and that envelope decays with remaining.
        let (amp, fx, fy) = LEGACY_SHAKE;
        for remaining in 1..=8 {
            let mag = shake_offset_parametric(remaining, 8, amp, fx, fy).length();
            let envelope = amp * (remaining as f32 / 8.0) * std::f32::consts::SQRT_2;
            assert!(
                mag <= envelope + 1e-5,
                "magnitude {mag} exceeds envelope {envelope} at remaining={remaining}"
            );
        }
    }

    #[test]
    fn registry_get_unknown_returns_none() {
        let reg = CueRegistry::default();
        assert!(reg.get("nope").is_none());
    }

    #[test]
    fn registry_register_and_get() {
        let mut reg = CueRegistry::default();
        let def = CueDef::Flash {
            peak: (1.0, 0.45, 0.45),
            ticks: 8,
        };
        reg.register("hit_flash", def.clone());
        assert_eq!(reg.get("hit_flash"), Some(&def));
    }

    #[test]
    fn registry_register_idempotent_equal_def() {
        let mut reg = CueRegistry::default();
        let def = CueDef::ParticleBurst {
            effect_id: "baby_flame_impact".to_string(),
        };
        reg.register("burst", def.clone());
        // Re-registering the same id with an EQUAL def is a no-op (no panic).
        reg.register("burst", def.clone());
        assert_eq!(reg.get("burst"), Some(&def));
    }

    #[test]
    #[should_panic(expected = "CueRegistry conflict for id")]
    fn registry_register_conflict_panics() {
        let mut reg = CueRegistry::default();
        reg.register(
            "cue",
            CueDef::Flash {
                peak: (1.0, 0.45, 0.45),
                ticks: 8,
            },
        );
        reg.register(
            "cue",
            CueDef::Flash {
                peak: (1.0, 0.0, 0.0),
                ticks: 4,
            },
        );
    }
}
