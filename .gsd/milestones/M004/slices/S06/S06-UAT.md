# S06: S06 — UAT

**Milestone:** M004
**Written:** 2026-05-25T20:50:12.005Z

# UAT Type
Manual windowed visual signoff framework verification with automated regression backstop.

# Preconditions
- Repository is at the S06 closeout state.
- `docs/uat/M004-vfx-signoff.md` exists in the tracked tree.
- `scripts/capture-windowed-m004-vfx.sh` exists and is executable.
- A human operator is available for any future visual signoff run.
- Auto-mode must not launch the windowed binary (K001).

# Steps
1. Open `docs/uat/M004-vfx-signoff.md` and confirm it provides a `cargo winx` launch path, per-skill sections for Sharp Claws, Baby Flame, and Baby Burner, explicit acceptance bars, the D037 caveat, and top-level/per-skill signoff fields.
2. Inspect `scripts/capture-windowed-m004-vfx.sh` and confirm it is a human-only helper that writes timestamped logs under `.gsd/milestones/M004/slices/S06/uat-evidence/`, tees stdout/stderr, uses `cargo winx`, and prominently states that auto-mode must NOT invoke it.
3. Run the automated regression proof only: `cargo test --test animation vfx_asset_load`, `cargo test --test animation vfx_asset_eval`, `cargo test --test animation render_no_vfx_kind_guard`, `cargo check --features windowed`, `cargo test --features windowed --test windowed_only vfx_asset_impact_render`, and `cargo test --features windowed --test windowed_only vfx_rendering_acceptance`.
4. Leave the signoff artifact in its honest closeout state unless a human actually runs the windowed session: top-level status remains framework complete / human capture pending and each skill verdict remains PENDING or is later updated by a human to PASS-with-notes / FAIL / WAIVED.

# Expected Outcomes
- The runbook fully describes the manual review path and does not overclaim completed visual signoff.
- The capture helper is safe to inspect, syntax-valid, and ready for a human-run capture session.
- All automated rendering/data regression commands pass.
- The final state clearly distinguishes automated proof from manual visual judgment.

# Edge Cases
- If the helper script loses its K001 banner or stops using `cargo winx`, the framework is invalid.
- If any regression command fails, S06 cannot be considered complete because the manual-signoff framework no longer sits on a proven rendering baseline.
- If a human later waives or fails a skill, the tracked signoff artifact must record that explicitly instead of silently implying a pass.

# Not Proven By This UAT
- Actual on-screen visual quality of Sharp Claws, Baby Flame, or Baby Burner in the windowed build.
- Human perception of HDR bloom/readability during live playback.
- Any claim that `cargo winx` was run during auto-mode.
