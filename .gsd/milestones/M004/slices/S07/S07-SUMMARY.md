---
id: S07
parent: M004
milestone: M004
provides:
  - A rerunnable M004 validation closeout surface that reconciles requirement scope, boundary map, variant seam, D037 rescope, and UAT waiver evidence.
  - A roadmap-visible boundary summary plus executable guards that fail loudly when validator-facing documentation drifts.
requires:
  []
affects:
  []
key_files:
  - .gsd/milestones/M004/M004-ROADMAP.md
  - .gsd/milestones/M004/slices/S04/verify_s04_docs.py
  - .gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md
  - .gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md
  - .gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md
  - .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py
  - docs/uat/M004-vfx-signoff.md
key_decisions:
  - Use `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md` as the canonical validator-facing closeout surface while keeping S04 as the historical scope/boundary source.
  - Treat the S03 variant selector as a deterministic future-consumer seam, not as missing M004 runtime integration.
  - Close K001 visual-UAT honestly with a formal WAIVED artifact and explicitly forbid any claim that auto-mode ran `cargo winx`.
  - Use `projectile_on_expire_chains_the_impact_then_flash_fan` as the current validator-facing Baby Flame proof token and reject the stale `...impact_fan` name.
patterns_established:
  - Canonical validation-remediation docs should aggregate scope mapping, boundary map disposition, rescope decisions, and waiver status into one validator-facing surface.
  - Executable documentation guards with negative self-tests are the preferred way to prevent milestone closeout drift across roadmap, remediation, and UAT artifacts.
  - Manual-only windowed validation surfaces under K001 should close with explicit WAIVED artifacts rather than ambiguous pending prose or fabricated PASS claims.
observability_surfaces:
  - .gsd/milestones/M004/slices/S04/verify_s04_docs.py
  - .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py
drill_down_paths:
  - .gsd/milestones/M004/slices/S07/tasks/T01-SUMMARY.md
  - .gsd/milestones/M004/slices/S07/tasks/T02-SUMMARY.md
  - .gsd/milestones/M004/slices/S07/tasks/T03-SUMMARY.md
  - .gsd/milestones/M004/slices/S07/tasks/T04-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-25T21:26:44.704Z
blocker_discovered: false
---

# S07: S07

**Closed M004 validation-remediation gaps with a canonical closeout artifact, inline roadmap boundary map, formal visual-UAT waiver, and executable doc guards that match current VFX proof surfaces.**

## What Happened

S07 finalized the validator-facing closeout surface for M004 without overclaiming runtime work. T01 repaired the historical S04 documentation guard so it tracks the current Baby Flame windowed proof token `projectile_on_expire_chains_the_impact_then_flash_fan` and tolerates later-slice superseding wording. T02 authored `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md` as the canonical remediation artifact, explicitly scoping M004 as local/inherited re-verification against already-validated requirements, classifying the S03 variant selector as a deterministic future-consumer seam, and citing D037 as the strict-additive rescope. T03 replaced the roadmap `Boundary Map` placeholder with a compact inline producer→consumer summary, refreshed the S04 boundary artifact to reflect S05/S06 truth, and converted `docs/uat/M004-vfx-signoff.md` from pending-runbook state into an explicit WAIVED closeout that records reviewer/date/evidence fields while stating that auto-mode did not launch `cargo winx`. T04 added `.gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py`, including negative self-tests for placeholder regressions, missing D037 citations, and forbidden auto-mode windowed-run claims, then used it to catch and repair real roadmap drift before rerunning the full documentation and Rust regression suite.

## Verification

Fresh verification ran in `gsd_exec:fec12eb1-5e3c-4ed5-a7c7-06713c4c04a8` and exited 0. The suite passed `python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py`, `python3 .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py`, `test -s .gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md`, `test -s docs/uat/M004-vfx-signoff.md`, `cargo test --test animation vfx_asset_load -- --nocapture`, `cargo test --test animation vfx_asset_eval -- --nocapture`, `cargo test --test animation render_no_vfx_kind_guard -- --nocapture`, `cargo check --features windowed`, `cargo test --features windowed --test windowed_only vfx_asset_impact_render -- --nocapture`, and `cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture`. This proves the roadmap boundary map is no longer missing, the remediation/waiver/docs are mutually consistent, stale Baby Flame proof-token drift is removed, D037 is cited everywhere required, and the data-driven VFX regression set still passes without launching the windowed binary.

## Requirements Advanced

- R004 — Re-verified deterministic VFX proof-token and evaluator evidence surfaces in current validation docs.
- R005 — Re-verified the windowed-only rendering boundary through roadmap/remediation/UAT closeout and headless verifier guards.
- R014 — Closed the human-visual boundary honestly via a tracked WAIVED artifact under K001 rather than a fabricated PASS.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

T04 discovered and repaired real documentation drift after task completion claims: the roadmap still contained the old `Boundary Map` placeholder, so the slice was only closed after fixing that drift and rerunning verification.

## Known Limitations

Visual signoff is intentionally closed via a tracked WAIVED artifact under K001 rather than a live human PASS; strict additive material remains deferred by D037.

## Follow-ups

A future human may replace the WAIVED entries in `docs/uat/M004-vfx-signoff.md` with PASS-with-notes or FAIL after a real `cargo winx` review. If strict additive material becomes a milestone requirement later, it should be addressed in a dedicated rendering refactor that supersedes D037.

## Files Created/Modified

- `.gsd/milestones/M004/M004-ROADMAP.md` — Replaced the boundary-map placeholder with a compact inline validator-facing summary table.
- `.gsd/milestones/M004/slices/S04/verify_s04_docs.py` — Updated the S04 guard to require the current Baby Flame proof token and tolerate superseding closeout wording.
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md` — Aligned S04 boundary dispositions with S05/S06 truth, including D037 deferral and WAIVED visual-UAT closure.
- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` — Pointed validators to the S07 remediation closeout as the canonical current answer surface and repaired stale proof-token references.
- `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md` — Authored the canonical validation-remediation closeout covering requirement scope, variant seam, S06 evidence, D037 rescope, and UAT disposition.
- `.gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py` — Added an executable closeout guard with negative self-tests for boundary, D037, waiver, stale-token, and forbidden auto-mode-claim regressions.
- `docs/uat/M004-vfx-signoff.md` — Converted the visual signoff runbook into a formal WAIVED closeout artifact with per-skill waiver entries and evidence fields.
