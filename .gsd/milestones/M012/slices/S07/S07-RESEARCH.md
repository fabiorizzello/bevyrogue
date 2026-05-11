# S07 Research — CLI/windowed affordance integration

## Summary

S07 is a targeted integration slice. The core legality/query API already exists in `src/combat/action_query.rs` and engine parity was completed in S06; the remaining work is to make CLI and windowed consumers build their action/target affordances from that API instead of local `ko`/team/ultimate checks.

Primary requirements supported:
- **R084** — CLI/windowed must consume the shared DSL-backed action/target query before execution.
- **R085** — UI-facing affordances must not lie: disabled/deferred/hidden actions and legal targets must come from query output, not local hardcoding.

Memory notes relevant to this slice:
- MEM066: hard boundary is no skill-ID-specific legality rules in CLI/windowed.
- MEM081/MEM084: keep action/status and per-target affordances together so UI can explain legality and toughness visibility without a second rules path.
- MEM072: windowed feature often breaks after display/query signature changes; always rerun `cargo check --features "dev windowed"`.

Baseline evidence collected during research:
- `cargo check --features "dev windowed"` currently passes with warnings. This means S07 starts from a compiling windowed path, but many `action_query` symbols are currently warned as unused in the windowed binary, confirming UI is not consuming the query surface yet.

## Skill Discovery

Installed skills directly relevant to implementation discipline:
- `verify-before-complete`: use before claiming S07 done; it requires fresh verification output in the completion message.
- `test`: useful for running/generating targeted integration tests if executors want skill-guided test work.

External skills discovered but **not installed**:
- `npx skills add mindrally/skills@rust` — 249 installs; broadly relevant to Rust implementation.
- `npx skills add bfollington/terma@bevy` — 117 installs; directly relevant to Bevy app/ECS patterns.
- `npx skills add sickn33/antigravity-awesome-skills@bevy-ecs-expert` — 108 installs; relevant to Bevy ECS query/scheduling issues.
- `npx skills add laurigates/claude-plugins@bevy-game-engine` — 18 installs; lower-install but Bevy-specific.

No library docs lookup was needed: Bevy 0.18 and `bevy_egui` are already established in the codebase, and the work is mostly internal adapter wiring.

## Implementation Landscape

### Shared query surface: `src/combat/action_query.rs`

Key exported types/functions already available:
- `CombatQuerySnapshot` / `UnitQuerySnapshot`: pure data snapshot consumed by affordance queries.
- `ActionQueryKind::{Basic, Skill(&SkillId), Ultimate}`: UI/CLI action vocabulary.
- `ActionStatus`, `TargetStatus`, `ResourceStatus`: `Enabled`, `Disabled { reason }`, `Deferred { reason }`, `Hidden { reason }`.
- `ActionAffordance`: combines action status, aggregate target status, per-target affordances, resource status/details, implementation status, and toughness affordance.
- `query_action_affordance(snapshot, skill_book, actor_id, kind)`: main consumer API for CLI/windowed.
- `query_target_affordance` / `query_all_target_affordances`: lower-level helpers.
- `query_intent_legality(...)`: engine parity API already used by `resolve_action_system`.
- `build_snapshot_from_ecs(...)`: S06 ECS adapter used by engine validation.

Important constraint: `build_snapshot_from_ecs()` currently sets every unit snapshot `sp` to `i32::MAX` and ignores `_sp_pool`. That was intentional for S06 because runtime early validation must preserve legacy SP shortfall behavior in `step_app()`. CLI/windowed affordance rendering should not use this exact behavior if they need truthful SP/resource disablement. Natural fix: add a separate adapter or a parameterized helper that can build snapshots with real `sp_pool.current` for UI/CLI while preserving engine bypass semantics.

The query API already preserves target affordances when action/resource is disabled. This matters for Patamon revive: `patamon_revive` has `sp_cost: 6` in `assets/data/skills.ron`, likely making the action disabled by SP in normal max-5 SP state, but the per-target query can still show KO allies as valid targets and live allies/enemies as disabled for the correct reasons.

### Engine adapter pattern: `src/combat/turn_system/mod.rs`

`resolve_action_system()` already demonstrates how to build a readonly ECS snapshot without borrow conflicts:
- calls `actors.as_readonly()` and `energy_q.as_readonly()`;
- collects `(UnitId, Team, &Unit, Option<&UnitSkills>, Option<&UltimateCharge>, Option<&Toughness>, ko/stunned/commander bools, Option<&Energy>, Option<&RoundEnergyTracker>)` into a `Vec`;
- calls `build_snapshot_from_ecs()`;
- passes the snapshot into `query_intent_legality()`.

This pattern should be reused or extracted for S07. Avoid duplicating query logic in CLI/windowed; duplicate only the read-only ECS-to-snapshot collection if extraction is too invasive.

### CLI consumer: `src/bin/combat_cli.rs`

Relevant system: `player_action_system()`.

Current local legality/affordance logic that should be replaced or wrapped by query output:
- Non-interactive default target selection at lines around 246-252 uses `u.id != actor_id && **team == Team::Ally && ko.is_none()`. This appears suspicious for a default basic attack because it selects a live ally, not an enemy. If kept, engine validation can reject it; but for S07 the non-interactive path should probably choose the first `Enabled` target for `ActionQueryKind::Basic` from `query_action_affordance()`.
- Interactive action menu always shows `Basic Attack` and all actor skills from `UnitSkills`, then only shows ultimate when `actor_ult.ready()` locally says ready. This duplicates part of query status. Prefer rendering all relevant action entries with query status labels/reasons, and only allowing selection of `Enabled` actions (or at least clearly marking disabled/deferred/hidden and not emitting intents for them).
- Interactive target list currently filters `ko.is_none()` and `u.id != actor_id`, then includes all live units from both sides. This is the biggest S07 mismatch: revive-like skills need KO allies, and offensive skills should not list allies as selectable. Replace with `affordance.targets` filtered/presented by `TargetStatus`; at minimum enabled entries should come from `TargetStatus::Enabled`, and disabled/deferred entries can be displayed with reason text.
- CLI imports currently lack `SkillBookHandle`, `SkillBook`, `Energy`, `RoundEnergyTracker`, `Stunned`, and maybe `Commander` in the player action query. These are needed if building the same rich snapshot as engine/windowed.

Likely CLI plan:
1. Add `skill_books: Res<Assets<SkillBook>>` and `skill_book_handle: Option<Res<SkillBookHandle>>` to `player_action_system()`.
2. Expand the units query to include `Option<&Stunned>`, `Option<&Toughness>`, and maybe include `Entity` if reading energy/tracker through a second query; or include energy components directly if component set permits.
3. Build a `CombatQuerySnapshot` for the active actor with all units and real `sp_pool.current`.
4. Build action entries for `Basic`, each non-basic skill, and `Ultimate` by calling `query_action_affordance()`.
5. Choose/select targets from `selected_affordance.targets` rather than local KO/team filters.

### Windowed consumer: `src/ui/combat_panel.rs`

Relevant system: `combat_panel()`.

Current local legality/affordance logic that should be replaced or wrapped by query output:
- `can_choose_action = phase == WaitingAction && active_ally.is_some()` is a local phase/actor check.
- `Basic` button is enabled solely by `can_choose_action`.
- Skill menu is enabled solely by `can_choose_action && !ally.skills.is_empty()`.
- Ultimate button is enabled solely by `active_ally.ultimate_ready`.
- Targets are only enemies; target buttons use `add_enabled(can_pick_target && !enemy.is_ko, chip)`. This prevents revive targeting KO allies and hardcodes offensive-only target assumptions.
- Enemy toughness display already uses `visible_toughness()`, which aligns with S02, but the action query also exposes target toughness affordance/reason. S07 can either continue passive display via `visible_toughness` or move selected-action target display to `TargetAffordance.toughness_view`; the latter is better aligned with MEM084.

Likely windowed plan:
1. Import `action_query::{ActionQueryKind, ActionStatus, TargetStatus, query_action_affordance, ...}` and data `SkillBookHandle`/`SkillBook` already exist.
2. Expand `CombatPanelUnitsQuery` to include `Option<&Commander>`, `Option<&Energy>`, `Option<&RoundEnergyTracker>`. It already has `Unit`, `Team`, `Toughness`, `UltimateCharge`, `UnitSkills`, `Ko`, `Stunned`.
3. Build a snapshot once per frame for the active ally. Note: current active ally is derived from `order.future_preview.first()`, not `order.active_unit`. Engine query uses `TurnOrder.active_unit` when present, otherwise actor id. Windowed may need to either align to `order.active_unit` or mark the snapshot actor `is_active: true` for the active UI actor. Planner should inspect whether `future_preview.first()` is still the intended active source in windowed; S06 noted stale turn-order assumptions are fragile.
4. Build per-action affordances and drive button/menu `add_enabled` from `ActionStatus::Enabled` instead of local checks.
5. For target clicking, either:
   - render both allies and enemies as clickable when `pending_action.kind` has a `TargetStatus::Enabled` entry for that unit; or
   - minimally, keep layout but add clickable affordance handling to allies as well as enemies so revive can target KO allies.
6. If a target is disabled/deferred/hidden, present reason text/tooltip and do not emit `ActionIntent`.

Windowed currently only sends intents when clicking enemies. To satisfy “revive can target KO allies in CLI/query tests without special-case UI code,” executors probably need ally cards to become target buttons driven by the selected action’s target affordance, not a separate revive branch.

### App/windowed wiring: `src/windowed.rs`, `src/main.rs`

`src/windowed.rs` registers `combat_panel` in `EguiPrimaryContextPass` and combat systems in `Update`. It already imports `CombatEvent` unused and has warnings. `cargo check --features "dev windowed"` passes now.

Potential scheduling constraint: `combat_panel` is an egui pass and emits `ActionIntent`; `resolve_action_system` is in `Update`. Existing flow compiles and presumably works. S07 should not need schedule changes unless the panel needs a helper system/resource.

### Tests to extend/add

Existing tests provide pure query and engine parity coverage:
- `tests/action_affordance_query.rs`: rich pure query tests, including revive, damaged targets, unimplemented shapes/effects, reason codes, target list retention.
- `tests/engine_legality_integration.rs`: injected illegal intents vs preflight reason parity.
- `tests/revive_semantics.rs` and `tests/patamon_revive.rs`: revive behavior.
- `tests/target_shape_truthfulness.rs`, `tests/toughness_enemy_only.rs`.

S07-specific tests should focus on adapter/consumer code rather than re-testing the pure query:
- Add a CLI/windowed adapter unit/integration test if a shared adapter helper is extracted from UI/CLI. It should build a snapshot from ECS-like inputs with real SP and assert Patamon revive target affordances include KO ally enabled and live ally disabled as `TargetNotKo`.
- Add a test that a default/auto CLI action chooses an enabled target from query output, not the old live-ally filter.
- If UI helpers are factored into pure functions (recommended), test that button/target enablement maps from `ActionStatus`/`TargetStatus`, not KO/team booleans.
- Contract test/grep-style assertion could check `src/bin/combat_cli.rs` and `src/ui/combat_panel.rs` no longer contain target list legality filters like `.filter(|..., ko, ...| ko.is_none())` for action targeting or enemy-only `add_enabled(can_pick_target && !enemy.is_ko, ...)`.

## Natural Seams / Suggested Task Boundaries

1. **Snapshot adapter seam**
   - Build or extract a reusable UI/CLI snapshot builder that preserves S06 engine behavior but allows real SP for affordance display.
   - Risk: borrow conflicts and accidentally changing engine SP parity. Keep engine early guard on SP-bypass path unless explicitly retested.

2. **CLI affordance seam**
   - Change `player_action_system()` action and target menus to use `query_action_affordance()`.
   - Include non-interactive/default target selection, because current default likely selects a live ally for Basic.
   - This is easiest to test without egui.

3. **Windowed affordance seam**
   - Change `combat_panel()` to compute selected/pending action affordance and use it for action/target enablement.
   - Add target affordance to ally cards as well as enemy cards so revive-style KO ally targeting is naturally supported.
   - Keep presentation minimal; this slice is correctness/compilation, not UI polish.

4. **Verification seam**
   - Add focused tests around adapter/consumer helpers.
   - Run `cargo test-dev` and `cargo check --features "dev windowed"` fresh before completion per `verify-before-complete`.

## Risks / Gotchas

- **SP truth vs engine parity:** `build_snapshot_from_ecs()` intentionally bypasses SP (`i32::MAX`). Reusing it unchanged in UI/CLI would make SP-cost affordances lie. Changing it blindly could alter S06 engine behavior and existing SP lifecycle expectations. Prefer a new helper or parameter.
- **Patamon revive cost:** canonical `patamon_revive` has `sp_cost: 6`, higher than the default SP max seen in `SpPool`. The query can still show target legality even when action/resource is disabled; tests should assert target affordance rather than assuming the action is executable.
- **Windowed active unit source:** current UI derives active ally from `order.future_preview.first()`, while engine legality uses `turn_order.active_unit` with fallback to actor ID. This could produce `NotActiveUnit` if snapshot `is_active` is computed differently. Decide and test the active unit convention before wiring button enablement.
- **No per-skill UI branches:** Do not add a `patamon_revive` or `Revive` special case in UI/CLI. If a behavior is needed, it must come from `SkillTargeting` / `ActionAffordance`.
- **Clickable allies:** The current windowed UI only allows clicking enemies as targets. Supporting revive without special-casing requires making ally cards targetable based on `TargetStatus::Enabled` for the pending action.
- **Borrow shape:** UI/CLI queries may need `Entity` to join unit rows to `Energy` / `RoundEnergyTracker` if those are separate components. Follow the S06 pattern: collect readonly data into a transient Vec, then call the pure query.

## Verification Plan

Minimum fresh verification for S07 executors:
- Targeted tests for new adapter/helper behavior, e.g. `cargo test-dev --test <new_or_existing_test_name>`.
- `cargo test-dev` full suite.
- `cargo check --features "dev windowed"` because windowed is in-scope and baseline currently passes.
- Optional manual CLI smoke: `cargo run --bin combat_cli` in interactive mode if environment permits; not required for automated completion.

Acceptance evidence to look for:
- CLI/windowed code calls `query_action_affordance()` or a helper built directly on it.
- Target lists come from `ActionAffordance.targets` / `TargetStatus`, not local live-enemy/live-unit filters.
- Revive-like target affordance enables KO allies and disables live allies/enemies with reason codes from `LegalityReasonCode`.
- Windowed feature still compiles.
- No skill-ID-specific legality branches were introduced.

## Recommendation

Build S07 around a small shared adapter/helper layer first, then update CLI, then windowed. The adapter should make the distinction between **engine validation snapshot with SP bypass** and **UI/CLI affordance snapshot with real SP** explicit. Once that seam exists, both CLI and windowed can become thin consumers of `query_action_affordance()` and the planner can keep tests focused on consumer mapping rather than re-proving the legality engine.