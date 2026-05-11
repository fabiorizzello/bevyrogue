# S01 Research — Test and artifact failure inventory

## Summary

S01 owns/supports these active requirements from the preloaded contract:

- **Owns R089** — M013 closure truth: current artifacts are contradictory/missing and must be made truthful.
- **Owns R090** — test failure classification: classify blocked/failing/stale signals before fixing/removing.
- **Supports R091** — green or fully explained regression baseline: this slice establishes the initial ledger.
- **Supports R098/R099** — CLI/shared-surface and artifact closure gaps: current CLI smoke has an asset-path/runtime gap and M013 artifacts are absent from the actual checkout.
- **Supports R100** — verification must stay deterministic/headless-first.

The first suite blocker is confirmed: `cargo test --no-run` fails before normal compilation because `Cargo.toml` declares `[[test]] name = "battery_loop_resolution" path = "tests/battery_loop_resolution.rs"`, but that file does not exist. Individual target compilation reveals additional blockers hidden behind that stale declaration: 25 integration test targets fail to compile, 26 compile, and 3 of the compiling targets fail at runtime. The current M013 artifact state on disk is also not what the preloaded milestone context described: `.gsd/milestones/M013/` contains only `M013-DISCUSSION.md` and `M013-PARKED.md`; no `M013-VALIDATION.md`, `MILESTONE-SUMMARY.md`, or `M013-CONTEXT.md` exists in the actual checkout.

## Recommendation

Plan S01 as a **failure-ledger + first unblocker** slice, not as a broad fix slice:

1. **Remove or replace the stale `battery_loop_resolution` manifest declaration first**, after naming the replacement coverage already present in `tests/battery_loop_kernel.rs` and `src/combat/battery_loop.rs`. No source file currently references `battery_loop_resolution`, and the existing `battery_loop_kernel` target compiles/runs green.
2. **Repair mechanical stale fixtures next**, in small batches by schema drift category (`SkillDef`, `UnitDef`, `RoundFlags`, duplicate fields) before addressing semantic gameplay failures.
3. **Treat Holy Support and Twin Core failures as architecture-drift candidates** for S02/S03, not blind compatibility work. They encode old APIs (`HolySupportAffordance`, `Effect::HolySupportRequest`, `TwinCoreState.resonance/heat`) that may contradict the M015 direction.
4. **Record M013 artifact mismatch explicitly** in closure work: current disk state is missing validation/summary artifacts, while preloaded context says validation existed and was needs-attention. S06 should supersede/repair truth rather than assuming either source is complete.
5. **Defer full `cargo test --no-fail-fast` until the no-run blockers are fixed.** For now, use the individual-target inventory below as the preliminary hidden-failure map.

## Implementation Landscape

### Test manifest and tree

- `Cargo.toml`
  - Has 10 explicit `[[test]]` declarations.
  - 9 declared files exist.
  - 1 declared file is missing: `tests/battery_loop_resolution.rs`.
- `tests/`
  - 51 `tests/*.rs` files exist.
  - Most integration tests are auto-discovered; several stale ones are not explicitly listed in `Cargo.toml` but still compile/run when the full suite is unblocked.

### Battery loop coverage

- `src/combat/battery_loop.rs`
  - Exists; 262 lines.
  - Contains `BatteryLoopDesignTag`, `BatteryLoopRequestKind`, `BatteryLoopState`, `BatteryLoopHook`, transition application systems, and snapshot support.
- `tests/battery_loop_kernel.rs`
  - Exists; 151 lines.
  - Compiles and runs green: 4 tests passed.
  - Tests cover clean start + third-hit energy grant, cap/blocked reason on fourth hit, underflow rejection, and tactical-cycle reset visibility after Bevy message flush.
- `tests/battery_loop_resolution.rs`
  - Missing.
  - No grep hits for `battery_loop_resolution` outside the manifest; likely a stale manifest/test target unless S02/S03 proves a missing resolution-layer contract.

### Schema/API surfaces causing stale tests

- `src/data/skills_ron.rs`
  - `SkillDef` currently requires `animation_sequence: Option<Vec<String>>` and `qte: Option<String>`.
  - 16 tests construct `SkillDef` without those fields.
  - `Effect` no longer exposes the Holy Support variants expected by `tests/holy_support_roster_contract.rs` (`HolySupportTag`, `HolySupportRequest`).
  - `TargetShape::SelfTarget` is no longer available to that stale test.
- `src/data/units_ron.rs`
  - `UnitDef` currently requires `twin_core: TwinCoreRosterMetadata` and `holy_support: HolySupportRosterMetadata`.
  - At least 2 tests construct `UnitDef` without those fields.
  - `follow_up_chains` and `roster_smoke` now duplicate `enemy_traits` and `charged_attack` in fixtures.
- `src/combat/action_query.rs`
  - The old direct `HolySupportAffordance` / `query_holy_support_affordance` API expected by `tests/holy_support_affordance.rs` no longer exists.
  - `ResourceKind::Grace` and `ResourceKind::MartyrLight` expected by that test no longer exist.
- `src/combat/twin_core.rs`
  - `TwinCoreState` currently has `active_thermal_spark_targets`, `cross_resonance`, `fire_spend_markers`, `ice_spend_markers`, `twin_burst_used_this_cycle`, `shatter_used_this_cycle`, and `last_signal`.
  - Stale tests still expect old `resonance` and `heat` fields.
- `docs/combat_ui_readiness_gap_matrix.md`
  - Missing.
  - `tests/ui_readiness_gap_matrix_docs.rs` uses `include_str!("../docs/combat_ui_readiness_gap_matrix.md")`, so it fails at compile time.

### M013 artifact landscape

Actual checkout inventory differs from preloaded context:

- `.gsd/STATE.md` exists and says active milestone is M015; registry marks M013 paused (`⏸️`) and M015 active (`🔄`).
- `.gsd/milestones/M013/M013-DISCUSSION.md` exists.
- `.gsd/milestones/M013/M013-PARKED.md` exists.
- `.gsd/milestones/M013/M013-VALIDATION.md` **does not exist**.
- `.gsd/milestones/M013/MILESTONE-SUMMARY.md` **does not exist**.
- `.gsd/milestones/M013/M013-CONTEXT.md` **does not exist**.

This is an artifact truth gap, not a gameplay regression. S06 should decide whether to repair M013 artifacts in place or supersede them with M015 closure evidence.

### CLI smoke gap

- `src/bin/combat_cli.rs` compiles when invoked through `cargo run --bin combat_cli`.
- Running from the assigned auto-mode worktree fails before shared-surface proof:
  - panic at `src/bin/combat_cli.rs:651`
  - `assets/data/units.ron not found`
  - the code reads `std::fs::read_to_string("assets/data/units.ron")` relative to process cwd.
- Root assets do exist at package root (`assets/data/units.ron`, `skills.ron`, `party.ron`), but not in the assigned `.gsd/worktrees/M013` cwd.
- This is a likely CLI portability/working-directory gap for S05. Either the CLI should resolve assets via `CARGO_MANIFEST_DIR`/a passed data-dir, or verification must run from package root. Given auto-mode constraints say not to `cd`, code-level path resilience is safer.

## Failure Ledger

### Blocking command

| Command | Exit | Classification | Evidence |
|---|---:|---|---|
| `cargo test --no-run` | 101 | **Stale manifest/declaration** | `Cargo.toml` declares `tests/battery_loop_resolution.rs`, but the file is absent. This prevents the suite from reaching later compile/runtime failures. |

### Explicit `[[test]]` declarations

| Target | Status | Classification | Notes |
|---|---|---|---|
| `twin_core_roster_contract` | compile ok, runtime ok | Covered/current | 1 passed. |
| `twin_core_resolution` | compile ok, runtime ok | Covered/current | 1 passed. |
| `twin_core_integration` | compile ok, runtime fail | **Real regression or stale assertion** | Fails `canonical_fire_ice_twin_core_loop_is_visible_through_validation_snapshots` at `tests/twin_core_integration.rs:153`. Needs S02/S03 review against current Twin Core model. |
| `holy_support_mechanics` | compile ok, runtime fail | **Real regression or missing Bevy resource setup** | 4 failures panic in Bevy `system_param.rs:851`; likely missing required resource/system setup in test harness or runtime plugin. |
| `holy_support_resolution` | compile ok, runtime fail | **Mixed real regression/stale assertion** | Patamon test assertion fails at line 144; Angemon test panics in Bevy system param. |
| `holy_support_affordance` | compile fail | **Obsolete test API** | Imports removed `HolySupportAffordance` / `query_holy_support_affordance`; expects removed `ResourceKind::Grace/MartyrLight`. |
| `holy_support_roster_contract` | compile fail | **Obsolete RON/test contract** | Expects removed `Effect::HolySupportTag`, `Effect::HolySupportRequest`, and `TargetShape::SelfTarget`. |
| `battery_loop_resolution` | missing | **Stale manifest/declaration or missing deleted coverage** | No file and no code hits. Existing `battery_loop_kernel` replacement coverage is green; planner should confirm whether resolution-layer coverage is still needed. |
| `battery_loop_kernel` | compile ok, runtime ok | Covered/current replacement candidate | 4 passed; likely replacement coverage for stale `battery_loop_resolution`. |
| `predator_loop_kernel` | compile ok, runtime ok | Covered/current | 7 passed. |

### Auto-discovered test compile blockers

Aggregate categories from individual `cargo test --no-run --test <name>`:

| Category | Affected targets | Classification |
|---|---|---|
| `SkillDef` fixture missing `animation_sequence`/`qte` | `action_affordance_consumers`, `action_affordance_query`, `boundary_contract`, `combat_coherence`, `damage_breakdown_log`, `encounter_e2e`, `engine_legality_integration`, `event_stream`, `patamon_revive`, `resource_caps`, `revive_semantics`, `sp_economy`, `status_effect_apply`, `status_effect_integration`, `toughness_categories`, `ultimate_meter` | Mechanical stale test fixtures after presentation metadata fields were added. |
| `UnitDef` fixture missing `twin_core`/`holy_support` | `bootstrap_spawn_composition`, `tempo_resistance` | Mechanical stale test fixtures after roster metadata fields were added. |
| Duplicate `UnitDef` fields | `follow_up_chains`, `roster_smoke` | Mechanical fixture drift: `enemy_traits` and `charged_attack` specified twice. |
| `RoundFlags` fixture missing new fields | `resource_caps` | Mechanical stale fixture. |
| Obsolete `TwinCoreState.resonance/heat` API | `twin_core_mechanics`, `validation_snapshot` | Stale tests or architecture-drift candidate; current state uses `cross_resonance` plus spark/spend/guard fields. |
| Missing doc artifact | `ui_readiness_gap_matrix_docs` | Artifact gap: missing `docs/combat_ui_readiness_gap_matrix.md`; test references old R085/D053/D054 vocabulary. |

### Individually compiling tests with runtime status

- **Runtime green (23 targets):** `twin_core_roster_contract`, `twin_core_resolution`, `battery_loop_kernel`, `predator_loop_kernel`, `commander_flow`, `enemy_ai`, `enemy_counterplay_affordance`, `follow_up_triggers`, `form_identity`, `party_config_validation`, `party_selection_validation`, `pipeline_dispatch`, `roster_catalog`, `scenario_boss_ttk`, `scenario_miniboss_ttk`, `scenario_minion_ttk`, `skill_legality_contract_docs`, `status_accuracy`, `status_effect_turn_tick`, `target_shape_truthfulness`, `toughness_enemy_only`, `triangle_matchup`, `turn_system_av`.
- **Runtime red (3 targets):** `twin_core_integration`, `holy_support_mechanics`, `holy_support_resolution`.
- Runtime red tests should be classified by S02/S03 before fixes because they touch exactly the architecture-drift zone: kernel hooks, validation snapshots, and line/mechanic modules.

## Natural Seams for Planning

1. **Manifest/test-target unblocker**
   - Files: `Cargo.toml`, possibly `tests/battery_loop_resolution.rs` only if replacement coverage is insufficient.
   - Work: remove stale target or recreate a current-architecture test.
   - Verify: `cargo test --no-run` should progress past the missing-target error.

2. **Mechanical fixture schema updates**
   - Files: tests constructing `SkillDef`, `UnitDef`, `RoundFlags`.
   - Work: add current fields/default helpers; prefer shared test constructors to prevent repeated drift.
   - Verify: individual `cargo test --no-run --test <affected>` for the category.

3. **Obsolete Holy Support API tests**
   - Files: `tests/holy_support_affordance.rs`, `tests/holy_support_roster_contract.rs`, plus current `src/combat/action_query.rs`, `src/data/skills_ron.rs`, `assets/data/skills.ron` for contract comparison.
   - Work: decide whether tests should be rewritten to current shared affordance/custom-signal boundary or removed with named replacement coverage.
   - Verify: compile and then runtime for those tests.

4. **Twin Core stale-state assertions**
   - Files: `tests/twin_core_mechanics.rs`, `tests/validation_snapshot.rs`, `tests/twin_core_integration.rs`, `src/combat/twin_core.rs`, `src/combat/observability.rs`.
   - Work: update expectations from old `resonance/heat` to current spark/cross-resonance/snapshot contract only if S02 confirms current model is canonical.
   - Verify: `cargo test --test twin_core_mechanics --test validation_snapshot --test twin_core_integration` (or individual equivalents).

5. **GSD/document artifacts**
   - Files: `.gsd/milestones/M013/*`, `.gsd/STATE.md`, `docs/combat_ui_readiness_gap_matrix.md`, `tests/ui_readiness_gap_matrix_docs.rs`.
   - Work: classify whether missing docs are obsolete M012/M013 artifacts or still-required contract docs. S06 should repair/supersede M013 artifacts.
   - Verify: doc test compiles or is removed/replaced with current M015 artifact checks.

6. **CLI cwd/data-dir gap**
   - Files: `src/bin/combat_cli.rs`.
   - Work: resolve asset paths robustly or add CLI arg/env for data dir. Later S05 must prove shared action query/event/beat/snapshot surfaces.
   - Verify: `cargo run --bin combat_cli` from the assigned worktree should no longer panic on missing assets.

## Risks and Pitfalls

- **Do not fix obsolete tests by restoring old APIs blindly.** `HolySupportAffordance`, `Effect::HolySupportRequest`, and `TwinCoreState.resonance/heat` may be stale from prior passes and conflict with the desired RON custom signal → per-Digimon/module hook → kernel observable model.
- **The stale `battery_loop_resolution` declaration hides many later failures.** Removing it is necessary but not sufficient.
- **Current disk GSD state conflicts with preloaded milestone context.** Treat artifact truth as an explicit ledger item; do not assume missing validation files exist elsewhere.
- **Bevy message/resource setup remains fragile.** Memory notes warn that tests which enqueue combat messages must advance schedules before snapshot assertions; Holy Support runtime panics may also be missing resources/plugins rather than gameplay bugs.
- **The CLI run is cwd-sensitive.** `cargo run` from auto-mode worktree compiles but panics because assets are looked up relative to cwd, not package root.

## Verification Plan for Executors

Use this order:

1. `cargo test --no-run`
   - Expected before first fix: fails on missing `battery_loop_resolution`.
   - Expected after manifest fix: reveals compile errors from stale test fixtures.
2. For each mechanical batch:
   - `cargo test --no-run --test <target>` for each affected target.
3. After compile blockers are cleared:
   - `cargo test --no-fail-fast` to classify all remaining runtime failures.
4. For battery loop replacement proof:
   - `cargo test --test battery_loop_kernel` (currently green: 4 passed).
5. For current runtime reds:
   - `cargo test --test twin_core_integration -- --nocapture`
   - `cargo test --test holy_support_mechanics -- --nocapture`
   - `cargo test --test holy_support_resolution -- --nocapture`
6. For CLI gap:
   - `cargo run --bin combat_cli` from auto-mode cwd should not panic on `assets/data/units.ron` after S05 repairs.

## Skill Discovery

Installed/project skills do not include Rust- or Bevy-specific skills. `npx skills find` found potentially relevant external skills, but none were installed:

- Rust:
  - `npx skills add apollographql/skills@rust-best-practices` (9.6K installs) — broadly relevant for Rust review.
  - `npx skills add affaan-m/everything-claude-code@rust-testing` (2.8K installs) — directly relevant to test repair.
- Bevy:
  - `npx skills add bfollington/terma@bevy` (122 installs)
  - `npx skills add sickn33/antigravity-awesome-skills@bevy-ecs-expert` (111 installs)
  - `npx skills add laurigates/claude-plugins@bevy-ecs-patterns` (26 installs)

The most directly relevant candidate for this milestone is a Bevy ECS expert/testing skill, but S01 had enough local evidence without installing anything.

## Evidence Sources

- `gsd_exec 4d879e48-736b-493b-a15e-5f612dcb9317` — `cargo test --no-run` missing `battery_loop_resolution` blocker.
- `gsd_exec 0b76ea53-c656-4931-93bc-8ae1c7d2ce2d` — manifest test declaration and test tree inventory.
- `gsd_exec 8d68db4e-4c37-43c0-a443-ef133570d428` — battery loop files and M013 artifact existence summary.
- `gsd_exec 76dfae30-9eb6-4fa6-a2fb-8074a73e7ac8` — individual integration-test no-run compile inventory.
- `gsd_exec d7a9a0e2-0fa5-4184-81e7-e0c2a0cbc2b7` — runtime inventory for compiling tests.
- `gsd_exec 03a3746d-0b08-4f20-9e23-121d10413788` — detailed runtime failure excerpts for red compiling tests.
- `gsd_exec 5ca5a73e-745d-47b7-8feb-6179479a4553` — aggregate compile failure categories.
- `gsd_exec b43e6835-585f-4bb4-9da7-817dc1b48293` — missing UI readiness doc and stale-symbol scan.
- `gsd_exec 277587f7-a4f3-45e1-ae4d-d7e385e964c2` and `697f7eb9-795b-4a2d-88e5-9dcd1cf8bc4e` — CLI asset-path smoke failure and asset/code path evidence.
- Memory notes used: `MEM008`, `MEM012`, `MEM043`, `MEM050`.