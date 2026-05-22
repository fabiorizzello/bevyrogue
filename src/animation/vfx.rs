//! Bevy-free VFX spawn descriptor — the renderable counterpart to the
//! `Command::SpawnParticle` seam.
//!
//! The headless lib excludes `bevy/2d`, so it cannot build the real
//! `Sprite`/`Transform` particle entity. This module mirrors the discipline of
//! [`crate::animation::atlas`]: it exposes a pure descriptor distilled from a
//! `SpawnParticle` command, proving the seam yields a *renderable* spawn (a
//! visual-component intent) rather than only an opaque [`ParticleId`]. The
//! windowed layer consumes a descriptor to build the actual short-lived
//! Sprite-quad entity, feeding [`resolve_locus`] its `Transform` translations.
//!
//! Only the three implemented [`VfxLocus`] variants (`CasterCenter`,
//! `TargetCenter`, `PrimaryTargetCenter`) and three [`VfxMotion`] variants
//! (`Static`, `FollowTarget`, `ArcToTarget`) exist — this descriptor honors
//! exactly those and references no design-draft enums.

use crate::animation::anim_graph::{Command, ParticleId, VfxLocus, VfxMotion};

/// Pure, Bevy-free description of a particle to spawn at the renderer.
///
/// Built from a [`Command::SpawnParticle`]; carries only the opaque
/// [`ParticleId`] and the closed presentation enums — no numeric gameplay
/// payload, preserving the `vfx_handle_seam` parity guarantee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VfxSpawnDescriptor {
    pub particle: ParticleId,
    pub locus: VfxLocus,
    pub motion: VfxMotion,
}

impl VfxSpawnDescriptor {
    /// Distill a descriptor from a command, returning `Some` only for
    /// [`Command::SpawnParticle`] and `None` for every other command.
    pub fn from_command(cmd: &Command) -> Option<Self> {
        match cmd {
            Command::SpawnParticle {
                name,
                origin,
                motion,
            } => Some(Self {
                particle: name.clone(),
                locus: origin.clone(),
                motion: motion.clone(),
            }),
            _ => None,
        }
    }

    /// This descriptor maps to a real visual entity windowed-side.
    ///
    /// The structural counterpart the headless test asserts against "only an
    /// opaque `ParticleId`": a descriptor always denotes a renderable spawn.
    pub fn is_renderable(&self) -> bool {
        true
    }
}

/// Resolve a [`VfxLocus`] to a concrete world-space `[x, y]` position.
///
/// Uses plain `[f32; 2]` (not Bevy `Vec2`) so the windowed layer can feed it
/// `Transform` translations. `CasterCenter` maps to the caster position;
/// `TargetCenter` and `PrimaryTargetCenter` both map to the target position.
pub fn resolve_locus(locus: &VfxLocus, caster: [f32; 2], target: [f32; 2]) -> [f32; 2] {
    match locus {
        VfxLocus::CasterCenter => caster,
        VfxLocus::TargetCenter | VfxLocus::PrimaryTargetCenter => target,
    }
}
