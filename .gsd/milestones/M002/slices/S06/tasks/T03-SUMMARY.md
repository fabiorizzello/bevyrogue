---
id: T03
parent: S06
milestone: M002
key_files:
  - .gsd/milestones/M002/slices/S06/regression-matrix.md
key_decisions:
  - Treated the lone R005 grep hit (src/combat/runtime/mod.rs:20) as a PASS because it is a `//!` doc-comment, not an actual `use` import — recorded both command and output verbatim in the matrix so the judgment is auditable.
  - Did not launch the windowed binary from auto-mode (K001); the regression matrix explicitly delegates live windowed smoke to the user-driven UAT log under uat-evidence/ (T01).
duration: 
verification_result: passed
completed_at: 2026-05-21T12:04:49.472Z
blocker_discovered: false
---

# T03: Ran the M002 R016 invariant gate: 7 cargo commands plus R005/R006/I3 structural checks all PASS, recorded in regression-matrix.md.

**Ran the M002 R016 invariant gate: 7 cargo commands plus R005/R006/I3 structural checks all PASS, recorded in regression-matrix.md.**

## What Happened

Executed the full M002 closeout verification matrix and captured results in `.gsd/milestones/M002/slices/S06/regression-matrix.md`.

Cargo commands (all exit 0):
1. `cargo test` (default/headless) — full suite green; final binary 56 passed; doc-tests empty (24.5s)
2. `cargo test --features windowed --test windowed_only` — 23 passed (1.4s)
3. `cargo test --test timeline` — 47 passed including `timeline_two_clock_parity`, `timeline_loop_hop_cue_parity`, `timeline_cue_barrier_pipeline` (the I3 extended parity proof) (0.2s)
4. `cargo test --features windowed --test bootstrap_encounter` — 16 passed, 1 ignored (0.3s)
5. `cargo test --test digimon_kits` — 70 passed (0.2s)
6. `cargo build --no-default-features` — clean (0.2s)
7. `cargo build --features windowed` — clean (0.3s)

R016 structural checks:
- R005 dep-gating: the prescribed grep yields a single hit, `src/combat/runtime/mod.rs:20`, which is a `//!` doc-comment forbidding such imports — not an actual `use`. No real winit/wgpu/egui/bevy_egui leakage outside `src/windowed/` or `src/ui/`. PASS.
- R006 repo hygiene: `find . -maxdepth 1 -name '*.md'` returns empty. PASS.
- I3 extension: all three timeline parity files exist and pass in command #3. PASS.

Per K001, no windowed binary was launched from auto-mode; the live windowed UAT log under `uat-evidence/` (T01) is the user-driven R014 evidence and is intentionally separate from this mechanical matrix.

## Verification

Ran every command listed in T03-PLAN.md step 1, captured stdout tails and exit codes, executed the R005 grep and R006 find verbatim, and confirmed the three timeline parity test files exist on disk and pass inside the timeline harness. Final task verification command (`test -s ... && grep -q 'cargo test' && grep -q 'R005' && grep -q 'R006' && grep -q 'I3' && grep -q 'PASS'`) returned VERIFY OK.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --no-default-features` | 0 | pass | 24464ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass | 1406ms |
| 3 | `cargo test --test timeline` | 0 | pass | 225ms |
| 4 | `cargo test --features windowed --test bootstrap_encounter` | 0 | pass | 302ms |
| 5 | `cargo test --test digimon_kits` | 0 | pass | 213ms |
| 6 | `cargo build --no-default-features` | 0 | pass | 246ms |
| 7 | `cargo build --features windowed` | 0 | pass | 328ms |
| 8 | `grep -rn 'use winit|use wgpu|use bevy_egui|use egui' src/ | grep -v cfg-windowed | grep -v /windowed/ | grep -v /ui/` | 0 | pass (only doc-comment match) | 50ms |
| 9 | `find . -maxdepth 1 -name '*.md'` | 0 | pass (empty) | 20ms |
| 10 | `test -s regression-matrix.md && grep -q 'cargo test' && grep -q R005 && grep -q R006 && grep -q I3 && grep -q PASS` | 0 | pass | 10ms |

## Deviations

None.

## Known Issues

None from this task. F2 (unsafe raw-pointer ExtRegistries dance in timeline_exec.rs) from T02's architectural review remains the standing follow-up for M003/S01 triage; it is below the regression gate.

## Files Created/Modified

- `.gsd/milestones/M002/slices/S06/regression-matrix.md`
