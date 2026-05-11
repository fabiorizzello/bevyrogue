---
estimated_steps: 5
estimated_files: 2
skills_used:
  - test
  - verify-before-complete
---

# T03: Drive windowed action buttons and ally/enemy targets from query affordances

Executor task-plan frontmatter must include `skills_used: [test, verify-before-complete]`.

Update `combat_panel()` so the active actor's action controls and target picking are derived from the same `query_action_affordance()` output used by CLI. Expand `CombatPanelUnitsQuery` or adjacent queries to include the snapshot inputs missing today (`Commander`, `Energy`, `RoundEnergyTracker`, and real SP) while preserving headless-first feature gating. Use `ActionStatus::Enabled` for Basic/Skill/Ultimate button enablement and show query reasons for disabled/deferred/hidden states using concise labels or egui hover text. When an action is pending, render both ally cards and enemy cards as potential targets and only emit `ActionIntent` when the matching `TargetAffordance.status` is `Enabled`; disabled/deferred/hidden targets should remain visible with reason text and must not be clickable. Align the active actor source with `TurnOrder.active_unit` where available, falling back only as needed for existing preview display, to avoid `NotActiveUnit` button disablement caused by stale preview state.

Failure Modes (Q5): If the skill book is temporarily unavailable, controls should disable with an explainable unavailable state instead of using local ultimate/KO/team logic. If the selected pending action becomes disabled after state changes, clear or disable the pending action without emitting a stale intent.

Load Profile (Q6): Shared resources are egui frame rendering and ECS snapshot traversal. Per frame cost should be one snapshot for the active actor plus affordance calls for visible actions; avoid rebuilding per card or per button.

Negative Tests (Q7): Consumer tests should cover helper mapping for target enablement so KO ally targeting is proven without needing to drive egui. The final verification must include windowed compilation to catch feature-gated imports/query signature drift.

## Inputs

- ``src/ui/combat_panel.rs``
- ``src/combat/action_query.rs``
- ``tests/action_affordance_consumers.rs``
- ``assets/data/skills.ron``

## Expected Output

- ``src/ui/combat_panel.rs``
- ``tests/action_affordance_consumers.rs``

## Verification

cargo test-dev --test action_affordance_consumers && cargo check --features "dev windowed"

## Observability Impact

Windowed affordances should visibly expose the same reason-code state used by tests. Future agents can inspect disabled action labels/tooltips and target reason text, then correlate any emitted illegal intent with `ActionLog::ActionFailed` / `OnActionFailed`.
