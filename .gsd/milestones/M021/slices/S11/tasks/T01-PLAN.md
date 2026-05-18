---
estimated_steps: 4
estimated_files: 4
skills_used: []
---

# T01: Add shared `query_skill_preview` seam and non-mutating preview-stream coverage

Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.

Why: S11 needs one canonical consumer seam for UI and AI, and today the only timeline runner path lives inside timeline-backed action execution where it immediately applies intents.

Do: introduce `src/combat/preview.rs` as the shared preview helper module; factor or mirror the compiled-timeline lookup/interning path from `src/combat/turn_system/pipeline.rs` so preview and execute resolve the same skill timeline IDs; expose `query_skill_preview` that runs `BeatRunner` with `SkillCtxMode::Preview` and returns the pending stream without touching `intent_applier`; add focused integration coverage that proves preview output matches execute-mode intent shape for a timeline-backed skill and that preview leaves world state unchanged after the call.

Done when: preview callers can obtain a deterministic `VecDeque<Intent>` for a skill cast without applying combat mutations, and the new test fixture proves both parity and no-world-mutation behavior.

## Inputs

- `src/combat/turn_system/pipeline.rs`
- `src/combat/mod.rs`
- `src/combat/api/mod.rs`
- `src/combat/api/runner.rs`
- `src/combat/api/skill_ctx.rs`
- `tests/timeline_mode_parity.rs`
- `src/data/skill_timeline.rs`

## Expected Output

- `src/combat/preview.rs`
- `src/combat/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/skill_preview.rs`

## Verification

cargo test --test skill_preview

## Observability Impact

Creates one inspectable preview seam for future tests and consumers, making preview-vs-execute mismatches localizable to timeline resolution or runner output rather than UI/AI code.
