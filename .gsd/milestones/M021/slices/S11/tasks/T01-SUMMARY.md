---
id: T01
parent: S11
milestone: M021
key_files:
  - src/combat/preview.rs
  - src/combat/mod.rs
  - src/combat/turn_system/pipeline.rs
  - tests/skill_preview.rs
key_decisions:
  - Shared preview uses the same timeline lookup and interning path as execute-mode before running `BeatRunner` in `SkillCtxMode::Preview`.
  - Preview consumers receive the pending intent stream directly and do not touch `intent_applier`, keeping world mutation out of the seam.
duration: 
verification_result: passed
completed_at: 2026-05-17T07:00:32.470Z
blocker_discovered: false
---

# T01: Added a shared skill-preview query seam that reuses timeline resolution and returns non-mutating preview intent streams.

**Added a shared skill-preview query seam that reuses timeline resolution and returns non-mutating preview intent streams.**

## What Happened

Introduced `src/combat/preview.rs` as the shared preview helper module. The new `query_skill_preview` function resolves the same compiled skill timeline path used by execute-mode, falls back to SkillBook compilation when a precompiled library entry is absent, runs `BeatRunner` under `SkillCtxMode::Preview`, and returns the pending `VecDeque<Intent>` without invoking `intent_applier`. I also wired the module into `combat::mod` for external access and updated the turn pipeline to reuse the shared timeline resolver so preview and execute stay aligned on the same timeline IDs. Finally, I added `tests/skill_preview.rs`, which proves preview/output parity against execute-mode intent shape for a timeline-backed skill and verifies the world stays unchanged after the preview call.

## Verification

`cargo test --test skill_preview -- --nocapture` passed. The integration test confirmed preview intent-shape parity with execute mode and checked that target HP and the shared intent queue remained unchanged after `query_skill_preview` returned.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test skill_preview -- --nocapture` | 0 | ✅ pass | 3232ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/preview.rs`
- `src/combat/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/skill_preview.rs`
