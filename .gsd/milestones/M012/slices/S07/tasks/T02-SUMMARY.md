---
id: T02
parent: S07
milestone: M012
key_files:
  - src/bin/combat_cli.rs
  - tests/action_affordance_consumers.rs
key_decisions:
  - Build one shared snapshot per turn and reuse query affordances for both action and target menus.
  - Prefer query-backed Basic fallback or a skipped turn over emitting a guessed intent when Basic has no enabled target.
  - Surface query reason labels directly in CLI output for disabled/deferred/hidden actions and targets.
duration: 
verification_result: passed
completed_at: 2026-05-01T13:54:23.721Z
blocker_discovered: false
---

# T02: Routed combat_cli action and target menus through query-backed affordances.

**Routed combat_cli action and target menus through query-backed affordances.**

## What Happened

Updated the CLI combat turn handler to build a single ECS snapshot with the real SpPool.current value plus the missing snapshot inputs (Commander, Toughness, Stunned, Energy, RoundEnergyTracker) and then query ActionAffordance data for Basic, skills, and Ultimate. The non-interactive path now uses the query-backed Basic affordance and its first enabled target, while the interactive path prints enabled/disabled/deferred/hidden action and target labels from the shared legality query, prompts only from enabled entries, and falls back to the first enabled choice on cancel.

I also removed the old local KO/team selection logic so revive-like skills can surface KO allies through TargetAffordance entries without a revive-specific branch. When the skill book is unavailable, the CLI now surfaces query-derived MissingSkill diagnostics instead of guessing an intent. The consumer tests were extended to cover the Basic enemy default, deferred-action exclusion, revive KO/live target handling, and a static no-hardcoding scan for the CLI source.

## Verification

`cargo test-dev --test action_affordance_consumers` passed with 5/5 tests green.
`cargo check --bin combat_cli` passed with exit code 0.

A repo-wide `cargo fmt --check` was attempted earlier, but it reported unrelated formatting drift across many pre-existing files; it was not part of the task gate and did not block the required verification commands.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test action_affordance_consumers` | 0 | ✅ pass | 4000ms |
| 2 | `cargo check --bin combat_cli` | 0 | ✅ pass | 980ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/bin/combat_cli.rs`
- `tests/action_affordance_consumers.rs`
