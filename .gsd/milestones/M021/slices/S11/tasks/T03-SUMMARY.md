---
id: T03
parent: S11
milestone: M021
key_files:
  - src/combat/enemy_ai.rs
  - src/combat/preview.rs
  - src/combat/turn_system/mod.rs
  - src/headless.rs
  - src/windowed.rs
  - src/bin/combat_cli.rs
  - tests/enemy_ai.rs
  - tests/enemy_ai_preview.rs
key_decisions:
  - Use `try_query_skill_preview` to distinguish unavailable preview data from zero-damage previews, preserving deterministic fallback when scoring cannot be computed.
  - Route enemy turns through a small request queue and exclusive resolver so preview scoring can use `&mut World` without turning `advance_turn_system` into an exclusive monolith.
  - Keep deterministic tie-breaks on preview-scored candidates: higher damage first, then ultimate > skill > basic, then lowest target id, then earliest skill order.
duration: 
verification_result: passed
completed_at: 2026-05-17T07:50:30.530Z
blocker_discovered: false
---

# T03: Added preview-scored enemy action selection with a narrow world-backed bridge and deterministic fallback.

**Added preview-scored enemy action selection with a narrow world-backed bridge and deterministic fallback.**

## What Happened

Extended the pure enemy AI to score action/target pairs by shared preview-stream damage, keeping the legacy toughness-ratio route as a deterministic fallback when no preview can be produced. Moved enemy-turn intent emission out of `advance_turn_system` into a small request-queue bridge that runs with `&mut World`, queries shared preview streams, and emits exactly one `ActionIntent` per queued enemy turn. Updated headless, windowed, and CLI combat registration paths to initialize and schedule the bridge, and added focused pure/runtime tests covering higher-damage preview wins plus the stable runtime intent path.

## Verification

Verified with targeted cargo commands: `cargo test --test enemy_ai_preview` passed 2 tests (pure preview scoring + runtime bridge). `cargo test --test enemy_ai` passed 3 regression tests for the legacy fallback route. `cargo check --features windowed` completed successfully, confirming the windowed registration path still compiles with the new bridge wiring.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test enemy_ai_preview` | 0 | ✅ pass | 664ms |
| 2 | `cargo test --test enemy_ai` | 0 | ✅ pass | 681ms |
| 3 | `cargo check --features windowed` | 0 | ✅ pass | 3174ms |

## Deviations

Replaced the plan's implicit single-system enemy AI wiring with a small request-queue bridge plus an exclusive world-backed resolver to keep the main AV turn system non-exclusive.

## Known Issues

None.

## Files Created/Modified

- `src/combat/enemy_ai.rs`
- `src/combat/preview.rs`
- `src/combat/turn_system/mod.rs`
- `src/headless.rs`
- `src/windowed.rs`
- `src/bin/combat_cli.rs`
- `tests/enemy_ai.rs`
- `tests/enemy_ai_preview.rs`
