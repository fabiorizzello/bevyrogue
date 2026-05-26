---
id: T03
parent: S05
milestone: M005
key_files:
  - src/windowed/render.rs
  - tests/dependency_gating.rs
  - tests/windowed_only.rs
key_decisions:
  - No source changes needed — this is a verification-only sweep; all four build/test flavors were already green against the T01/T02 migration.
  - K001 windowed visual sign-off is deferred to the user as a manual UAT step, per the never-run-windowed-from-auto-mode rule.
duration: 
verification_result: passed
completed_at: 2026-05-26T09:47:52.930Z
blocker_discovered: false
---

# T03: Ran the four-command full-migration regression sweep (all green) and documented the K001 manual windowed sign-off the user must perform.

**Ran the four-command full-migration regression sweep (all green) and documented the K001 manual windowed sign-off the user must perform.**

## What Happened

Executed the full-migration regression sweep for the S05 enoki migration with no source changes. All four commands exited 0: (1) `cargo test` headless suite — 51 passed, 0 failed, including the standalone `dependency_gating` binary; (2) `cargo build --features windowed` — the full enoki render stack compiles windowed-gated; (3) `cargo test --features windowed --test windowed_only` — 49 passed, covering the three Agumon contact-burst parse tests and the generalized per-effect-id seam source-contract test; (4) `cargo test --test dependency_gating` — 2 passed, confirming bevy_enoki is absent from the headless dev graph and present only in the windowed graph (R005/R016 dep-isolation invariant holds). No new source files were created; inputs src/windowed/render.rs, tests/dependency_gating.rs, and tests/windowed_only.rs were left unchanged.

K001 manual sign-off (UAT gate): auto-mode must not launch the windowed binary, so this step is handed to the user. The user must run `cargo winx` and confirm that Sharp Claws, Baby Flame, and Baby Burner contact bursts now render through bevy_enoki one-shots and look better than the flat-quad placeholder. The quad path remains behind the seam as the reversible fallback.

## Verification

Ran the four-command sweep via gsd_exec; each exited 0. Headless `cargo test`: 51 passed/0 failed. `cargo build --features windowed`: finished, exit 0. `cargo test --features windowed --test windowed_only`: 49 passed/0 failed. `cargo test --test dependency_gating`: 2 passed (bevy_enoki_absent_from_headless_graph, bevy_enoki_present_in_windowed_graph). K001 windowed visual sign-off is not executable from auto-mode and is documented above as a required manual user step.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | pass | 2835ms |
| 2 | `cargo build --features windowed` | 0 | pass | 2146ms |
| 3 | `cargo test --features windowed --test windowed_only` | 0 | pass | 327ms |
| 4 | `cargo test --test dependency_gating` | 0 | pass | 451ms |

## Deviations

None.

## Known Issues

none

## Files Created/Modified

- `src/windowed/render.rs`
- `tests/dependency_gating.rs`
- `tests/windowed_only.rs`
