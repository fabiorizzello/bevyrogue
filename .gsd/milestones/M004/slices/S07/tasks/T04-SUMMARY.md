---
id: T04
parent: S07
milestone: M004
key_files:
  - .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py
  - .gsd/milestones/M004/M004-ROADMAP.md
  - .gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md
  - .gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md
  - .gsd/milestones/M004/slices/S04/verify_s04_docs.py
key_decisions:
  - The S07 closeout guard treats WAIVED visual UAT as the honest success surface and explicitly forbids any artifact claim that auto-mode ran `cargo winx`.
  - The current Baby Flame closeout proof token for validator-facing docs is the windowed `projectile_on_expire_chains_the_impact_then_flash_fan` test name; the older `...impact_fan` citation is stale drift.
duration: 
verification_result: passed
completed_at: 2026-05-25T21:24:23.049Z
blocker_discovered: false
---

# T04: Added the S07 validation-remediation guard, repaired roadmap/S04 closeout drift, and reran the full M004 VFX regression proof.

**Added the S07 validation-remediation guard, repaired roadmap/S04 closeout drift, and reran the full M004 VFX regression proof.**

## What Happened

Added `.gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py` as the new closeout verifier for M004/S07. The guard checks the roadmap boundary-map section, remediation closeout sections and tokens, D037 citation presence, required evidence-file existence, current vs stale Baby Flame proof-token references, the WAIVED signoff disposition, and forbidden auto-mode `cargo winx` claims. It also includes inline `--self-test` negative cases for the required failure modes. While exercising the new guard, I found real repo drift and repaired it: `.gsd/milestones/M004/M004-ROADMAP.md` still said `Not provided.` for Boundary Map despite the T03 summary claiming otherwise; `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md` and `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` still cited the obsolete `projectile_on_expire_chains_the_impact_fan` windowed proof token; and `.gsd/milestones/M004/slices/S04/verify_s04_docs.py` still expected the pre-D037 HDR wording. After repairing those active validator-facing artifacts, I reran the repaired S04 guard, the new S07 guard, and the full Rust regression set covering authored VFX data, deterministic evaluation, no-hardcoded-VFX dispatch, windowed placement/render proof, and HDR/Bloom acceptance.

## Verification

Ran the new S07 guard in `--self-test` mode to prove the negative checks fire for a lingering roadmap placeholder, missing D037 remediation citation, and a forbidden auto-mode windowed-run claim. Then ran `python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py` and `python3 .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py`, both of which passed. Finally reran the required cargo regression set: `cargo test --test animation vfx_asset_load -- --nocapture`, `cargo test --test animation vfx_asset_eval -- --nocapture`, `cargo test --test animation render_no_vfx_kind_guard -- --nocapture`, `cargo check --features windowed`, `cargo test --features windowed --test windowed_only vfx_asset_impact_render -- --nocapture`, and `cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture`; all exited 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `python3 .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py --self-test` | 0 | ✅ pass | 33ms |
| 2 | `python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py` | 0 | ✅ pass | 26ms |
| 3 | `python3 .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py` | 0 | ✅ pass | 32ms |
| 4 | `cargo test --test animation vfx_asset_load -- --nocapture` | 0 | ✅ pass | 884ms |
| 5 | `cargo test --test animation vfx_asset_eval -- --nocapture` | 0 | ✅ pass | 173ms |
| 6 | `cargo test --test animation render_no_vfx_kind_guard -- --nocapture` | 0 | ✅ pass | 169ms |
| 7 | `cargo check --features windowed` | 0 | ✅ pass | 448ms |
| 8 | `cargo test --features windowed --test windowed_only vfx_asset_impact_render -- --nocapture` | 0 | ✅ pass | 592ms |
| 9 | `cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture` | 0 | ✅ pass | 262ms |

## Deviations

In addition to creating the new S07 guard, I repaired three pieces of active documentation drift discovered during verification: the roadmap boundary-map placeholder, stale S04 proof-token citations, and one outdated S04 guard token for the D037 HDR/Bloom proxy wording.

## Known Issues

None.

## Files Created/Modified

- `.gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py`
- `.gsd/milestones/M004/M004-ROADMAP.md`
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`
- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`
- `.gsd/milestones/M004/slices/S04/verify_s04_docs.py`
