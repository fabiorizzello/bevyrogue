---
estimated_steps: 5
estimated_files: 2
skills_used:
  - test
  - verify-before-complete
---

# T02: Wire combat_cli action and target menus through query affordances

Executor task-plan frontmatter must include `skills_used: [test, verify-before-complete]`.

Replace `player_action_system()`'s local action/target affordance decisions with the shared query-backed helper from T01. Add `SkillBook` access, real `SpPool.current`, and any missing unit snapshot inputs (`Toughness`, `Stunned`, `Energy`, `RoundEnergyTracker`, `Commander`) needed by the helper. In non-interactive mode, emit the first enabled target for `ActionQueryKind::Basic` from `ActionAffordance.targets`; if no enabled target exists, do not recreate the old live-ally fallback silently—choose a query-explained safe fallback or exit the turn in a way consistent with existing CLI behavior. In interactive mode, render Basic, skills, and Ultimate with enabled/disabled/deferred/hidden labels/reasons from `ActionStatus` and resource details, allow only enabled action choices, and build the target prompt from `TargetAffordance` entries so KO allies can appear for revive-like skills without a revive branch.

Failure Modes (Q5): If assets are not loaded yet or a selected skill is missing from the book, the CLI should not panic or emit a guessed intent; it should show the query-derived unavailable reason or fall back to an enabled Basic affordance if one exists. Canceled prompts should continue to choose an enabled query-backed default.

Load Profile (Q6): Shared resources are the ECS snapshot and terminal prompt lists. Per turn cost should be one snapshot plus one affordance per displayed action; avoid rebuilding the snapshot separately for every target prompt.

Negative Tests (Q7): Extend consumer tests/helper tests to cover Basic default choosing an enemy enabled by query, disabled/deferred actions not selected, and revive target entries retaining KO allies plus disabled live allies/enemies. Add a static no-hardcoding assertion if practical to catch `patamon_revive` or reason-code branches in CLI.

## Inputs

- ``src/bin/combat_cli.rs``
- ``src/combat/action_query.rs``
- ``tests/action_affordance_consumers.rs``
- ``assets/data/skills.ron``

## Expected Output

- ``src/bin/combat_cli.rs``
- ``tests/action_affordance_consumers.rs``

## Verification

cargo test-dev --test action_affordance_consumers && cargo check --bin combat_cli

## Observability Impact

CLI output should surface query reason labels for disabled/deferred/hidden actions and targets, giving future agents a direct terminal-visible explanation before any `ActionIntent` is emitted. Engine `OnActionFailed` remains the post-emission diagnostic safety net.
