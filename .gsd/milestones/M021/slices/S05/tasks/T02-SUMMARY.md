---
id: T02
parent: S05
milestone: M021
key_files:
  - src/data/skills_ron.rs
  - src/data/skill_timeline.rs
  - src/data/mod.rs
  - src/combat/api/timeline.rs
  - assets/data/skills.ron
  - tests/compiled_timeline_boot_validation.rs
key_decisions:
  - D012
duration: 
verification_result: passed
completed_at: 2026-05-15T15:22:49.944Z
blocker_discovered: false
---

# T02: Added optional SkillBook timeline schemas plus load-time compilation into TimelineLibrary, with fast failure on bad hook/selector/predicate refs.

**Added optional SkillBook timeline schemas plus load-time compilation into TimelineLibrary, with fast failure on bad hook/selector/predicate refs.**

## What Happened

Extended `SkillDef` with an optional `timeline` field and introduced `src/data/skill_timeline.rs` to lower asset-side beat/edge data into owned `CompiledTimeline<String>` values. Updated the data load path so `SkillBook` asset events now validate the catalog, compile every timeline-backed skill, and replace the shared `TimelineLibrary` immediately on load/modify events; this preserves legacy skills with no timeline field while making RON typos fail fast with skill id plus beat/edge context. Added asset-backed coverage using the canonical `assets/data/skills.ron` for the happy path and a mutated in-memory RON fixture for the typo path, and aligned the Renamon ult label to `Tohakken` to match the demo wording.

## Verification

Ran `cargo test --test compiled_timeline_boot_validation` and confirmed both the canonical asset compilation path and the typo path pass/fail as expected. The happy-path test validates canonical `skills.ron`, compiles the timeline-backed skills into the library shape, and asserts the expected timeline ids and structure; the negative test mutates the canonical asset to a bad hook id and confirms the compiler reports the owning skill and beat site before runtime.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test compiled_timeline_boot_validation` | 0 | ✅ pass | 439ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/data/skill_timeline.rs`
- `src/data/mod.rs`
- `src/combat/api/timeline.rs`
- `assets/data/skills.ron`
- `tests/compiled_timeline_boot_validation.rs`
