# S03 — Research

**Date:** 2026-05-08

## Summary

S03 owns the local normalization part of R093–R096: clear drift should be repaired now, while the full one-module-per-Digimon migration remains deferred by R101. The important current-state finding is that the code already has most of the shared kernel pieces (`CombatKernelTransition`, `CombatKernelRegistry`, mechanic hooks, mechanic state resources, validation snapshots), but live action resolution does not yet route through them. `OnCombatBeat` and `OnKernelTransition` are typed event vocabulary in `src/combat/events.rs`, yet `resolve_action_system` / `pipeline::step_app` currently emit only action lifecycle and core damage/status events. Kernel applier systems also exist for Twin Core, Holy Support, Battery Loop, Predator Loop, and Precision Mind Game, but `register_combat_kernel_runtime` only wires Battery Loop and Predator Loop today.

The best S03 shape is a small, explicit blueprint seam plus runtime bridge rather than a broad rewrite. Add typed custom-signal declarations to RON as data (not `Effect` gameplay authority), copy those declarations into `ResolvedAction`, let per-Digimon Rust blueprint modules translate those signals into generic kernel transitions, then dispatch those transitions through the existing kernel registry and emit canonical `OnKernelTransition` events. Seed the first concrete seam with a Holy Support Digimon such as Patamon (and optionally Angemon if scope allows): RON declares a Patamon custom signal, `src/combat/blueprints/patamon.rs` owns the per-Digimon interpretation, the shared Holy Support hook converts the resulting tag/kernel transition, and `ValidationSnapshot` proves canonical state changed.

There are compile blockers that will obscure S03 verification until they are handled or consciously left to S06. The fresh `cargo test --no-run` run still fails before the stale Holy/Twin tests are reached because `docs/combat_ui_readiness_gap_matrix.md` is missing and several `SkillDef` fixtures omit `animation_sequence` / `qte`. S01 already classifies most fixture repairs as S03-owned mechanical drift, while the missing UI readiness doc is classified as S06-owned; planners should decide whether S03 temporarily fixes only the S03-owned fixture drift and runs targeted tests, or also restores the docs include to recover whole-suite `--no-run` earlier.

## Recommendation

Use a hybrid of “per-Digimon registry” and “existing kernel hook registry.” Reject two tempting shortcuts: do not re-add old `Effect::HolySupportTag` / `Effect::HolySupportRequest` variants just to satisfy stale tests, because that makes RON effects a hidden gameplay script; and do not put skill-ID-specific branches into `resolution.rs` or the generic kernel, because that violates R094/R096. Instead:

1. Add a data-only custom signal field to `SkillDef`, e.g. `custom_signals: Vec<SkillCustomSignal>` with typed per-Digimon sub-enums such as `SkillCustomSignal::Patamon(PatamonBlueprintSignal::BuildGrace)`.
2. Add a `combat::blueprints` module with per-Digimon files (`patamon.rs`, optionally `angemon.rs`) plus a tiny registry/router. The router may key by `UnitId`/blueprint id, but only delegates; it must not own the behavior.
3. Extend `ResolvedAction` to carry copied custom signals from the chosen skill. `apply_effects` should continue to own normal damage/resource/status results and should not interpret the custom signals.
4. In the action pipeline, after an action succeeds, ask the blueprint layer for kernel transitions and emit the registry-dispatched `OnKernelTransition` chain. Also emit `OnCombatBeat` / `CombatKernelTransition::Beat` for action lifecycle seams so the canonical beat vocabulary is live.
5. Wire `register_combat_kernel_runtime` into main/headless/windowed/CLI app setup and include all applier systems there. Update tests that manually add applier systems after runtime registration to avoid duplicate state application.

The design-an-interface guidance applies here: the deep interface is the narrow blueprint contract (`ResolvedAction + typed custom signals -> Vec<CombatKernelTransition>`), not a large public API. This keeps the hard parts hidden in per-Digimon modules while preserving the existing generic kernel and hook mechanics.

## Implementation Landscape

### Key Files

- `src/data/skills_ron.rs` — RON skill schema. Add `#[serde(default)] pub custom_signals: Vec<SkillCustomSignal>` to `SkillDef`. Keep `animation_sequence` and `qte` as presentation metadata. Do not encode the new signals as `Effect` variants.
- `assets/data/skills.ron` — source content. Add the first custom-signal declaration to a Patamon/Holy skill, likely `patamon_ult` or `holy_breeze`, using the new data-only field. If Angemon is included, keep in mind `angemon_ult` is currently `Deferred(reason: UnimplementedEffect)` with row/mixed damage+revive semantics, so it is riskier as the first end-to-end pipeline proof.
- `src/combat/state.rs` — `ResolvedAction` currently carries resolved numeric/effect fields but no metadata/custom signals. Add a copied `custom_signals` field so downstream pipeline code can call blueprints without re-looking up the RON asset.
- `src/combat/resolution.rs` — `resolve_action` is the natural place to copy `skill.custom_signals` into `ResolvedAction`. `apply_effects` should remain unchanged for custom-signal semantics; it should not decide Holy/Twin gameplay from RON.
- `src/combat/blueprints/mod.rs` (new) — small registry/router and public helper such as `resolve_blueprint_transitions(action: &ResolvedAction) -> Vec<CombatKernelTransition>` or a resource-backed registry. This is the seam that later per-Digimon modules expand.
- `src/combat/blueprints/patamon.rs` (new) — first concrete per-Digimon owner. It should translate Patamon custom signals into generic transitions such as a Holy Support design tag/kernel transition, then let `HolySupportHook` own the shared primitive behavior.
- `src/combat/mod.rs` — export the new `blueprints` module.
- `src/combat/kernel.rs` — `register_combat_kernel_runtime` currently registers all hooks but only adds Battery Loop and Predator Loop applier systems. It should also add `apply_twin_core_transitions_system`, `apply_holy_support_transitions_system`, and `apply_precision_mind_game_transitions_system`, and ideally expose a helper for dispatching/writing registry-expanded kernel events.
- `src/combat/events.rs` — event vocabulary already has `OnCombatBeat` and `OnKernelTransition`. No new event kind is needed for the minimum seam unless the executor wants a blueprint-specific debug event; avoid adding one unless tests prove it adds value.
- `src/combat/turn_system/mod.rs` — `resolve_action_system` currently emits `OnActionDeclared`, `OnActionPreApp`, `OnActionApplied`, and `OnActionResolved`, but no beats/kernel transitions. Add lifecycle beat emission and pass kernel/blueprint context to `pipeline::step_app`.
- `src/combat/turn_system/pipeline.rs` — `step_app` is the natural place to emit `CombatBeatId::Impact`/`Damage` around `apply_effects` and to trigger blueprint transitions after a successful action. There are two nearly duplicated branches (self-target modifier path and normal attacker/defender path); update both or extract a helper to avoid drift.
- `src/main.rs`, `src/headless.rs`, `src/windowed.rs` — app setup currently initializes general combat resources/messages but does not register kernel runtime. Main can call `combat::kernel::register_combat_kernel_runtime(&mut app)` after `CombatEvent` messages are added; if mode-specific setup needs ownership, keep the call in both `register_combat_systems` functions instead.
- `src/bin/combat_cli.rs` — the CLI builds its own App and currently does not register the kernel runtime. S03 can wire it now for D3/D9, but S05 still owns the full CLI proof and cwd-sensitive asset loading gap.
- `src/combat/observability.rs` — snapshots already include Twin Core, optional Holy Support, optional Predator Loop, and optional Precision Mind Game. Once runtime registration happens, snapshots should stop missing `TwinCoreState` and should expose Holy Support after the seeded blueprint proof.
- `tests/holy_support_resolution.rs` — stale/current hybrid. It is useful as a destination for the new Patamon blueprint proof, but its existing expectation that `apply_effects` directly returns `OnKernelTransition` is the wrong direction. Rewrite it to prove `RON custom_signals -> Patamon blueprint -> kernel registry -> HolySupportState -> ValidationSnapshot`.
- `tests/holy_support_roster_contract.rs` — stale. It imports removed `Effect::HolySupportTag`, `Effect::HolySupportRequest`, and `TargetShape::SelfTarget`; rewrite around the new `custom_signals` field rather than restoring removed variants.
- `tests/holy_support_affordance.rs` — stale. It imports removed `HolySupportAffordance` / `query_holy_support_affordance`; either rewrite against `ActionAffordance.resource_details` / validation snapshots or defer if S03 only seeds the blueprint seam.
- `tests/twin_core_mechanics.rs`, `tests/twin_core_integration.rs`, `tests/validation_snapshot.rs` — stale Twin Core assertions still expect `resonance`/`heat`; update to current `cross_resonance`, `active_thermal_spark_targets`, spend markers, and guard fields if S03 touches Twin Core proof.
- `tests/status_effect_apply.rs`, `tests/toughness_categories.rs`, `tests/event_stream.rs`, plus other fixture-heavy tests listed in `docs/m015_failure_ledger.md` — mechanical compile blockers from missing `SkillDef.animation_sequence` / `SkillDef.qte` fields. Add `animation_sequence: None, qte: None` or central fixture helpers before relying on full-suite no-run.
- `docs/m015_failure_ledger.md` and `docs/combat_mixed_pattern_drift_ledger.md` — use D1/D2/D3/D4/D5/D6/D7/D9/D10 as the task checklist. Do not erase the S01/S02 classifications when rewriting tests.

### Build Order

1. **Unblock S03-owned compilation enough for targeted tests.** Fix mechanical `SkillDef` / `UnitDef` fixture drift from S01’s S03-owned rows first. If full `cargo test --no-run` remains blocked only by `docs/combat_ui_readiness_gap_matrix.md`, record that as S06-owned unless the planner chooses to restore the doc early.
2. **Normalize kernel runtime registration.** Update `register_combat_kernel_runtime` to install all hook appliers and call it from app/CLI setup. Then update tests that currently add duplicate appliers after registration.
3. **Add lifecycle beat/kernel event bridge.** Emit `OnCombatBeat` and `CombatKernelTransition::Beat` from the live action pipeline, and optionally advance `CombatKernelState` with `TacticalCycle` transitions at lifecycle seams. This retires D1/D2/D3/D9 before adding per-Digimon behavior.
4. **Add data-only custom signal schema.** Introduce typed custom signals in `skills_ron.rs` and update at least one canonical skill in `assets/data/skills.ron`. Keep this separate from `Effect` so R095 remains true.
5. **Seed one per-Digimon blueprint.** Implement Patamon first because it is a narrow Holy Support proof and avoids Angemon’s deferred row/mixed-effect complexity. The minimum proof should show a Patamon custom signal creates a generic kernel transition, Holy Support hook expands/applies it, and validation snapshot reports the state.
6. **Rewrite stale Holy/Twin tests to current contracts.** Replace removed API expectations with the new blueprint/kernel/snapshot contract. For Twin Core, assert current names (`cross_resonance`, spark targets, spend markers, guards), not old `resonance`/`heat`.
7. **Refresh docs/audit if behavior changes.** Update `docs/combat_authority_map.md` and/or `docs/combat_mixed_pattern_drift_ledger.md` to mark the seeded blueprint seam and any drift split forward.

### Verification Approach

- Fresh evidence already collected: `cargo test --no-run` exits 101 in this worktree. First blockers observed were missing `docs/combat_ui_readiness_gap_matrix.md` and `SkillDef` fixture initializers missing `animation_sequence` / `qte` (`.gsd/exec/02b5e5e6-a357-4bc7-acc0-055e59e2b7f9.stderr`, summarized by `.gsd/exec/c8bba18a-f512-49a8-961f-aa4d000ead76.stdout`).
- After fixture/runtime registration repairs, run targeted compile/tests before the whole suite:
  - `cargo test --test holy_support_mechanics`
  - `cargo test --test holy_support_resolution` or the renamed/new Patamon blueprint proof test
  - `cargo test --test twin_core_mechanics --test twin_core_integration --test validation_snapshot` if Twin Core stale assertions are rewritten in S03
  - `cargo test --test battery_loop_kernel --test predator_loop_kernel` to ensure registration changes do not regress existing kernel domains
- New or rewritten proof should assert the full chain: parsed RON custom signal exists, `ResolvedAction` carries it, per-Digimon blueprint emits a generic transition, `CombatKernelRegistry::dispatch` expands it, `CombatEventKind::OnKernelTransition` is written, `app.update()` flushes the message, and `format_validation_snapshot` shows the resulting mechanic state.
- Run `python3 scripts/verify_combat_authority_audit.py` after updating docs.
- If S03 fixes all S03-owned compile blockers, run `cargo test --no-run` and record any remaining blockers with owners. Do not claim a green full baseline unless the missing UI doc and all stale tests have also been repaired.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Shared mechanic state machines | `src/combat/kernel.rs` + `CombatKernelRegistry` + mechanic hooks | The hook/transition seam already exists and is the canonical shared extension surface; use it instead of adding central skill-ID branches. |
| State proof / diagnostics | `src/combat/observability.rs` `ValidationSnapshot` | Snapshots already expose mechanic state and are the expected headless proof surface. |
| Presentation vocabulary | `CombatBeatId` / `OnCombatBeat` | Beat IDs already exist; S03 should make them live, not invent another event vocabulary. |
| RON parsing | Existing `serde`/`ron` skill schema | Add a typed field with `#[serde(default)]`; do not create a parallel parser or stringly typed scripting layer. |

## Constraints

- Headless-first: any new blueprint/kernel wiring must run without the `windowed` feature and must not introduce UI/winit dependencies outside the feature gate.
- R095/R104: `animation_sequence`, `qte`, and future presentation metadata cannot decide damage, legality, state transitions, or rules.
- R096: the generic kernel cannot become a central per-Digimon match ladder. A small router may delegate to per-Digimon modules, but behavior ownership must live in those modules.
- Bevy message flushing matters. Tests that write `CombatEvent` or `ActionIntent` must call `app.update()` before asserting snapshots or resources.
- Existing tests sometimes call `register_combat_kernel_runtime` and then manually add the same applier system. Once runtime registration is normalized, those duplicate additions can double-apply transitions and must be removed.
- `ValidationSnapshot` currently requires `TwinCoreState`; app modes that capture snapshots need kernel runtime registration before snapshot capture.

## Common Pitfalls

- **Restoring obsolete Effect variants** — `Effect::HolySupportTag` / `Effect::HolySupportRequest` were classified as stale. Re-adding them to make old tests compile would turn RON effects into gameplay authority and preserve drift.
- **Central skill-ID branching** — putting `if skill_id == "patamon_ult"` inside `resolution.rs`, `pipeline.rs`, or `kernel.rs` contradicts the per-Digimon module direction. If routing by skill/unit is needed, keep it in the blueprint layer.
- **Duplicate applier systems** — after fixing `register_combat_kernel_runtime`, old tests that add `apply_holy_support_transitions_system` or `apply_twin_core_transitions_system` again can mutate state twice.
- **One-frame event lag assumptions** — if the bridge is implemented as an event-reader system that emits more `CombatEvent`s, tests may need an extra `app.update()`. Direct emission from the pipeline is easier to prove deterministically.
- **Using Angemon as the first pipeline proof** — `angemon_ult` currently has row/mixed damage+revive deferred semantics. It can be covered later, but Patamon is a lower-risk first blueprint seam.

## Open Risks

- The exact custom-signal enum shape is a design choice. Per-Digimon variants (`Patamon(BuildGrace)`) align best with D011/R094, while mechanic variants (`HolySupport(BuildGrace)`) are easier but drift toward line/mechanic ownership. Prefer per-Digimon variants for the seed.
- Full `cargo test --no-run` may reveal additional blockers after the first compile errors are fixed. S01 already lists likely fixture and stale-test families, but current Rust compilation stops early.
- Scheduling/order may matter once kernel applier systems are in the real app. Direct pipeline emission plus explicit schedule ordering is safer than an unordered bridge system.
- The CLI still has S05-owned startup/cwd gaps. S03 should register kernel runtime there if touching setup, but should not claim the full shared-surface CLI proof.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Rust | `apollographql/skills@rust-best-practices` (9.6K installs), `jeffallan/claude-skills@rust-engineer` (2.9K installs), `affaan-m/everything-claude-code@rust-testing` (2.8K installs) | Available via `npx skills add ...`; not installed |
| Bevy | `bfollington/terma@bevy` (122 installs), `sickn33/antigravity-awesome-skills@bevy-ecs-expert` (111 installs), `laurigates/claude-plugins@bevy-ecs-patterns` (26 installs) | Available via `npx skills add ...`; not installed |
