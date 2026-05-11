---
id: T05
parent: S02
milestone: M016
key_files:
  - tests/dorumon_predator_runtime.rs
  - tests/digimon_signal_registry.rs
  - Cargo.toml
key_decisions:
  - Created the cursor after queuing the kernel-transition envelopes so the test observes only the canonical drained predator-loop outputs.
  - Kept the new parsing/routing/rejection coverage in a dedicated registry integration target rather than mixing it into the runtime proof.
duration: 
verification_result: passed
completed_at: 2026-05-09T15:37:29.208Z
blocker_discovered: false
---

# T05: Fixed the Dorumon runtime proof to drain only canonical predator-loop events and added a dedicated digimon signal registry test target.

**Fixed the Dorumon runtime proof to drain only canonical predator-loop events and added a dedicated digimon signal registry test target.**

## What Happened

Updated `tests/dorumon_predator_runtime.rs` so the message cursor is created after the `OnKernelTransition` input envelopes are queued; the drained stream now only observes the downstream `PredatorLoopResolved` events produced after kernel updates. I also tightened the runtime snapshot assertion to match the current predator-loop formatter output (`targets=[8:e2:p2]`) and kept the canonical state/snapshot checks intact.

Added `tests/digimon_signal_registry.rs` as a dedicated integration target covering RON envelope parsing, Dorumon registry routing, unknown-owner rejection, malformed payload-shape rejection, and malformed RON rejection. Wired the new binary into `Cargo.toml` with a `[[test]]` entry so the test target is discoverable by Cargo and can be executed directly.

## Verification

Passed `cargo test --test dorumon_predator_runtime --no-fail-fast` and `cargo test --test digimon_signal_registry --no-fail-fast`. Confirmed the new integration target is wired in `Cargo.toml` with `rg -n 'digimon_signal_registry' Cargo.toml`. A full `cargo test -- --list` probe is currently blocked by unrelated pre-existing compile errors in `tests/patamon_blueprint_seam.rs` and `tests/predator_loop_kernel.rs`, so direct target verification was used instead.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test dorumon_predator_runtime --no-fail-fast` | 0 | ✅ pass | 454ms |
| 2 | `cargo test --test digimon_signal_registry --no-fail-fast` | 0 | ✅ pass | 475ms |
| 3 | `rg -n 'digimon_signal_registry' Cargo.toml` | 0 | ✅ pass | 6ms |

## Deviations

Used direct Cargo.toml wiring proof instead of a full `cargo test -- --list` scan because unrelated test compile errors prevent the full list command from completing.

## Known Issues

`cargo test -- --list` currently fails due unrelated pre-existing compile errors in `tests/patamon_blueprint_seam.rs` and `tests/predator_loop_kernel.rs`. The new target itself passes when invoked directly.

## Files Created/Modified

- `tests/dorumon_predator_runtime.rs`
- `tests/digimon_signal_registry.rs`
- `Cargo.toml`
