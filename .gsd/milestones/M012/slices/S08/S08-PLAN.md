# S08: Enemy counterplay declaration surface

**Goal:** Expose enemy counterplay traits and charged attack telegraphs through the shared query surface so UI/CLI/engine-adjacent consumers can distinguish implemented mechanics (Tempo Anchor, Break Seal) from deferred or hidden declarations without hardcoding skill IDs or free-text traits.
**Demo:** After this: enemy traits and charged attacks have a queryable UI contract even if only Tempo Anchor/Break Seal execute today.

## Must-Haves

- R085 primary coverage: typed enemy counterplay trait declarations and charged attack declarations exist in tracked data/schema and do not rely on `signature_traits` free text for UI-readiness.
- R084 supporting coverage: declarations survive from `assets/data/units.ron` through runtime ECS/snapshot wiring into pure query functions that return stable `LegalityReasonCode` values.
- Query tests in `tests/enemy_counterplay_affordance.rs` prove `TempoAnchor` and shielded `BreakSeal` are `Implemented`, while `TypeTrap`, `ReactiveArmor`, and charged attacks remain `Deferred` with `EnemyTraitDeferred` / `ChargedTelegraphDeferred`.
- Consumer regression tests in `tests/action_affordance_consumers.rs` prove CLI/windowed snapshot construction can expose the same declaration surface without local legality or skill-ID hardcoding.
- Canonical data/tests continue to parse and avoid falsely claiming that Armored toughness implements Reactive Armor.
- Final verification passes: `cargo test-dev --test enemy_counterplay_affordance`, `cargo test-dev --test action_affordance_consumers`, `cargo test-dev --test roster_catalog`, `cargo test-dev`, and `cargo check --features "dev windowed"`.

## Proof Level

- This slice proves: Integration contract: this slice proves the data-to-runtime-to-query contract and thin consumer exposure path, not full behavior implementations for Type Trap, Reactive Armor, or Charged Attacks. Real runtime is required for snapshot/component extraction tests; human/UAT is not required.

## Integration Closure

Upstream surfaces consumed: `UnitDef`/`assets/data/units.ron`, `bootstrap::spawn_unit_from_def()`, `UnitQuerySnapshot`, CLI/windowed snapshot builders, and the existing `ImplementationStatus`/`ResourceStatus`/`LegalityReasonCode` vocabulary. New wiring introduced: a typed enemy-counterplay component/snapshot field and pure query helpers for enemy trait and charged telegraph affordances. Remaining end-to-end work belongs to S09 docs/data alignment and future mechanics slices that actually implement deferred behaviors.

## Verification

- Runtime signals: no new combat events are required, but query output must carry machine-readable `ImplementationStatus`, `ResourceStatus`, `ResourceKind`, and `LegalityReasonCode` values. Inspection surfaces: failing tests in `tests/enemy_counterplay_affordance.rs` and `tests/action_affordance_consumers.rs`, plus optional CLI/windowed display labels returned from the query surface. Failure visibility: a future agent should be able to tell whether drift is in RON schema, spawn/component propagation, snapshot extraction, or consumer formatting. Redaction constraints: none; all data is gameplay metadata.

## Tasks

- [x] **T01: Add typed enemy counterplay declarations to unit data** `est:1h`
  ---
estimated_steps: 5
estimated_files: 4
skills_used:
  - tdd
  - test
  - verify-before-complete
---

Executor task-plan frontmatter must include `skills_used: [tdd, test, verify-before-complete]`.

Why: S08 must stop treating free-text `signature_traits` as a UI contract. This task creates the typed RON/schema surface that later tasks can propagate into runtime snapshots. Keep `signature_traits` intact for flavor/catalog checks, but make `enemy_traits` and `charged_attack` the only machine-readable enemy counterplay declarations.

Failure Modes (Q5): Existing RON fixtures and helper constructors can fail to deserialize/compile if new fields lack `#[serde(default)]`; avoid that by defaulting optional/vector declaration fields. Canonical data can falsely imply implementation if `ReactiveArmor` is mapped from `ToughnessCategory::Armored`; explicitly avoid that mapping and encode it as deferred if declared.

Load Profile (Q6): Static roster parsing is in-memory and small; per-operation cost is trivial clone/deserialize work. At 10x roster size, readability and validation clarity fail before runtime performance.

Negative Tests (Q7): Cover backward-compatible parsing when declaration fields are omitted, canonical enemy declarations with expected implemented/deferred statuses, and a guard that Armored toughness is not interpreted as implemented Reactive Armor.

Implement typed declarations in or near `src/data/units_ron.rs`: `EnemyCounterplayKind` (`TypeTrap`, `ReactiveArmor`, `BreakSeal`, `TempoAnchor`), a reusable declaration status mirroring implemented/deferred/hidden with `LegalityReasonCode`, `EnemyTraitDeclaration`, and `ChargedAttackDeclaration` with `SkillId`, `lead_turns`, and status. Add `#[serde(default)] pub enemy_traits: Vec<EnemyTraitDeclaration>` and `#[serde(default)] pub charged_attack: Option<ChargedAttackDeclaration>` to `UnitDef`. Update `round_trip_unit_def()`, `taichi_def()`/test fixtures that construct `UnitDef`, and canonical `assets/data/units.ron` so Devimon declares `TempoAnchor` implemented plus deferred Type Trap/Reactive Armor/charged attack if present, while Ogremon can carry a deferred charged attack declaration and Goblimon remains empty. Do not change canonical `toughness_category` to `Shielded` just to prove Break Seal; that proof belongs in fixture query tests to avoid boss TTK regressions.
  - Files: `src/data/units_ron.rs`, `assets/data/units.ron`, `src/combat/mod.rs`, `tests/roster_catalog.rs`
  - Verify: cargo test-dev --test roster_catalog && cargo test-dev units_ron

- [x] **T02: Propagate declarations into snapshots and query affordances** `est:2h`
  ---
estimated_steps: 6
estimated_files: 5
skills_used:
  - tdd
  - test
  - verify-before-complete
---

Executor task-plan frontmatter must include `skills_used: [tdd, test, verify-before-complete]`.

Why: The schema is not useful to UI/CLI unless it survives spawn and appears in the same shared query layer as action legality. This task adds the runtime component/snapshot seam and pure query helpers that distinguish implemented, deferred, and hidden enemy counterplay.

Failure Modes (Q5): ECS snapshot callers can silently drop declarations if their query tuple is not extended consistently; keep tuple additions explicit and update all call sites. Missing components should degrade to empty declaration vectors, not panics. Hidden/deferred declarations must preserve their own stable reason codes instead of inventing display strings.

Load Profile (Q6): Shared resources are ECS component queries and per-frame snapshot allocations. Per operation, declaration lists are short vectors cloned into `UnitQuerySnapshot`; at 10x combatants, repeated snapshot rebuilding is the first cost, so keep query helpers pure and consumers on one snapshot per frame/turn.

Negative Tests (Q7): Include empty declaration lists, hidden declarations, deferred charged telegraphs, implemented Tempo Anchor from declaration/runtime fact, and shielded-toughness Break Seal fixture. Include a regression that `Armored` toughness does not yield implemented Reactive Armor.

Add a lightweight component such as `EnemyCounterplayKit` (either in a new `src/combat/enemy_counterplay.rs` or an existing appropriate module) containing cloned typed `enemy_traits` and `charged_attack` declarations. Insert it in `bootstrap::spawn_unit_from_def()` for enemy/unit defs as appropriate and re-export the module if needed. Extend `UnitQuerySnapshot` with declaration fields and update the fallback constructor plus all test fixtures. Extend `build_snapshot_from_ecs()` / `build_snapshot_from_ecs_with_sp()` tuple inputs to accept `Option<&EnemyCounterplayKit>` and populate snapshot fields. Add pure query helpers in `src/combat/action_query.rs`, e.g. `query_enemy_trait_affordances(&UnitQuerySnapshot) -> Vec<EnemyTraitAffordance>` and `query_charged_telegraph_affordance(&UnitQuerySnapshot) -> Option<ChargedTelegraphAffordance>` or similarly named structs. Reuse existing `ImplementationStatus`, `ResourceStatus`, `ResourceKind::EnemyTrait`, `ResourceKind::ChargedTelegraph`, `LegalityReasonCode::EnemyTraitDeferred`, and `LegalityReasonCode::ChargedTelegraphDeferred`; do not implement Type Trap, Reactive Armor, or charged attack execution. Add `tests/enemy_counterplay_affordance.rs` with fixture-level contract tests for implemented Tempo Anchor, implemented Break Seal via `ToughnessCategory::Shielded`, deferred Type Trap/Reactive Armor, deferred/hidden charged telegraph, empty minion declarations, and canonical RON query projection.
  - Files: `src/combat/action_query.rs`, `src/combat/bootstrap.rs`, `src/combat/mod.rs`, `src/data/units_ron.rs`, `tests/enemy_counterplay_affordance.rs`
  - Verify: cargo test-dev --test enemy_counterplay_affordance && cargo test-dev --test action_affordance_query

- [x] **T03: Expose query-backed declarations in consumer snapshots** `est:1h30m`
  ---
estimated_steps: 5
estimated_files: 4
skills_used:
  - test
  - verify-before-complete
---

Executor task-plan frontmatter must include `skills_used: [test, verify-before-complete]`.

Why: S07 established that CLI/windowed consumers must stay thin and use the shared query surface. S08 must prove enemy trait and charged telegraph declarations are available to those consumers without local skill-ID or free-text trait heuristics.

Failure Modes (Q5): Consumer ECS queries can compile in headless but fail under `windowed` if tuple updates are missed; run both headless tests and `cargo check --features "dev windowed"`. Missing `SkillBook`/roster data should leave declaration display empty or reason-coded, not panic. Formatting must not decide legality locally.

Load Profile (Q6): Shared resources are in-memory snapshot vectors reused by CLI/windowed affordance rendering. Per operation is one snapshot build plus short declaration formatting. At 10x combatants, repeated per-widget queries would be wasteful, so derive declarations from the existing per-turn/frame snapshot rather than rebuilding per enemy card.

Negative Tests (Q7): Add source-scan or behavioral tests that consumers do not match `devimon`, `ogremon`, charged skill IDs, or `signature_traits` to decide declarations. Verify deferred/hidden reason codes remain visible in formatted labels and empty enemies do not emit false warnings.

Update CLI and windowed ECS unit queries to include the new `EnemyCounterplayKit` snapshot input from T02. Add small display/formatting helpers that consume `query_enemy_trait_affordances()` and `query_charged_telegraph_affordance()` results; they may render labels/reason codes but must not branch on enemy names, skill IDs, or free-text `signature_traits`. Keep this minimal: a CLI line or windowed enemy-card line is enough if low risk, but consumer tests are the proof. Extend `tests/action_affordance_consumers.rs` to build snapshots with declaration components, assert consumer-facing helpers expose implemented/deferred/hidden states from the query output, and extend the existing no-hardcoding scan to cover enemy counterplay declarations. Final verification should include scenario TTK tests to ensure no accidental canonical `Shielded`/counterplay behavior change altered boss/miniboss combat pacing.
  - Files: `src/bin/combat_cli.rs`, `src/ui/combat_panel.rs`, `tests/action_affordance_consumers.rs`, `tests/enemy_counterplay_affordance.rs`
  - Verify: cargo test-dev --test action_affordance_consumers && cargo test-dev --test scenario_boss_ttk --test scenario_miniboss_ttk && cargo check --features "dev windowed"

## Files Likely Touched

- src/data/units_ron.rs
- assets/data/units.ron
- src/combat/mod.rs
- tests/roster_catalog.rs
- src/combat/action_query.rs
- src/combat/bootstrap.rs
- tests/enemy_counterplay_affordance.rs
- src/bin/combat_cli.rs
- src/ui/combat_panel.rs
- tests/action_affordance_consumers.rs
