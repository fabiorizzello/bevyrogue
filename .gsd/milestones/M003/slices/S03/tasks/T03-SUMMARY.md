---
id: T03
parent: S03
milestone: M003
key_files:
  - src/windowed/render.rs
key_decisions:
  - Reused the existing public `latest_baby_burner_flash_trigger` helper so the detonate world-VFX path mirrors the egui chip trigger extraction without altering chip behavior.
  - Synthesized the Baby Burner detonate particle through a `Command::SpawnParticle` -> `VfxSpawnDescriptor` seam to preserve the no-numeric-payload contract already established for windowed VFX.
duration: 
verification_result: passed
completed_at: 2026-05-22T13:30:01.065Z
blocker_discovered: false
---

# T03: Added Baby Burner detonate world-particle spawning to the windowed renderer while keeping the existing egui flash chip path untouched.

**Added Baby Burner detonate world-particle spawning to the windowed renderer while keeping the existing egui flash chip path untouched.**

## What Happened

Added a new windowed `spawn_detonate_particles` system in `src/windowed/render.rs` and registered it in `RenderPlugin` after `resolve_action_system`, so Baby Burner detonate blueprint signals now spawn short-lived world-space sprite particles alongside the pre-existing egui flash chip. The implementation reuses the public `latest_baby_burner_flash_trigger(events.read())` helper, resolves caster/target sprite transforms from the existing `AgumonSprite` query, synthesizes a `baby_burner_detonate` `VfxSpawnDescriptor` via a `Command::SpawnParticle`, and feeds it into the T02 `spawn_vfx_particle` helper with `VfxLocus::TargetCenter` and `VfxMotion::Static`. Added trace logging for each detonate-driven particle spawn plus debug logging when a source or target sprite cannot be resolved. Also added a unit test proving the synthesized detonate descriptor is renderable, keeps `TargetCenter`/`Static`, and that a non-numeric particle-id `SpawnParticle` serialization contains no ASCII digits, preserving the no-numeric gameplay-payload seam.

## Verification

Verified a clean windowed build, the windowed binary test target, and the untouched egui chip path. `cargo build --features windowed` succeeded after the render-system changes. `cargo test --features windowed --bin bevyrogue` passed all 18 tests, including the new detonate descriptor structural test and the existing T02 render tests. A grep of `src/ui/combat_panel/mod.rs` confirmed `observe_baby_burner_flash` and `BabyBurnerFlashState` remain present and unmodified as the parallel chip path.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | ✅ pass | 1498ms |
| 2 | `cargo test --features windowed --bin bevyrogue` | 0 | ✅ pass | 1159ms |
| 3 | `rg -n "observe_baby_burner_flash|BabyBurnerFlashState" src/ui/combat_panel/mod.rs` | 0 | ✅ pass | 4ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/windowed/render.rs`
