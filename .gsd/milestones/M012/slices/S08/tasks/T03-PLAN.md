---
estimated_steps: 5
estimated_files: 4
skills_used:
  - test
  - verify-before-complete
---

# T03: Expose query-backed declarations in consumer snapshots

Why: S07 established that CLI/windowed consumers must stay thin and use the shared query surface. S08 must prove enemy trait and charged telegraph declarations are available to those consumers without local skill-ID or free-text trait heuristics.

Failure Modes (Q5): Consumer ECS queries can compile in headless but fail under `windowed` if tuple updates are missed; run both headless tests and `cargo check --features "dev windowed"`. Missing `SkillBook`/roster data should leave declaration display empty or reason-coded, not panic. Formatting must not decide legality locally.

Load Profile (Q6): Shared resources are in-memory snapshot vectors reused by CLI/windowed affordance rendering. Per operation is one snapshot build plus short declaration formatting. At 10x combatants, repeated per-widget queries would be wasteful, so derive declarations from the existing per-turn/frame snapshot rather than rebuilding per enemy card.

Negative Tests (Q7): Add source-scan or behavioral tests that consumers do not match `devimon`, `ogremon`, charged skill IDs, or `signature_traits` to decide declarations. Verify deferred/hidden reason codes remain visible in formatted labels and empty enemies do not emit false warnings.

Update CLI and windowed ECS unit queries to include the new `EnemyCounterplayKit` snapshot input from T02. Add small display/formatting helpers that consume `query_enemy_trait_affordances()` and `query_charged_telegraph_affordance()` results; they may render labels/reason codes but must not branch on enemy names, skill IDs, or free-text `signature_traits`. Keep this minimal: a CLI line or windowed enemy-card line is enough if low risk, but consumer tests are the proof. Extend `tests/action_affordance_consumers.rs` to build snapshots with declaration components, assert consumer-facing helpers expose implemented/deferred/hidden states from the query output, and extend the existing no-hardcoding scan to cover enemy counterplay declarations. Final verification should include scenario TTK tests to ensure no accidental canonical `Shielded`/counterplay behavior change altered boss/miniboss combat pacing.

## Inputs

- `src/combat/action_query.rs`
- `src/combat/bootstrap.rs`
- `src/bin/combat_cli.rs`
- `src/ui/combat_panel.rs`
- `tests/action_affordance_consumers.rs`
- `tests/enemy_counterplay_affordance.rs`

## Expected Output

- `src/bin/combat_cli.rs`
- `src/ui/combat_panel.rs`
- `tests/action_affordance_consumers.rs`

## Verification

cargo test-dev --test action_affordance_consumers && cargo test-dev --test scenario_boss_ttk --test scenario_miniboss_ttk && cargo check --features "dev windowed"
