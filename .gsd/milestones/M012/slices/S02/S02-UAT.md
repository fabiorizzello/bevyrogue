# S02: Enemy-only Toughness and TargetShape truthfulness — UAT

**Milestone:** M012
**Written:** 2026-04-30T21:32:43.650Z

# S02 UAT: Enemy-only Toughness and TargetShape truthfulness

## Preconditions
- Working tree is at M012 / S02 implementation state.
- Test data includes canonical allies, at least one positive-max enemy, and at least one zero-max enemy fixture.
- Cargo can run `cargo test-dev` and `cargo check --features "dev windowed"`.

## Scenario 1 — Ally toughness is hidden in headless and validation surfaces
1. Run `cargo test-dev --test bootstrap_spawn_composition --test validation_snapshot --test toughness_enemy_only`.
2. Inspect the validation snapshot assertions for an ally unit with internal toughness data.
3. Expected: the ally is reported as `N/A`/hidden, not as an exposed break target.
4. Expected: no error is raised just because the ally has internal toughness/weakness data.

## Scenario 2 — Zero-max enemy toughness is hidden, not exposed
1. Use the validation snapshot fixture for a zero-max enemy.
2. Expected: the enemy does not render a numeric break bar or weakness list as if it were a real break target.
3. Expected: hidden/`N/A` formatting is used consistently in headless and snapshot output.

## Scenario 3 — Positive enemy toughness stays truthful
1. Use a positive-max enemy fixture (e.g. the canonical enemy break target in the tests).
2. Expected: the enemy displays current/max toughness numerically and still exposes weakness information.
3. Expected: the combat panel and validation snapshot agree on visibility.

## Scenario 4 — Ally damage works without break side effects
1. Run `cargo test-dev --test toughness_enemy_only`.
2. Observe the ally-targeted damage case.
3. Expected: HP changes normally.
4. Expected: ally toughness does not decrement, `OnBreak` is not emitted, and stun-from-break is not applied.

## Scenario 5 — Enemy break still works
1. In the same toughness regression suite, run the enemy break case.
2. Expected: a weakness-matching hit that crosses toughness to zero or below still produces the break outcome.
3. Expected: the engine continues to treat enemy break as authoritative.

## Scenario 6 — Unsupported target shapes are rejected before mutation
1. Run `cargo test-dev --test target_shape_truthfulness`.
2. Use the Row and AllEnemies fixtures.
3. Expected: each action fails before mutation with a stable reason containing `UnimplementedTargetShape`.
4. Expected: no SP, HP, toughness, or action lifecycle side effects occur for the rejected action.

## Scenario 7 — Single-target skills still execute normally
1. In `target_shape_truthfulness`, run the Single-target fixture.
2. Expected: the action executes, mutates state, and emits ordinary lifecycle/core events.
3. Expected: the new rejection path does not block valid single-target combat.

## Scenario 8 — Windowed path still compiles
1. Run `cargo check --features "dev windowed"`.
2. Expected: the feature-gated UI path compiles successfully with the optional toughness query shape.
3. Expected: no hardcoded skill-ID UI exception was introduced to make the build pass.

## Edge Cases
- Missing enemy toughness on a unit that should expose a real bar must still fail loudly in diagnostics.
- Ally units with internal toughness/weakness data must remain damage-valid while staying hidden as break targets.
- Row/AllEnemies/SelfOnly remain explicitly deferred until later targeting work; they must not silently degrade to single-target behavior.
