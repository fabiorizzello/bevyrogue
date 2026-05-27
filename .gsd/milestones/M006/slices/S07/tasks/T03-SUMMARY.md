---
id: T03
parent: S07
milestone: M006
key_files:
  - src/data/mod.rs
  - src/ui/combat_panel/render.rs
  - tests/assets_data/missing_skill_log_dedup.rs
  - tests/assets_data.rs
  - tests/action_query/combat_panel_skill_book_seam.rs
key_decisions:
  - Extracted the diagnostic format + dedup decision into a pure, non-gated helper (src/data/missing_skill_log_once) so it is testable under headless cargo test — the entire combat_panel UI module is #[cfg(feature="windowed")]-gated and never compiled by cargo test.
  - Folded the dedup Local<HashSet> into the existing skill_book_params tuple (now a 3-tuple) instead of adding a 17th system param, respecting Bevy's 16-argument system limit (consistent with T02's tuple groupings).
  - Keyed dedup on '<skill_id>@<handle_repr>' so distinct skills and distinct books each surface once; absent handle is labelled as the fallback empty book to distinguish load-time misses from canonical-book misses.
duration: 
verification_result: passed
completed_at: 2026-05-27T08:20:50.480Z
blocker_discovered: false
---

# T03: Combat panel now logs genuine MissingSkill with skill id + consulted book handle, deduplicated once per pair

**Combat panel now logs genuine MissingSkill with skill id + consulted book handle, deduplicated once per pair**

## What Happened

On the legitimately-missing branch, the combat panel now emits a diagnostic naming both the skill id and the book handle consulted, so a true miss is distinguishable from the old arbitrary-partial-book defect that S07 fixed.

Implementation: extracted a pure, headless-testable helper `missing_skill_log_once(seen, skill_id, book_handle)` into `src/data/mod.rs` (non-gated, next to `SkillBookHandle`). It formats `combat panel MissingSkill: skill '<id>' not found in consulted skill book (handle <repr>)`, keys a dedup set on `<skill>@<handle-repr>`, and returns `Some(message)` only the first time a pair is seen (`None` thereafter). An absent handle is reported as the fallback empty book ("SkillBookHandle not yet loaded"), distinguishing a miss during asset loading from a miss against the loaded canonical book.

In `render.rs` (windowed-only), after `selected_action_affordance` is computed I detect `ActionStatus::Disabled { reason: MissingSkill }`, derive the skill id from `affordance.kind` (Skill→id, Basic/Ultimate→label), and `warn!` the returned message. Dedup state lives in a `Local<HashSet<String>>` — the panel runs every frame, so without dedup it would spam. Bevy's 16-argument system limit was already saturated (T02 grouped params), so the `Local` was folded into the existing `skill_book_params` tuple as a 3-tuple `(Res<Assets<SkillBook>>, Option<Res<SkillBookHandle>>, Local<HashSet<String>>)` rather than added as a 17th param.

## Verification

cargo check --features windowed compiles render.rs cleanly (exit 0). Full headless cargo test is green (all scope harnesses pass). cargo clippy --features windowed exits 0 with no new warnings in touched files (pre-existing warnings only). Added 3 headless tests in tests/assets_data/missing_skill_log_dedup.rs covering: log-once-then-dedup, distinct skill ids each log once, and absent-handle names the fallback book — all pass.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features windowed` | 0 | pass | 3280ms |
| 2 | `cargo test (full headless suite)` | 0 | pass | 5000ms |
| 3 | `cargo test --test assets_data (3 new dedup tests)` | 0 | pass | 10ms |
| 4 | `cargo clippy --features windowed` | 0 | pass | 190ms |

## Deviations

Fixed a pre-existing red test outside T03's nominal scope: tests/action_query/combat_panel_skill_book_seam.rs::render_panel_path_resolves_renamon_skill_without_missing_skill was logically contradictory — it fed Agumon's PARTIAL book to query_action_affordance yet asserted no MissingSkill, which is impossible since the partial book genuinely lacks diamond_storm. T02 fixed render.rs (windowed, untestable headless) but this test never exercised that path. Confirmed it was already failing on HEAD before my changes (git stash). Repointed the test to the canonical aggregate_skill_book() — exactly what render.rs now consults via SkillBookHandle post-T02 — matching the test's own docstring intent. The companion test partial_book_confirms_missing_skill_is_the_root_cause still covers the old arbitrary-partial-book bug.

## Known Issues

The render.rs warn! call itself is exercised only under the windowed feature and cannot be asserted by headless tests; headless coverage is via the extracted pure helper. Pre-existing unrelated warning "unused import: BeatEdge" remains in the timeline module (not touched by this task).

## Files Created/Modified

- `src/data/mod.rs`
- `src/ui/combat_panel/render.rs`
- `tests/assets_data/missing_skill_log_dedup.rs`
- `tests/assets_data.rs`
- `tests/action_query/combat_panel_skill_book_seam.rs`
