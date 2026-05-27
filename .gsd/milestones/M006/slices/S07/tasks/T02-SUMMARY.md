---
id: T02
parent: S07
milestone: M006
key_files:
  - src/ui/combat_panel/render.rs
key_decisions:
  - Grouped skill_books and skill_book_handle into a tuple system parameter to respect Bevy's 16-argument system limit, consistent with the existing panel_state and intent_writers groupings.
  - Used Option<Res<SkillBookHandle>> (optional) to preserve the graceful fallback during asset loading, matching the pattern in combat_cli/player.rs.
duration: 
verification_result: passed
completed_at: 2026-05-27T08:12:52.626Z
blocker_discovered: false
---

# T02: Repointed combat panel skill-book lookup to canonical SkillBookHandle, eliminating false MissingSkill for Renamon.

**Repointed combat panel skill-book lookup to canonical SkillBookHandle, eliminating false MissingSkill for Renamon.**

## What Happened

The combat panel's `render.rs` was resolving the active skill book via `skill_books.iter().next()`, which returns an arbitrary partial per-digimon book from `Assets<SkillBook>`. When Renamon is acting, this typically returned Agumon's partial book, causing `query_action_affordance` to return `ActionStatus::Disabled { reason: MissingSkill }` for skills like `diamond_storm` that only appear in the canonical merged book.

The fix mirrors the pattern already used by `preview_cache.rs`, `combat_cli/player.rs`, and `animation/plugin.rs`:
1. Added `SkillBookHandle` to the import in `render.rs` (alongside the existing `SkillBook` import).
2. Grouped `(Res<Assets<SkillBook>>, Option<Res<SkillBookHandle>>)` into a single `skill_book_params` tuple parameter to stay within Bevy's 16-argument system limit — the function was already at 16 params (with panel_state and intent_writers already grouped as tuples), so adding a 17th was not legal.
3. In the function body, destructured the tuple and replaced the `.iter().next()` chain with `skill_book_handle.as_ref().and_then(|handle| skill_books.get(&handle.0))`, preserving the `&fallback_skill_book` fallback for the legitimately-missing-during-load case.

T01's test file (`tests/action_query/combat_panel_skill_book_seam.rs`) was already present with `follow_up: None` (the T01 agent had already corrected the type mismatch before this task ran). Both T01 tests passed green after the fix.

## Verification

Ran `cargo test --test action_query` (46 tests, 0 failures) confirming both T01 seam tests pass: `render_arbitrary_partial_book_reports_false_missing_skill_for_renamon` and `canonical_skill_book_resolves_renamon_diamond_storm_without_missing_skill`. Ran full `cargo test` (all test suites, 0 failures) confirming no regressions.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test action_query` | 0 | 46/46 passed including both T01 seam tests | 150ms |
| 2 | `cargo test` | 0 | All test suites green, 0 failures | 15000ms |

## Deviations

T01's test file already had follow_up: None when this task ran — no edit was needed. The Bevy 16-argument limit required grouping skill_book_params into a tuple, which was not explicitly called out in the plan but is consistent with prior groupings in the same function.

## Known Issues

None.

## Files Created/Modified

- `src/ui/combat_panel/render.rs`
