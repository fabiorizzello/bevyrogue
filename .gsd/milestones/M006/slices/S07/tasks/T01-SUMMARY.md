---
id: T01
parent: S07
milestone: M006
key_files:
  - /home/fabio/dev/bevyrogue/tests/action_query/combat_panel_skill_book_seam.rs
  - /home/fabio/dev/bevyrogue/tests/action_query.rs
key_decisions:
  - Only added test file; did not touch render.rs (T02 owns that edit)
  - Used Agumon's partial book as the 'arbitrary first book' stand-in — it is the most likely first loaded and definitively lacks Renamon skills
  - Structured as 3 tests: red reproducer, green root-cause proof, green canonical sanity — so the suite communicates both the bug and the fix direction clearly
duration: 
verification_result: passed
completed_at: 2026-05-27T08:13:16.468Z
blocker_discovered: false
---

# T01: Added red headless test reproducing false MissingSkill when render.rs uses iter().next() partial book for Renamon

**Added red headless test reproducing false MissingSkill when render.rs uses iter().next() partial book for Renamon**

## What Happened

Explored the combat panel source files to understand the bug. `render.rs` (lines 70-74) resolves the active skill book via `skill_books.iter().next()`, which picks an arbitrary partial `SkillBook` from `Assets<SkillBook>` — typically Agumon's partial book, since it loads first. When Renamon is active, her skill IDs (`diamond_storm`, `renamon_ult`, etc.) are absent from Agumon's partial book, so `query_action_affordance` returns `ActionStatus::Disabled { reason: MissingSkill }` — a false positive. By contrast, `preview_cache.rs` already uses `SkillBookHandle` to obtain the canonical merged book and therefore works correctly.

Added `tests/action_query/combat_panel_skill_book_seam.rs` with three tests:
1. `render_panel_path_resolves_renamon_skill_without_missing_skill` — RED (asserts the fix: no MissingSkill via partial book path; fails today because the partial book doesn't contain Renamon's skills)
2. `partial_book_confirms_missing_skill_is_the_root_cause` — green (documents the bug mechanism: partial Agumon book returns MissingSkill for diamond_storm)
3. `canonical_skill_book_resolves_renamon_diamond_storm_as_enabled` — green (proves the canonical merged book resolves correctly and indicates the fix direction for T02)

Registered the new module in `tests/action_query.rs`. T02 must change `render.rs` to use `SkillBookHandle` instead of `iter().next()`, at which point test #1 turns green.

## Verification

Ran `cargo test --test action_query combat_panel_skill_book_seam`. 1 red (reproducing bug), 2 green (root cause proof + fix direction). Exit code 101 confirms red test is failing as expected.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test action_query combat_panel_skill_book_seam` | 101 | 1 FAILED (red reproducer), 2 passed (mechanism + canonical sanity) | 3500ms |

## Deviations

T02's fix had not yet landed when the tests ran; the red test is genuinely red as expected.

## Known Issues

None.

## Files Created/Modified

- `/home/fabio/dev/bevyrogue/tests/action_query/combat_panel_skill_book_seam.rs`
- `/home/fabio/dev/bevyrogue/tests/action_query.rs`
