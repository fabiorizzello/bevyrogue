---
estimated_steps: 17
estimated_files: 1
skills_used: []
---

# T03: R016 invariant gate + final M002 regression matrix

Why: R016 requires that R002 (headless-first), R004 (determinism), R005 (windowed dep-gating), R006 (no .md in repo root), and I3 two-clock parity all hold at M002 closeout. This task is the mechanical proof.

Do:
1. Run the full M002 verification matrix and capture results in `.gsd/milestones/M002/slices/S06/regression-matrix.md`:
   - `cargo test` (M001 + M002 headless harnesses)
   - `cargo test --features windowed --test windowed_only`
   - `cargo test --test timeline` (includes timeline_two_clock_parity, timeline_loop_hop_cue_parity, timeline_cue_barrier_pipeline — the I3 extended parity proof)
   - `cargo test --features windowed --test bootstrap_encounter`
   - `cargo test --test digimon_kits`
   - `cargo build --no-default-features`
   - `cargo build --features windowed`
2. R016 structural checks (record in the matrix doc):
   - R005 dep-gating: `grep -rn 'use winit\|use wgpu\|use bevy_egui\|use egui' src/ | grep -v '#\[cfg(feature = "windowed")\]' | grep -v '/windowed/' | grep -v '/ui/'` must report no leakage (record the command + output).
   - R006 repo hygiene: `find . -maxdepth 1 -name '*.md' -not -path './node_modules/*'` must list only files that pre-existed before M002 (record the listing).
   - I3 extension: confirm `tests/timeline/timeline_two_clock_parity.rs`, `tests/timeline/timeline_loop_hop_cue_parity.rs`, `tests/timeline/timeline_cue_barrier_pipeline.rs` exist and pass — these together cover the cue handshake extension R016 mandates.
3. Note in the matrix: live windowed UAT is performed by the user per K001 — the captured log under `uat-evidence/` is the R014 evidence and is independent of this regression matrix.
4. Done when: regression-matrix.md exists with PASS/FAIL per row for every command above, and explicit PASS rows for R005/R006/I3 structural checks.

If any check fails, do NOT complete the slice — escalate the failure as a blocker via `gsd_replan_slice` so the offending defect is fixed before R015 review attaches.

## Inputs

- `tests/timeline/timeline_two_clock_parity.rs`
- `tests/timeline/timeline_loop_hop_cue_parity.rs`
- `tests/timeline/timeline_cue_barrier_pipeline.rs`
- `src/windowed/mod.rs`
- `src/ui/mod.rs`
- `Cargo.toml`

## Expected Output

- `.gsd/milestones/M002/slices/S06/regression-matrix.md`

## Verification

test -s .gsd/milestones/M002/slices/S06/regression-matrix.md && grep -q 'cargo test' .gsd/milestones/M002/slices/S06/regression-matrix.md && grep -q 'R005' .gsd/milestones/M002/slices/S06/regression-matrix.md && grep -q 'R006' .gsd/milestones/M002/slices/S06/regression-matrix.md && grep -q 'I3' .gsd/milestones/M002/slices/S06/regression-matrix.md && grep -q 'PASS' .gsd/milestones/M002/slices/S06/regression-matrix.md
