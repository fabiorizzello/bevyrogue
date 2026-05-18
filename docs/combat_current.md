# Combat Current State

This is the current combat architecture entrypoint. It replaces the older historical design/status docs as the first read for new combat work. Updated after the 5-wave decoupling refactor on the `gsd/refactor/disaccoppiare-src-combat-observability-r` branch.

## Authority stack

```text
Per-digimon RON data (assets/data/digimon/{name}/, assets/data/enemies/{name}/)
-> per-Digimon Rust blueprint (src/combat/blueprints/{name}/)
-> generic execution runtime: Intent, ExtRegistries, SignalBus, IntentQueue (src/combat/api/)
-> canonical ECS state, CombatEvent stream, ValidationSnapshot
-> CLI, UI, tests, logs, and presentation consumers
```

Gameplay authority lives in typed Rust combat paths. RON declares data and typed intent. Consumers observe and render shared surfaces; they do not decide legality or outcomes.

## Module map

### `src/combat/api/` — Execution runtime (historical name; actually the combat kernel)

Contains no Digimon-specific names or logic. All mutations go through `Intent`; all extension points use `ExtPoint` + `Registry<E>`. Sub-modules:

| Sub-module        | Responsibility                                                     |
|-------------------|--------------------------------------------------------------------|
| `intent`          | `CastId` + closed `Intent` enum (~18 variants)                    |
| `registry`        | `ExtPoint` trait, `Registry<E>`, `ExtRegistries` Resource (9 axes) |
| `signal`          | `SignalBus` + `SignalTaxonomy` for blueprint-owned custom signals  |
| `event_filter`    | Typed runtime filters for passive subscriptions                    |
| `rng`             | `CastRng` — SplitMix64 deterministic per-cast RNG                 |
| `applier`         | Exclusive `intent_applier` system that drains `IntentQueue`        |
| `runner`          | Timeline-backed skill execution (FSM stepping, beat evaluation)    |
| `timeline`        | Compiled timeline schema and evaluation helpers                    |
| `blueprint_state` | `BlueprintState` resource for per-owner opaque state bags          |
| `builtins`        | `register_kernel_builtins` — shared hooks/selectors/predicates     |
| `clock`           | `Clock` resource (headless vs windowed timing)                     |
| `event_bridge`    | `combat_event_to_signal_system` — CombatEvent -> SignalBus relay   |
| `passive_runner`  | `PassiveListeners`, `passive_dispatch_system`                      |
| `skill_ctx`       | `SkillCtx` + `SkillCtxMode` — mutable world access for beat hooks |

No `bevy::winit`, `bevy::render`, or `bevy_egui` imports appear in this tree.

### `src/combat/api/registry.rs` — Extension axes (`ExtRegistries`)

Nine axes, each a unit struct implementing `ExtPoint`:

| Axis                   | Fn signature                                         | Purpose                                      |
|------------------------|------------------------------------------------------|----------------------------------------------|
| `HookExt`              | `fn(&BeatEvent, &mut SkillCtx)`                     | Lifecycle hooks (OnTurnStart, OnDamageDealt)  |
| `SelectorExt`          | `fn(&SelectorCtx) -> Vec<UnitId>`                   | Target selectors (primary, all_enemies, etc.) |
| `PredicateExt`         | `fn(&BeatEvent, &SkillCtx) -> bool`                 | Edge-gate predicates                          |
| `FormulaExt`           | `fn()` (placeholder)                                 | Damage/heal formula                           |
| `TickExt`              | `fn()` (placeholder)                                 | Per-turn status tick                          |
| `AiUtilityExt`         | `fn()` (placeholder)                                 | AI utility scorer                             |
| `CueExt`               | `fn(&CueCtx) -> &'static str`                       | Presentation cue -> animation handle          |
| `PreDamageReactionExt` | `fn(&mut World, UnitId, CastId) -> Option<i32>`     | Blueprint pre-damage mitigation reactions     |
| `ValidationExt`        | `fn(&World) -> Option<ValidationSection>`            | Per-owner validation snapshot sections        |

Blueprints register into these axes at startup via `register_all_blueprint_exts` / `register_all_blueprint_validation_exts`. Built-in (generic) entries are registered via `register_kernel_builtins`. After `App::finish()`, registries are immutable.

### `src/combat/blueprints/` — Per-Digimon composition

Centralized in `blueprints/mod.rs`. Each Digimon module provides:

- **`OWNER` constant** + **`dispatch` function** — signal routing from RON `custom_signals` into `CombatKernelTransition` values.
- **`register_*_ext(regs)`** — registers hooks, predicates, selectors, and validation contributors into `ExtRegistries`.
- **`register_passive_runtime(app)`** — wires passive Bevy systems (where applicable).
- **`*Plugin`** — Bevy `Plugin` adding runtime systems and resources.

Composition entry point: `register_blueprints(app)` calls `add_runtime_plugins`, `register_all_blueprint_validation_exts`, and `register_canonical_passive_runners` in one shot. `CombatPlugin::build` invokes it.

Signal dispatch: `dispatch_custom_signal` iterates a static `BLUEPRINTS` table keyed by owner string. No shared-system character branching.

### `src/combat/kernel/` — Tactical cycle primitives

`CombatKernelRegistry` + `CombatKernelTransition` + `CombatKernelHook` trait. Shared hook dispatch; blueprint-specific transitions use the `Blueprint { owner, name, payload }` envelope.

### `src/combat/observability.rs` — Blueprint-agnostic validation

`ValidationSnapshot` aggregates generic combat state (phase, SP, turn preview, unit snapshots) plus `owner_sections: Vec<ValidationSection>`. Each blueprint registers a `ValidationExt` function that returns its own `ValidationSection` with arbitrary key/value fields. No hardcoded Digimon names in the observability module.

### `src/combat/counterplay.rs` — Typed enemy counterplay

Single module (formerly split across `counterplay.rs` + `enemy_counterplay.rs`). Declares `EnemyCounterplayKind` (TypeTrap, ReactiveArmor, BreakSeal, TempoAnchor) and `ImplementationStatus` (Implemented / Deferred / Hidden). Used by unit data and query surfaces.

### `src/data/` — RON asset loading

Per-digimon asset split:

```
assets/data/digimon/{name}/unit.ron   — unit definition (HP, attribute, kit, counterplay)
assets/data/digimon/{name}/skills.ron — skill definitions (costs, targets, signals, timelines)
assets/data/enemies/{name}/unit.ron
assets/data/enemies/{name}/skills.ron
```

Loaders in `src/data/mod.rs` enumerate paths via `DIGIMON_UNIT_PATHS`, `ENEMY_UNIT_PATHS`, `DIGIMON_SKILL_PATHS`, `ENEMY_SKILL_PATHS` and assemble them at runtime into `UnitRoster` and `SkillBook`.

### Other core modules

| Module              | Responsibility                                                  |
|---------------------|-----------------------------------------------------------------|
| `action_query`      | Pure action legality / affordance query vocabulary               |
| `state`             | `CombatState`, `CombatPhase`, `InFlightAction`, `ResolvedAction` |
| `turn_system/`      | Turn pipeline: advance, resolve, check-victory                   |
| `turn_order`        | AV gauge queue + `TurnAdvanced` event                            |
| `follow_up`         | FIFO reaction queue + depth guard                                |
| `damage`            | Attribute matchup, resistance, element calculations              |
| `toughness`         | Break gauge (HSR-like)                                           |
| `energy`            | Per-unit Energy (max 100, gain caps per source)                  |
| `sp`                | Shared SP pool (cap 5, gen on Basic, +2 extra/round)             |
| `ultimate`          | Ultimate charge meter + accumulation triggers                    |
| `status_effect`     | Buff/debuff with duration, tick at turn end                      |
| `resistance`        | Tempo resistance: diminishing returns on repeated Delay          |
| `buffs`             | Damage-reduction bag (`DrBag`)                                   |
| `modifiers`         | Ordered modifier aggregation, one-shot incoming-damage ledger    |
| `events`            | `CombatEvent` / `CombatEventKind` — single-source-of-truth bus  |
| `floating`          | Floating damage numbers (spawned on hit, decayed by system)      |
| `jsonl_logger`      | JSONL dump on stdout behind `BEVYROGUE_JSONL` env                |
| `log`               | `ActionLog` ring buffer + `LogEntry` enum                        |
| `bootstrap`         | Encounter spawn from `SelectionRequest`                          |
| `enemy_ai`          | Enemy AI: decision routing -> `ActionIntent`                     |
| `preview`           | Shared skill-preview seam for UI/AI consumers                    |
| `plugin`            | `CombatPlugin` Bevy plugin (wires everything)                    |

## Canonical boundaries

- **RON** owns numbers, targeting declarations, costs, metadata, presentation metadata, and typed `custom_signals`. It is not a gameplay scripting engine.
- **Blueprints** own unique Digimon behavior. Each blueprint registers hooks, passives, and signals through `ExtRegistries`. No shared-system character branching.
- **Execution runtime (`api/`)** is the generic kernel. All mutations flow through `Intent` -> `IntentQueue` -> `intent_applier`. Extension points use `ExtPoint` + `Registry<E>`.
- **Events/beats:** `CombatEventKind::OnCombatBeat` and `OnKernelTransition` are live combat output. Presentation consumes them; presentation does not author them.
- **Snapshots:** `ValidationSnapshot` is diagnostic truth over live state, not a second gameplay source. Blueprint sections are injected via `ValidationExt`.
- **CLI/UI** must use shared action query, events, beats, kernel state, and snapshots. No CLI/windowed skill-ID-specific legality logic.
- **Pre-damage reactions** route through `ExtRegistries::pre_damage_reactions` in `applier.rs`, not hardcoded blueprint calls.

## Key invariants

1. **K001 / P001:** `src/combat/api/` contains zero Digimon-specific names. All blueprint logic is in `src/combat/blueprints/`.
2. **Single mutation path:** all combat state mutations flow through `Intent` -> `IntentQueue` -> exclusive `intent_applier` system.
3. **Determinism:** `CombatRng` from fixed seed; no wall-clock or unseeded RNG in combat paths.
4. **Headless first:** no `bevy::winit`, `bevy::render`, or `bevy_egui` imports outside `#[cfg(feature = "windowed")]` gates.
5. **Boot-time validation:** `CombatPlugin::finish` validates all `CompiledTimeline` references against `ExtRegistries`; panics on dangling hook/selector/predicate IDs.
6. **Zero warnings:** `cargo check` produces 0 warnings after Wave 5 + Wave 10 dead-code cleanup.

## Migrated identities

| Digimon   | Mechanic           | Blueprint module                          |
|-----------|--------------------|-------------------------------------------|
| Patamon   | Holy Support       | `src/combat/blueprints/patamon.rs`        |
| Dorumon   | Predator Loop      | `src/combat/blueprints/dorumon/`          |
| Tentomon  | Battery Loop       | `src/combat/blueprints/tentomon.rs`       |
| Renamon   | Precision/MindGame | `src/combat/blueprints/renamon.rs`        |
| Agumon    | Twin Core (Fire)   | `src/combat/blueprints/agumon.rs`         |
| Gabumon   | Twin Core (Ice)    | `src/combat/blueprints/gabumon.rs`        |

Shared Twin Core mechanics live in `src/combat/blueprints/twin_core/`.

Each blueprint owns signal interpretation, kernel mutation for its mechanic, and `ValidationExt` registration. The kernel mutates only through the generic `CombatKernelTransition::Blueprint { owner, name, payload }` envelope.

## Acceptance criteria per blueprint

- RON declares typed custom-signal intent only.
- Per-Digimon Rust blueprint owns unique interpretation.
- Blueprint registers hooks, selectors, predicates, and validation via `ExtRegistries`.
- Shared kernel transition/hook owns canonical mutation.
- `CombatEvent` and `ValidationSnapshot` expose the result.
- Action query and CLI proof observe shared surfaces.
- Presentation metadata remains non-authoritative.

## Proof surfaces

- Authority map: `docs/contracts/combat_authority_map.md`.
- Blueprint runtime proofs: `tests/dorumon_blueprint.rs`, `tests/dorumon_predator_runtime.rs`, `tests/tentomon_blueprint.rs`, `tests/renamon_precision_runtime.rs`, `tests/twin_core_integration.rs`, `tests/battery_loop_kernel.rs`, `tests/predator_loop_kernel.rs`.
- Presentation boundary: `docs/contracts/presentation_metadata_boundary.md`, `tests/presentation_metadata_boundary.rs`.
- CLI shared-surface proof: `docs/contracts/combat_cli_shared_surface_proof.md`, `tests/combat_cli_shared_surface.rs`.
- UI/CLI legality: `docs/contracts/combat_ui_readiness_gap_matrix.md`, `docs/contracts/skill_legality_contract.md`.
- Signal dispatch: `tests/blueprint_signal_dispatcher.rs`, `tests/digimon_signal_registry.rs`.
- Timeline validation: `tests/compiled_timeline_boot_validation.rs`, `tests/compiled_timeline_active_canon.rs`.
- Passive system: `tests/passive_kitsune_grace.rs`, `tests/passive_event_filters.rs`, `tests/passive_canon_support.rs`, `tests/passive_reactive_canon.rs`.
- Intent pipeline: `tests/intent_applier_canary.rs`, `tests/cast_id_propagation.rs`, `tests/pipeline_dispatch.rs`.
- Validation snapshot: `tests/validation_snapshot.rs`.

## Verification

```bash
cargo check                   # 0 warnings
cargo test                    # full integration suite
cargo test --no-fail-fast     # all tests, no early exit
BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli
```
