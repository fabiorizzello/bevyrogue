---
estimated_steps: 5
estimated_files: 3
skills_used:
  - test
  - verify-before-complete
---

# T01: Extract a shared UI/CLI affordance snapshot and selection helper

Executor task-plan frontmatter must include `skills_used: [test, verify-before-complete]`.

Create the reusable seam that lets CLI/windowed build affordance snapshots with real UI resources without changing S06's engine validation behavior. Keep the existing `build_snapshot_from_ecs()` SP-bypass path intact for `resolve_action_system()`, then add either a parameterized builder or a new public helper that accepts an explicit SP mode/value for UI/CLI. Add small pure helpers for consumer selection if useful, such as converting an action kind plus affordance into enabled target ids/labels, but do not encode skill IDs, target sides, KO rules, or ultimate readiness outside `query_action_affordance()`.

Failure Modes (Q5): If `SkillBookHandle` or a skill definition is missing, helpers must produce disabled/hidden query results from the existing query API rather than panicking. If the active actor is missing, the snapshot fallback should remain diagnosable through existing query reason codes.

Load Profile (Q6): Shared resources are only in-memory ECS/query snapshots; per-operation cost is one short snapshot allocation and one affordance traversal per rendered action. At 10x combatant count, the first concern is repeated snapshot rebuilding in UI frames, so helpers should build one snapshot per actor/frame and reuse it across action affordance calls.

Negative Tests (Q7): Include tests for real SP lower than a revive skill cost, SP-bypass remaining separate for engine parity, missing/disabled target reasons retained while the action resource is disabled, and an enabled Basic target selected from query output rather than local team/KO assumptions.

## Inputs

- ``src/combat/action_query.rs``
- ``src/combat/mod.rs``
- ``src/combat/turn_system/mod.rs``
- ``tests/action_affordance_query.rs``
- ``tests/engine_legality_integration.rs``
- ``assets/data/skills.ron``

## Expected Output

- ``src/combat/action_query.rs``
- ``src/combat/mod.rs``
- ``tests/action_affordance_consumers.rs``

## Verification

cargo test-dev --test action_affordance_consumers

## Observability Impact

The helper keeps reason-code diagnostics (`ActionStatus`, `ResourceStatus`, `TargetStatus`, `LegalityReasonCode`) available to both consumers and tests. Future agents inspect `tests/action_affordance_consumers.rs` failures to identify whether drift is in snapshot construction, resource truth, or target selection.
