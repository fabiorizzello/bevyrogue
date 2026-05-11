# S02 Research — Enemy-only Toughness and TargetShape truthfulness

## Summary

S02 owns the first implementation part of R085 and supports R084. The concrete hazards are real: canonical `UnitDef`s currently give every ally a `toughness_max` and `weaknesses`, `spawn_unit_from_def` always inserts `Toughness`, and several display/snapshot paths assume every unit has a toughness bar. Separately, `TargetShape` is parsed from RON but `resolve_action`/`apply_effects` collapse every `Damage { target: ... }` into one `base_damage` applied only to the selected `ActionIntent.target`.

This slice should prove two things before S03-S07 build the DSL/query layer on top:

1. **Toughness is not exposed as an ally break affordance.** Either remove `Toughness` structurally from allies or, at minimum, make every display/query/snapshot path treat toughness as enemy-only and no-op toughness damage against allies.
2. **Non-single target shapes cannot silently execute as single-target.** Either implement truthful fanout for `Row`/`AllEnemies`, or reject/declare them as unimplemented before mutation. Because canonical data already contains six `Row` skills and follow-up tests use `heat_viper`, gating all `Row` skills will require coordinated data/test migration.

Recommended planner approach: split S02 into two vertical tasks: first make toughness exposure enemy-only with compatibility adaptations, then add a narrow TargetShape truthfulness layer plus tests that locks current behavior before S03 adds richer legality metadata.

## Requirements Targeting

- **R085 primary:** S02 directly advances enemy-only Toughness and TargetShape truthfulness. It should produce test evidence that ally units do not expose enemy toughness bars/break affordances and that `Row`/`AllEnemies` are not silently single-target.
- **R084 support:** S02 should avoid introducing UI/CLI skill-ID hardcoding. Any shape/toughness rule introduced here should be reusable by the later pure legality query, even if S04 owns the final API.
- Existing R073/R070/R071 are regression-sensitive: action lifecycle, follow-up FIFO, and resource tests exercise `Toughness` heavily and must remain green.

## Skill / Process Notes

- The installed **tdd** skill is relevant for this slice: write the failing observable contract first (ally toughness hidden/no component; non-single shape rejected or fanout), then make the minimal engine changes.
- The installed **test** skill is relevant for choosing targeted integration tests before full `cargo test-dev`.
- The installed **verify-before-complete** skill applies to executors: do not mark S02 complete without fresh command output in that same completion step.
- Skill discovery for directly relevant tech:
  - `mindrally/skills@rust` — 243 installs — `npx skills add mindrally/skills@rust`
  - `bfollington/terma@bevy` — 117 installs — `npx skills add bfollington/terma@bevy`
  - `sickn33/antigravity-awesome-skills@bevy-ecs-expert` — 108 installs — `npx skills add sickn33/antigravity-awesome-skills@bevy-ecs-expert`
  - Do not install automatically; these are only suggestions for the user.

## Implementation Landscape

### Data and DSL

- `src/data/skills_ron.rs`
  - Defines `TargetShape::{Single, Row, AllEnemies, SelfOnly}`.
  - `Effect::Damage { amount, target }` carries shape, but helpers in `src/combat/resolution.rs` currently discard it by returning only the first damage amount.
  - `SkillDef` has no explicit legality metadata yet; S03 owns that richer contract.
- `assets/data/skills.ron`
  - `Row` appears in six canonical/compatibility skills:
    - `heat_viper`
    - `greymon_ult`
    - `mega_blaster_aoe`
    - `kabuterimon_ult`
    - `kyubimon_ult`
    - `angemon_ult`
  - `AllEnemies` and `SelfOnly` are in the enum but unused in canonical data.
  - `angemon_ult` is especially risky: it mixes `Damage(Row)`, `ToughnessHit`, and `Revive(20)`. Current scalar resolution treats it as revive-first/target-state dependent, not a truthful mixed AoE/revive model.
- `src/data/units_ron.rs` / `assets/data/units.ron`
  - Every canonical ally has nonzero `toughness_max` and `weaknesses`; enemies are `Devimon tough=35`, `Goblimon tough=0`, `Ogremon tough=20`.
  - If S02 structurally removes ally `Toughness`, do not assume `UnitDef.weaknesses` can disappear; current damage tag weakness calculation also reads weaknesses from the `Toughness` component.

### Toughness spawn and runtime use

- `src/combat/bootstrap.rs`
  - `spawn_unit_from_def` always inserts `Toughness::with_category(def.toughness_max, def.weaknesses.clone(), def.toughness_category)` for all teams, including Taichi.
  - `taichi_def()` sets `toughness_max: 1`; if toughness becomes enemy-only, commander/tamer spawn is another special case to adapt.
- `src/combat/toughness.rs`
  - `Toughness` stores both break bar and `weaknesses` used by damage classification.
  - `apply_hit` already supports no-break categories and `break_sealed`; it has no team awareness.
  - Memory note: break occurs only when the same hit crosses from positive to `<=0` and the attack tag is a weakness. Non-weak hits can drain the bar without ever triggering break later.
- `src/combat/resolution.rs`
  - `apply_effects` requires `&mut Toughness` for the defender. It uses `defender_tough.weaknesses` for `calculate_damage` and `classify`, then applies `resolved.toughness_damage`.
  - If allies lose the `Toughness` component, this function must accept optional/no toughness. Otherwise enemy attacks against allies will silently abort in the pipeline before damage.
- `src/combat/turn_system/pipeline.rs`
  - `ResolveActorsQuery` already represents toughness as `Option<&mut Toughness>`, but `step_app` immediately does `let Some(mut defender_tough) = defender_tough else { return; };`.
  - That silent return is a major seam: change it to allow non-tough targets for HP damage/status/revive while no-oping break/toughness mutation when absent or hidden.
  - The system currently emits string reasons like `"Target is KO"`, not stable codes; S06/S04 later will normalize reasons. S02 should avoid adding bespoke one-off strings where possible.
- `src/combat/turn_system/mod.rs`
  - `advance_turn_system` snapshots `Option<&Toughness>` into `toughness_current/max` defaulting to `0/1` and feeds ally targets to enemy AI. This currently makes enemy AI choose by ally toughness ratio.
  - `src/combat/enemy_ai.rs` target selection is explicitly based on lowest `toughness_current / toughness_max`; if ally toughness becomes absent/hidden, all allies tie at default ratio unless the selection heuristic is changed. Decide whether this slice preserves old AI behavior or intentionally switches to HP/id targeting for allies.

### Display / affordance surfaces that will leak ally toughness unless adapted

- `src/headless.rs`
  - `HeadlessUnitsQuery` requires `&Toughness`; the startup roster snapshot prints `team={:?} tough={}/{} weak={:?}` for all units. This will either exclude units without Toughness or fail to compile if spawn removes the component.
  - Change query to `Option<&Toughness>` and only print enemy toughness/weakness, or print `tough=N/A` for allies.
- `src/ui/combat_panel.rs` (feature `windowed`)
  - `CombatPanelUnitsQuery` currently requires `&Toughness` for every displayed unit.
  - UI only renders toughness bars for enemies, which is good, but the required query will drop allies if allies lack the component. Change to `Option<&Toughness>` and require/unwrap only for `Team::Enemy` display.
- `src/bin/combat_cli.rs`
  - Dashboard query already uses `Option<&Toughness>` and prints `TGH: N/A` if missing.
  - Interactive target picker currently lists all non-KO units regardless of action legality; S07 owns full query integration, but S02 tests should not rely on CLI target picker correctness.
- `src/combat/observability.rs`
  - `capture_validation_snapshot` treats missing `Toughness` as `ValidationSnapshotError::MissingToughness` and formats toughness for all units.
  - Structural ally toughness removal requires either optional fields/default display (`tough=N/A`) or a team-aware snapshot model. Tests in `tests/validation_snapshot.rs` assert exact strings with ally toughness today.

### Tests likely affected

- Many integration tests manually spawn allies with `Toughness::new(...)`; structural removal from `spawn_unit_from_def` will not automatically change those fixtures. Executors should choose whether to update only canonical spawn tests first or broaden all helper fixtures.
- Exact snapshot expectations to update:
  - `tests/validation_snapshot.rs` currently expects ally toughness bars and also has a test where a unit missing Toughness errors.
- Canonical spawn/composition tests:
  - `tests/bootstrap_spawn_composition.rs`, `tests/roster_smoke.rs`, `tests/scenario_*`, and helpers in `tests/combat_coherence.rs`, `tests/pipeline_dispatch.rs`, `tests/follow_up_chains.rs`, `tests/follow_up_triggers.rs` often build `Toughness` manually from `UnitDef`.
- TargetShape-sensitive tests:
  - `tests/follow_up_triggers.rs` uses `heat_viper` as an enemy skill. Since `heat_viper` is `Damage(Row)`, any “reject non-single Row” approach will break those tests unless the fixture skill/data is changed.
  - The current suite does not appear to assert true AoE fanout for `Row`/`AllEnemies`; new tests must be added.

## Natural Seams / Task Boundaries

### Task 1 — Lock enemy-only Toughness exposure

Goal: establish what “enemy-only Toughness” means in runtime/display terms.

Recommended tests first:

- Canonical bootstrap: allies spawned by `spawn_unit_from_def`/`apply_composition` do not expose a toughness affordance, while enemies with positive `toughness_max` do.
- Goblimon/zero-max enemy behavior: either no `Toughness` component or present-but-hidden/no break bar, but test the chosen contract explicitly.
- Enemy attacking an ally still deals HP damage and can trigger low-HP follow-up even if the ally has no toughness component or no exposed toughness.
- Observability/windowed query compatibility: snapshot/UI query should not require ally `Toughness`.

Implementation seams:

- `src/combat/bootstrap.rs` for spawn policy.
- `src/combat/resolution.rs` and `src/combat/turn_system/pipeline.rs` for optional/no-op toughness application.
- `src/headless.rs`, `src/ui/combat_panel.rs`, `src/combat/observability.rs` for display/snapshot truthfulness.
- `src/combat/enemy_ai.rs` if ally target ranking should no longer use toughness ratio.

### Task 2 — Lock TargetShape truthfulness

Goal: make non-single shapes impossible to mistake for single-target execution.

Two viable approaches:

1. **Implement minimal fanout now** (`Row` and `AllEnemies` both mean “all living opposing units” until rows exist). This preserves canonical `Row` skills as usable and gives truthful AoE behavior, but requires pipeline changes because `ActionIntent` currently carries only one target and `ResolvedAction` carries only one `target`.
2. **Gate non-single shapes now** (`Row`/`AllEnemies` return `UnimplementedTargetShape`/action failed before mutation). This is smaller but requires changing canonical `Row` skills or tests that use them, otherwise existing combat behavior will regress.

Recommended for S02 planning: prefer **gating + data migration only if the user accepts canonical Row skills becoming unavailable/single**, otherwise implement a small fanout. Because the roadmap says Row/AllEnemies must not silently behave as single-target, doing nothing is not acceptable.

Implementation seams:

- Add a shape-preserving helper in `src/combat/resolution.rs` (e.g. `skill_damage_shape(&effects) -> Option<TargetShape>`) or carry shape on `ResolvedAction` in `src/combat/state.rs`.
- If gating: `step_declaration` can reject non-single before `InFlightAction` is emitted. It currently returns `None` silently; for truthful engine behavior, emit an `OnActionFailed` or make `step_app` emit before mutation. Later S06 can replace display strings with stable reason codes.
- If fanout: `step_app` needs target collection by team and repeated application per target, plus lifecycle/event depth semantics for multiple targets. This is riskier because follow-up triggers, KO, break, floating damage, status application, and turn advance all currently assume one defender.
- `SelfOnly` can stay deferred if canonical data does not use it, but add a test proving it does not route through ordinary enemy target selection silently.

## Risks and Surprises

- **Toughness carries weaknesses.** Removing ally `Toughness` structurally also removes the only runtime weakness list used by `calculate_damage`. If enemy-vs-ally weakness damage matters now, introduce a separate damage-affinity component or keep internal `Toughness` but hide/no-op break for allies. This is the biggest design seam for the planner.
- **Optional Toughness already exists in queries but not in application.** `ResolveActorsQuery` is optional, but `step_app` silently returns if target toughness is absent. This makes partial structural removal dangerous unless fixed immediately.
- **UI query currently requires Toughness even though it only renders enemy bars.** Removing ally components without updating `CombatPanelUnitsQuery` will make allies disappear from the windowed panel.
- **TargetShape is entangled with single-target lifecycle.** `ActionIntent`, `ResolvedAction`, logs, events, status application, turn advance, and floating damage all assume one `target`. Full AoE is a cross-cutting change; gating is much smaller but affects canonical skills.
- **Existing tests use Row skill IDs as ordinary single-target fixtures.** `heat_viper` appears repeatedly in follow-up tests; this will catch S02 changes but may require fixture updates.
- **CLI has an unrelated non-interactive target bug.** Non-interactive mode currently picks `Team::Ally` targets for a player action. S07 probably owns CLI legality integration; avoid mixing this into S02 unless it blocks tests.

## Recommendation

Plan S02 as targeted implementation, not broad legality architecture:

1. Decide the enemy-only toughness representation up front.
   - Most robust long-term: `Toughness` component only for enemies with a break bar; display/snapshot code handles `None`; damage weakness is separated or consciously deferred.
   - Lowest regression risk: keep `Toughness` component internally but add team-aware exposure/no-op break rules so allies do not show as break targets. This may conflict with the roadmap’s preferred “remove from allies” direction, so document the tradeoff if chosen.
2. Add tests around canonical bootstrap/display/snapshot rather than only unit-level `Toughness::apply_hit` tests.
3. Preserve `TargetShape` in `ResolvedAction` and add an explicit non-single behavior path.
4. For `Row`/`AllEnemies`, choose either minimal fanout or explicit rejection. If explicit rejection, update canonical data/tests so no shipped skill silently claims `Row` while executing as single-target.
5. Leave full S04 affordance statuses/reason-code types for the later query slice, but use reason text/code names aligned with `docs/skill_legality_contract.md` (`UnimplementedTargetShape`, `WrongSide`, etc.) so migration is mechanical.

## Verification Plan

Targeted commands for executors after each task:

- Toughness-focused:
  - `cargo test-dev --test bootstrap_spawn_composition --test validation_snapshot --test enemy_ai`
  - `cargo test-dev --test follow_up_triggers --test combat_coherence` if enemy attacks against allies or low-HP follow-ups are touched.
- TargetShape-focused:
  - Add a new integration test, suggested name `tests/target_shape_truthfulness.rs`, covering either:
    - `Row`/`AllEnemies` apply to all live opposing enemies and not allies/KO units; or
    - non-single shape fails before mutation with an `UnimplementedTargetShape`-aligned reason.
  - Then run `cargo test-dev --test target_shape_truthfulness --test follow_up_triggers`.
- Final S02 verification:
  - `cargo test-dev`
  - If `src/ui/combat_panel.rs` query types changed, also run `cargo check --features "dev windowed"` even though windowed is only required by later milestone acceptance; this catches the most likely optional-toughness compile regression.
