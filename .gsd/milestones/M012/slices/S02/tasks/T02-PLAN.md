---
estimated_steps: 5
estimated_files: 6
skills_used:
  - tdd
  - test
  - verify-before-complete
---

# T02: Hide ally toughness from headless, validation, and windowed display surfaces

Executor skills_used frontmatter expectation: `tdd`, `test`, `verify-before-complete`.

Why: R085 is about UI truth, not just engine math. Even if the runtime keeps internal weakness data, player-facing and diagnostic surfaces must not show allies as enemy-style break targets.

Do:
1. Update headless roster printing in `src/headless.rs` so the unit query tolerates optional/hidden toughness and prints ally toughness/weakness as hidden or `N/A` instead of a numeric break bar.
2. Update `capture_validation_snapshot` and formatting in `src/combat/observability.rs` so missing or ally-hidden toughness is not an error, ally snapshots do not expose enemy break affordance data, and enemies with positive bars still report toughness truthfully. Preserve `MissingToughness` only for units where the helper says an exposed enemy bar is required.
3. Update the windowed query/rendering path in `src/ui/combat_panel.rs` to use `Option<&Toughness>` and the shared helper so allies cannot disappear from the panel and cannot render toughness bars.
4. Extend/update `tests/bootstrap_spawn_composition.rs`, `tests/validation_snapshot.rs`, and/or `tests/toughness_enemy_only.rs` to assert that canonical allies do not expose toughness affordances while Devimon/Ogremon-style positive-max enemies do. Include the zero-max enemy contract chosen by the helper (hidden/no exposed bar).
5. Run a windowed compile check because optional-toughness query changes often compile in headless but fail behind the feature gate.

Failure Modes:
- Dependency: feature-gated `src/ui/combat_panel.rs`. Headless tests will not type-check this path; `cargo check --features "dev windowed"` is required.
- Dependency: exact validation snapshot strings. Update tests intentionally so failures identify ally-hidden vs enemy-exposed mismatches.

Load Profile:
- Shared resources: display/snapshot iteration over all units.
- Per-operation cost: one helper branch per displayed/snapshotted unit; trivial.
- 10x breakpoint: none expected; keep formatting O(unit_count).

Negative Tests:
- Boundary conditions: ally with internal toughness data formats as hidden/`N/A`; enemy with zero max formats as hidden/no exposed bar; enemy with positive max formats numeric current/max and weaknesses.
- Error paths: a positive-max enemy missing `Toughness` should still produce a diagnostic failure rather than silently hiding a real enemy break bar.

## Inputs

- `src/combat/toughness.rs`
- `src/headless.rs`
- `src/combat/observability.rs`
- `src/ui/combat_panel.rs`
- `tests/bootstrap_spawn_composition.rs`
- `tests/validation_snapshot.rs`
- `tests/toughness_enemy_only.rs`

## Expected Output

- `src/headless.rs`
- `src/combat/observability.rs`
- `src/ui/combat_panel.rs`
- `tests/bootstrap_spawn_composition.rs`
- `tests/validation_snapshot.rs`
- `tests/toughness_enemy_only.rs`

## Verification

cargo test-dev --test bootstrap_spawn_composition --test validation_snapshot --test toughness_enemy_only && cargo check --features "dev windowed"

## Observability Impact

Validation snapshot and headless roster text become the diagnostic surfaces for toughness exposure. Failure output should make it clear whether a unit is hidden (`N/A`) or has an exposed enemy break bar.
