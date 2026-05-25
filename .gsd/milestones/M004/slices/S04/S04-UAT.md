# S04: S04 — UAT

**Milestone:** M004
**Written:** 2026-05-25T17:46:56.806Z

# UAT Type
Repository-local documentation and contract verification.

# Preconditions
1. Worktree contains the completed S04 artifacts and S03 summary update.
2. Run from the repository root.
3. Python 3 is available.

# Steps
1. Execute `python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py`.
2. Confirm the script exits with code 0.
3. Confirm stdout reports: `OK: S04 validation docs, proof references, and S03 dependency metadata are consistent.`
4. If needed, inspect `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` and `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md` to verify they include pending-scope sections for S05 and S06 and the K001 manual visual boundary.
5. Inspect `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md` frontmatter to confirm `requires:` is populated with explicit S01 and S02 consumed contracts rather than `[]`.

# Expected Outcomes
1. The checker reports success with no missing paths or missing tokens.
2. The scope artifact clearly distinguishes delivered S01-S04 evidence from deferred Sharp Claws, HDR bloom/additive rendering, and manual visual signoff work.
3. The boundary artifact lists the required producer→consumer seams and validator-facing non-claims.
4. S03 summary metadata declares upstream contracts consumed from S01 and S02.

# Edge Cases
1. If any cited proof file or representative test name is renamed or removed, the checker must fail with a targeted message.
2. If pending S05/S06 scope language is deleted, the checker must fail rather than silently allowing over-claiming.
3. If S03 frontmatter regresses to `requires: []`, the checker must fail explicitly.

# Not Proven By This UAT
- No runtime rendering behavior is exercised.
- No Sharp Claws VFX delivery or rescope is proven.
- No HDR bloom/additive rendering implementation is proven.
- No human `cargo winx` visual signoff is captured; K001 remains deferred to later work.
