---
id: T05
parent: S10
milestone: M021
key_files:
  - src/combat/observability.rs
  - tests/validation_snapshot.rs
  - tests/combat_cli_shared_surface.rs
  - tests/holy_support_affordance.rs
  - tests/holy_support_resolution.rs
  - tests/patamon_blueprint_seam.rs
  - tests/renamon_precision_runtime.rs
  - tests/dorumon_predator_runtime.rs
  - tests/predator_loop_kernel.rs
  - tests/holy_support_mechanics.rs
  - tests/presentation_metadata_boundary.rs
key_decisions:
  - Use generic shared labels (`support`, `predator`, `mind_game`, `battery`) in validation/CLI output while leaving blueprint-owned mechanics and signal names unchanged.
  - Assert the rendered diagnostic contract in tests rather than the retired mechanic identifiers.
duration: 
verification_result: mixed
completed_at: 2026-05-17T06:41:28.296Z
blocker_discovered: true
---

# T05: Genericized validation/CLI snapshot labels to support/predator/mind_game/battery and updated the affected snapshot assertions.

**Genericized validation/CLI snapshot labels to support/predator/mind_game/battery and updated the affected snapshot assertions.**

## What Happened

Updated `src/combat/observability.rs` so the rendered validation snapshot uses generic shared labels instead of the retired mechanic names. Aligned the CLI proof check and the Patamon/Renamon/Dorumon-facing snapshot assertions to the new diagnostic contract across the touched tests. I also kept the underlying blueprint-owned state and signal names intact; only the shared rendered surface changed.

## Verification

`cargo test --test validation_snapshot` and `cargo test --test combat_cli_shared_surface` passed. `cargo check` and `cargo check --features windowed` passed. The exact structural grep gate from the task plan still reports canonical shared-module type names outside blueprints, so that literal repository-wide check remains noisy even though the shared validation/CLI surfaces were updated.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test validation_snapshot` | 0 | ✅ pass | 3144ms |
| 2 | `cargo test --test combat_cli_shared_surface` | 0 | ✅ pass | 644ms |
| 3 | `cargo check` | 0 | ✅ pass | 4771ms |
| 4 | `cargo check --features windowed` | 0 | ✅ pass | 6039ms |
| 5 | `rg -n "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat --glob '!src/combat/blueprints/**'` | 1 | ❌ fail | 19ms |

## Deviations

The task-plan grep gate is broader than the surfaces changed here and still matches canonical shared-module type names in non-blueprint runtime files.

## Known Issues

The literal `rg` gate from the plan still finds `TwinCore`, `BatteryLoop`, `HolySupport`, `PredatorLoop`, `PrecisionMindGame`, and `KitsuneGrace` in shared runtime modules outside `blueprints/`; these are canonical module/type names rather than validation/CLI output strings.

## Files Created/Modified

- `src/combat/observability.rs`
- `tests/validation_snapshot.rs`
- `tests/combat_cli_shared_surface.rs`
- `tests/holy_support_affordance.rs`
- `tests/holy_support_resolution.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/renamon_precision_runtime.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/predator_loop_kernel.rs`
- `tests/holy_support_mechanics.rs`
- `tests/presentation_metadata_boundary.rs`
