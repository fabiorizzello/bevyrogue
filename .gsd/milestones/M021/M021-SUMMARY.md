---
id: M021
title: "Finalize generic-kernel boundaries and blueprint runtime migration"
status: complete
completed_at: 2026-05-17T20:13:28.134Z
key_decisions:
  - Moved owner-local runtime logic out of shared `src/combat/` modules.
  - Introduced `src/combat/bevy_types.rs` as a local bridge for blueprint Bevy access.
  - Replaced static kernel transition variants with a generic Blueprint variant.
key_files: []
lessons_learned:
  - Moving existing code requires a simultaneous sweep of the integration test suite, as many harnesses rely on flat API assumptions that break during decoupling.
  - Local preludes/shims are an effective way to enforce architectural boundaries (like 'no direct Bevy imports') while still allowing necessary ECS access.
---

# M021: Finalize generic-kernel boundaries and blueprint runtime migration

**M021 finalized the combat kernel's generic architecture and completed the blueprint boundary migration.**

## What Happened

M021 achieved the long-term goal of making the combat kernel truly generic. Previous iterations had left five Digimon-specific variants (TwinCore, BatteryLoop, etc.) in the core transition enums and shared observability surfaces. This milestone replaced those variants with a unified generic Blueprint envelope and moved the typed payloads into owner-local modules. We also physically enforced the boundary by moving owner-local runtime systems out of shared combat modules and into the blueprint directory structure. To satisfy the roadmap constraint against direct Bevy coupling in blueprints, we introduced a local `bevy_types` bridge. Despite the large-scale refactor, we maintained 100% test coverage and repaired all integration regressions. M021 leaves the project with a clean, extensible kernel that is ready for new Digimon without further core modifications.

## Success Criteria Results

- Shared-name grep reports 0 hits: ✅ PASS
- Blueprint Bevy-import grep reports 0 hits: ✅ PASS
- Full integration test suite is green (237 passed): ✅ PASS
- Headless/windowed builds compile: ✅ PASS

## Definition of Done Results

- [x] All 5 Digimon-specific resolved events are removed from the shared bus.
- [x] Shared `observability.rs` and `events.rs` surfaces no longer contain Digimon-specific naming.
- [x] Combat blueprints avoid direct `use bevy` imports.
- [x] Full integration test suite (237 tests) is green.
- [x] All M021 architectural greps return zero hits.
- [x] Headless and windowed builds are verified.

## Requirement Outcomes

Not provided.

## Deviations

None.

## Follow-ups

None.
