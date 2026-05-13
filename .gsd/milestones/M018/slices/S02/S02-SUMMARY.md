---
id: S02
parent: M018
milestone: M018
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - src/combat/unit.rs
  - src/combat/bootstrap.rs
  - tests/slot_index_tiebreak.rs
  - src/data/skills_ron.rs
  - src/combat/resolution.rs
  - src/combat/action_query.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - src/combat/follow_up.rs
  - tests/target_shape_blast_spillover.rs
  - tests/target_shape_aoe_all_order.rs
  - assets/data/skills.ron
  - src/bin/combat_cli.rs
key_decisions:
  - AoE(All) is a DSL alias for TargetShape::AllEnemies — no new variant added to avoid wide diff across 11 deferred skills
  - SlotIndex inserted post-spawn by apply_composition (not passed into spawn_unit_from_def) to keep 6+ test callers stable
  - Option (a) hoisting: SP/ult/streak consumed once before per-target loop; apply_damage_only() handles per-target damage only
  - SlotIndex added as 14th element to both ResolveActorsQuery aliases (turn_system/mod.rs, follow_up.rs) — must stay in sync
  - resolve_targets() is a pure fn over TargetableSnapshot (plain struct, no ECS) for testability without a running App
  - apply_damage_only() is a new public fn in resolution.rs; apply_effects() unchanged so all existing unit tests stay green
patterns_established:
  - Pure resolver pattern: pure fn + plain snapshot struct for multi-target resolution, testable without ECS
  - Resource-hoist pattern: consume-once before loop, damage-only per target
  - Post-spawn insertion pattern: use commands.entity().insert() for new components to avoid cascading signature changes
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-13T20:17:03.485Z
blocker_discovered: false
---

# S02: TargetShape resolver: Blast e AoE(All) con tie-break slot_index

**Extended TargetShape resolver to Blast and AoE(All) with slot_index tie-break, fan-out apply_effects in pipeline, and CLI aoe-blast scenario with byte-for-byte deterministic JSONL output.**

## What Happened

S02 delivered five cohesive tasks that together extend the combat resolver from Single-target-only to Blast (primary + slot_index ±1 spillover) and AoE/AllEnemies (every alive enemy), with full determinism and no resource-double-spending.

**T01 — SlotIndex Component:** A new `SlotIndex(u8)` Bevy Component was declared in `src/combat/unit.rs` (derives Ord+Hash for sorted consumers). `apply_composition` in `bootstrap.rs` assigns it post-spawn via `commands.entity().insert()` rather than threading it through `spawn_unit_from_def`, keeping the signature of 6+ existing test callers stable. A `slot_index_tiebreak` integration test asserts per-team ranges are exactly {0,1,2} for a 3+3 encounter.

**T02 — TargetShape::Blast + pure resolve_targets():** `TargetShape::Blast` was added to `skills_ron.rs`. The roadmap label "AoE(All)" is confirmed as a DSL alias for the existing `AllEnemies` variant — no new variant added, avoiding a wide diff across 11 deferred skills. `resolve_targets(shape, primary, snapshot) -> Vec<UnitId>` is a pure fn operating on a plain `TargetableSnapshot` struct (no ECS), enabling 6 table-driven tests covering edge slots, KO'd adjacents, and AllEnemies ordering without a running App.

**T03 — Validation gate widening:** Three sites previously gated non-Single shapes behind `UnimplementedTargetShape`: `skills_ron::validate_skill_def`, `resolution::target_shape_is_executable_now`, and `action_query::target_status_for_*`. All three now use the identical allowlist `Single|Blast|AllEnemies`; Row and SelfOnly remain deferred. One stale test assertion (checking for "TargetShape::Single" string in an error message about Row) was corrected to check for "Row".

**T04 — Fan-out in pipeline::step_app:** The single `apply_effects` call in the pipeline was split into Option (a) — hoisting: SP/ult charge/basic streak consumed once in Phase 1 (before the target loop), then `apply_damage_only()` (new public fn in `resolution.rs`) called per resolved target. Snapshot built in a read-only ECS pass; `actor_pairs Vec<(Entity, UnitId)>` pre-collected for entity lookup to avoid conflicting borrows. SlotIndex added as 14th element to both `ResolveActorsQuery` aliases (turn_system/mod.rs and follow_up.rs). Two new integration tests: `target_shape_blast_spillover` (4 scenarios inc. KO adjacent absorb) and `target_shape_aoe_all_order` (slot_index ordering stability).

**T05 — Fixture skills + combat_cli aoe-blast:** Added `nova_burst` (Blast) and `dark_flood` (AllEnemies/AoE) as Implemented fixture skills in `assets/data/skills.ron`, with `Effect::Damage{target}` fields consistent with their shapes. The `combat_cli --scenario aoe-blast` dispatcher branch resolves targets, applies per-target damage, and emits JSONL. Output is 25 lines per run, byte-for-byte identical across 10 invocations — determinism gate passed.

## Verification

All verification checks passed:
- `cargo test --test target_shape_blast_spillover --test target_shape_aoe_all_order --test slot_index_tiebreak`: 6 tests across 3 binaries, all passed (exit 0).
- `cargo test` full suite: 0 failures, all test binaries green including M017 regression tests (status_slowed_delay: 1 passed, tempo_resistance: 14 passed, turn_advance_split: 6 passed).
- `cargo check --features windowed`: Finished cleanly, warnings only, 0 errors.
- `combat_cli --scenario aoe-blast` × 2, diff: DETERMINISM PASS — 25 lines of JSONL, byte-for-byte identical. Output shows resolved target list (slot_index asc) and per-target damage step-by-step.
- Greppable invariant confirmed: Blast|AllEnemies present at all three gate sites in skills_ron.rs, resolution.rs, action_query.rs.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

spawn_unit_from_def signature not changed; SlotIndex inserted post-spawn by apply_composition caller. One stale test assertion updated: validate_rejects_implemented_non_single_shape checked for "TargetShape::Single" string (old error message fragment) — updated to check for "Row" (the shape actually under test).

## Known Limitations

ApplyStatus effects not fanned out per target in the multi-target loop (only damage is) — deferred; Bounce(N) and extended selectors are S03/S04 scope.

## Follow-ups

None.

## Files Created/Modified

None.
