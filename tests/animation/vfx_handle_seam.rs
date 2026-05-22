//! Evidence test for the VFX handle seam — the producer→consumer contract for
//! opaque presentation ids (`SpawnParticle { name, origin, motion }`).
//!
//! The seam must carry only an opaque `ParticleId` plus the *closed*
//! `VfxLocus`/`VfxMotion` presentation enums: gameplay numbers never leak
//! through it, and unknown variants must fail to deserialize so the consumer
//! surface stays closed. This test is cited by the M002 boundary map.

use bevyrogue::animation::anim_graph::{Command, ParticleId, VfxLocus, VfxMotion};

/// A `SpawnParticle` value round-trips losslessly through RON, preserving the
/// opaque `ParticleId(String)` and the chosen closed `VfxLocus`/`VfxMotion`
/// variants.
#[test]
fn spawn_particle_ron_round_trips_losslessly() {
    let original = Command::SpawnParticle {
        name: ParticleId("explosion_burst".to_string()),
        origin: VfxLocus::PrimaryTargetCenter,
        motion: VfxMotion::ArcToTarget,
    };

    let serialized = ron::ser::to_string(&original).expect("SpawnParticle should serialize to RON");
    let deserialized: Command =
        ron::de::from_str(&serialized).expect("serialized SpawnParticle should deserialize");

    assert_eq!(
        original, deserialized,
        "SpawnParticle must round-trip losslessly through RON; got {serialized}"
    );
}

/// Deserializing an unknown `VfxLocus` variant must fail — the locus vocabulary
/// is closed, so a windowed consumer can never receive an out-of-contract value.
#[test]
fn unknown_vfx_locus_variant_fails_to_deserialize() {
    let ron_str =
        r#"SpawnParticle(name: "explosion_burst", origin: OffscreenVoid, motion: Static)"#;
    let result: Result<Command, _> = ron::de::from_str(ron_str);
    assert!(
        result.is_err(),
        "unknown VfxLocus variant must fail to deserialize (closed-enum guarantee), got {result:?}"
    );
}

/// Deserializing an unknown `VfxMotion` variant must fail for the same reason.
#[test]
fn unknown_vfx_motion_variant_fails_to_deserialize() {
    let ron_str =
        r#"SpawnParticle(name: "explosion_burst", origin: CasterCenter, motion: TeleportBlink)"#;
    let result: Result<Command, _> = ron::de::from_str(ron_str);
    assert!(
        result.is_err(),
        "unknown VfxMotion variant must fail to deserialize (closed-enum guarantee), got {result:?}"
    );
}

/// The seam carries no numeric gameplay payload: with a non-numeric opaque id,
/// the serialized form holds only the id string and the closed enum variant
/// names — never a `ParamRef`/`Literal`/digit that could smuggle a game number.
#[test]
fn spawn_particle_has_no_numeric_gameplay_payload() {
    let cmd = Command::SpawnParticle {
        name: ParticleId("sparkle".to_string()),
        origin: VfxLocus::CasterCenter,
        motion: VfxMotion::FollowTarget,
    };

    let serialized = ron::ser::to_string(&cmd).expect("SpawnParticle should serialize to RON");

    assert!(
        !serialized.contains("Literal"),
        "SpawnParticle must not carry a ParamRef/Literal gameplay value; got {serialized}"
    );
    assert!(
        !serialized.chars().any(|c| c.is_ascii_digit()),
        "SpawnParticle with a non-numeric id must serialize without any digit \
         (no numeric gameplay payload leaks through the seam); got {serialized}"
    );
}
