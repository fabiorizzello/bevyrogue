# S07: S07 — UAT

**Milestone:** M004
**Written:** 2026-05-25T21:26:44.704Z

# UAT — S07 Validation remediation closeout

- **UAT Type:** artifact-driven closeout verification
- **Preconditions:**
  1. Repository is at the S07 closeout state.
  2. No human windowed session is required; K001 remains in force.
  3. Python 3 and Rust tooling are available locally.

## Steps

1. Run `python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py`.
   - **Expected:** exits 0 and confirms S04 validation docs, proof references, and dependency metadata are consistent.
2. Run `python3 .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py`.
   - **Expected:** exits 0 and confirms roadmap boundary-map, remediation closeout, waiver disposition, and proof-token surfaces are consistent.
3. Open `.gsd/milestones/M004/M004-ROADMAP.md` and inspect the `## Boundary Map` section.
   - **Expected:** no `Not provided.` placeholder remains; the inline table includes rows for VfxAsset/schema/eval, placement/appearance registry, projectile/impact chain, Baby Burner detonate, Sharp Claws slash, HDR/Bloom overbright rendering proxy, variant selection seam, and K001 visual-UAT boundary.
4. Open `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md`.
   - **Expected:** it includes explicit sections for Requirement scope, Boundary map, Variant seam disposition, S06 evidence, D037 rendering rescope, UAT disposition, and Verification commands; it states auto-mode did not run `cargo winx`.
5. Open `docs/uat/M004-vfx-signoff.md`.
   - **Expected:** top-level status is `WAIVED`; per-skill Sharp Claws, Baby Flame, and Baby Burner entries are `WAIVED`; reviewer/date/evidence fields are present; no human visual PASS is claimed.
6. Run the regression suite:
   - `cargo test --test animation vfx_asset_load -- --nocapture`
   - `cargo test --test animation vfx_asset_eval -- --nocapture`
   - `cargo test --test animation render_no_vfx_kind_guard -- --nocapture`
   - `cargo check --features windowed`
   - `cargo test --features windowed --test windowed_only vfx_asset_impact_render -- --nocapture`
   - `cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture`
   - **Expected:** every command exits 0.

## Edge Cases

1. Reintroduce `Not provided.` under the roadmap boundary section.
   - **Expected:** the S07 verifier fails with a named roadmap boundary-map error.
2. Replace `projectile_on_expire_chains_the_impact_then_flash_fan` with the stale `projectile_on_expire_chains_the_impact_fan` token in validator-facing docs.
   - **Expected:** the S04 or S07 guard fails with a stale/missing proof-token message.
3. Add text claiming auto-mode ran `cargo winx` to any closeout artifact.
   - **Expected:** the S07 verifier fails with a forbidden auto-mode windowed-run claim.

## Not Proven By This UAT

- Human-eye visual quality in a real windowed session.
- Strict custom additive particle material delivery beyond the D037-accepted HDR/Bloom/overbright proxy.
- Any claim that auto-mode launched the windowed binary.

