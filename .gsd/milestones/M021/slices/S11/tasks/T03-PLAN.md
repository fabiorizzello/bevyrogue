---
estimated_steps: 4
estimated_files: 8
skills_used: []
---

# T03: Route enemy action choice through preview-stream scoring and deterministic turn-system wiring

Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.

Why: the current enemy AI ignores skill timelines and only targets the lowest-toughness ally, so S11's AI half is incomplete even once preview querying exists.

Do: extend `src/combat/enemy_ai.rs` with preview-based candidate scoring helpers that rank skill/ultimate/basic options by summed preview-stream damage while keeping deterministic tie-breaks; add the narrow world-backed turn-system bridge needed to run preview queries during enemy turns without converting the whole turn advance system into an exclusive monolith; wire that bridge through headless, windowed, and CLI combat-system registration paths; preserve a deterministic fallback when preview data is unavailable; add focused tests that prove higher-damage preview candidates win and that the enemy turn flow still emits one stable `ActionIntent` in the registered runtime path.

Done when: enemy turns use the shared preview contract to choose the best available action/target pair, fallback behavior is deterministic, and both pure AI and runtime integration tests pass.

## Inputs

- `src/combat/enemy_ai.rs`
- `src/combat/turn_system/mod.rs`
- `src/headless.rs`
- `src/windowed.rs`
- `src/bin/combat_cli.rs`
- `src/combat/preview.rs`
- `tests/enemy_ai.rs`

## Expected Output

- `src/combat/enemy_ai.rs`
- `src/combat/preview.rs`
- `src/combat/turn_system/mod.rs`
- `src/headless.rs`
- `src/windowed.rs`
- `src/bin/combat_cli.rs`
- `tests/enemy_ai.rs`
- `tests/enemy_ai_preview.rs`

## Verification

cargo test --test enemy_ai_preview

## Observability Impact

Makes AI choice failures diagnosable by asserting preview-derived scores and fallback paths directly, instead of inferring behavior from toughness-ratio heuristics.
