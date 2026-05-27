# S07: Combat panel reads canonical SkillBookHandle so Renamon skills are legal

**Goal:** Make the combat panel read the canonical aggregated SkillBookHandle instead of grabbing an arbitrary SkillBook asset via skill_books.iter().next() (src/ui/combat_panel/render.rs:70-74). Today that arbitrary pick makes Renamon's skills surface as false MissingSkill.
**Demo:** Headless test: Renamon action query returns its real skills, no false MissingSkill

## Must-Haves

- Headless test: Renamon's action query / panel skill resolution returns its real skills with zero false MissingSkill entries, using the same SkillBookHandle that preview_cache.rs and combat_cli already consume.

## Proof Level

- This slice proves: headless test

## Verification

- Where a skill genuinely cannot be resolved, the MissingSkill path should log which skill id and which book handle were consulted, so a true miss is distinguishable from the old arbitrary-book bug.

## Tasks

- [x] **T01: Reproduce false MissingSkill in a headless test** `est:S`
  Add a headless test that loads the full roster and asserts Renamon's skills resolve through the combat panel path without MissingSkill. Confirm it fails today because render.rs picks an arbitrary SkillBook.
  - Files: `tests/action_query/case.rs`, `src/ui/combat_panel/render.rs`
  - Verify: cargo test --test action_query (new case red, reproducing false MissingSkill)

- [x] **T02: Repoint combat panel to canonical SkillBookHandle** `est:M`
  Replace skill_books.iter().next() with a lookup against the canonical aggregated SkillBookHandle resource (the same one preview_cache.rs and combat_cli use). Preserve current rendering for the legitimately-missing case.
  - Files: `src/ui/combat_panel/render.rs`, `src/ui/combat_panel/preview_cache.rs`
  - Verify: cargo test --test action_query (T01 green); cargo test (headless suite green)

- [ ] **T03: Diagnose true MissingSkill with context** `est:S`
  On the legitimately-missing branch, log the skill id and the book handle consulted (deduplicated) so a real miss is debuggable and distinct from the old arbitrary-book defect.
  - Files: `src/ui/combat_panel/render.rs`
  - Verify: cargo test (headless green)

## Files Likely Touched

- tests/action_query/case.rs
- src/ui/combat_panel/render.rs
- src/ui/combat_panel/preview_cache.rs
