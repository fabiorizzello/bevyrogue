---
estimated_steps: 4
estimated_files: 1
skills_used: []
---

# T01: Write validation scope artifact

Expected executor skills frontmatter: write-docs, grill-me, verify-before-complete.

Why: Validation currently cannot tell whether M004 is validating global requirements or re-verifying local/inherited constraints because .gsd/REQUIREMENTS.md has zero active requirements while M004 context references R002, R004, and R005 labels. This task creates the scope artifact that makes the validation claim precise before any boundary proof is consumed.

Do: Create .gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md. Include a concise summary, an explicit table of scope items, and a pending-scope section. State that M004 owns no active global requirements unless new requirement records are added through GSD requirement tools. Treat R002, R004, and R005 in M004 context as inherited or local constraint labels rather than newly validated global requirement ids. Separate supporting/re-verified invariants from pending S05/S06 items: VFX presentation seam and no gameplay payload, headless deterministic VFX math, windowed dependency gating, K001 manual visual signoff, Sharp Claws VFX, and HDR bloom/additive rendering. Cite evidence owners using paths and test names from existing code, but do not invent new implementation claims.

Done when: The artifact is non-empty, reader-testable for a fresh validator, and clearly states that S01 to S04 do not validate visual signoff, Sharp Claws, or HDR/additive rendering.

## Inputs

- `.gsd/REQUIREMENTS.md`
- `.gsd/milestones/M004/M004-CONTEXT.md`
- `.gsd/milestones/M004/M004-ROADMAP.md`
- `.gsd/milestones/M004/slices/S04/S04-RESEARCH.md`
- `assets/digimon/agumon/vfx.ron`
- `src/windowed/render.rs`

## Expected Output

- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`

## Verification

test -s .gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md

## Observability Impact

Improves validation diagnostics by making scope ambiguity explicit and localizing pending visual and rendering obligations to S05 and S06.
