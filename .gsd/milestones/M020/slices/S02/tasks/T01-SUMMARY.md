---
id: T01
parent: S02
milestone: M020
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
  - Added pub use identity::{...} to blueprint mod.rs files rather than routing call-sites to ::identity:: sub-path, keeping the canonical surface at blueprints::<name>::<Type>
  - Removed shims atomically in a single commit unit; coexistence period (phase 1 + phase 2 in same edit) is not needed because all call-sites are updated in the same PR
duration: 
verification_result: passed
completed_at: 2026-05-14T10:48:33.839Z
blocker_discovered: false
---

# T01: Removed three legacy pub use shims (twin_core / holy_support / predator_loop) from src/combat/mod.rs and rerouted all 13 call-sites to canonical blueprints::<name>::<Type> paths.

**Removed three legacy pub use shims (twin_core / holy_support / predator_loop) from src/combat/mod.rs and rerouted all 13 call-sites to canonical blueprints::<name>::<Type> paths.**

## What Happened

Executed a three-phase compiler-driven refactor. Phase 1: extended agumon/mod.rs, patamon/mod.rs, and dorumon/mod.rs with targeted `pub use identity::{...}` blocks so that `combat::blueprints::<name>::<Type>` paths exist as stable canonical surface. Phase 2: removed the three `pub use blueprints::<name>::identity as <alias>` shims (plus their doc-comment blocks) from src/combat/mod.rs (lines ~83-108). Phase 3: updated all call-sites — 2 in src/ (gabumon.rs, observability.rs) and 11 in tests/ (twin_core_mechanics, twin_core_integration, holy_support_affordance, holy_support_mechanics, holy_support_resolution, patamon_blueprint_seam, dorumon_predator_runtime, predator_loop_kernel, validation_snapshot, status_observability_canon, presentation_metadata_boundary) — replacing the legacy `combat::twin_core::`, `combat::holy_support::`, and `combat::predator_loop::` prefixes with their canonical blueprints paths. The scout pass identified more affected files than the 9 listed in the plan (11 test files total), which were fixed by the compiler acting as oracle.

## Verification

cargo check (headless, exit 0), cargo check --features windowed (exit 0), cargo test (all tests pass, 0 failed), and `rg` grep on legacy patterns exits 1 (CLEAN: no occurrences found).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 1300ms |
| 2 | `cargo check --features windowed` | 0 | pass | 1460ms |
| 3 | `cargo test` | 0 | pass — all test suites ok, 0 failed | 60000ms |
| 4 | `! rg -n 'combat::twin_core|combat::holy_support|combat::predator_loop' src tests` | 0 | pass — zero legacy occurrences | 50ms |

## Deviations

Plan listed 9 affected test files; actual count was 11 (validation_snapshot.rs, status_observability_canon.rs, and presentation_metadata_boundary.rs also used legacy paths). All were fixed.

## Known Issues

none

## Files Created/Modified

- `src/combat/mod.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/gabumon.rs`
- `src/combat/observability.rs`
- `tests/twin_core_mechanics.rs`
- `tests/twin_core_integration.rs`
- `tests/holy_support_affordance.rs`
- `tests/holy_support_mechanics.rs`
- `tests/holy_support_resolution.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/predator_loop_kernel.rs`
- `tests/validation_snapshot.rs`
- `tests/status_observability_canon.rs`
- `tests/presentation_metadata_boundary.rs`
