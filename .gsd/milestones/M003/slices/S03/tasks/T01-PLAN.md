---
estimated_steps: 3
estimated_files: 4
skills_used: []
---

# T01: Add Bevy-free VfxSpawnDescriptor + resolve_locus and its headless structural test

Why: the VFX seam (Command::SpawnParticle) is fully defined but has no renderable counterpart and nothing in src/ consumes it; the lib needs a pure, Bevy-free descriptor so tests/ (lib-only link) can prove the seam yields a renderable spawn rather than an opaque ParticleId — this is the slice's only CI-provable deliverable and de-risks the windowed split, exactly as S01's AtlasGeometry did.

Do: Create src/animation/vfx.rs (NO bevy/2d imports — same discipline as src/animation/atlas.rs). Define `pub struct VfxSpawnDescriptor { pub particle: ParticleId, pub locus: VfxLocus, pub motion: VfxMotion }` with `pub fn from_command(cmd: &Command) -> Option<Self>` that returns Some only for Command::SpawnParticle (cloning name/origin/motion) and None otherwise, plus `pub fn is_renderable(&self) -> bool { true }` documenting the visual-component intent (the structural counterpart the headless test asserts vs. 'only an opaque ParticleId'). Add a pure `pub fn resolve_locus(locus: &VfxLocus, caster: [f32; 2], target: [f32; 2]) -> [f32; 2]` mapping CasterCenter->caster, TargetCenter->target, PrimaryTargetCenter->target (use plain [f32;2], no Bevy Vec2, so the windowed layer feeds it Transform translations). Doc-comment the file noting the real Sprite/Transform entity is built windowed-side and that only the three implemented VfxLocus/VfxMotion variants exist (do NOT reference the design-draft enums). Re-export via `pub use vfx::*;` in src/animation/mod.rs (after `pub use atlas::*` line). Then create tests/animation/vfx_spawn_descriptor.rs and register it in tests/animation.rs with `#[path = "animation/vfx_spawn_descriptor.rs"] mod vfx_spawn_descriptor;`. Tests assert: (a) from_command(&Command::SpawnParticle{..}) yields Some with is_renderable()==true and VfxLocus/VfxMotion/ParticleId preserved; (b) from_command on a non-SpawnParticle Command (e.g. Command::Shake or EmitDamage) returns None; (c) resolve_locus returns caster for CasterCenter and target for TargetCenter & PrimaryTargetCenter; (d) re-run the no-numeric-payload invariant against the descriptor's reconstructed/serialized form (build a Command::SpawnParticle from the descriptor with a non-numeric ParticleId, ron-serialize, assert no ascii digit and no 'Literal') — preserving vfx_handle_seam parity.

Done when: cargo test --test animation is green with the new test file present and all existing atlas_binding (6) and vfx_handle_seam (4) tests still passing, and `grep -L bevy::` confirms vfx.rs has no Bevy-2d import.

## Inputs

- `src/animation/anim_graph.rs`
- `src/animation/atlas.rs`
- `src/animation/mod.rs`
- `tests/animation.rs`
- `tests/animation/vfx_handle_seam.rs`
- `tests/animation/atlas_binding.rs`

## Expected Output

- `src/animation/vfx.rs`
- `src/animation/mod.rs`
- `tests/animation/vfx_spawn_descriptor.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation

## Observability Impact

None (pure lib + test; no runtime signals).
