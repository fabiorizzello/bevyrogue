# S07: Combat panel reads canonical SkillBookHandle so Renamon skills are legal — UAT

**Milestone:** M006
**Written:** 2026-05-27T08:34:21.880Z


# S07 UAT: Combat Panel Skill Resolution via Canonical SkillBookHandle

**UAT Type:** Headless automated test (no windowed binary required)

## Preconditions

- Full project builds headless (`cargo test`) with no compilation errors.
- `tests/action_query/combat_panel_skill_book_seam.rs` is present.
- `tests/assets_data/missing_skill_log_dedup.rs` is present.
- Renamon roster entry (`src/data/units.ron` or equivalent) references `diamond_storm` or equivalent skill.

## Test Steps

1. Run `cargo test --test action_query` — verify all 47 tests pass including:
   - `render_arbitrary_partial_book_reports_false_missing_skill_for_renamon` — confirms partial-book path does produce MissingSkill (root cause preserved as regression guard)
   - `canonical_skill_book_resolves_renamon_diamond_storm_without_missing_skill` — confirms canonical book path resolves the skill correctly
2. Run `cargo test --test assets_data` — verify all 49 tests pass including:
   - `missing_skill_log_dedup::logs_once_then_dedup` — same skill+handle pair is suppressed after the first log
   - `missing_skill_log_dedup::distinct_skill_ids_each_log_once` — different skill ids each get their own log entry
   - `missing_skill_log_dedup::absent_handle_names_fallback_book` — absent SkillBookHandle is reported with human-readable label
3. Run full `cargo test` — verify all suites exit 0 with no failures.

## Expected Outcomes

- All three test groups pass with 0 failures.
- No MissingSkill entry appears for Renamon skills when the canonical SkillBookHandle is consulted.
- Dedup helper suppresses repeated logs for the same (skill, book) pair.
- The partial-book root-cause test still fails as expected (serves as a regression guard documenting the original bug).

## Edge Cases

- **SkillBookHandle not yet loaded:** `render.rs` falls back to an empty `SkillBook`; any skill lookup returns MissingSkill with "SkillBookHandle not yet loaded" in the log — distinct from a canonical-book miss.
- **Skill genuinely absent from canonical book:** `missing_skill_log_once` logs once with the actual handle repr, then dedup suppresses subsequent frames.

## Not Proven By This UAT

- Live windowed rendering: the `warn!` call in `render.rs` requires the windowed feature and a running Bevy app — confirmed only by `cargo check --features windowed` (compilation) and manual inspection. K001 sign-off applies.
- Renamon visual presentation: covered by S05/S06 UAT.

