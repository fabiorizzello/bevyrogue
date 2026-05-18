# ARCHITECTURE

## Overall style

- **Single Rust crate / application** centered on a Bevy ECS runtime
- **Headless-first architecture**
  - Default build avoids windowing/render dependencies
  - Windowed UI is an optional feature-gated presentation layer
- **Data-driven + typed-code hybrid**
  - RON files define content, numbers, targeting metadata, and typed custom-signal intent
  - Rust modules own gameplay authority and unique combat behavior

## Primary architectural split

The codebase is organized around four main concerns:

1. **Combat runtime** — `src/combat/`
2. **Asset/data loading** — `src/data/`
3. **Application shells**
   - `src/headless.rs`
   - `src/windowed.rs`
   - `src/bin/combat_cli.rs`
4. **Optional UI presentation** — `src/ui/`

## Core data flow

Observed flow from docs and code:

```text
RON assets
-> typed asset loading (DataPlugin)
-> compiled skill timelines / typed registries
-> combat kernel + Bevy ECS state/resources/messages
-> CombatEvent / snapshots / logs
-> presentation consumers (CLI, egui UI, tests)
```

`docs/combat_current.md` describes the current authority stack as:

```text
RON data and typed custom signals
-> per-Digimon Rust blueprint logic
-> generic CombatKernelTransition values and shared hooks
-> canonical ECS state, CombatEvent stream, and ValidationSnapshot
-> CLI, UI, tests, logs, and presentation consumers
```

## Key architectural patterns

## 1. ECS + plugin composition

- Bevy `Plugin` composition is the main assembly mechanism
- `CombatPlugin` initializes runtime resources and systems
- `DataPlugin` owns asset loading / readiness hydration
- App shells register different plugins/system sets depending on build mode

## 2. Message/event-driven runtime

- Bevy messages are used heavily for runtime flow
- Main message surfaces visible in `src/main.rs` and related modules include:
  - `TurnAdvanced`
  - `ActionIntent`
  - `FollowUpIntent`
  - `FollowUpTrace`
  - `CombatEvent`
  - `ActionValueUpdated`
- `CombatEvent` is explicitly documented as the **single-source-of-truth bus** for downstream consumers

## 3. Presentation is read-only with respect to game authority

Per `CLAUDE.md` and `docs/combat_current.md`:

- UI/logging should **read events and state**, not author gameplay outcomes
- CLI/UI must use shared action queries, kernel state, and snapshots
- Presentation metadata is non-authoritative

## 4. Data-driven combat definitions

- `assets/data/units.ron` defines roster/unit data
- `assets/data/skills.ron` defines skill metadata/effects/targeting/custom signals
- `assets/data/party.ron` defines party configuration
- `src/data/skills_ron.rs` and `src/data/units_ron.rs` provide the typed schema layer

## 5. Blueprint-owned unique behavior

- Per-Digimon behavior is split into dedicated blueprint modules under `src/combat/blueprints/`
- Shared mechanics live in generic modules such as:
  - `battery_loop.rs`
  - `precision_mind_game.rs`
  - `kernel.rs`
- The docs explicitly state that new identities should extend via blueprint + signal registration rather than shared-system character branches

## 6. Query boundary for legality / affordances

- `src/combat/action_query.rs` exposes read-model style legality and affordance queries
- CLI and UI can inspect actionable state without re-encoding combat rules
- This creates a boundary between:
  - authoritative mutation systems
  - read-only presentation/query consumers

## Module boundaries

## App layer

- `src/main.rs` — top-level app assembly
- `src/headless.rs` — headless runtime wiring and smoke script
- `src/windowed.rs` — desktop UI wiring and validation mode
- `src/bin/combat_cli.rs` — separate CLI harness built on shared library modules

## Library boundary

- `src/lib.rs` re-exports:
  - `combat`
  - `data`
  - `party_validation`
  - `CombatPlugin`

## Combat boundary

`src/combat/mod.rs` groups modules by responsibility:

- **Framework API** — `combat/api/`
- **Core kernel & primitives** — types, team, unit, kit, state, action query, RNG
- **Turn pipeline** — turn order, resolution, AV, speed, resistance
- **Combat mechanics** — damage, toughness, stun, status effects, SP, ultimates, energy, follow-ups
- **Enemy & encounter** — bootstrap, AI, counterplay, blueprints
- **Observability** — events, logs, snapshots, JSONL, floating damage

## Data boundary

- `src/data/mod.rs` owns asset-plugin registration and readiness synchronization
- Typed asset handles (`UnitRosterHandle`, `SkillBookHandle`, `PartyConfigHandle`) decouple loading from consumers
- Skill-book load also triggers validation and timeline compilation

## Test boundary

- Most verification is in **integration tests** under `tests/`
- Shared helpers live in `tests/common/mod.rs`
- Tests commonly construct a small Bevy app and run the same systems used in production

## Notable design decisions visible in code/comments

- **Deterministic RNG** is centralized via `CombatRng`
- **Boot-time validation** is used for timeline/registry integrity (`CombatPlugin::finish` panics on dangling references)
- **Headless default** is enforced in Cargo/dependency configuration and `.cargo/config.toml`
- **File-watching assets** are enabled for local iteration (`watch_for_changes_override: Some(true)`)
