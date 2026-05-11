# S02 Research: Combat authority and mixed-pattern audit

## Summary

S02 owns **R092 (Combat single-source-of-truth audit)** and supports **R093–R098** by identifying where authority actually lives before S03/S04/S05 normalize it. Current code has two partially overlapping combat models:

1. **Live action pipeline authority**: `ActionIntent` → `query_intent_legality` → `pipeline::step_declaration`/`step_app` → `resolution::apply_effects` → `CombatEvent`/`ActionLog`/ECS state. This is what `headless` and `combat_cli` run today.
2. **Typed kernel-transition authority seam**: `CombatKernelTransition` + `CombatKernelRegistry` + hook modules (`twin_core`, `battery_loop`, `holy_support`, `predator_loop`, `precision_mind_game`) + validation snapshot fields. This is well-typed and testable, but mostly test-injected and not wired into the live action path.

The central drift is not that the kernel types are bad; it is that the live action loop does not consistently emit `OnCombatBeat`/`OnKernelTransition`, and app wiring does not register/apply all kernel consumers. S03 should normalize the smallest shared seam: live action lifecycle emits canonical beat/kernel transition events, hook-derived transitions are re-emitted through `CombatEventKind::OnKernelTransition`, and all kernel state appliers are registered in headless/CLI runtime. Per-Digimon blueprint modules can then own unique signal routing without moving gameplay rules into RON or CLI.

## Requirements targeted

- **R092 — S02 primary:** map current gameplay authority, RON data, custom signals, per-Digimon logic, kernel transitions/state, beats, snapshots, and CLI consumers.
- **R093 — supports S03:** identify clear local drift versus rewrite-scale drift.
- **R094 — supports S03:** identify current absence of per-Digimon blueprint ownership and where to seed it.
- **R095 — supports S04:** distinguish RON content/metadata from gameplay authority.
- **R096 — supports S03/S05:** check whether generic kernel authority is actually shared.
- **R097 — supports S04:** locate presentation beat/QTE metadata and whether it can affect outcomes.
- **R098 — supports S05:** identify what CLI already consumes from shared surfaces and what it bypasses/misses.

## Skill discovery

Installed skills did not include a Rust/Bevy-specific skill. `npx skills find` results worth considering, but **not installed**:

- Rust: `npx skills add apollographql/skills@rust-best-practices` (9.6K installs), generally relevant for Rust quality but not Bevy-specific.
- Rust testing: `npx skills add affaan-m/everything-claude-code@rust-testing` (2.8K installs), relevant if S03/S06 fixture/test repair becomes large.
- Bevy: `npx skills add sickn33/antigravity-awesome-skills@bevy-ecs-expert` (111 installs), most directly relevant to Bevy ECS/event scheduling.
- Bevy: `npx skills add bfollington/terma@bevy` (122 installs), general Bevy skill.

No external library docs were needed; this is an internal Rust/Bevy architecture audit using existing code.

## Source-of-truth map

### RON data and metadata

- `src/data/skills_ron.rs:10-150` defines the current skill DSL. `SkillDef` owns `id`, `name`, `damage_tag`, `sp_cost`, targeting, implementation status, `effects`, plus `animation_sequence` and `qte` presentation metadata (`src/data/skills_ron.rs:133-149`).
- Current `Effect` variants are gameplay primitives only: `Damage`, `ToughnessHit`, `GainSP`, `UltGain`, `Stun`, `Revive`, `GrantFreeSkill`, `ApplyStatus`, `TurnAdvance`, `GrantEnergy`, `SelfAdvance` (`src/data/skills_ron.rs:114-131`). There are **no current `HolySupportRequest`, `HolySupportTag`, TwinCore, BatteryLoop, PredatorLoop, or Precision effects**.
- `assets/data/skills.ron` contains optional `animation_sequence` and `qte` strings around lines 91-160. They are parsed into `SkillDef` but not read by `resolution.rs`, `turn_system`, or CLI gameplay logic. That supports S04's non-authority claim but needs a regression test/doc.
- `src/data/units_ron.rs:8-60` defines line/mechanic metadata for Twin Core and Holy Support (`TwinCoreRosterMetadata`, `HolySupportRosterMetadata`). `UnitDef` embeds `follow_up`, `form_identity`, `twin_core`, and `holy_support` (`src/data/units_ron.rs:62-112`). This is declarative metadata, not behavior.
- `src/combat/bootstrap.rs:145-189` materializes RON into ECS: `Unit`, `Team`, `Toughness`, `RoundFlags`, `UltimateCharge`, `UnitSkills`, `Energy`, `RoundEnergyTracker`, optional `Commander`, `TempoResistance`, `FormIdentityKit`, and enemy counterplay kit. It does **not** instantiate per-Digimon blueprint components or kernel line metadata components.

### Current live gameplay authority path

- `src/combat/turn_system/mod.rs:25-39` defines `ActionIntent` (`Basic`, `Skill`, `Ultimate`).
- `resolve_action_system` performs early shared legality through `build_snapshot_from_ecs` + `query_intent_legality` before resolving (`src/combat/turn_system/mod.rs:105-171`). This is the strongest existing single-source query seam.
- `pipeline::step_declaration` converts `ActionIntent` + `UnitSkills` + `SkillBook` into `InFlightAction` via `resolution::resolve_action` (`src/combat/turn_system/pipeline.rs:36-89`). This is where RON effects become typed resolved action fields.
- `pipeline::step_app` mutates ECS state and emits `CombatEvent` through `apply_effects` (`src/combat/turn_system/pipeline.rs:91-629`). This is the actual runtime authority for damage, toughness, KO, revive, SP/ult/energy, status, and turn-advance today.
- `src/combat/resolution.rs:53-184` maps `Effect` values into action fields (`skill_base_damage`, `skill_toughness_hit`, revive, status, turn advance, energy, self advance). This confirms RON currently drives only generic effect data.
- `src/combat/events.rs:24-124` is the public event bus. It already includes lifecycle events (`OnActionDeclared`, `OnActionPreApp`, `OnActionApplied`, `OnActionResolved`), `OnCombatBeat`, `OnKernelTransition`, `BatteryLoopResolved`, and `PredatorLoopResolved`, but live action systems do not emit `OnCombatBeat` or generic `OnKernelTransition` today.

### Follow-up and form identity logic

- `src/combat/kit.rs:6-51` defines data-side triggers (`FollowUpTrigger`, `FormIdentityTrigger`) and ECS kits (`UnitSkills`, `FormIdentityKit`).
- `src/combat/follow_up.rs:88-183` evaluates follow-up triggers from `CombatEvent` (`OnBreak`, `OnAllyLowHp`, `OnEnemyKill`). It is event-driven and generic.
- `src/combat/follow_up.rs:303-356` evaluates Form Identity triggers from events and RON skill data (`OnFirstHitVsTagThisRound`, `OnStatusApplied`, `OnFirstSkillCastWithTag`, `OnAttackVsAttribute`). This is currently the closest thing to per-Digimon behavior, but it is generic central trigger code, not per-Digimon blueprint ownership.
- `src/combat/follow_up.rs:459-526` resolves follow-up/form-identity actions by reusing the same `step_declaration`/`step_app` path. Good: no separate mini engine. Drift: unique identity trigger routing is centralized.

### Kernel transition and hook seam

- `src/combat/kernel.rs:893-908` defines `CombatKernelTransition` variants for tactical cycle, strain, flow, fatigue, tags, beats, Twin Core, Battery Loop, Holy Support, Predator Loop, and Precision Mind Game.
- `src/combat/kernel.rs:1045-1068` defines `CombatKernelRegistry::dispatch`, which returns the input transition plus hook-produced transitions.
- `src/combat/kernel.rs:1070-1092` registers hooks and resources in `register_combat_kernel_runtime`, but only adds BatteryLoop and PredatorLoop applier systems. It does **not** add `apply_twin_core_transitions_system`, `apply_holy_support_transitions_system`, or `apply_precision_mind_game_transitions_system`.
- `src/combat/kernel.rs:1095-1113` defines hook domains (`Digimon`, `Enemy`, `Party`, `Shared`). Current hooks all report `Shared`; no per-Digimon hook modules exist yet.
- `src/combat/twin_core.rs`, `holy_support.rs`, `battery_loop.rs`, `predator_loop.rs`, and `precision_mind_game.rs` are line/mechanic modules. They own state resources, transition appliers, and hooks. They are useful shared primitives but do not satisfy the per-Digimon blueprint target by themselves.

### Mechanic module roles today

- `src/combat/twin_core.rs:20-63`: design tags map to `CombatTagId`; `TwinCoreHook` converts tag/beat/tactical-cycle transitions into `TwinCoreTransition`. `apply_twin_core_transitions_system` consumes `CombatEventKind::OnKernelTransition` and mutates `TwinCoreState` (`src/combat/twin_core.rs:163-244`). Tests expect an older `resonance/heat` format in some places, but current state is `cross_resonance`, spark targets, spend markers, guards, and `last_signal`.
- `src/combat/holy_support.rs:21-60`: design tags map to Holy Support transitions. `HolySupportState` and snapshot exist. Applier consumes `OnKernelTransition` (`src/combat/holy_support.rs:168-185`). Some tests expect old/unimplemented RON effects and affordance APIs.
- `src/combat/battery_loop.rs:17-24`: design tags/request kind exist but no RON/custom signal feeds them. State/applier are coherent and `tests/battery_loop_kernel.rs` is the replacement coverage for removed stale target.
- `src/combat/predator_loop.rs:19-194`: target state, prey lock, exploit stacks, berserk; applier consumes kernel state for strain when resolving `EnterBerserk` and emits `PredatorLoopResolved` (`src/combat/predator_loop.rs:225-257`). This is the most complete event/snapshot loop.
- `src/combat/precision_mind_game.rs:71-93`: hook currently does nothing and runtime registration exists only as a separate helper. It is a seeded placeholder rather than an integrated behavior.

### Observability and snapshots

- `src/combat/observability.rs:30-82` defines `ValidationSnapshot` with `units`, `twin_core`, optional `holy_support`, `predator_loop`, `precision_mind_game`, and optional `battery_loop`.
- `capture_validation_snapshot` pulls ECS resources and returns structured snapshots (`src/combat/observability.rs:157-281`). It requires `TwinCoreState`; optional surfaces use `world.get_resource`.
- `format_validation_snapshot` and helper formatters expose kernel/mechanic state for CLI/tests/logs (`src/combat/observability.rs:305-809`). This is an appropriate shared consumer surface.
- `src/headless.rs:16-180` logs one validation snapshot once data is ready, but headless does not register `register_combat_kernel_runtime`; unless another path initializes `TwinCoreState`, `capture_validation_snapshot` can fail with missing resource.

### CLI consumer surface

- `src/bin/combat_cli.rs:367-399` builds action entries using `query_action_affordance`, and `player_action_system` builds a shared `CombatQuerySnapshot` through `build_snapshot_from_ecs_with_sp` (`src/bin/combat_cli.rs:419-522`). This is good: CLI action availability is not local hardcoded combat logic.
- `src/bin/combat_cli.rs:522-649` chooses a legal action/target and writes a shared `ActionIntent`; it does not compute damage/outcomes itself.
- `src/bin/combat_cli.rs:659-776` wires the same core systems as headless (`resolve_action_system`, follow-up, ult, turn, dashboard, event log), but does not register kernel runtime, does not capture/print validation snapshots, and does not prove beats/kernel transitions.
- S01 observed `cargo run --bin combat_cli --quiet` still panics at Bevy runtime. S05 needs to fix/prove CLI after S03/S04 normalize surfaces.

## Drift ledger for S03/S04/S05 planning

| ID | Severity | Area | Evidence | Classification | Recommended owner |
|---|---:|---|---|---|---|
| D1 | High | Live action path vs kernel seam | `CombatEventKind` defines `OnCombatBeat`/`OnKernelTransition` (`src/combat/events.rs:103-110`), but grep found no live emissions outside tests; current systems emit lifecycle events only (`turn_system/mod.rs`, `pipeline.rs`). | Clear local architecture drift: canonical event variants exist but are not runtime authority. | S03 first task |
| D2 | High | Kernel runtime registration | `register_combat_kernel_runtime` registers hooks/resources but only BatteryLoop and PredatorLoop appliers (`src/combat/kernel.rs:1070-1092`); TwinCore/HolySupport/Precision appliers are omitted. | Clear local wiring drift. | S03 first task |
| D3 | High | App/CLI/headless wiring | `main.rs`, `headless.rs`, and `combat_cli.rs` do not call `register_combat_kernel_runtime`; CLI/headless therefore run the live action path without kernel resources/registry. | Clear shared-surface gap; blocks S05 proof. | S03/S05 |
| D4 | High | Per-Digimon blueprint ownership | Current unique behavior is `form_identity_listener_system` central trigger matching (`src/combat/follow_up.rs:303-356`) plus line metadata in `UnitDef`; no `src/combat/blueprints/*` or per-Digimon modules. | Missing target seam; not necessarily broken, but violates D011 direction if left unseeded. | S03 |
| D5 | Medium | RON custom signals | RON has `form_identity`, `follow_up`, TwinCore/HolySupport metadata and generic `Effect`; no general custom-signal vocabulary. Tests expecting `Effect::HolySupportRequest/Tag` are obsolete against current schema (`tests/holy_support_roster_contract.rs:43,63,67`). | Mixed old test/model drift. | S03 after D1-D3 |
| D6 | Medium | Holy Support affordance tests | Tests expect `query_holy_support_affordance`, `HolySupportAffordance`, `ResourceKind::Grace`, `ResourceKind::MartyrLight`, none present in `action_query.rs`; compile output confirms unresolved imports. | Architecture-drift candidate: either implement query surface or re-scope obsolete test. | S03/S05 |
| D7 | Medium | Twin Core stale contract | `tests/twin_core_integration.rs` expects formatted `resonance=5` and `heat=10`, while current `TwinCoreState` caps `cross_resonance` at 2 and formats `resonance={} spark_targets...`. S01 classified `resonance/heat` as stale. | Obsolete test or incomplete migration. | S03 |
| D8 | Medium | Presentation metadata non-authority | `SkillDef.animation_sequence` and `qte` parse from RON, but no gameplay code reads them. This is good, but currently unprotected except indirectly by absence. | Needs explicit S04 contract test/doc, not code rewrite. | S04 |
| D9 | Medium | Snapshot assumptions | `ValidationSnapshot` requires `TwinCoreState`; headless/CLI do not initialize kernel runtime, so shared snapshot proof can fail even if combat works. | Runtime wiring drift. | S03/S05 |
| D10 | Low | Compile fixture drift | Many tests need new `SkillDef` fields (`animation_sequence`, `qte`) and `UnitDef` fields (`twin_core`, `holy_support`); `roster_smoke.rs` duplicates fields. | Mechanical fixture drift from S01; not architecture authority. | S03/S06 |
| D11 | Low | Precision module | `PrecisionMindGameHook` does nothing and runtime registration is separate. | Seeded placeholder; safe to defer if documented. | S03 follow-up boundary |

## Natural implementation seams

### Seam A — Kernel event bridge and runtime registration

Files: `src/combat/kernel.rs`, `src/combat/turn_system/mod.rs`, `src/combat/turn_system/pipeline.rs`, `src/headless.rs`, `src/bin/combat_cli.rs`, possibly `src/main.rs`/`windowed`.

Goal: make live combat emit/consume canonical beat/kernel surfaces without changing content semantics.

Likely work:

- Add all missing applier systems to `register_combat_kernel_runtime`: TwinCore, HolySupport, Precision, plus existing Battery/Predator.
- Call `register_combat_kernel_runtime(app)` in headless and CLI app setup before systems that read snapshots/events. Windowed may need it too if future UI snapshots consume it.
- Introduce a small bridge function/system that turns lifecycle points into `CombatEventKind::OnCombatBeat { beat }` and possibly `CombatKernelTransition::Beat(beat)` routed through `CombatKernelRegistry::dispatch` into `OnKernelTransition` events.
- Do **not** make CLI call registry directly; CLI should remain a consumer of query/event/snapshot surfaces.

Why first: D1-D3/D9 block every downstream proof.

### Seam B — RON custom signal → per-Digimon blueprint seed

Files: new `src/combat/blueprints/mod.rs` (or `src/combat/digimon_blueprints/*`), `src/combat/mod.rs`, maybe `bootstrap.rs`, `follow_up.rs`, tests.

Goal: satisfy D011/D094 with a small seed, not a full migration.

Candidate minimal seed:

- Create a `CombatBlueprint`/`DigimonBlueprintHook`-style module that maps a `UnitId`/species marker + RON-declared `FormIdentityConfig` or mechanic metadata to typed kernel/custom signal outputs.
- Move one current central Form Identity case (e.g. Angemon or DORUgamon because existing RON comments already call out per-Digimon follow-up skills) behind that blueprint module while keeping `step_app`/`CombatEvent` as the authority path.
- Keep mechanic modules (`holy_support`, `twin_core`, etc.) as shared primitives called by blueprints/hooks, not as primary identity ownership.

Caution: avoid adding a RON gameplay DSL. RON should name data/signals; Rust blueprint validates and emits transitions/actions.

### Seam C — Test/schema cleanup and obsolete test decisions

Files: fixture-heavy tests (`tests/action_affordance_query.rs`, `tests/action_affordance_consumers.rs`, `tests/bootstrap_spawn_composition.rs`, `tests/sp_economy.rs`, `tests/tempo_resistance.rs`, `tests/roster_smoke.rs`, etc.), obsolete Holy/Twin tests.

Goal: get `cargo test --no-run` past compile blockers without enshrining old models.

Plan:

- Mechanical fixtures: add `animation_sequence: None`, `qte: None`, `twin_core: Default::default()`, `holy_support: Default::default()`; remove duplicate struct fields in `roster_smoke.rs`.
- Obsolete Holy Support tests: either implement a current `action_query` affordance surface if required by S05, or rewrite tests to prove snapshot/event surfaces instead. Do not re-add `Effect::HolySupportRequest/Tag` unless S03 explicitly chooses that as the custom signal vocabulary.
- Twin Core tests: update to current `cross_resonance`/spark/marker contract or declare old `resonance/heat` coverage obsolete with replacement tests.

### Seam D — Presentation beat/QTE boundary

Files: `src/data/skills_ron.rs`, `assets/data/skills.ron`, `src/combat/resolution.rs`, tests, docs.

Goal: S04 proof that `animation_sequence`/`qte` are non-authoritative.

Suggested proof:

- Test two otherwise identical `SkillDef`s differing only in `animation_sequence`/`qte` produce identical `ResolvedAction`/`ResolutionOutcome`/events.
- Assert `resolution.rs` does not branch on those fields via behavior test rather than brittle source grep.
- Document `animation_sequence` and `qte` as presentation metadata only.

### Seam E — CLI shared-surface proof

Files: `src/bin/combat_cli.rs`, `tests/action_affordance_consumers.rs`, possible CLI smoke test/script.

Goal: S05 proof through shared action query, event, beat, kernel-observable state, and snapshot surfaces.

Existing good surface:

- CLI uses `build_snapshot_from_ecs_with_sp` + `query_action_affordance` and writes `ActionIntent`.

Missing proof:

- CLI does not capture/print validation snapshots.
- CLI does not register kernel runtime.
- CLI does not consume/display `OnCombatBeat` or kernel transition events.
- CLI runtime panic from S01 must be fixed/classified.

## What to build/prove first

1. **Kernel runtime bridge first** (D1-D3/D9). Without this, snapshots and CLI cannot prove the agreed architecture even if tests compile.
2. **Fixture compile cleanup second**, but only mechanical fields and duplicate struct fields. This unblocks targeted tests without deciding obsolete architecture tests prematurely.
3. **Per-Digimon blueprint seed third** after event bridge exists, so the seed can emit into the same canonical surfaces.
4. **Presentation non-authority proof fourth**, because current code likely already satisfies it; add protection after live surfaces are stable.
5. **CLI proof last**, after shared surfaces are actually wired.

## Verification plan for downstream slices

Use `gsd_exec` for noisy commands.

Immediate research-safe checks already found:

- Prior `cargo test --no-run` run (`.gsd/exec/c91f74f8-...stderr`) shows current compile blockers: stale fixture fields, duplicate `UnitDef` fields, obsolete Holy Support APIs/effect variants.
- Grep confirms `OnCombatBeat` is defined but not emitted by runtime code, and `OnKernelTransition` emissions are test-injected or module appliers only.

Recommended S03/S04/S05 verification ladder:

1. `cargo test --test battery_loop_kernel` should remain green (S01 replacement coverage).
2. After fixture cleanup: `cargo test --no-run` should move past compile blockers.
3. Target kernel wiring tests:
   - `cargo test --test battery_loop_kernel`
   - `cargo test --test predator_loop_kernel`
   - updated/rewritten Twin Core/Holy Support kernel/snapshot tests.
4. Presentation boundary: new deterministic test proving metadata-only changes do not alter resolution/event outputs.
5. CLI: `cargo run --bin combat_cli --quiet` in non-interactive mode should exit without panic and print/emit evidence of query affordances, events, beats/kernel transitions, and validation snapshot state.
6. Full baseline later: `cargo test --no-fail-fast`, with every remaining failure classified per R090/R091.

## Planner cautions

- Do not restore obsolete APIs just to satisfy tests (`Effect::HolySupportRequest`, `Effect::HolySupportTag`, `TargetShape::SelfTarget`, `ResourceKind::Grace/MartyrLight`) unless a deliberate S03 decision makes those the new custom-signal/query contract.
- Do not move gameplay authority into RON `animation_sequence`/`qte`; they are presentation metadata.
- Do not put per-Digimon branches inside `resolution.rs`, `turn_system`, or CLI. Seed per-Digimon behavior in a new typed module/hook and keep shared systems branch-light.
- Bevy message ordering matters: tests that write `CombatEventKind::OnKernelTransition` must call `app.update()` before snapshot assertions (matches existing memory/gotcha).
- `register_combat_kernel_runtime` currently does not add all appliers and is not called by headless/CLI; this is likely the smallest high-value normalization point.

## Research artifacts generated

- `.gsd/exec/8ab468ee-cd85-4c6e-b3aa-21e85b1a1782.stdout` — declaration/symbol summary for key combat/data/CLI/test files.
- `.gsd/exec/f3b04691-382b-43e7-9714-7ce352ed2e56.stdout` — grep of custom/signal/blueprint/presentation/authority terms.
- `.gsd/exec/ba1d7688-3f14-4023-875a-0a60f6bb9833.stdout` — kernel runtime registration and transition writer references.
- `.gsd/exec/51fa754b-0d35-4379-9d2e-5794c2338da6.stdout` — authority-relevant test summary.
- `.gsd/exec/26007825-e760-4f46-a820-44d3bc857206.stdout` — obsolete effect variants and current effect consumers.
- `.gsd/exec/36345b1d-a681-4f99-b76c-ed7de012741d.stdout` — beat/kernel event emission search.
- `.gsd/exec/67b25384-e60b-46b3-9d8b-900a82c70571.stdout` — Rust/Bevy skill discovery.
