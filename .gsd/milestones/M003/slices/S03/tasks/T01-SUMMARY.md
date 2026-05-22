---
id: T01
parent: S03
milestone: M003
key_files:
  - src/animation/vfx.rs
  - src/animation/mod.rs
  - tests/animation/vfx_spawn_descriptor.rs
  - tests/animation.rs
key_decisions:
  - vfx.rs already existed and matched the plan; only needed module wiring (pub mod vfx; pub use vfx::*;) rather than a rewrite
  - Used Command::Shake and Command::EmitDamage as the non-SpawnParticle None cases; EmitDamage fixture uses TargetShape::Primary (plan's PrimaryOnly does not exist)
duration: 
verification_result: passed
completed_at: 2026-05-22T12:33:13.522Z
blocker_discovered: false
---

# T01: Wired the pre-existing Bevy-free VfxSpawnDescriptor + resolve_locus into the lib and added its headless structural test proving the SpawnParticle seam yields a renderable descriptor

**Wired the pre-existing Bevy-free VfxSpawnDescriptor + resolve_locus into the lib and added its headless structural test proving the SpawnParticle seam yields a renderable descriptor**

## What Happened

The descriptor module `src/animation/vfx.rs` already existed on disk (matching the plan exactly: `VfxSpawnDescriptor{particle,locus,motion}`, `from_command` returning Some only for `Command::SpawnParticle`, `is_renderable()->true`, and a pure `resolve_locus(&VfxLocus, [f32;2], [f32;2]) -> [f32;2]` using plain arrays, no Bevy types). However it was NOT wired into the module tree. I added `pub mod vfx;` and `pub use vfx::*;` to `src/animation/mod.rs`.

I created `tests/animation/vfx_spawn_descriptor.rs` with four tests covering the plan's required assertions: (a) `from_command` on a SpawnParticle yields Some with is_renderable()==true and ParticleId/VfxLocus/VfxMotion preserved; (b) `from_command` on Command::Shake and Command::EmitDamage returns None; (c) resolve_locus maps CasterCenter->caster and TargetCenter/PrimaryTargetCenter->target; (d) reconstructs a SpawnParticle from the descriptor with a non-numeric ParticleId, ron-serializes it, and asserts no ascii digit and no "Literal" — re-running the vfx_handle_seam no-numeric-payload invariant on the descriptor path. Registered it in `tests/animation.rs` via `#[path = ...] mod vfx_spawn_descriptor;`.

Corrected one local mismatch: the plan's example used a TargetShape variant name `PrimaryOnly`; the actual enum variant is `Primary`, so the EmitDamage fixture uses `TargetShape::Primary`. rustfmt reordered the `pub use vfx::*;` line (placed after validation alphabetically) and the test import order — accepted those.

## Verification

Ran `cargo test --test animation`: 65 passed, 0 failed — including the 4 new vfx_spawn_descriptor tests, all 6 existing atlas_binding tests, and all 4 existing vfx_handle_seam tests still green. `grep -L bevy:: src/animation/vfx.rs` printed the filename, confirming the module has no Bevy-2d import (same discipline as atlas.rs). `cargo fmt -- --check` clean after formatting. `cargo clippy --tests` produces no warnings referencing vfx files (remaining warnings are pre-existing elsewhere in the crate).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation` | 0 | pass | 10ms |
| 2 | `grep -L bevy:: src/animation/vfx.rs` | 1 | pass (printed filename: no bevy:: import) | 50ms |
| 3 | `cargo fmt -- --check` | 0 | pass (after fmt applied) | 2000ms |
| 4 | `cargo clippy --tests (grep vfx)` | 0 | pass (no warnings on vfx files) | 120000ms |

## Deviations

Plan referenced TargetShape::PrimaryOnly which does not exist; used the actual variant TargetShape::Primary. vfx.rs was found pre-existing on disk (created earlier per recent commit) so this task only wired it in + added tests rather than authoring the module. rustfmt reordered the new re-export and test imports.

## Known Issues

none

## Files Created/Modified

- `src/animation/vfx.rs`
- `src/animation/mod.rs`
- `tests/animation/vfx_spawn_descriptor.rs`
- `tests/animation.rs`
