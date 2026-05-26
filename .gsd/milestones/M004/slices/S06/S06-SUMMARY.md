---
id: S06
parent: M004
milestone: M004
provides:
  - A reusable manual-signoff framework for future milestone windowed UAT: tracked runbook, human-only capture helper, and explicit pending/waived closeout semantics.
  - Fresh regression proof that S05’s data-driven VFX/rendering contracts still hold at M004 closeout.
requires:
  - slice: S05
    provides: The automated rendering/data contract suite that S06 re-ran green as its regression baseline.
affects:
  []
key_files:
  - docs/uat/M004-vfx-signoff.md
  - scripts/capture-windowed-m004-vfx.sh
  - tests/windowed_only/vfx_asset_impact_render.rs
key_decisions:
  - Preserved the K001 honesty boundary: auto-mode may close the framework and regression proof, but it must not claim manual windowed visual PASS.
  - Standardized the human capture path on the `.cargo` `cargo winx` alias plus slice-local `.gsd/.../uat-evidence` logs.
  - Aligned the Baby Flame windowed-only proof with the authored two-stage `impact -> impact_flash` chain instead of treating `baby_flame.impact` as the shard fan-out.
patterns_established:
  - For K001-blocked manual/windowed signoff slices, close them with a tracked runbook, a human-only capture helper, and explicit pending/waived status rather than false closure.
  - Keep automated regression evidence and manual visual verdicts as separate artifacts so slice completion can be honest and machine-checkable.
observability_surfaces:
  - `scripts/capture-windowed-m004-vfx.sh` writes timestamped stdout/stderr logs to `.gsd/milestones/M004/slices/S06/uat-evidence/` for later human-run evidence capture.
drill_down_paths:
  - .gsd/milestones/M004/slices/S06/tasks/T01-SUMMARY.md
  - .gsd/milestones/M004/slices/S06/tasks/T02-SUMMARY.md
  - .gsd/milestones/M004/slices/S06/tasks/T03-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-25T20:50:12.005Z
blocker_discovered: false
---

# S06: S06

**Closed M004’s remaining windowed-VFX signoff gap with a tracked human-only runbook, a K001-bannered capture helper, and fresh headless/windowed regression proof that keeps manual visual verdicts explicitly pending until a human records them.**

## What Happened

S06 finished the closeout framework for M004’s manual windowed VFX review without violating K001. T01 confirmed the tracked artifact at `docs/uat/M004-vfx-signoff.md` already satisfied the slice contract: it tells a human reviewer to use `cargo winx`, documents per-skill trigger/acceptance guidance for Sharp Claws, Baby Flame, and Baby Burner, carries the D037 caveat that true additive blending is not the pass bar, and leaves each verdict honestly PENDING under a top-level ‘framework complete / human capture pending’ status. T02 added the executable helper `scripts/capture-windowed-m004-vfx.sh`, following the established human-only capture pattern by using the `.cargo` alias, teeing stdout/stderr into `.gsd/milestones/M004/slices/S06/uat-evidence/`, and carrying an explicit K001 banner that auto-mode must not invoke it. T03 then re-verified the framework and re-ran the full S05 automated rendering regression set. During that verification pass, the windowed-only Baby Flame proof surface was aligned with the authored two-stage impact chain (`baby_flame.impact` central burst chaining `baby_flame.impact_flash` fan-out), after which all required cargo checks/tests passed. The slice closes the milestone’s remaining documentation/process gap around visual signoff while keeping the human-eye verdict separate from automated proof: no windowed binary was launched by auto-mode, and no claim of visual PASS was made.

## Verification

Fresh closeout verification passed via `gsd_exec` run `07476357-cd04-4a6b-b89e-c94a6cd2fc5a`: (1) `docs/uat/M004-vfx-signoff.md` is present, non-empty, references `cargo winx`, names Sharp Claws/Baby Flame/Baby Burner, includes the D037 caveat, and includes waiver/pending/signoff language; (2) `scripts/capture-windowed-m004-vfx.sh` is executable, passes `bash -n`, includes the K001 / `auto-mode must NOT invoke` banner, targets `M004/slices/S06/uat-evidence`, and invokes `cargo winx`; (3) the full S05 regression set passed: `cargo test --test animation vfx_asset_load`, `cargo test --test animation vfx_asset_eval`, `cargo test --test animation render_no_vfx_kind_guard`, `cargo check --features windowed`, `cargo test --features windowed --test windowed_only vfx_asset_impact_render`, and `cargo test --features windowed --test windowed_only vfx_rendering_acceptance`. All commands exited 0. No `cargo winx` execution occurred, by design.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

T03 corrected a stale windowed-only verification surface so the Baby Flame proof matches the current authored `impact -> impact_flash` chain; runtime behavior was not intentionally changed as part of slice closeout.

## Known Limitations

Human visual signoff remains pending until a person runs `cargo winx` and records PASS-with-notes / FAIL / WAIVED in `docs/uat/M004-vfx-signoff.md`. K001 intentionally prevents auto-mode from proving that visual bar.

## Follow-ups

Have a human operator run `scripts/capture-windowed-m004-vfx.sh` or `cargo winx`, review Sharp Claws/Baby Flame/Baby Burner against the runbook, save the evidence log, and update `docs/uat/M004-vfx-signoff.md` with the real per-skill verdicts.

## Files Created/Modified

- `docs/uat/M004-vfx-signoff.md` — Tracked M004 runbook/signoff artifact with per-skill acceptance bars, D037 caveat, and honest pending human-only verdict fields.
- `scripts/capture-windowed-m004-vfx.sh` — Human-only `cargo winx` capture helper with explicit K001 banner and slice-local evidence logging.
- `tests/windowed_only/vfx_asset_impact_render.rs` — Updated proof assertions to match the authored Baby Flame `impact -> impact_flash` chain used by current windowed verification.
