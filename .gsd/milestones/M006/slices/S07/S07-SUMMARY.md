---
id: S07
parent: M006
milestone: M006
provides:
  - Canonical SkillBookHandle used in all combat panel skill resolution paths; genuine MissingSkill misses log with skill id and book handle, deduped once per pair
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - Grouped SkillBookHandle and Local<HashSet> into the existing skill_book_params tuple to respect Bevy's 16-argument system limit — consistent with prior panel_state and intent_writers groupings.
  - Extracted missing_skill_log_once as a non-windowed-gated pure helper so it is testable headless, since the entire combat_panel module is cfg(feature=windowed) and never compiled by cargo test.
  - Dedup key is '<skill_id>@<handle_repr>' so distinct skills and distinct books each surface once per session.
  - Absent SkillBookHandle is reported as 'not yet loaded' to distinguish load-time misses from genuine canonical-book misses.
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-27T08:34:21.879Z
blocker_discovered: false
---

# S07: Combat panel reads canonical SkillBookHandle so Renamon skills are legal

**Replaced the arbitrary iter().next() partial-book lookup in combat_panel/render.rs with the canonical SkillBookHandle resource, eliminating false MissingSkill for Renamon; added dedup diagnostic logging for genuine misses.**

## What Happened


S07 fixed a correctness defect in the combat panel's skill resolution path: `render.rs` was calling `skill_books.iter().next()` to obtain the active skill book, which returns an arbitrary per-digimon partial book from `Assets<SkillBook>`. When Renamon was acting, this typically returned Agumon's partial book, which lacks `diamond_storm` and similar Renamon skills, causing `query_action_affordance` to return `ActionStatus::Disabled { reason: MissingSkill }` for all Renamon skills — a false negative.

**T01** established a red reproducer: a headless test (`tests/action_query/combat_panel_skill_book_seam.rs`) that feeds Agumon's partial book to the panel resolution path and asserts Renamon's skills show as MissingSkill — confirming the bug. Two companion tests proved the correct direction: feeding the canonical aggregate book resolves `diamond_storm` cleanly.

**T02** applied the one-line fix in `render.rs`: replaced `skill_books.iter().next()` with `skill_book_handle.as_ref().and_then(|handle| skill_books.get(&handle.0))`, mirroring the pattern already used in `preview_cache.rs`, `combat_cli/player.rs`, and `animation/plugin.rs`. Because the function was already at Bevy's 16-argument system limit, the new `Option<Res<SkillBookHandle>>` parameter was grouped with the existing `Res<Assets<SkillBook>>` into a `skill_book_params` tuple — consistent with the existing `panel_state` and `intent_writers` groupings. The `&fallback_skill_book` graceful fallback for the not-yet-loaded case was preserved. After the fix, all 47 action_query tests (including both T01 seam tests) passed.

**T03** made genuine MissingSkill misses diagnosable. A pure, non-windowed-gated helper `missing_skill_log_once(seen, skill_id, book_handle)` was extracted into `src/data/mod.rs`. It formats a log message naming both the skill id and the consulted book handle, keys a dedup set on `<skill>@<handle-repr>` to suppress per-frame spam, and labels an absent handle as "SkillBookHandle not yet loaded" to distinguish load-time misses from canonical-book misses. In `render.rs` (windowed path only), the helper is called on the `ActionStatus::Disabled { reason: MissingSkill }` branch and the result is emitted via `warn!`. A `Local<HashSet<String>>` dedup state was folded into the existing `skill_book_params` tuple as a 3-tuple rather than adding a 17th system parameter. Three headless tests in `tests/assets_data/missing_skill_log_dedup.rs` cover: log-once-then-dedup, distinct skill ids each surface once, and absent-handle naming — all green.

A pre-existing test (`render_panel_path_resolves_renamon_skill_without_missing_skill`) was found to be logically contradictory (fed a partial book but asserted no MissingSkill) and was fixed in T03 to use the same canonical aggregate book that `render.rs` now consults, matching its own docstring intent. The companion test proving the partial-book root cause was preserved.


## Verification


1. `cargo test --test action_query` — 47/47 passed (includes both T01 seam tests: `render_arbitrary_partial_book_reports_false_missing_skill_for_renamon` and `canonical_skill_book_resolves_renamon_diamond_storm_without_missing_skill`)
2. `cargo test --test assets_data` — 49/49 passed (includes 3 new dedup tests in `missing_skill_log_dedup.rs`)
3. `cargo test` (full headless suite) — all suites green, exit 0
4. `cargo check --features windowed` — exit 0 (T03 windowed path compiles cleanly)
5. `cargo clippy --features windowed` — exit 0 (no new warnings in touched files)


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

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

- `src/ui/combat_panel/render.rs` — Replaced skill_books.iter().next() with canonical SkillBookHandle lookup; grouped skill_book_params as 3-tuple including Local<HashSet> dedup; added warn! on genuine MissingSkill branch
- `src/data/mod.rs` — Added missing_skill_log_once() pure helper for dedup-once MissingSkill diagnostic logging
- `tests/action_query/combat_panel_skill_book_seam.rs` — Added seam tests: red reproducer for false MissingSkill via partial book, canonical book resolves Renamon skills; fixed pre-existing contradictory test
- `tests/assets_data/missing_skill_log_dedup.rs` — New test file: 3 headless tests for missing_skill_log_once dedup behaviour
- `tests/assets_data.rs` — Wired in new missing_skill_log_dedup test module
