---
id: T02
parent: S05
milestone: M015
key_files:
  - src/bin/combat_cli.rs
key_decisions:
  - Use live ECS readiness (`DataReady`, spawned units, non-empty `ActionLog`) as the proof trigger before calling `capture_validation_snapshot(world)`, avoiding any CLI-only combat state or fake snapshot.
  - Return `AppExit` from `combat_cli` main so proof-mode failures become real process failures instead of marker-only diagnostics.
duration: 
verification_result: passed
completed_at: 2026-05-08T20:00:21.234Z
blocker_discovered: false
---

# T02: Added env-gated combat_cli proof mode that captures a live validation snapshot after shared action resolution and exits deterministically.

**Added env-gated combat_cli proof mode that captures a live validation snapshot after shared action resolution and exits deterministically.**

## What Happened

Added `BEVYROGUE_CLI_PROOF=1` gating in `src/bin/combat_cli.rs` with a bounded `BEVYROGUE_CLI_TICK_LIMIT` configuration resource and proof state. The proof system is an exclusive ECS observer that waits for shared runtime readiness (`DataReady`, spawned units, and non-empty `ActionLog`), then calls the existing `capture_validation_snapshot(world)` and prints `[CLI_PROOF] validation_snapshot: {format_validation_snapshot(...)}` from the live Bevy world before writing `AppExit::Success`. It does not construct a fake snapshot or introduce a CLI-only combat path; gameplay still flows through the shared bootstrap, action query, turn resolution, follow-up, ultimate, kernel, JSONL, and event systems.

Added explicit failure markers for bounded proof failures: `[CLI_PROOF] readiness_timeout:` includes readiness booleans/counters, and `[CLI_PROOF] validation_snapshot_error:` reports snapshot capture failures before writing `AppExit::error()`. During negative-path verification, Bevy local behavior showed that writing `AppExit::error()` from a system does not make the process non-zero unless `main` returns `app.run()`, so `combat_cli` now follows `src/main.rs` and returns `AppExit`. Proof mode also disables terminal prompts so a terminal-launched proof run remains deterministic, while non-proof mode keeps the existing interactive behavior and the old non-interactive timeout remains disabled only when proof config is present.

Added short binary unit tests for proof tick-limit parsing to pin positive, missing, zero, and invalid values.

## Verification

Verified the edited binary with `cargo check --bin combat_cli` and the new proof tick-limit unit tests with `cargo test --bin combat_cli proof_tick_limit`. Verified the authoritative real-binary proof command using `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` plus combined-output assertions: required `Action affordances`, `OnCombatBeat`, `OnKernelTransition`, `OnActionResolved`, `OnDamageDealt`, `[CLI_PROOF] validation_snapshot:`, and `holy_support=grace=` were present; forbidden `panicked`, `Message not initialized`, obsolete `[QUERY] Skill book unavailable`, `validation_snapshot_error`, and `readiness_timeout` markers were absent. Verified bounded failure behavior with `BEVYROGUE_CLI_TICK_LIMIT=1`: the underlying CLI exited non-zero and emitted `[CLI_PROOF] readiness_timeout:` without a snapshot marker. Verified non-proof non-interactive smoke still exits 0 and emits no `[CLI_PROOF]` markers.

Slice-level verification is partially advanced for T02: the runtime CLI proof command now passes. The `tests/combat_cli_shared_surface.rs`, docs proof page, and M015 verifier scripts are downstream S05 tasks (T03/T04), so they were not expected to pass or exist in this task.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --bin combat_cli` | 0 | ✅ pass | 313ms |
| 2 | `cargo test --bin combat_cli proof_tick_limit` | 0 | ✅ pass | 397ms |
| 3 | `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 BEVYROGUE_CLI_TICK_LIMIT=1 cargo run --bin combat_cli (wrapper asserted underlying exit=1, readiness_timeout marker present, snapshot marker absent)` | 0 | ✅ pass | 691ms |
| 4 | `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli (wrapper asserted required/forbidden shared-surface markers)` | 0 | ✅ pass | 492ms |
| 5 | `BEVYROGUE_JSONL=1 cargo run --bin combat_cli (wrapper asserted no CLI_PROOF markers in non-proof mode)` | 0 | ✅ pass | 6486ms |

## Deviations

Added `fn main() -> AppExit` propagation in `src/bin/combat_cli.rs` because local verification showed proof failure markers alone were not enough to produce a non-zero process status when `app.run()` was ignored. This supports the planned non-success failure requirement and stays within the expected output file.

## Known Issues

Existing unrelated library warnings remain during Cargo commands. Broad full-suite fixture drift noted in T01 remains outside this task and is S06-owned.

## Files Created/Modified

- `src/bin/combat_cli.rs`
