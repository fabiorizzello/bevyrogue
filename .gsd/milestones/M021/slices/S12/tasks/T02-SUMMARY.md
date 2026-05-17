---
id: T02
parent: S12
milestone: M021
key_files:
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
key_decisions:
  - Expose validation diagnostics through the ExtRegistries ValidationExt axis so each blueprint contributes its own owner-keyed section.
  - Keep validation snapshot formatting deterministic by sorting owner sections and fields before rendering.
  - Treat missing validation contributors or resources as absence/none rather than shared hard failures.
duration: 
verification_result: passed
completed_at: 2026-05-17T09:11:10.141Z
blocker_discovered: false
---

# T02: Moved validation snapshots onto registry-owned owner sections and updated the validation fixture to install blueprint validation contributors.

**Moved validation snapshots onto registry-owned owner sections and updated the validation fixture to install blueprint validation contributors.**

## What Happened

Validation capture is now driven by ExtRegistries-owned ValidationExt contributors instead of shared hardcoded snapshot state. The combat runtime already wires all blueprint validation contributors into the registry, and the snapshot formatter consumes the registry-produced owner sections with deterministic sorting. I updated the validation snapshot test fixture to install the validation registry explicitly so direct-world tests exercise the new contract, then revalidated the full slice test set.

This keeps optional blueprint state absent when its contributor or resource is missing, while still rendering the owned sections when the registry is present. The direct validation snapshot expectations now match the registry-driven behavior again, including the Twin Core section in the explicit test fixtures.

## Verification

Ran cargo test --test validation_snapshot and the full requested slice command: cargo test --test validation_snapshot --test predator_loop_kernel --test patamon_blueprint_seam --test twin_core_integration --test dorumon_predator_runtime --test renamon_precision_runtime. Both passed after the fixture update, confirming the registry-owned validation sections render deterministically and the adjacent blueprint runtime seams remain green.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test validation_snapshot` | 0 | ✅ pass | 684ms |
| 2 | `cargo test --test validation_snapshot --test predator_loop_kernel --test patamon_blueprint_seam --test twin_core_integration --test dorumon_predator_runtime --test renamon_precision_runtime` | 0 | ✅ pass | 194ms |

## Deviations

Used the existing runtime registration path for validation contributors and adjusted the direct-world validation snapshot fixture to install ExtRegistries explicitly, rather than changing snapshot semantics to synthesize missing registry data.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/registry.rs`
- `src/combat/observability.rs`
- `src/combat/blueprints/mod.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/gabumon/mod.rs`
- `src/combat/blueprints/twin_core/mod.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/renamon.rs`
- `src/combat/blueprints/tentomon.rs`
- `tests/validation_snapshot.rs`
