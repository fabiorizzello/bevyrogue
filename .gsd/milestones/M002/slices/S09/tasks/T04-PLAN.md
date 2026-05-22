---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: VFX handle seam serde round-trip evidence test

Why: the VFX seam (`SpawnParticle { name: ParticleId, origin: VfxLocus, motion: VfxMotion }`) is the producer→consumer contract for opaque presentation ids — gameplay numbers never leak through it and rendering is no-op/validate-only until a windowed consumer exists; the boundary map must cite an enforcing test. Skills: tdd. Do: add `tests/animation/vfx_handle_seam.rs` (register in `tests/animation.rs` via `#[path]`) using `bevyrogue::animation::anim_graph` types: assert a `SpawnParticle` value round-trips losslessly through RON (`ron::ser::to_string` then `ron::de::from_str`) preserving the opaque `ParticleId(String)` and the closed `VfxLocus`/`VfxMotion` variants; assert that deserializing an unknown `VfxLocus`/`VfxMotion` variant string fails (closed-enum guarantee); and assert there is no numeric gameplay payload on the variant (only an opaque id + closed presentation enums). Done-when: `cargo test --test animation vfx_handle_seam` green.

## Inputs

- `src/animation/anim_graph.rs`

## Expected Output

- `tests/animation/vfx_handle_seam.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation vfx_handle_seam

## Observability Impact

N/A — pure contract test, no runtime signals.
