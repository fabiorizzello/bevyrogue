---
id: T02
parent: S09
milestone: M021
key_files:
  - src/combat/blueprints/tentomon.rs
  - src/combat/battery_loop.rs
  - tests/tentomon_blueprint.rs
  - tests/battery_loop_kernel.rs
key_decisions:
  - Tentomon Battery Loop raw transitions now use the shared `CombatKernelTransition::Blueprint` envelope owned by `tentomon` with stable signal names.
  - Battery Loop runtime decoding is owner-gated: only Tentomon Blueprint transitions mutate BatteryLoopState, and foreign owners are ignored.
duration: 
verification_result: passed
completed_at: 2026-05-16T22:34:15.167Z
blocker_discovered: false
---

# T02: Moved Tentomon Battery Loop transport onto shared Tentomon-owned Blueprint transitions while preserving the deterministic state machine and passive block reactions.

**Moved Tentomon Battery Loop transport onto shared Tentomon-owned Blueprint transitions while preserving the deterministic state machine and passive block reactions.**

## What Happened

Reworked Tentomon's custom-signal dispatcher so `build_static_charge`, `build_circuit_charge`, and `spend_circuit_charge` now emit `CombatKernelTransition::Blueprint` events owned by `tentomon` with stable amount payloads. Updated `apply_battery_loop_transitions_system` to accept only Tentomon-owned Blueprint transitions, decode their names/payloads back into the existing BatteryLoop state machine, and ignore foreign owners or malformed payloads without mutating state. Changed the wrapped-cycle hook to emit a Tentomon-owned Blueprint `cycle_reset` transition instead of a kernel-local BatteryLoop transition. Refreshed the Tentomon and Battery Loop tests to assert the new raw transition shape, added a foreign-owner negative check, and kept the passive deterministic block-reaction coverage green without changing the mitigation path or `BlockReactionTriggered` diagnostics.

## Verification

Verified the transport migration by running the slice's focused integration tests: Tentomon blueprint mapping, Battery Loop kernel/runtime behavior, and passive reactive canon. All three suites passed, confirming the Blueprint envelope is emitted and consumed correctly, state mutation still converges once per event, wrapped-cycle reset still works, foreign Blueprint transitions are ignored, and passive block reaction determinism remains intact.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test tentomon_blueprint` | 0 | ✅ pass | 5304ms |
| 2 | `cargo test --test battery_loop_kernel` | 0 | ✅ pass | 631ms |
| 3 | `cargo test --test passive_reactive_canon` | 0 | ✅ pass | 966ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/blueprints/tentomon.rs`
- `src/combat/battery_loop.rs`
- `tests/tentomon_blueprint.rs`
- `tests/battery_loop_kernel.rs`
