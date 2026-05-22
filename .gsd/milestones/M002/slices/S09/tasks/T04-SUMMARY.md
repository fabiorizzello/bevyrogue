---
id: T04
parent: S09
milestone: M002
key_files:
  - tests/animation/vfx_handle_seam.rs
  - tests/animation.rs
key_decisions:
  - Asserted absence of numeric gameplay payload structurally by checking the serialized RON contains no `Literal` and no ASCII digit when given a non-numeric ParticleId, rather than relying solely on the type system
  - Used inline RON string literals for unknown-variant negative cases to keep the test hermetic, matching anim_gameplay_command_forbidden.rs conventions
duration: 
verification_result: passed
completed_at: 2026-05-22T08:25:46.116Z
blocker_discovered: false
---

# T04: Added tests/animation/vfx_handle_seam.rs proving the SpawnParticle VFX seam round-trips losslessly through RON, rejects unknown closed-enum variants, and carries no numeric gameplay payload

**Added tests/animation/vfx_handle_seam.rs proving the SpawnParticle VFX seam round-trips losslessly through RON, rejects unknown closed-enum variants, and carries no numeric gameplay payload**

## What Happened

Created `tests/animation/vfx_handle_seam.rs` and registered it in `tests/animation.rs` via `#[path]` (R003). The test exercises the VFX handle seam (`Command::SpawnParticle { name: ParticleId, origin: VfxLocus, motion: VfxMotion }`) — the producer→consumer contract for opaque presentation ids — with four cases:

1. `spawn_particle_ron_round_trips_losslessly`: a `SpawnParticle` value serializes via `ron::ser::to_string` and deserializes via `ron::de::from_str` back to an equal `Command`, preserving the opaque `ParticleId(String)` and the chosen closed `VfxLocus`/`VfxMotion` variants.
2. `unknown_vfx_locus_variant_fails_to_deserialize`: an out-of-vocabulary locus (`OffscreenVoid`) fails to deserialize, proving the locus enum is closed.
3. `unknown_vfx_motion_variant_fails_to_deserialize`: an out-of-vocabulary motion (`TeleportBlink`) fails to deserialize, proving the motion enum is closed.
4. `spawn_particle_has_no_numeric_gameplay_payload`: with a non-numeric opaque id, the serialized form contains no `Literal` and no ASCII digit — observable evidence that no gameplay number (ParamRef/Literal) can leak through the seam.

Used inline RON string literals for the negative cases (hermetic, no asset files), consistent with the existing `anim_gameplay_command_forbidden.rs` and T03 fixture conventions. Imported the types through `bevyrogue::animation::anim_graph::{...}` as specified by the plan.

## Verification

Ran `cargo test --test animation vfx_handle_seam` — all 4 tests passed, 0 failed (47 filtered out). Exit code 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation vfx_handle_seam` | 0 | pass | 3339ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/animation/vfx_handle_seam.rs`
- `tests/animation.rs`
