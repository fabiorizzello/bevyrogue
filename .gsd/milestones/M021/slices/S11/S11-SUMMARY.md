---
id: S11
parent: M021
milestone: M021
provides:
  - A reusable preview-stream contract for downstream consumer wiring.
  - Cacheable preview summaries for the windowed UI.
  - Deterministic preview-based enemy scoring with fallback behavior.
requires:
  []
affects:
  []
key_files:
  - src/combat/preview.rs
  - src/combat/mod.rs
  - src/combat/turn_system/pipeline.rs
  - src/ui/combat_panel.rs
  - src/windowed.rs
  - src/combat/enemy_ai.rs
  - src/combat/turn_system/mod.rs
  - src/headless.rs
  - src/bin/combat_cli.rs
  - tests/skill_preview.rs
  - tests/windowed_preview_cache.rs
  - tests/enemy_ai.rs
  - tests/enemy_ai_preview.rs
key_decisions:
  - Shared preview uses the same timeline lookup and interning path as execute-mode, then runs BeatRunner in `SkillCtxMode::Preview`.
  - Windowed preview summaries are cached only after a successful refresh and are rendered only when actor, skill kind, and target still match the cached context.
  - Enemy turn scoring must distinguish unavailable preview data from a zero-damage preview so deterministic fallback remains intact.
patterns_established:
  - Single shared preview contract for both UI and AI consumers.
  - Preview cache invalidation on actor/kind/target mismatch.
  - Deterministic fallback when preview scoring is unavailable.
  - Request-queue plus exclusive resolver bridge for narrow world-backed preview scoring.
observability_surfaces:
  - Focused integration tests for preview parity and non-mutation.
  - `PreviewCache` resource state in the windowed client.
  - Enemy-AI preview-score assertions and fallback-path assertions.
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-17T08:15:13.642Z
blocker_discovered: false
---

# S11: UI and AI consumers via SkillCtx Preview

**Added a shared preview-stream contract for combat UI and enemy AI so both consumers can read preview damage without mutating combat state.**

## What Happened

T01 introduced the shared `query_skill_preview` seam in `src/combat/preview.rs`, reusing the same compiled-timeline lookup and BeatRunner execution path as live skill resolution while running in `SkillCtxMode::Preview` and returning the pending intent stream without touching world state. T02 wired the windowed combat panel through a cached preview bridge that refreshes only after a successful shared-preview read, then renders previewed damage only when the actor, skill kind, and target still match the cached context. T03 replaced the old toughness-ratio-only enemy scoring path with preview-based action evaluation, using a narrow request-queue plus exclusive resolver bridge so the main turn system stays non-exclusive while still preserving deterministic fallback behavior whenever preview data is unavailable.

Across the slice, the consumer-facing preview path is now shared end to end: the UI and the enemy AI both consume the same preview-stream contract, the cache is validated against the current combat context before rendering, and the AI ranking path uses preview availability as a first-class signal instead of conflating it with zero-damage results. The shipped behavior was verified both through focused integration tests and through compile checks for the headless and windowed entrypoints.

## Verification

Verified with `cargo test --test skill_preview -- --nocapture` (passed), `cargo test --features windowed --test windowed_preview_cache -- --nocapture` (passed), `cargo test --test enemy_ai -- --nocapture` (passed), `cargo test --test enemy_ai_preview -- --nocapture` (passed), `cargo check` (passed), and `cargo check --features windowed` (passed). The test suite confirmed preview-stream parity, non-mutating preview execution, cached UI refresh behavior, deterministic enemy preview scoring, legacy fallback preservation, and successful headless/windowed compilation.

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

T03 used a narrow request-queue plus exclusive resolver bridge for enemy turns instead of a single monolithic system, so the main turn system stays non-exclusive.

## Known Limitations

The slice is proven by integration tests and compile checks rather than a manual end-user playthrough; extended runtime behavior beyond the exercised test cases remains unverified.

## Follow-ups

None.

## Files Created/Modified

None.
