---
id: T01
parent: S04
milestone: M005
key_files:
  - Cargo.toml
  - tests/dependency_gating.rs
key_decisions:
  - Gate bevy_enoki strictly behind the windowed feature via dep:bevy_enoki (R005), leaving headless/dev/default feature sets untouched
  - Use cargo tree --invert with --offline as the regression guard; assert on exit status + stdout-contains rather than cargo's exact error string (not a stable contract)
  - Place the dep-gating test as its own standalone single-binary domain (R003) so it runs on the default headless build, not the windowed harness
duration: 
verification_result: passed
completed_at: 2026-05-26T09:18:19.922Z
blocker_discovered: false
---

# T01: Added bevy_enoki 0.6 as a windowed-only optional dep and a standalone cargo-tree dep-gating test proving it never leaks into the headless graph

**Added bevy_enoki 0.6 as a windowed-only optional dep and a standalone cargo-tree dep-gating test proving it never leaks into the headless graph**

## What Happened

Retired the central S04 risk (bevy_enoki's full-render-stack dep leaking into the headless build) before any effect authoring. Two Cargo.toml edits: added `bevy_enoki = { version = "0.6", optional = true }` to [dependencies] and appended `"dep:bevy_enoki"` to the `windowed` feature list. The headless `bevy` feature set, `dev`, and `default` were left untouched. Created `tests/dependency_gating.rs` as its own single-binary test domain (R003) — not under `tests/windowed_only/` and not gated by `#![cfg(feature = "windowed")]`, so it runs on the default `dev` headless build. The test shells out (via std::process::Command, `--offline`) to `cargo tree -e normal --no-default-features --features <set> --invert bevy_enoki` twice: the `dev` query must FAIL / not contain bevy_enoki (absence), and the `windowed` query must succeed and contain bevy_enoki (presence). Assertions key on exit status + stdout-contains, not cargo's exact error phrasing, and log captured stdout/stderr on failure so a future agent can read the graph without re-running. An online `cargo tree --features windowed` confirmed bevy_enoki resolves to v0.6.0 and populated the lockfile/registry so the `--offline` test queries succeed.

## Verification

Ran `cargo tree -e normal --no-default-features --features windowed --invert bevy_enoki` (online): resolved bevy_enoki v0.6.0, present. Ran the headless `--features dev` invert query: exit 101, "package ID specification `bevy_enoki` did not match any packages" (absent). `cargo test --test dependency_gating`: 2 passed, 0 failed. `cargo check` (headless default `dev`): finished clean, bevy_enoki not in the graph, manifest valid.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test dependency_gating` | 0 | pass | 290ms |
| 2 | `cargo tree -e normal --no-default-features --features dev --invert bevy_enoki` | 101 | pass (expected absence — no match) | 1500ms |
| 3 | `cargo tree -e normal --no-default-features --features windowed --invert bevy_enoki` | 0 | pass (bevy_enoki v0.6.0 present) | 8000ms |
| 4 | `cargo check` | 0 | pass (headless build valid, no enoki) | 3830ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `Cargo.toml`
- `tests/dependency_gating.rs`
