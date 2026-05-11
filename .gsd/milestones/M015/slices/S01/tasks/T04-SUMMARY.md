---
id: T04
parent: S01
milestone: M015
key_files:
  - scripts/verify_m015_failure_ledger.py
  - docs/m015_failure_ledger.md
key_decisions:
  - Scope placeholder detection to structured table cells so the verifier does not false-positive on the ledger's own guardrail prose.
  - Keep the stale `battery_loop_resolution` manifest target absent and verify replacement coverage through `tests/battery_loop_kernel.rs` instead.
duration: 
verification_result: mixed
completed_at: 2026-05-08T11:01:29.525Z
blocker_discovered: false
---

# T04: Added a deterministic M015 failure-ledger verifier and refreshed the ledger with fresh pass/fail evidence.

**Added a deterministic M015 failure-ledger verifier and refreshed the ledger with fresh pass/fail evidence.**

## What Happened

I created `scripts/verify_m015_failure_ledger.py` as a lightweight tracked checker that reads only `docs/m015_failure_ledger.md` and `Cargo.toml`. The script validates the required S01 ledger sections, checks that the verification snapshot contains the fresh command evidence rows and pass/fail markers, rejects placeholder classifications in structured table cells, and fails if `Cargo.toml` still references the removed `battery_loop_resolution` target. After the first run exposed a false positive from the ledger's own guardrail prose, I narrowed the placeholder check to table-style classification cells and reran the verifier successfully. I then ran `cargo test --test battery_loop_kernel` and `cargo test --no-run`, updated the ledger's verification snapshot with the fresh results, and reran the verifier to confirm the saved ledger still passes. The no-run sweep still fails on the current compile-red inventory, which remains classified in the ledger and is no longer attributed to the removed manifest target.

## Verification

Fresh verification completed successfully for the tracked ledger checker and the replacement battery-loop test. `python3 scripts/verify_m015_failure_ledger.py` passed after narrowing placeholder detection to structured classification cells. `cargo test --test battery_loop_kernel` passed with all four tests green. `cargo test --no-run` still exits 101, and the ledger records the current compile reds as classified blockers rather than the stale `battery_loop_resolution` manifest issue.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `python3 scripts/verify_m015_failure_ledger.py` | 0 | ✅ pass | 18ms |
| 2 | `cargo test --test battery_loop_kernel` | 0 | ✅ pass | 170ms |
| 3 | `cargo test --no-run` | 101 | ❌ fail | 3228ms |

## Deviations

I tightened the placeholder-classification check after the first verifier pass false-positive, limiting it to table-style classification cells so the ledger's own guardrail prose does not trip the checker.

## Known Issues

`cargo test --no-run` still fails on the existing mechanical fixture drift / runtime-red inventory recorded in `docs/m015_failure_ledger.md`; that is expected for this slice and is not caused by the removed `battery_loop_resolution` target.

## Files Created/Modified

- `scripts/verify_m015_failure_ledger.py`
- `docs/m015_failure_ledger.md`
