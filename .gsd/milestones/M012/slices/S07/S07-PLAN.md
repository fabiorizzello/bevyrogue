# S07: CLI/windowed affordance integration

**Goal:** CLI and windowed combat affordances consume the shared DSL-backed legality query instead of local KO/team/ultimate checks, so action enablement, target enablement, disabled/deferred reasons, and revive-like KO-ally targets are truthful before an intent is emitted.
**Demo:** After this: CLI/windowed-facing code asks the legality query for action/target affordances; revive can target KO allies in CLI/query tests without special-case UI code, and windowed compiles.

## Must-Haves

- ## Must-Haves
- CLI non-interactive default action selection chooses its target from `ActionAffordance.targets` with `TargetStatus::Enabled`, not local team/KO filters.
- CLI interactive action and target menus are built from `query_action_affordance()` (or a small helper directly backed by it), show disabled/deferred/hidden reasons, and do not emit intents for disabled/deferred/hidden choices.
- Windowed action buttons/menus are enabled from `ActionStatus::Enabled`; target clicking is driven by `TargetStatus::Enabled` for both ally and enemy cards so revive-like skills can naturally target KO allies.
- UI/CLI snapshot construction uses real `SpPool.current` for affordance display while preserving S06 engine validation's SP-bypass snapshot behavior.
- No CLI/windowed code adds skill-ID-specific legality branches; legality must come from `SkillTargeting` / `ActionAffordance` / `TargetAffordance`.
- Revive-like target affordance is covered by integration tests that prove KO allies are enabled and live allies/enemies are disabled with query reason codes.
- ## Threat Surface
- **Abuse**: User-selected CLI/windowed targets are untrusted input to the combat pipeline; the UI must not emit locally invented legal intents, and S06 engine validation remains the authoritative safety net if a bad intent is still injected.
- **Data exposure**: No secrets or personal data are touched; the only exposed data is combat state already visible to the player.
- **Input trust**: Terminal menu selection and egui clicks are trusted only after they map to `ActionStatus::Enabled` and `TargetStatus::Enabled` from the query surface.
- ## Requirement Impact
- **Requirements touched**: R084, R085.
- **Re-verify**: `tests/action_affordance_consumers.rs` (new), existing `tests/action_affordance_query.rs`, existing `tests/engine_legality_integration.rs`, full `cargo test-dev`, and `cargo check --features "dev windowed"`.
- **Decisions revisited**: D053's shared DSL/query source of truth is exercised by real CLI/windowed consumers; D055/D056 remain compatible because toughness and damaged/KO targeting are read from query output rather than UI branches.
- ## Verification
- Add `tests/action_affordance_consumers.rs` with real assertions for the UI/CLI adapter/helper contract: real SP appears in affordance snapshots; S06 engine SP-bypass behavior remains separate; Basic default target selection picks an enabled enemy; Patamon-style revive target lists include a KO ally as enabled and disabled live ally/enemy entries with canonical reason codes.
- Run `cargo test-dev --test action_affordance_consumers`.
- Run `cargo test-dev --test action_affordance_query` and `cargo test-dev --test engine_legality_integration` to prove the shared query and runtime rejection parity still hold.
- Run full `cargo test-dev`.
- Run `cargo check --features "dev windowed"`.
- Static sanity check: `rg -n "patamon_revive|TargetNotKo|WrongSide|ko\.is_none\(\).*target|can_pick_target && !enemy\.is_ko" src/bin/combat_cli.rs src/ui/combat_panel.rs` should show no skill-ID-specific or legacy target-filter affordance paths in CLI/windowed code.

## Proof Level

- This slice proves: integration — this slice proves the pure DSL-backed legality contract is consumed by the real CLI and windowed entrypoints, with targeted integration tests plus full headless and windowed compile verification.

## Integration Closure

Upstream surfaces consumed: `src/combat/action_query.rs`, `src/combat/turn_system/mod.rs`, `assets/data/skills.ron`, existing RON-backed `SkillBook` handles, `SpPool`, `TurnOrder`, per-unit `UnitSkills`, `UltimateCharge`, `Toughness`, `Ko`, `Stunned`, `Commander`, `Energy`, and `RoundEnergyTracker` components. New wiring introduced: a shared affordance snapshot/selection helper plus CLI/windowed consumers that ask `query_action_affordance()` before rendering or emitting `ActionIntent`. Remaining milestone work after this slice: S08 enemy counterplay declarations and S09 doc/data/UI handoff alignment.

## Verification

- Runtime signals remain the existing `CombatEventKind::OnActionFailed` and `ActionLog::ActionFailed` reason-code path from S06; S07 improves preflight diagnostics by surfacing the same `ActionStatus`, `ResourceStatus`, `TargetStatus`, and `LegalityReasonCode` reasons in CLI/windowed affordances before emission. Future agents can inspect `tests/action_affordance_consumers.rs`, CLI labels, windowed disabled tooltips/text, and engine failure events to localize drift between preflight and runtime enforcement.

## Tasks

- [x] **T01: Extract a shared UI/CLI affordance snapshot and selection helper** `est:1h 30m`
  Executor task-plan frontmatter must include `skills_used: [test, verify-before-complete]`.

Create the reusable seam that lets CLI/windowed build affordance snapshots with real UI resources without changing S06's engine validation behavior. Keep the existing `build_snapshot_from_ecs()` SP-bypass path intact for `resolve_action_system()`, then add either a parameterized builder or a new public helper that accepts an explicit SP mode/value for UI/CLI. Add small pure helpers for consumer selection if useful, such as converting an action kind plus affordance into enabled target ids/labels, but do not encode skill IDs, target sides, KO rules, or ultimate readiness outside `query_action_affordance()`.

Failure Modes (Q5): If `SkillBookHandle` or a skill definition is missing, helpers must produce disabled/hidden query results from the existing query API rather than panicking. If the active actor is missing, the snapshot fallback should remain diagnosable through existing query reason codes.

Load Profile (Q6): Shared resources are only in-memory ECS/query snapshots; per-operation cost is one short snapshot allocation and one affordance traversal per rendered action. At 10x combatant count, the first concern is repeated snapshot rebuilding in UI frames, so helpers should build one snapshot per actor/frame and reuse it across action affordance calls.

Negative Tests (Q7): Include tests for real SP lower than a revive skill cost, SP-bypass remaining separate for engine parity, missing/disabled target reasons retained while the action resource is disabled, and an enabled Basic target selected from query output rather than local team/KO assumptions.
  - Files: `src/combat/action_query.rs`, `src/combat/mod.rs`, `tests/action_affordance_consumers.rs`
  - Verify: cargo test-dev --test action_affordance_consumers

- [x] **T02: Wire combat_cli action and target menus through query affordances** `est:1h 30m`
  Executor task-plan frontmatter must include `skills_used: [test, verify-before-complete]`.

Replace `player_action_system()`'s local action/target affordance decisions with the shared query-backed helper from T01. Add `SkillBook` access, real `SpPool.current`, and any missing unit snapshot inputs (`Toughness`, `Stunned`, `Energy`, `RoundEnergyTracker`, `Commander`) needed by the helper. In non-interactive mode, emit the first enabled target for `ActionQueryKind::Basic` from `ActionAffordance.targets`; if no enabled target exists, do not recreate the old live-ally fallback silently—choose a query-explained safe fallback or exit the turn in a way consistent with existing CLI behavior. In interactive mode, render Basic, skills, and Ultimate with enabled/disabled/deferred/hidden labels/reasons from `ActionStatus` and resource details, allow only enabled action choices, and build the target prompt from `TargetAffordance` entries so KO allies can appear for revive-like skills without a revive branch.

Failure Modes (Q5): If assets are not loaded yet or a selected skill is missing from the book, the CLI should not panic or emit a guessed intent; it should show the query-derived unavailable reason or fall back to an enabled Basic affordance if one exists. Canceled prompts should continue to choose an enabled query-backed default.

Load Profile (Q6): Shared resources are the ECS snapshot and terminal prompt lists. Per turn cost should be one snapshot plus one affordance per displayed action; avoid rebuilding the snapshot separately for every target prompt.

Negative Tests (Q7): Extend consumer tests/helper tests to cover Basic default choosing an enemy enabled by query, disabled/deferred actions not selected, and revive target entries retaining KO allies plus disabled live allies/enemies. Add a static no-hardcoding assertion if practical to catch `patamon_revive` or reason-code branches in CLI.
  - Files: `src/bin/combat_cli.rs`, `tests/action_affordance_consumers.rs`
  - Verify: cargo test-dev --test action_affordance_consumers && cargo check --bin combat_cli

- [x] **T03: Drive windowed action buttons and ally/enemy targets from query affordances** `est:2h`
  Executor task-plan frontmatter must include `skills_used: [test, verify-before-complete]`.

Update `combat_panel()` so the active actor's action controls and target picking are derived from the same `query_action_affordance()` output used by CLI. Expand `CombatPanelUnitsQuery` or adjacent queries to include the snapshot inputs missing today (`Commander`, `Energy`, `RoundEnergyTracker`, and real SP) while preserving headless-first feature gating. Use `ActionStatus::Enabled` for Basic/Skill/Ultimate button enablement and show query reasons for disabled/deferred/hidden states using concise labels or egui hover text. When an action is pending, render both ally cards and enemy cards as potential targets and only emit `ActionIntent` when the matching `TargetAffordance.status` is `Enabled`; disabled/deferred/hidden targets should remain visible with reason text and must not be clickable. Align the active actor source with `TurnOrder.active_unit` where available, falling back only as needed for existing preview display, to avoid `NotActiveUnit` button disablement caused by stale preview state.

Failure Modes (Q5): If the skill book is temporarily unavailable, controls should disable with an explainable unavailable state instead of using local ultimate/KO/team logic. If the selected pending action becomes disabled after state changes, clear or disable the pending action without emitting a stale intent.

Load Profile (Q6): Shared resources are egui frame rendering and ECS snapshot traversal. Per frame cost should be one snapshot for the active actor plus affordance calls for visible actions; avoid rebuilding per card or per button.

Negative Tests (Q7): Consumer tests should cover helper mapping for target enablement so KO ally targeting is proven without needing to drive egui. The final verification must include windowed compilation to catch feature-gated imports/query signature drift.
  - Files: `src/ui/combat_panel.rs`, `tests/action_affordance_consumers.rs`
  - Verify: cargo test-dev --test action_affordance_consumers && cargo check --features "dev windowed"

- [x] **T04: Verify no CLI/windowed legality hardcoding remains and run full S07 gates** `est:45m`
  Executor task-plan frontmatter must include `skills_used: [test, verify-before-complete]`.

Perform the final integration pass for S07. Clean up any duplicated local legality logic left after T02/T03, especially target filters based on `ko.is_none()`, enemy-only clickability, local ultimate readiness as an enablement source, skill-ID-specific branches, or direct matching on reason codes to decide legality. Keep display formatting allowed, but all legal/illegal/deferred/hidden decisions must originate from `ActionAffordance`, `TargetAffordance`, resource details, or helpers backed directly by `query_action_affordance()`.

Failure Modes (Q5): If static scans find legacy filters, treat them as blockers because they can diverge from the DSL query. If full tests fail, preserve the query source-of-truth and fix adapters rather than weakening engine parity.

Load Profile (Q6): Final scans and tests are local development commands; no runtime shared resources beyond cargo build/test outputs.

Negative Tests (Q7): The static scan is the negative test for forbidden skill-ID-specific/hardcoded affordance paths; targeted tests are the negative tests for disabled actions and invalid targets. Do not claim completion without fresh verification output per `verify-before-complete`.
  - Files: `src/bin/combat_cli.rs`, `src/ui/combat_panel.rs`, `tests/action_affordance_consumers.rs`
  - Verify: cargo test-dev --test action_affordance_consumers && cargo test-dev --test action_affordance_query && cargo test-dev --test engine_legality_integration && cargo test-dev && cargo check --features "dev windowed"

## Files Likely Touched

- src/combat/action_query.rs
- src/combat/mod.rs
- tests/action_affordance_consumers.rs
- src/bin/combat_cli.rs
- src/ui/combat_panel.rs
