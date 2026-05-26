---
id: T04
parent: S04
milestone: M006
key_files:
  - tests/windowed_only/agumon_module_extraction.rs
  - tests/windowed_only.rs
  - src/windowed/render.rs
key_decisions:
  - Use an include_str! source-contract test in tests/windowed_only to enforce the S04 Agumon extraction/no-hardcoding seam for binary-only windowed code, instead of runtime verification.
duration: 
verification_result: passed
completed_at: 2026-05-26T12:56:34.480Z
blocker_discovered: false
---

# T04: Added the agumon_module_extraction source-contract test, wired it into the windowed_only harness, and cleared the remaining engine-file AGUMON_ comment leakage so the S04 grep gate is enforced in CI.

**Added the agumon_module_extraction source-contract test, wired it into the windowed_only harness, and cleared the remaining engine-file AGUMON_ comment leakage so the S04 grep gate is enforced in CI.**

## What Happened

Created `tests/windowed_only/agumon_module_extraction.rs` as an `include_str!` source-contract test over `src/windowed/render.rs`, `src/windowed/mod.rs`, `src/windowed/digimon/mod.rs`, and `src/windowed/digimon/agumon/mod.rs`. The test asserts that the engine files no longer contain Agumon-specific tokens/helpers/paths, that the Agumon module now owns the registry-population tokens and atlas-path string, and that `src/windowed/digimon/mod.rs` still exposes the `mod agumon` + `fn register_all` seam. Registered the new case in `tests/windowed_only.rs`. While adding the contract, I found the last S04 grep-gate violations were comment-only `AGUMON_*` mentions in `src/windowed/render.rs`; I rewrote those comments to describe species-specific consts generically without changing behavior.

## Verification

Ran the narrow source-contract test first, then the full `windowed_only` integration harness, then the dependency-gating suite, and finally a `windowed` feature build. All checks passed: the new `agumon_module_extraction` test passed 3/3, `windowed_only` passed 62/62, `dependency_gating` passed 2/2, and `cargo build --features windowed` completed successfully with no warnings in output.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only agumon_module_extraction` | 0 | ✅ pass | 64164ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | ✅ pass | 2840ms |
| 3 | `cargo test --test dependency_gating` | 0 | ✅ pass | 26131ms |
| 4 | `cargo build --features windowed` | 0 | ✅ pass | 45125ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `tests/windowed_only/agumon_module_extraction.rs`
- `tests/windowed_only.rs`
- `src/windowed/render.rs`
