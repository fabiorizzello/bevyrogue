---
estimated_steps: 5
estimated_files: 7
skills_used:
  - tdd
  - test
  - verify-before-complete
---

# T01: Enforce enemy-only toughness semantics in the combat pipeline

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

## Inputs

- `src/combat/toughness.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `tests/follow_up_triggers.rs`
- `tests/combat_coherence.rs`

## Expected Output

- `src/combat/toughness.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `tests/toughness_enemy_only.rs`

## Verification

cargo test-dev --test toughness_enemy_only --test follow_up_triggers --test combat_coherence

## Observability Impact

Changes event emission semantics for hidden/missing toughness: HP events should still appear, but `OnBreak`/`Stunned` should only appear for enemy break bars. A future agent can inspect `CombatEvent`/`ActionLog` in the new tests to diagnose regressions.
