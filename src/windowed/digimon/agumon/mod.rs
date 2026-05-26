//! Agumon presentation: cosmetic cues, effect-id mappings, and skill-node
//! vocabulary. Registered into the generic engine registries via `register`.
//!
//! S04 extracts this out of the windowed engine files (`src/windowed/mod.rs`,
//! `src/windowed/render.rs`); the engine systems stay generic and read from the
//! registries this module populates.

use bevy::prelude::*;

/// Populate the engine registries with all Agumon-specific presentation data.
/// Called once from `crate::windowed::digimon::register_all`.
pub(in crate::windowed) fn register(app: &mut App) {
    app.add_systems(Startup, register_agumon_cues);
}

/// Register the three Agumon-specific cosmetic cues with the legacy
/// `hit_feedback` const values: the colour flash, the positional sprite-shake,
/// and the camera-shake. Behaviour-preserving param sourcing (D048 model a) —
/// the parametric fns at these params are bit-for-bit identical to the legacy
/// `flash_tint`/`shake_offset`. Registration is collision-free (D047 panics on
/// a conflicting def).
fn register_agumon_cues(mut registry: ResMut<bevyrogue::ui::cues::CueRegistry>) {
    use bevyrogue::ui::cues::CueDef;
    registry.register(
        "hit_flash",
        CueDef::Flash {
            peak: (1.0, 0.45, 0.45),
            ticks: 8,
        },
    );
    registry.register(
        "hit_shake",
        CueDef::SpriteShake {
            amp: 4.0,
            freq_x: 1.7,
            freq_y: 2.3,
            ticks: 8,
        },
    );
    registry.register(
        "camera_impact",
        CueDef::CameraShake {
            amp: 4.0,
            freq_x: 1.7,
            freq_y: 2.3,
            ticks: 8,
        },
    );
}
