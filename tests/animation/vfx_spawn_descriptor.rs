//! Structural test for the Bevy-free [`VfxSpawnDescriptor`] seam.
//!
//! Proves the `Command::SpawnParticle` seam yields a *renderable* spawn
//! descriptor (with `VfxLocus`/`VfxMotion`/`ParticleId` honored and
//! `is_renderable()`) rather than only an opaque `ParticleId`, that non-spawn
//! commands produce nothing, that `resolve_locus` maps the closed locus set, and
//! that the descriptor's reconstructed serialized form carries no numeric
//! gameplay payload — preserving `vfx_handle_seam` parity.

use bevyrogue::animation::anim_graph::{Command, ParamRef, ParticleId, VfxLocus, VfxMotion};
use bevyrogue::animation::vfx::{VfxSpawnDescriptor, resolve_locus};

/// `from_command` on a `SpawnParticle` yields `Some` with the id and closed
/// presentation enums preserved and `is_renderable() == true`.
#[test]
fn from_command_yields_renderable_descriptor_for_spawn_particle() {
    let cmd = Command::SpawnParticle {
        name: ParticleId("explosion_burst".to_string()),
        origin: VfxLocus::PrimaryTargetCenter,
        motion: VfxMotion::ArcToTarget,
    };

    let descriptor = VfxSpawnDescriptor::from_command(&cmd)
        .expect("SpawnParticle must distill into a VfxSpawnDescriptor");

    assert!(
        descriptor.is_renderable(),
        "a descriptor must denote a renderable spawn, not an opaque ParticleId"
    );
    assert_eq!(
        descriptor.particle,
        ParticleId("explosion_burst".to_string())
    );
    assert_eq!(descriptor.locus, VfxLocus::PrimaryTargetCenter);
    assert_eq!(descriptor.motion, VfxMotion::ArcToTarget);
}

/// `from_command` returns `None` for any command that is not `SpawnParticle`.
#[test]
fn from_command_returns_none_for_non_spawn_particle() {
    let shake = Command::Shake {
        intensity: ParamRef::Literal(3),
        duration_ms: ParamRef::Literal(120),
    };
    assert!(
        VfxSpawnDescriptor::from_command(&shake).is_none(),
        "Shake is not a particle spawn and must yield None"
    );

    let emit_damage = Command::EmitDamage {
        hits: ParamRef::Literal(1),
        mul: ParamRef::Literal(100),
        status: None,
        chance_pct: None,
        duration: None,
        target: bevyrogue::animation::anim_graph::TargetShape::Primary,
    };
    assert!(
        VfxSpawnDescriptor::from_command(&emit_damage).is_none(),
        "EmitDamage is not a particle spawn and must yield None"
    );
}

/// `resolve_locus` maps `CasterCenter` to the caster point and both target
/// variants to the target point, using plain `[f32; 2]`.
#[test]
fn resolve_locus_maps_the_closed_locus_set() {
    let caster = [10.0_f32, -4.0];
    let target = [88.0_f32, 5.0];

    assert_eq!(
        resolve_locus(&VfxLocus::CasterCenter, caster, target),
        caster
    );
    assert_eq!(
        resolve_locus(&VfxLocus::TargetCenter, caster, target),
        target
    );
    assert_eq!(
        resolve_locus(&VfxLocus::PrimaryTargetCenter, caster, target),
        target
    );
}

/// Reconstructing a `SpawnParticle` from the descriptor and serializing it must
/// carry no numeric gameplay payload — no `Literal` and no digit when the id is
/// non-numeric — re-running the `vfx_handle_seam` invariant on this path.
#[test]
fn reconstructed_descriptor_has_no_numeric_gameplay_payload() {
    let descriptor = VfxSpawnDescriptor::from_command(&Command::SpawnParticle {
        name: ParticleId("sparkle".to_string()),
        origin: VfxLocus::CasterCenter,
        motion: VfxMotion::FollowTarget,
    })
    .expect("SpawnParticle must distill into a descriptor");

    let reconstructed = Command::SpawnParticle {
        name: descriptor.particle.clone(),
        origin: descriptor.locus.clone(),
        motion: descriptor.motion.clone(),
    };

    let serialized =
        ron::ser::to_string(&reconstructed).expect("reconstructed SpawnParticle should serialize");

    assert!(
        !serialized.contains("Literal"),
        "reconstructed descriptor must not carry a ParamRef/Literal gameplay value; got {serialized}"
    );
    assert!(
        !serialized.chars().any(|c| c.is_ascii_digit()),
        "reconstructed descriptor with a non-numeric id must serialize without any digit; got {serialized}"
    );
}
