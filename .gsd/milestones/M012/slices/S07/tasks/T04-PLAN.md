---
estimated_steps: 5
estimated_files: 3
skills_used:
  - test
  - verify-before-complete
---

# T04: Verify no CLI/windowed legality hardcoding remains and run full S07 gates

Executor task-plan frontmatter must include `skills_used: [test, verify-before-complete]`.

Perform the final integration pass for S07. Clean up any duplicated local legality logic left after T02/T03, especially target filters based on `ko.is_none()`, enemy-only clickability, local ultimate readiness as an enablement source, skill-ID-specific branches, or direct matching on reason codes to decide legality. Keep display formatting allowed, but all legal/illegal/deferred/hidden decisions must originate from `ActionAffordance`, `TargetAffordance`, resource details, or helpers backed directly by `query_action_affordance()`.

Failure Modes (Q5): If static scans find legacy filters, treat them as blockers because they can diverge from the DSL query. If full tests fail, preserve the query source-of-truth and fix adapters rather than weakening engine parity.

Load Profile (Q6): Final scans and tests are local development commands; no runtime shared resources beyond cargo build/test outputs.

Negative Tests (Q7): The static scan is the negative test for forbidden skill-ID-specific/hardcoded affordance paths; targeted tests are the negative tests for disabled actions and invalid targets. Do not claim completion without fresh verification output per `verify-before-complete`.

## Inputs

- ``src/bin/combat_cli.rs``
- ``src/ui/combat_panel.rs``
- ``src/combat/action_query.rs``
- ``tests/action_affordance_consumers.rs``

## Expected Output

- ``src/bin/combat_cli.rs``
- ``src/ui/combat_panel.rs``
- ``tests/action_affordance_consumers.rs``

## Verification

cargo test-dev --test action_affordance_consumers && cargo test-dev --test action_affordance_query && cargo test-dev --test engine_legality_integration && cargo test-dev && cargo check --features "dev windowed"

## Observability Impact

The final verification leaves fresh command output proving both preflight affordance diagnostics and runtime failure diagnostics remain aligned; future agents should start with `tests/action_affordance_consumers.rs` and the static scan if drift reappears.
