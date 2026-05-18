---
id: S12
parent: M021
milestone: M021
provides:
  - Generic roster metadata for future blueprint owners.
  - Registry-owned validation sections for downstream proof and CLI surfaces.
  - Deterministic snapshot output that stays diff-friendly in minimal worlds.
requires:
  []
affects:
  []
key_files:
  - src/data/units_ron.rs
  - src/combat/api/registry.rs
  - src/combat/observability.rs
  - src/combat/blueprints/mod.rs
  - src/combat/blueprints/agumon/mod.rs
  - src/combat/blueprints/gabumon/mod.rs
  - src/combat/blueprints/twin_core/mod.rs
  - src/combat/blueprints/patamon/mod.rs
  - src/combat/blueprints/dorumon/mod.rs
  - src/combat/blueprints/renamon.rs
  - src/combat/blueprints/tentomon.rs
  - tests/validation_snapshot.rs
  - tests/holy_support_affordance.rs
  - tests/holy_support_mechanics.rs
  - tests/holy_support_resolution.rs
  - tests/combat_cli_shared_surface.rs
key_decisions:
  - Use owner-keyed generic roster metadata so RON round-trips stay deterministic without digimon-specific fields.
  - Expose validation diagnostics through ExtRegistries ValidationExt contributors and sort owner sections/fields before rendering.
  - Treat missing optional validation contributors or resources as absence/none rather than shared hard failures.
patterns_established:
  - Registry-driven validation snapshots should be tested with explicit ExtRegistries installation in direct-world fixtures.
  - Owner-keyed validation sections must render deterministically regardless of contributor registration order.
  - Retired shared snapshot fields should be removed from both source and tests, not just ignored.
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-17T09:12:02.840Z
blocker_discovered: false
---

# S12: RosterEntry blueprint-keyed + ValidationSnapshot from registry

**Removed digimon-named roster and validation seams; validation snapshots now come from registry-owned owner-keyed sections, and the focused proof suite stays green.**

## What Happened

This slice closed the shared-schema and shared-observability seam cleanup for M021. On the roster side, the shared UnitDef blueprint metadata moved to an owner-keyed generic payload, with deterministic ordering and backward-compatible empty defaults so constructors and fixtures no longer name Twin Core or Holy Support directly. On the validation side, snapshot capture now consumes registry-owned ValidationExt contributors, and the formatter renders owner-keyed sections in deterministic order while treating missing optional blueprint state as absence instead of a shared crash. The proof layer was then realigned to the new contract: holy-support tests now assert through owner-keyed validation sections, the CLI/shared-surface checks reject retired snapshot fields, and the focused validation/runtime suite confirms the registry-driven contract together with adjacent blueprint seams. The final closeout run also rechecked the structural greps and both cargo check modes to confirm the old named seams are gone and the build remains healthy.

## Verification

Verified with gsd_exec evidence: structural greps returned zero matches for retired roster and validation seams (`roster_grep_count=0`, `validation_grep_count=0`), and the focused closeout suite passed (`cargo test --test validation_snapshot --test predator_loop_kernel --test patamon_blueprint_seam --test twin_core_integration --test dorumon_predator_runtime --test renamon_precision_runtime` => OK; `cargo check` => OK; `cargo check --features windowed` => OK). Earlier task summaries also recorded passing evidence for the roster-focused test set and the CLI/shared-surface proof set, so the full slice remains green end-to-end.

## Requirements Advanced

- M021 shared-schema and shared-observability cleanup advanced by removing digimon-named roster/validation seams. — 

## Requirements Validated

- M021 slice contract validated by passing focused tests, cargo checks, and structural greps. — 

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

- `src/data/units_ron.rs` — owner-keyed blueprint roster metadata
- `src/combat/api/registry.rs` — ValidationExt axis
- `src/combat/observability.rs` — registry-driven snapshot capture/formatting
- `tests/validation_snapshot.rs` — explicit registry-installing fixture and expectations
- `tests/holy_support_affordance.rs` — owner-keyed validation assertions
- `tests/combat_cli_shared_surface.rs` — retired-field surface checks
