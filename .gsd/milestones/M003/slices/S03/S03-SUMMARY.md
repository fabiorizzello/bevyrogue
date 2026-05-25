---
id: S03
parent: M003
milestone: M003
provides:
  - Renderable particle spawning for both authored node-entry VFX and Baby Burner detonate flashes.
  - A headless-tested `VfxSpawnDescriptor`/`resolve_locus` seam that keeps windowed rendering out of lib-only tests while proving render intent.
  - The final missing visual surface required for M003 pending manual `cargo winx` confirmation.
requires:
  - slice: S01
    provides: The atlas-bound Agumon sprite presentation layer that the new particles resolve against for source/target world positions.
  - slice: S02
    provides: The existing rendered skill/ultimate timing and cue-release pipeline that now gains visible VFX on Baby Flame and Baby Burner.
affects:
  []
key_files:
  - src/animation/vfx.rs
  - src/animation/mod.rs
  - tests/animation/vfx_spawn_descriptor.rs
  - tests/animation.rs
  - src/windowed/render.rs
key_decisions:
  - Kept the VFX seam Bevy-free by validating `Command::SpawnParticle` -> `VfxSpawnDescriptor` in lib tests instead of pushing render types into headless code.
  - Reused the same `SpawnParticle`/`VfxSpawnDescriptor` seam for synthetic Baby Burner detonate flashes so authored and reactive VFX share one payload contract.
  - Shared a deterministic per-frame animation tick budget between atlas presentation and short-lived particle advancement so node-entry effects stay frame-accurate.
patterns_established:
  - Presentation-only combat VFX can share the authored `Command::SpawnParticle` -> `VfxSpawnDescriptor` seam instead of inventing a separate render payload.
  - When windowed side effects must track rendered animation frames, advance them from the same per-frame tick budget used by sprite playback.
  - Reactive UI-only triggers can be upgraded into world rendering by synthesizing the same headless-tested descriptor contract rather than bypassing the seam.
observability_surfaces:
  - `windowed.agumon_playback` trace logging on particle spawn and despawn.
  - `windowed.agumon_playback` debug logging when a SpawnParticle or detonate flash cannot resolve its target sprite.
drill_down_paths:
  - .gsd/milestones/M003/slices/S03/tasks/T01-SUMMARY.md
  - .gsd/milestones/M003/slices/S03/tasks/T02-SUMMARY.md
  - .gsd/milestones/M003/slices/S03/tasks/T03-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-22T13:32:09.829Z
blocker_discovered: false
---

# S03: S03

**Turned the inert VFX seam into renderable windowed particles, with a Bevy-free headless descriptor contract and shared tick-driven spawning for Baby Flame and Baby Burner flash effects.**

## What Happened

S03 closed the last missing M003 visual surface by connecting the existing animation-side particle seam to real windowed rendering without leaking Bevy dependencies into the lib contract. T01 kept the VFX boundary Bevy-free by wiring `src/animation/vfx.rs` through `src/animation/mod.rs` and adding headless structural coverage proving `Command::SpawnParticle` can be converted into a renderable `VfxSpawnDescriptor`, that `resolve_locus` preserves the authored `VfxLocus` and `VfxMotion` semantics, and that the serialized seam still carries no numeric gameplay payload. T02 extended `src/windowed/render.rs` so authored node-entry `SpawnParticle` commands are no longer inert: Agumon presentation detects entered nodes exactly once, resolves source/target sprite transforms, converts descriptors into short-lived colored sprite-quad particles, and advances/despawns them from a shared deterministic per-frame tick budget so presentation VFX stay frame-accurate with atlas playback. T03 then reused that same renderable seam for the Baby Burner detonate path by synthesizing a detonate `SpawnParticle`/`VfxSpawnDescriptor` from the existing `latest_baby_burner_flash_trigger` signal, spawning world particles while leaving the egui `BabyBurnerFlashState` chip path intact. The result is one unified particle pipeline for both authored timeline entry effects and reactive detonate flashes, with trace/debug observability on spawn, despawn, and missing-target cases, plus preserved headless/windowed boundary discipline.

## Verification

Passed all slice-level verification gates from the plan. `cargo test --test animation` passed with 65 tests green, including the structural `vfx_spawn_descriptor` coverage plus existing atlas-binding and VFX seam regression tests. Full `cargo test` passed across the project, confirming the new lib/windowed split did not leak windowed dependencies into headless paths. `cargo build --features windowed` completed successfully. `cargo test --features windowed` passed, covering the windowed helper/unit-test surface for particle spawning, locus resolution, detonate synthesis, and render-side regression checks. Observability surfaces were also verified in code: `src/windowed/render.rs` still emits `trace!`/`debug!` events on particle spawn, despawn, and unresolved-target cases under the existing `windowed.agumon_playback` target, and `src/ui/combat_panel/mod.rs` still contains the untouched `observe_baby_burner_flash` / `BabyBurnerFlashState` chip path alongside the new world-particle rendering.

## Requirements Advanced

- R012 — Extended the previously validated opaque particle-handle seam into a renderable descriptor contract and windowed particle pipeline while preserving the no-numeric-payload serialization invariant.
- R016 — Re-verified boundary hygiene through full headless tests plus windowed build/test coverage so the new particle rendering path did not leak windowed dependencies into headless code.

## Requirements Validated

- R012 — `cargo test --test animation` passed with the structural `vfx_spawn_descriptor` coverage plus existing VFX seam regression tests, confirming locus/motion preservation and no numeric gameplay payload in the serialized seam.
- R016 — `cargo test`, `cargo build --features windowed`, and `cargo test --features windowed` all passed, confirming no windowed dependency leak or regression from the new particle rendering path.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

Auto-mode cannot perform the required human visual confirmation of `cargo winx`; world-pixel verification remains a manual K001 gate even though the headless and windowed automated checks passed.

## Follow-ups

None.

## Files Created/Modified

- `src/animation/vfx.rs` — Exposed the Bevy-free particle descriptor/locus seam used by headless tests and windowed rendering.
- `src/animation/mod.rs` — Re-exported the VFX seam so the lib contract is reachable from tests and downstream rendering code.
- `tests/animation/vfx_spawn_descriptor.rs` — Added structural tests for renderable descriptor conversion, locus resolution, and no-numeric-payload guarantees.
- `tests/animation.rs` — Included the new VFX structural test module in the headless animation test target.
- `src/windowed/render.rs` — Consumed authored and synthetic particle events into short-lived sprite-quad VFX, added tick-driven motion/TTL handling, and preserved chip-path observability.
