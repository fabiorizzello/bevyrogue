---
id: S02
parent: M020
milestone: M020
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - src/combat/mod.rs
  - src/combat/blueprints/agumon/mod.rs
  - src/combat/blueprints/patamon/mod.rs
  - src/combat/blueprints/dorumon/mod.rs
  - src/combat/blueprints/gabumon.rs
  - src/combat/observability.rs
  - tests/twin_core_mechanics.rs
  - tests/twin_core_integration.rs
  - tests/holy_support_affordance.rs
  - tests/holy_support_mechanics.rs
  - tests/holy_support_resolution.rs
  - tests/patamon_blueprint_seam.rs
  - tests/dorumon_predator_runtime.rs
  - tests/predator_loop_kernel.rs
  - tests/validation_snapshot.rs
  - tests/status_observability_canon.rs
  - tests/presentation_metadata_boundary.rs
key_decisions:
  - Added pub use identity::{...} to blueprint mod.rs files rather than routing call-sites to ::identity:: sub-path, keeping the canonical surface at blueprints::<name>::<Type> and hiding the identity sub-module from consumers
  - All three phases (extend mod.rs re-exports, remove shims, update call-sites) executed atomically in one commit unit — coexistence period was unnecessary given single-PR scope
patterns_established:
  - Compiler-driven refactor: remove the public alias first in a local branch, let the compiler enumerate all call-sites, then fix them all in one pass — more reliable than grep-based enumeration for Rust where path resolution is context-sensitive
observability_surfaces:
  - none — purely compile-time refactor; no new runtime signals introduced
drill_down_paths:
  - .gsd/milestones/M020/slices/S02/tasks/T01-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-14T10:51:51.350Z
blocker_discovered: false
---

# S02: Rimozione shim pub use legacy (twin_core / holy_support / predator_loop)

**Removed three legacy pub use shims (twin_core/holy_support/predator_loop) from src/combat/mod.rs and rerouted all 13 call-sites (2 in src/, 11 in tests/) to canonical blueprints::<name>::<Type> paths — combat namespace is now legacy-free.**

## What Happened

T01 executed the three-phase compiler-driven refactor as a single atomic unit. Phase 1 extended each blueprint mod.rs (agumon, patamon, dorumon) with targeted pub use identity::{...} re-exports so canonical paths existed before the shims were removed. Phase 2 deleted the three pub use shim lines (and their doc-comments) from src/combat/mod.rs at ~lines 87–106. Phase 3 updated all call-sites: 2 in src/ (gabumon.rs and observability.rs) and 11 in tests/ (three more files than the plan anticipated — validation_snapshot.rs, status_observability_canon.rs, and presentation_metadata_boundary.rs also referenced legacy paths). All substitutions replaced crate::combat::twin_core::X with crate::combat::blueprints::agumon::X (and analogues for patamon/dorumon), and bevyrogue:: prefix in test files. The refactor was purely lexical — no runtime behaviour changed. HolySupportTransition was present in identity.rs and successfully re-exported. Coexistence period was not needed because all call-sites were updated in the same commit unit.

## Verification

cargo check (headless, exit 0) — warnings are pre-existing or new unused-imports on prophylactic re-exports, no new errors. cargo check --features windowed (exit 0). cargo test — all suites passed, 0 failures across 60+ integration test binaries (including the two largest suites at 198 and 199 tests). rg -n 'combat::twin_core|combat::holy_support|combat::predator_loop' src tests — exit 1 (CLEAN, zero matches).

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

["Plan listed 9 affected test files; actual count was 11 — validation_snapshot.rs, status_observability_canon.rs, and presentation_metadata_boundary.rs also used legacy paths. All were fixed."]

## Known Limitations

["Blueprint mod.rs files now carry unused-import warnings for prophylactic re-export entries (TAG_* constants, some transition fns). These are benign: the symbols are available on the canonical path even if no current call-site uses them. Can be trimmed in a future housekeeping pass if desired."]

## Follow-ups

["M021 can now consume blueprint types via combat::blueprints::<name>::<Type> without any shim intermediaries — the canonical surface is stable."]

## Files Created/Modified

- `src/combat/mod.rs` — Removed three pub use shim blocks (twin_core/holy_support/predator_loop aliases) and their doc-comments
- `src/combat/blueprints/agumon/mod.rs` — Added pub use identity::{TwinCoreState, TwinCoreDesignTag, TwinCoreHook, ...} re-exports
- `src/combat/blueprints/patamon/mod.rs` — Added pub use identity::{HolySupportState, HolySupportSnapshot, HolySupportTransition, ...} re-exports
- `src/combat/blueprints/dorumon/mod.rs` — Added pub use identity::{PredatorLoopState, PredatorLoopSnapshot, PredatorTargetSnapshot, ...} re-exports
- `src/combat/blueprints/gabumon.rs` — Updated import from crate::combat::twin_core to crate::combat::blueprints::agumon
- `src/combat/observability.rs` — Updated blueprint imports to canonical blueprints::<name> paths
- `tests/twin_core_mechanics.rs` — Updated bevyrogue::combat::twin_core → bevyrogue::combat::blueprints::agumon
- `tests/twin_core_integration.rs` — Updated to canonical agumon path
- `tests/holy_support_affordance.rs` — Updated to canonical patamon path
- `tests/holy_support_mechanics.rs` — Updated to canonical patamon path
- `tests/holy_support_resolution.rs` — Updated to canonical patamon path
- `tests/patamon_blueprint_seam.rs` — Updated to canonical patamon path
- `tests/dorumon_predator_runtime.rs` — Updated to canonical dorumon path
- `tests/predator_loop_kernel.rs` — Updated to canonical dorumon path
- `tests/validation_snapshot.rs` — Unplanned: also used legacy paths — updated to canonical paths
- `tests/status_observability_canon.rs` — Unplanned: also used legacy paths — updated to canonical paths
- `tests/presentation_metadata_boundary.rs` — Unplanned: also used legacy paths — updated to canonical paths
