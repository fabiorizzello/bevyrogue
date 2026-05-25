---
id: T01
parent: S06
milestone: M004
key_files:
  - docs/uat/M004-vfx-signoff.md
key_decisions:
  - Kept manual `cargo winx` signoff as a tracked `docs/uat/` artifact with explicit K001 and D037 honesty boundaries instead of mixing it into automated S05 evidence.
duration: 
verification_result: passed
completed_at: 2026-05-25T20:34:15.058Z
blocker_discovered: false
---

# T01: Added the tracked M004 windowed VFX runbook/signoff artifact with per-skill acceptance bars and pending human-only verdict fields.

**Added the tracked M004 windowed VFX runbook/signoff artifact with per-skill acceptance bars and pending human-only verdict fields.**

## What Happened

Created `docs/uat/M004-vfx-signoff.md` as the tracked M004 manual VFX runbook and signoff/waiver artifact. The document instructs a human reviewer to use the sanctioned `cargo winx` path from `.cargo/config.toml`, explicitly preserves the K001 boundary that auto-mode must not launch the windowed binary, adds separate reviewer guidance and acceptance bars for Sharp Claws, Baby Flame, and Baby Burner, and includes the D037 caveat that strict additive blending is deferred so reviewers judge HDR bloom/overbright glow rather than true additive materials. The signoff section was intentionally initialized with a top-level 'Framework complete — human capture pending' status and per-skill `PENDING` verdicts so the artifact stays honest in autonomous execution.

## Verification

Ran the task-plan verification command against `docs/uat/M004-vfx-signoff.md`; it passed, confirming the file is present and contains `cargo winx`, all three skill names, the D037 caveat, and pending/signoff/waiver language.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -s docs/uat/M004-vfx-signoff.md && grep -qi 'cargo winx' docs/uat/M004-vfx-signoff.md && grep -qi 'sharp claws' docs/uat/M004-vfx-signoff.md && grep -qi 'baby flame' docs/uat/M004-vfx-signoff.md && grep -qi 'baby burner' docs/uat/M004-vfx-signoff.md && grep -qi 'D037' docs/uat/M004-vfx-signoff.md && grep -qiE 'waiver|pending|signoff' docs/uat/M004-vfx-signoff.md` | 0 | ✅ pass | 10ms |

## Deviations

None.

## Known Issues

Human windowed capture/signoff is still pending by design under K001; this task only ships the framework artifact, not completed visual evidence.

## Files Created/Modified

- `docs/uat/M004-vfx-signoff.md`
