# Inventory — combat decoupling and module cleanup

Date: 2026-05-18  
Branch: `gsd/refactor/disaccoppiare-src-combat-observability-r`

## Goal

Disaccoppiare il combat runtime condiviso dai dettagli specifici dei blueprint, rimuovere bootstrap hardcoded e residui del vecchio design, ridurre la dimensione dei file più critici, e lasciare una struttura `src/combat/*` più leggibile per umani e coding agents.

## User-directed constraints added after first pass

Questi vincoli sono esplicitamente richiesti e fanno parte del lavoro, non sono follow-up opzionali:

1. `src/combat/observability.rs` non deve conoscere tipi o naming policy blueprint-specifiche.
2. `src/combat/api/applier.rs` va ripulito da riferimenti hardcoded o residui concettuali collegati a blueprint-specific state (`BatteryLoopState` o equivalenti), se presenti.
3. I warning principali di dead code emersi dal refactor vanno risolti nello stesso lavoro, non lasciati sedimentare.
4. Vanno tolti i residui di vecchio design come export legacy `battery_loop` per compatibilità con old files.
5. `src/combat/kernel.rs` deve tornare a essere un motore FSM/shared-runtime generico, senza bootstrap diretto di blueprint specifici.
6. La struttura di `src/combat/` va resa più modulare, con file più piccoli e scope più chiari.
7. Va rivalutato il naming/ruolo di `combat/api`: oggi sembra più un framework runtime stile gameplay abilities / ECS execution layer che una “API”.
8. `enemy_counterplay` non deve rimanere percepito come logica kernel-centrica: ogni enemy/boss avrà la propria logica custom, il kernel deve offrire primitive, non policy.
9. Vanno tenuti presenti i capture:
   - `CAP-8d133d1a`
   - `CAP-af4db4ca`
   - `CAP-7c065a44` parte 2 (nota asset structure, non necessariamente da eseguire ora)

## Important verification note

Alcune descrizioni del problema sono state verificate come **ancora vere nel senso architetturale**, ma **non più nel punto esatto citato**:

- **Verificato:** `src/combat/kernel.rs` oggi contiene ancora coupling diretto ai blueprint tramite `register_canonical_passive_runners()`.
- **Verificato:** la registrazione hardcoded dei plugin blueprint-specifici oggi esiste, ma non in `kernel.rs`; vive in:
  - `src/combat/plugin.rs`
  - `src/combat/blueprints/mod.rs`
- **Non verificato nel codice corrente:** la vecchia forma “`kernel.rs` chiama direttamente `app.add_plugins(PatamonPlugin, DorumonPlugin, ...)`” non è più presente lì.
- **Da verificare durante il migrate:** quanto `ExtRegistries` sia ancora diventato una centralizzazione eccessiva, contro il paradigma “ogni Digimon è un Plugin Bevy che reagisce da fuori”. Oggi è certamente molto diffuso; resta da decidere cosa ridurre ora senza rompere M021.

## Scope summary

Primary themes:

1. `src/combat/kernel.rs` still mixes generic FSM/kernel primitives with blueprint bootstrap wiring.
2. `src/combat/plugin.rs` still owns blueprint/plugin registration order directly instead of delegating to a narrower runtime/bootstrap seam.
3. `src/combat/blueprints/mod.rs` is the real blueprint registry/composition root today, but still hardcodes plugin tuples, owner routing, and cleanup residue.
4. `src/combat/observability.rs` is generic in data collection but still hardcodes blueprint-specific section names and formatting policy.
5. `src/combat/api/applier.rs` still emits blueprint transition envelopes from shared code and is a mandatory audit target for hardcoded runtime assumptions.
6. `src/combat/mod.rs` still exports legacy identity modules (`battery_loop`, `precision_mind_game`, `enemy_counterplay`) that blur ownership and preserve old design names.
7. The `combat/api` naming and module shape are misleading enough to record as architectural debt in-scope for planning, but may require a shim-based migration rather than a brute-force rename.
8. Tests and docs still encode old structure/names and will block cleanup if not migrated last.

## File inventory by category

### A. Core runtime files that require structural changes

| File | LoC | Why it is in scope |
|---|---:|---|
| `src/combat/kernel.rs` | 719 | Defines generic transitions/state, but also owns `register_combat_kernel_runtime()` and hardcoded `register_canonical_passive_runners()` calling Agumon/Gabumon/Patamon/Renamon modules directly. Strong candidate for split into kernel primitives vs bootstrap/runtime glue. |
| `src/combat/plugin.rs` | 90 | `CombatPlugin` directly calls `crate::combat::blueprints::add_runtime_plugins(app)` and `register_canonical_passive_runners(app)`. So final runtime composition is still spread across plugin + kernel + blueprints. |
| `src/combat/mod.rs` | 125 | Public module surface still exports legacy/identity-shaped modules like `battery_loop`, `precision_mind_game`, and `enemy_counterplay`. This preserves old design language and hides real ownership under `blueprints/`. |
| `src/combat/blueprints/mod.rs` | 202 | Holds static blueprint dispatch registry (`BLUEPRINTS`), `add_runtime_plugins()`, ext registration, validation ext registration, and helper residue like `no_payload()`. This is the natural place for blueprint composition, but it needs cleanup and possibly submodule extraction. |

### B. Shared surfaces that still know too much about blueprint identity

| File | LoC | Why it is in scope |
|---|---:|---|
| `src/combat/observability.rs` | 543 | Collects generic validation sections correctly via `ExtRegistries`, but `format_validation_snapshot()` still hardcodes owners/labels like `twin_core`, `support`, `predator`, `mind_game`, `battery`. Also oversized and split-ready. |
| `src/combat/events.rs` | 167 | Re-exports `CombatKernelTransition` and defines the shared event seam. Must stay stable while kernel/blueprint internals move. Likely consumer-only changes, not redesign. |
| `src/combat/api/applier.rs` | 861 | Shared intent application surface. Direct `BatteryLoopState` references were not found in the current file, but it still emits blueprint transition envelopes from shared code (`apply_blueprint_signal`) and is a required dead-code / coupling audit target. |
| `src/combat/api/mod.rs` | 51 | The module name `api` is misleading relative to actual responsibility (gameplay runtime/framework). This may be addressed by re-organization and/or compatibility shims instead of a direct rename in the first waves. |
| `src/combat/api/registry.rs` | — | Defines `ExtRegistries`, which is now widely used across runtime, preview, timelines, validation, and tests. This is the central “DA_VERIFICARE” seam for over-centralization risk. |

### C. Blueprint-owned implementation files that reveal current true ownership

| File | LoC | Why it is in scope |
|---|---:|---|
| `src/combat/blueprints/tentomon.rs` | 695 | Owns `BatteryLoopState`, runtime hook registration, signal dispatch, validation section, and transition application. Confirms battery-loop logic belongs under blueprint ownership, not shared kernel/export paths. Also a likely source of dead-code residue after cleanup. |
| `src/combat/blueprints/twin_core/mod.rs` | — | Already uses validation ext pattern and blueprint transitions; relevant as the model for shared-but-blueprint-owned logic. |
| `src/combat/blueprints/{agumon,gabumon,patamon,dorumon,renamon}` | — | Used by hardcoded passive/plugin bootstrap paths and validation ext registration. These define the real downstream dependencies of any bootstrap refactor. |

### D. Files signaling outdated structure or ownership vocabulary

| File | Why it is in scope |
|---|---|
| `src/combat/battery_loop.rs` | Legacy export surface keeping Tentomon ownership blurred for old consumers/tests. |
| `src/combat/precision_mind_game.rs` | Legacy export surface preserving old Renamon-specific naming outside blueprint ownership. |
| `src/combat/enemy_counterplay.rs` | Must be reviewed as a typed data seam, not a kernel-policy seam. Current name/location still suggests centralization. |
| `src/combat/bootstrap.rs` | Imports `enemy_counterplay::EnemyCounterplayKit`; relevant to decide whether this remains a typed data seam or gets relocated. |
| `src/combat/action_query.rs`, `src/combat/follow_up.rs`, `src/combat/turn_system/mod.rs`, `src/ui/combat_panel.rs`, `src/bin/combat_cli.rs` | Active consumers of `enemy_counterplay::EnemyCounterplayKit`; these consumers shape how aggressively the module can move or be renamed. |

### E. Tests that currently encode old structure or old public paths

| File | LoC | Why it is in scope |
|---|---:|---|
| `tests/validation_snapshot.rs` | 405 | Exercises observability snapshot shape and formatting; will catch any owner-label or section-order churn. |
| `tests/battery_loop_kernel.rs` | 235 | Imports legacy `combat::battery_loop::*` path and validates Tentomon battery loop semantics. High-value migration guard, but currently tied to old public path. |
| `tests/tentomon_blueprint.rs` | 141 | Imports `combat::battery_loop::BatteryLoopState`; consumer of legacy re-export path. |
| `tests/passive_reactive_canon.rs` | — | Also imports `combat::battery_loop::BatteryLoopState`; consumer of legacy re-export path. |
| `tests/predator_loop_kernel.rs` | 304 | Observability/runtime guard for Dorumon/Predator naming and owner section behavior. |
| `tests/passive_canon_support.rs` | — | Covers `register_canonical_passive_runners()` and will detect bootstrap regressions. |
| `tests/action_affordance_consumers.rs`, `tests/engine_legality_integration.rs` | — | Consumers of `combat::enemy_counterplay::EnemyCounterplayKit`; relevant if moving or shrinking that seam. |
| `tests/holy_support_*`, `tests/twin_core_*`, `tests/dorumon_*`, `tests/renamon_*` | — | Broad set of consumers of `CombatKernelTransition::Blueprint` and validation snapshot output. These are mostly downstream verification, not first-wave edit targets. |

### F. Documentation and captured design intent that encode the target architecture

| File | LoC | Why it is in scope |
|---|---:|---|
| `docs/combat_current.md` | 84 | Current design summary; must match resulting structure after refactor. Also called out by the user as still encoding old assumptions about kernel ownership. |
| `docs/future_design_draft/02-03_blueprint_plugin.md` | 75 | Explicit target-spec saying kernel should not hardcode digimon/plugin-specific variants/bootstrap and that `BlueprintsPlugin` should own plugin composition. |
| `.gsd/CAPTURES.md` | 86 | Relevant captures: `CAP-8d133d1a`, `CAP-af4db4ca`, `CAP-7c065a44` constrain cleanup direction (remove digimon-specific kernel logic, keep enemy counterplay out of kernel, remember later asset split). |

## Exhaustive findings from search

### Hardcoded runtime/bootstrap coupling

#### Verified current code

- `src/combat/kernel.rs`
  - `register_combat_kernel_runtime(app)` initializes generic resources plus shared runtime resources.
  - `register_canonical_passive_runners(app)` directly calls:
    - `blueprints::agumon::register_passive_runtime(app)`
    - `blueprints::gabumon::register_passive_runtime(app)`
    - `blueprints::patamon::register_passive_runtime(app)`
    - `blueprints::renamon::register_passive_runtime(app)`
- `src/combat/plugin.rs`
  - `crate::combat::blueprints::add_runtime_plugins(app)`
  - `register_canonical_passive_runners(app)`
- `src/combat/blueprints/mod.rs`
  - `add_runtime_plugins(app)` hardcodes tuple plugin registration for `TwinCorePlugin`, `PatamonPlugin`, `DorumonPlugin`, `TentomonPlugin`, `RenamonPlugin`.
  - `BLUEPRINTS` hardcodes signal dispatch routing per owner.

#### Verified NOT current anymore

- `src/combat/kernel.rs` does **not** currently contain the hardcoded `app.add_plugins((PatamonPlugin, ...))` tuple the user quoted from an earlier revision/report.
- The architectural complaint remains valid, but the exact plugin-registration location has already shifted outward from `kernel.rs` into `plugin.rs` / `blueprints/mod.rs`.

### Legacy public-path residue

- `src/combat/mod.rs`
  - `pub mod enemy_counterplay;`
  - `pub mod battery_loop;`
  - `pub mod precision_mind_game;`
  - `pub mod api;`
- Tests still import legacy paths:
  - `tests/tentomon_blueprint.rs` → `combat::battery_loop::BatteryLoopState`
  - `tests/passive_reactive_canon.rs` → `combat::battery_loop::BatteryLoopState`
  - `tests/action_affordance_consumers.rs`, `tests/engine_legality_integration.rs` → `combat::enemy_counterplay::EnemyCounterplayKit`
- Runtime/UI consumers still import `enemy_counterplay::EnemyCounterplayKit` from shared combat paths:
  - `src/ui/combat_panel.rs`
  - `src/bin/combat_cli.rs`
  - `src/combat/bootstrap.rs`
  - `src/combat/action_query.rs`
  - `src/combat/follow_up.rs`
  - `src/combat/turn_system/mod.rs`

### Observability hardcoded naming

- `src/combat/observability.rs`
  - `format_validation_snapshot()` prints fixed sections for:
    - `twin_core`
    - `support`
    - `predator`
    - `mind_game`
    - `battery`
  - formatting helpers are specialized to these owners instead of operating on a generic section vocabulary.
- Multiple tests assert those strings/owners directly, so decoupling observability is not just internal cleanup: it requires migration of verification surfaces.

### Cleanup / dead code / likely residue

- `src/combat/blueprints/mod.rs`
  - `no_payload(...)` is explicitly marked `#[allow(dead_code)]` and called out as compatibility residue.
- `src/combat/api/applier.rs`
  - No direct `BatteryLoopState` reference was found in the current file.
  - `apply_blueprint_signal(...)` still duplicates blueprint-envelope emission in shared code and remains a valid coupling/dead-code audit target.
- `src/combat/api/mod.rs`
  - module naming mismatch (`api`) is architectural debt and now explicitly user-reported.

### ExtRegistries centralization risk — verified and still open

- `ExtRegistries` appears across runtime, timelines, preview, validation, data loading, follow-up execution, and a large test surface.
- This confirms the user’s “centralizzazione tramite string-ID” concern is not hypothetical.
- What remains unverified is **how much of this registry is still necessary** for compiled timelines vs what can be pulled outward into per-blueprint Bevy plugin registration. That is a planning/migration question, not yet answered by inventory alone.

## Dependency relationships

1. **Kernel/bootstrap split comes first**
   - `kernel.rs` must stop owning passive bootstrap before public surface cleanup can safely remove old names.
2. **Blueprint composition ownership must be centralized next**
   - `plugin.rs` and `blueprints/mod.rs` need a single authoritative composition seam for runtime plugins + passive bootstrap + validation ext registration.
3. **Observability formatting should move after composition is stable**
   - it depends on owner names and validation ext registration behavior staying predictable.
4. **Legacy public paths should be migrated after implementation moves compile cleanly**
   - tests/docs import these paths and should be updated last, optionally with short-lived shims.
5. **Dead-code removal is final-pass work**
   - only after callers and docs have moved can residues like `no_payload` and old export modules be deleted safely.
6. **Renaming/re-grouping `combat/api` should be treated as an optional late wave or shimmed move**
   - it touches too many imports to mix with the first decoupling wave.
7. **`enemy_counterplay` needs classification before relocation**
   - first decide whether it is a typed shared data seam worth keeping, or just old centralization residue; only then move/rename/remove it.

## Required outcomes implied by inventory

The user’s clarified request means the refactor is not “just” cleanup. The resulting state should satisfy all of these:

- shared `kernel` code is blueprint-agnostic in ownership and bootstrap responsibility
- `observability` depends on generic validation sections, not named blueprint-specific formatter branches
- old exports like `combat::battery_loop::*` are gone or reduced to short-lived migration shims during the refactor
- dead-code allowances introduced by prior refactors are either removed or justified by an active remaining consumer
- file boundaries in `src/combat/` are more explicit and smaller, especially around kernel/bootstrap/observability composition
- the role of `combat/api` is at least documented and plan-addressed, even if the rename itself lands in a later safe wave

## Suggested safe waves implied by inventory

1. **Wave 1 — Split `kernel.rs` by responsibility without behavior change**
   - Separate kernel primitives from runtime/bootstrap helpers.
2. **Wave 2 — Centralize blueprint runtime composition**
   - Move plugin/passive/bootstrap ownership behind one narrower seam, likely under `blueprints/` or a new combat composition module.
3. **Wave 3 — Split and genericize observability formatting**
   - Keep snapshot data contract stable while removing hardcoded section policy from the core formatter.
4. **Wave 4 — Migrate public paths/tests/docs**
   - Update imports from old exports like `combat::battery_loop` and shared `enemy_counterplay` exposure.
5. **Wave 5 — Remove residues and dead code**
   - Delete stale exports/helpers once search is clean.
6. **Optional Wave 6 — Rename/re-frame `combat/api`**
   - Only if the earlier waves leave the codebase stable and the rename can be shimmed safely.

## Estimated scope

### High-touch files

- `src/combat/kernel.rs` — 719 LoC
- `src/combat/api/applier.rs` — 861 LoC
- `src/combat/observability.rs` — 543 LoC
- `src/combat/blueprints/tentomon.rs` — 695 LoC
- `src/combat/blueprints/mod.rs` — 202 LoC
- `src/combat/mod.rs` — 125 LoC
- `src/combat/plugin.rs` — 90 LoC

### Likely edited files in this refactor

- Core source: ~10-18 files
- Tests/docs touched for migration/cleanup: ~8-16 files
- Potential new modules created to reduce file size: ~5-10 files

### Risk estimate

- **Behavioral risk:** medium
- **Compile risk:** medium-high during the middle waves because bootstrap ownership and public import paths are cross-cutting
- **Regression hotspots:** passive bootstrap ordering, validation snapshot string output, Tentomon battery loop imports, `enemy_counterplay` consumer paths, Dorumon/Patamon validation sections

## Explicitly not committed by this inventory

These are noted, but the inventory does **not** promise them in the same pass unless the plan can contain them safely:

- full immediate replacement of `ExtRegistries` with pure plugin-local Bevy registration
- asset-structure split from `CAP-7c065a44` part 2
- broad test pruning unrelated to kernel/blueprint decoupling
- giant one-shot rename of `combat/api` with no compatibility layer

## Inventory verdict

The requested scope is real, larger than the first inventory draft, and still mechanically actionable.

The highest-value first targets are now:

1. `src/combat/kernel.rs`
2. `src/combat/plugin.rs`
3. `src/combat/blueprints/mod.rs`
4. `src/combat/observability.rs`
5. legacy exports in `src/combat/mod.rs`
6. downstream consumers of `enemy_counterplay`
7. the `combat/api` naming debt, at least at planning level

The main thing to avoid is still the same: do **not** mix deep ownership moves, broad module renames, and behavior changes in the same first wave. But the updated inventory now captures the full requested work, including structure pressure, dead-code cleanup, and architectural naming debt.