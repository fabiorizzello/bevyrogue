---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T03: Verify S06 framework and re-prove S05 regression set green (no cargo winx)

Why: S06 must not silently break the S05 automated rendering contracts, and the slice must terminate in an honest, machine-checkable state (MEM078/MEM079) — framework present, human verdict pending or waived, no false closure. Do: (1) Confirm the two S06 deliverables exist and satisfy their content checks (runbook + capture script from T01/T02). (2) `bash -n scripts/capture-windowed-m004-vfx.sh` (syntax only — NEVER run it, K001) and grep the K001 banner. (3) Re-run S05's headless contract set and the windowed compile/headless harness set, all expected green: `cargo test --test animation vfx_asset_load`, `cargo test --test animation vfx_asset_eval`, `cargo test --test animation render_no_vfx_kind_guard`, `cargo check --features windowed`, `cargo test --features windowed --test windowed_only vfx_asset_impact_render`, `cargo test --features windowed --test windowed_only vfx_rendering_acceptance`. Use gsd_exec for the noisy cargo runs and gsd_exec_search first to avoid re-running. (4) Record the closure status: human visual signoff is PENDING/WAIVED per K001 — auto-mode cannot launch the window, so this task makes NO claim that the visual bar was met. Done-when: both deliverables verified present and well-formed, the script passes bash -n with the K001 banner, all six S05 regression commands exit 0, and the task summary states the human verdict is pending/waived with no overclaim. This is a pure verification task — it creates no new source files.

## Inputs

- `docs/uat/M004-vfx-signoff.md`
- `scripts/capture-windowed-m004-vfx.sh`
- `tests/animation/vfx_asset_load.rs`
- `tests/animation/vfx_asset_eval.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/windowed_only/vfx_rendering_acceptance.rs`

## Expected Output

- Update the implementation and proof artifacts needed for this task.

## Verification

test -s docs/uat/M004-vfx-signoff.md && test -x scripts/capture-windowed-m004-vfx.sh && bash -n scripts/capture-windowed-m004-vfx.sh && grep -q 'auto-mode must NOT invoke' scripts/capture-windowed-m004-vfx.sh && cargo test --test animation vfx_asset_load && cargo test --test animation vfx_asset_eval && cargo test --test animation render_no_vfx_kind_guard && cargo check --features windowed && cargo test --features windowed --test windowed_only vfx_asset_impact_render && cargo test --features windowed --test windowed_only vfx_rendering_acceptance
