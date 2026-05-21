---
id: T01
parent: S06
milestone: M002
key_files:
  - docs/uat/M002-S06-windowed-smoke.md
  - scripts/capture-windowed-smoke.sh
key_decisions:
  - Capture script preserves cargo exit code via PIPESTATUS so a panic surfaces as a non-zero exit even though tee succeeds.
  - Runbook prescribes one continuous combat exercising all three skills (with dummy pre-heated for Baby Burner reactive detonate) rather than separate encounters, matching the slice's smoke definition.
duration: 
verification_result: passed
completed_at: 2026-05-21T11:57:24.865Z
blocker_discovered: false
---

# T01: Authored windowed smoke UAT runbook and timestamped log capture helper for S06.

**Authored windowed smoke UAT runbook and timestamped log capture helper for S06.**

## What Happened

Created `docs/uat/M002-S06-windowed-smoke.md` documenting the windowed smoke runbook: launch command (preferred capture script + cargo fallback), pre-flight checklist (windowed build green, clean working tree on `assets/data/digimon/agumon/skills.ron`), a single full-encounter smoke scenario exercising Sharp Claws, Bouncing Fire (multi-hop visible), and Baby Burner Ultimate (charge → launch → recovery with reactive detonate against a pre-heated dummy), the mid-Ultimate hot-reload step (numeric edit, save, no panic, state intact), explicit pass/fail signals, and the canonical inspection surfaces named in the slice plan (PhaseStripDisplay, HpBarView, FloatingDamageView, TargetHurtState, TwinCoreBadgeState, CueBarrierStatus.hop_index). The runbook explicitly notes K001 (user, not auto-mode, executes the smoke) and points at the uat-evidence directory as canonical evidence. Also created `scripts/capture-windowed-smoke.sh` which ensures the evidence directory exists, runs `cargo run --features windowed --bin bevyrogue` and tees combined stdout/stderr into `windowed-smoke-<UTC>.log`, preserves the cargo exit code via PIPESTATUS, and is marked executable. Neither file existed previously, so both are fresh creates — no merge/extend decisions needed.

## Verification

Ran the provided one-liner: `test -f docs/uat/M002-S06-windowed-smoke.md && test -x scripts/capture-windowed-smoke.sh && grep -q 'Baby Burner' ... && grep -q 'hot-reload' ... && grep -q 'tee' scripts/capture-windowed-smoke.sh && echo OK`. Output: `OK`. Confirms file presence, executable bit, and required content markers. The windowed binary itself was not launched, per K001.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -f docs/uat/M002-S06-windowed-smoke.md && test -x scripts/capture-windowed-smoke.sh && grep -q 'Baby Burner' docs/uat/M002-S06-windowed-smoke.md && grep -q 'hot-reload' docs/uat/M002-S06-windowed-smoke.md && grep -q 'tee' scripts/capture-windowed-smoke.sh && echo OK` | 0 | pass | 50ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `docs/uat/M002-S06-windowed-smoke.md`
- `scripts/capture-windowed-smoke.sh`
