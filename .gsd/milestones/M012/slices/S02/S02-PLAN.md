# S02: Enemy-only Toughness and TargetShape truthfulness

**Goal:** Make combat runtime and display surfaces truthful about two UI-blocking mechanics: Toughness is an enemy break affordance only, and non-single TargetShape skills can no longer execute as if they were ordinary single-target skills.
**Demo:** After this: ally Toughness no longer leaks as a break target/affordance, and Row/AllEnemies semantics cannot silently behave as single-target without the query reporting that limitation.

## Must-Haves

- Ally units never expose toughness bars, weakness lists, or break affordances in headless/windowed/validation surfaces, while enemies with positive `Toughness::max` still do.
- HP damage/status/revive paths still work when the defender does not expose toughness; ally-directed hits do not mutate ally break state or emit `OnBreak`.
- The enemy-only toughness rule is represented by reusable, team-aware helper functions rather than UI/CLI skill-ID hardcoding, so S04 can consume the same semantics in the pure legality query.
- `TargetShape::Row`, `TargetShape::AllEnemies`, and `TargetShape::SelfOnly` are preserved from skill data into resolved action metadata and are explicitly rejected/deferred before mutation with an `UnimplementedTargetShape`-aligned failure reason until a later slice implements full targeting fanout.
- Canonical row-shaped skills and existing follow-up/resource/action-lifecycle regressions remain green after fixture/test adjustments; no Row/AllEnemies skill silently behaves as single-target.

## Proof Level

- This slice proves: Integration proof: deterministic Rust integration tests exercise Bevy spawn/application/display paths and target-shape rejection through the actual action pipeline, plus a windowed compile check for UI query compatibility.

## Integration Closure

Consumes the S01 contract vocabulary in `docs/skill_legality_contract.md` and gap matrix in `docs/combat_ui_readiness_gap_matrix.md`. Introduces reusable engine helpers and failure reason text that S03/S04/S06 can migrate into formal DSL/query reason codes. Full legal target enumeration, fanout AoE execution, and CLI/windowed affordance adapters remain for S03-S07.

## Verification

- Runtime diagnostics remain event-bus based. This slice should emit/log `OnActionFailed { reason: "UnimplementedTargetShape" ... }` for rejected non-single target shapes and update validation snapshots/headless output so ally toughness appears as `N/A`/hidden instead of an enemy break bar. Future agents can inspect `CombatEvent`, `ActionLog`, and validation snapshot output to distinguish target-shape deferral from ordinary KO/SP failures.

## Tasks

- [x] **T01: Enforce enemy-only toughness semantics in the combat pipeline** `est:2h`
  Executor skills_used frontmatter expectation: `tdd`, `test`, `verify-before-complete`.

Why: R085 requires ally units not to be exposed as break targets, but `Toughness` currently doubles as the runtime weakness carrier. This task keeps the component available internally while making the enemy-only exposure/application rule explicit and reusable.

Do:
1. Add team-aware helpers in `src/combat/toughness.rs`, e.g. `exposes_toughness_affordance(team: Team, toughness: Option<&Toughness>) -> bool` and `can_apply_toughness_damage(team: Team, toughness: Option<&Toughness>) -> bool`, returning true only for `Team::Enemy` with a positive max bar.
2. Update `apply_effects` in `src/combat/resolution.rs` to accept the defender team and optional toughness data; continue calculating HP damage with existing weakness data when present, but no-op toughness damage/break/classification-as-break for allies or missing/hidden toughness.
3. Update `step_app` in `src/combat/turn_system/pipeline.rs` so missing or hidden defender toughness does not silently abort an action. HP damage/status/revive should still resolve; `OnBreak` and `Stunned` insertion should only happen when the helper says toughness damage applies.
4. Add `tests/toughness_enemy_only.rs` with deterministic integration tests proving an enemy attack can damage an ally without emitting `OnBreak` or changing ally break state, and an ally attack still breaks an enemy when weakness/toughness conditions are met.
5. Run targeted regressions for action lifecycle, follow-up FIFO, and resource-sensitive toughness behavior before broadening scope.

Failure Modes:
- Dependency: existing `Toughness` weakness storage. If omitted for allies, damage classification may change; preserve current weakness inputs until a later component split exists.
- Dependency: Bevy query optional component handling. If `Option<&mut Toughness>` is unwrapped too early, ally-targeted actions may silently return; tests must catch this.

Load Profile:
- Shared resources: Bevy ECS world queries and event bus.
- Per-operation cost: one or two extra branch checks per resolved hit; trivial.
- 10x breakpoint: none expected, but avoid extra world scans in the per-hit path.

Negative Tests:
- Boundary conditions: ally with a `Toughness` component and weakness tags must not expose/apply break; enemy with positive max must still expose/apply break; missing toughness must not abort HP damage.
- Error paths: KO/SP/commander failures should continue using existing failure paths and not be masked by the new optional-toughness handling.
  - Files: `src/combat/toughness.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/mod.rs`, `tests/toughness_enemy_only.rs`, `tests/follow_up_triggers.rs`, `tests/combat_coherence.rs`
  - Verify: cargo test-dev --test toughness_enemy_only --test follow_up_triggers --test combat_coherence

- [x] **T02: Hide ally toughness from headless, validation, and windowed display surfaces** `est:90m`
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
  - Files: `src/headless.rs`, `src/combat/observability.rs`, `src/ui/combat_panel.rs`, `tests/bootstrap_spawn_composition.rs`, `tests/validation_snapshot.rs`, `tests/toughness_enemy_only.rs`
  - Verify: cargo test-dev --test bootstrap_spawn_composition --test validation_snapshot --test toughness_enemy_only && cargo check --features "dev windowed"

- [x] **T03: Preserve TargetShape metadata and reject non-single shapes before mutation** `est:2h`
  Executor skills_used frontmatter expectation: `tdd`, `test`, `verify-before-complete`.

Why: `TargetShape` is currently parsed but discarded, causing `Row` skills to mutate exactly one selected target. R085 requires Row/AllEnemies to execute truthfully or be explicitly unavailable. S02 chooses explicit deferral/rejection as the smallest safe step before S03/S04 add full DSL/query targeting metadata.

Do:
1. Add `target_shape: TargetShape` (or equivalent shape metadata) to `ResolvedAction` in `src/combat/state.rs` and populate it from the first `Effect::Damage { target, .. }` in `resolve_action`; default no-damage utility/revive skills to `TargetShape::Single` unless the skill data says otherwise.
2. Add a reusable predicate/helper near resolution or targeting code, e.g. `target_shape_is_executable_now(shape)`, that returns true only for `Single` in S02 and names `UnimplementedTargetShape` for `Row`, `AllEnemies`, and `SelfOnly`. Keep reason wording aligned with `docs/skill_legality_contract.md` for S04/S06 migration.
3. In `step_declaration`, after resolving the action but before `OnActionDeclared`/mutation, emit/log an `OnActionFailed { reason: "UnimplementedTargetShape:<Shape>" }` (or equivalent stable string containing `UnimplementedTargetShape`) and return `None` for non-single shapes. Do not consume SP, energy, ultimate charge, HP, toughness, or action lifecycle events for rejected shapes.
4. Add `tests/target_shape_truthfulness.rs` proving a Row skill and an inline/canonical AllEnemies fixture fail before mutation with the `UnimplementedTargetShape` reason, and proving a Single skill still executes normally. Avoid fixtures under ignored paths; inline any synthetic skill book data in the test or use tracked `assets/data/skills.ron`.
5. Update existing tests or tracked canonical data only where they used Row skills as ordinary single-target fixtures. Prefer changing test fixtures to a true Single skill over adding skill-ID exceptions. Do not add CLI/windowed hardcoding.

Failure Modes:
- Dependency: action lifecycle event ordering. Rejected shapes should not emit a misleading declared/pre-app/applied/resolved lifecycle as if execution happened; existing R070 tests must remain clear.
- Dependency: canonical Row skills. Tests such as `follow_up_triggers` may need fixture updates because `heat_viper` is Row in `assets/data/skills.ron`.

Load Profile:
- Shared resources: skill book lookup and event bus.
- Per-operation cost: one shape match per declared action; trivial.
- 10x breakpoint: none expected.

Negative Tests:
- Malformed inputs: no-damage skill without a Damage effect should not panic and should keep existing utility/revive behavior.
- Error paths: Row/AllEnemies/SelfOnly reject before mutation and before resource spend; unknown skill still follows existing `None` behavior unless already handled elsewhere.
- Boundary conditions: Single shape continues to execute and emit ordinary lifecycle/core events.
  - Files: `src/data/skills_ron.rs`, `src/combat/state.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/mod.rs`, `tests/target_shape_truthfulness.rs`, `tests/follow_up_triggers.rs`, `assets/data/skills.ron`
  - Verify: cargo test-dev --test target_shape_truthfulness --test follow_up_triggers --test pipeline_dispatch && cargo test-dev

## Files Likely Touched

- src/combat/toughness.rs
- src/combat/resolution.rs
- src/combat/turn_system/pipeline.rs
- src/combat/turn_system/mod.rs
- tests/toughness_enemy_only.rs
- tests/follow_up_triggers.rs
- tests/combat_coherence.rs
- src/headless.rs
- src/combat/observability.rs
- src/ui/combat_panel.rs
- tests/bootstrap_spawn_composition.rs
- tests/validation_snapshot.rs
- src/data/skills_ron.rs
- src/combat/state.rs
- tests/target_shape_truthfulness.rs
- assets/data/skills.ron
