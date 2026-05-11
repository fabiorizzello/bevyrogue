---
id: T03
parent: S05
milestone: M015
key_files:
  - tests/combat_cli_shared_surface.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-05-08T20:03:41.239Z
blocker_discovered: false
---

# T03: Added a real-binary combat_cli integration test that proves shared action, event, beat, kernel, and validation snapshot surfaces from a non-root cwd.

**Added a real-binary combat_cli integration test that proves shared action, event, beat, kernel, and validation snapshot surfaces from a non-root cwd.**

## What Happened

Created `tests/combat_cli_shared_surface.rs` as a targeted integration test that launches the Cargo-built `combat_cli` binary via `env!("CARGO_BIN_EXE_combat_cli")`. The test sets `BEVYROGUE_JSONL=1`, `BEVYROGUE_CLI_PROOF=1`, and a bounded `BEVYROGUE_CLI_TICK_LIMIT=120`, then runs the binary from `<manifest>/target` to prove the prior manifest-relative asset loading fix rather than relying on repository-root cwd luck. Runtime assertions inspect combined stdout/stderr for `Action affordances`, canonical event markers (`OnCombatBeat`, `OnKernelTransition`, `OnActionResolved`, `OnDamageDealt`, `OnSkillCast`), and the live `[CLI_PROOF] validation_snapshot:` marker with `holy_support=grace=`. Negative assertions explicitly fail on hidden-panic and startup drift markers (`panicked`, `Message not initialized`, obsolete skill-book fallback text, snapshot errors, readiness timeout, and generic proof failure text). Added a source-guard test that keeps `combat_cli` anchored to the shared query/snapshot surfaces and forbids direct mentions of presentation metadata authority fields (`animation_sequence`, `qte`).

## Verification

Ran the planned targeted command `cargo test --test combat_cli_shared_surface -- --nocapture` after creating the test file. It passed both tests: the source guard and the real-binary proof from non-root cwd. The proof test exercises the real CLI process and captures stdout/stderr for actionable failure output on launch, exit, missing marker, and forbidden marker failures.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test combat_cli_shared_surface -- --nocapture` | 0 | ✅ pass | 635ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `tests/combat_cli_shared_surface.rs`
