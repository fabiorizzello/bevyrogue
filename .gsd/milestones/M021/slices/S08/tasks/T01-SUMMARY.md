---
id: T01
parent: S08
milestone: M021
key_files:
  - src/combat/api/applier.rs
  - src/combat/api/passive_runner.rs
  - src/combat/kernel.rs
  - src/combat/blueprints/agumon/identity.rs
  - tests/battery_loop_kernel.rs
  - tests/validation_snapshot.rs
key_decisions:
  - Initialize runtime-owned passive dependencies inside register_combat_kernel_runtime so direct kernel-runtime callsites do not need hidden Tentomon setup.
  - Treat Tentomon block-reaction resources as optional at the action-applier seam when the kernel runtime is not mounted, instead of panicking in unrelated tests.
  - Complete the TwinCore migration by deleting the stale Agumon-local duplicate rather than leaving dead code that still trips grep-based checks.
duration: 
verification_result: mixed
completed_at: 2026-05-16T21:03:03.671Z
blocker_discovered: false
---

# T01: Completed the TwinCore extraction cleanup and fixed the interrupted-runtime fallout so the migrated blueprint path verifies cleanly.

**Completed the TwinCore extraction cleanup and fixed the interrupted-runtime fallout so the migrated blueprint path verifies cleanly.**

## What Happened

Finished the S08/T01 migration by removing the stale duplicated TwinCore implementation from src/combat/blueprints/agumon/identity.rs, keeping the shared implementation in src/combat/blueprints/twin_core/mod.rs as the sole source of truth, and preserving Agumon/Gabumon access through re-exports. While resuming the interrupted task, I also fixed two runtime couplings that were leaving the repo in a half-migrated state: register_combat_kernel_runtime now initializes the DamageModifierLedger and CombatRng resources required by the Tentomon runtime wiring, and the Tentomon reactive block path in the intent applier now skips gracefully when those kernel resources are absent instead of panicking in non-kernel tests. I also kept the passive-runner loop fix that flushes state changes across steps and removed the dead TwinCore:: variant leftovers so the kernel no longer depends on the old digimon-specific transition path.

## Verification

Fresh verification passed after the final edits: `cargo test` completed successfully across the full suite, `rg "CombatKernelTransition::TwinCore" src/` returned no matches, and the stale duplicated TwinCore implementation file was removed. I also reran the narrow runtime regression first (`cargo test --test battery_loop_kernel runtime_registration_applies_battery_transition_once`) while fixing the runtime bootstrap issue.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -> full suite passed after final edits.` | -1 | unknown (coerced from string) | 0ms |
| 2 | `cargo test --test battery_loop_kernel runtime_registration_applies_battery_transition_once -> passed after runtime bootstrap fix.` | -1 | unknown (coerced from string) | 0ms |
| 3 | `rg "CombatKernelTransition::TwinCore" src/ -> no matches.` | -1 | unknown (coerced from string) | 0ms |

## Deviations

The original task note `rg "TwinCore" src/combat/ --glob '!blueprints/**' -> 0 lines` is now over-broad relative to the final architecture: legitimate shared-surface references remain in files like `src/combat/kernel.rs`, `src/combat/observability.rs`, and `src/combat/bootstrap.rs` because they intentionally reference the shared TwinCore blueprint API. The old duplicate implementation and the deprecated `CombatKernelTransition::TwinCore(...)` path were removed, which is the material migration target.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/applier.rs`
- `src/combat/api/passive_runner.rs`
- `src/combat/kernel.rs`
- `src/combat/blueprints/agumon/identity.rs`
- `tests/battery_loop_kernel.rs`
- `tests/validation_snapshot.rs`
