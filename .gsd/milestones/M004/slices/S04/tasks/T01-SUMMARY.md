---
id: T01
parent: S04
milestone: M004
key_files:
  - .gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md
key_decisions:
  - M004 currently owns no active global requirements, so `R002`/`R004`/`R005` references in `M004-CONTEXT.md` are documented as inherited/local constraint labels rather than newly validated global requirement ids.
  - The S04 deliverable should combine scope statement, producer→consumer boundary map, and S03 consumed-contract clarification into one artifact so validators can trace delivered versus pending evidence without chasing multiple files.
duration: 
verification_result: passed
completed_at: 2026-05-25T17:27:25.347Z
blocker_discovered: false
---

# T01: Wrote `M004-VALIDATION-SCOPE.md` to pin M004's validation boundary, pending visual scope, and S03 upstream contract dependencies.

**Wrote `M004-VALIDATION-SCOPE.md` to pin M004's validation boundary, pending visual scope, and S03 upstream contract dependencies.**

## What Happened

Created `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` as the milestone-local validation contract for M004. The document states that `.gsd/REQUIREMENTS.md` currently has zero active requirements, so M004 does not own any new global requirement validation unless requirement records are later added through GSD tooling. It reclassifies the `R002`/`R004`/`R005` labels in `M004-CONTEXT.md` as inherited/local constraints, separates S01-S04 supporting or re-verified invariants from pending S05/S06 visual work, and explicitly names the excluded items: K001 manual visual signoff, Sharp Claws VFX, and HDR bloom/additive rendering. To satisfy the slice remediation goal without overstepping the single-file task contract, the same artifact also includes a producer→consumer boundary map and an explicit S03 consumed-contract table showing S03's dependence on S01's typed VfxAsset/eval path and S02's PlacementExt/registry-resolved render path.

## Verification

Verified the artifact exists and is non-empty with the task's required `test -s` check, then ran a semantic reader-test script to confirm the document includes the zero-active-requirements statement, inherited/local constraint wording for `R002`/`R004`/`R005`, the embedded producer→consumer boundary map, the S03 consumed-contract section, the pending S05/S06 sections, and the explicit exclusion of visual signoff, Sharp Claws, and HDR/additive rendering from S01-S04 proof.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -s .gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md && python3 - <<'PY'
from pathlib import Path
text = Path('.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md').read_text()
needles = [
    'Active requirements: 0',
    'inherited or local constraint labels',
    '## Producer → consumer boundary map',
    '## S03 consumed contracts from earlier slices',
    '### Pending S05',
    '### Pending S06',
    'Sharp Claws VFX',
    'HDR bloom / additive rendering',
    'K001 manual visual signoff',
    'Do **not** count S01-S04 as proof of **visual quality signoff, Sharp Claws completion, or HDR bloom/additive rendering**.',
]
for needle in needles:
    assert needle in text, needle
print('T01_SCOPE_DOC_OK')
PY` | 0 | ✅ pass | 19ms |

## Deviations

Embedded the producer→consumer boundary map and S03 consumed-contract statement inside `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` instead of creating separate remediation files or editing `S03-SUMMARY.md`, because T01's explicit deliverable was a single validation-scope artifact and the combined document resolves the ambiguity in one reader-testable place.

## Known Issues

None.

## Files Created/Modified

- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`
