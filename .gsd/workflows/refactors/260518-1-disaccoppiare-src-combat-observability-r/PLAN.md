# Plan — combat decoupling and module cleanup (waves)

Date: 2026-05-18
Branch: `gsd/refactor/disaccoppiare-src-combat-observability-r`
Source: `INVENTORY.md` (this directory)

## Baseline (verified 2026-05-18)

- `cargo check` → exit 0 (compiles clean).
- `cargo build` → 91 warnings (1 duplicate); ~10 cargo-fix-able.
- Warning surface is dominated by legacy blueprint re-export leakage (`twin_core`,
  `predator_loop`, `holy_support`, `battery_loop` glob re-exports) and unused
  framework scaffolding (`CastRng`, `CastIdGen`, dead `Registry` variants).

## Wave principles

- Each wave ends with `cargo check` green and the **targeted** tests green.
- Each wave is one commit: `refactor(combat): wave N — <desc>`.
- No behavior change except where a wave explicitly removes dead/legacy code.
- Public-path migrations (tests/docs) come **after** implementation compiles, with
  short-lived `#[deprecated]` shims where it shrinks blast radius.
- `combat/api` rename is deferred to an optional final wave; earlier waves must
  not depend on it.

## Wave 1 — Split `kernel.rs`, strip blueprint bootstrap from kernel

**Goal:** `kernel.rs` owns only generic FSM/runtime primitives. Blueprint
passive bootstrap leaves the kernel entirely.

Changes:
- Extract `register_combat_kernel_runtime()` generic resource init into a
  `kernel/runtime.rs` (or keep in `kernel.rs` if purely generic).
- Move `register_canonical_passive_runners()` out of `kernel.rs` into the
  blueprint composition root (Wave 2 target seam) — for Wave 1, relocate it to
  `blueprints/mod.rs` as `register_canonical_passive_runners()` and have
  `plugin.rs` call the blueprints-owned version. Kernel stops importing
  `blueprints::{agumon,gabumon,patamon,renamon}`.
- Keep `CombatKernelTransition` re-export seam stable (consumers untouched).

Files:
- `src/combat/kernel.rs` (split / shrink, remove blueprint imports)
- `src/combat/plugin.rs` (call blueprints-owned bootstrap)
- `src/combat/blueprints/mod.rs` (receive `register_canonical_passive_runners`)
- possibly new: `src/combat/kernel/` module dir if split warranted

Verify: `cargo check` + `cargo test --test passive_canon_support`
+ `cargo test --test passive_reactive_canon`.

## Wave 2 — Single blueprint composition seam

**Goal:** One authoritative place owns runtime-plugin registration + passive
bootstrap + validation-ext registration. `plugin.rs` delegates; it does not
enumerate blueprints.

Changes:
- In `blueprints/mod.rs`: consolidate `add_runtime_plugins()`,
  `register_canonical_passive_runners()`, and validation-ext registration into
  one `BlueprintsPlugin` (or `register_blueprints(app)`) entry point.
- `plugin.rs` `CombatPlugin` calls only that single seam + generic kernel runtime.
- Remove `no_payload()` dead-code residue if now unreferenced.

Files:
- `src/combat/blueprints/mod.rs`
- `src/combat/plugin.rs`

Verify: `cargo check` + `cargo test --test holy_support_*` (passive/plugin wiring)
+ a broad smoke run `cargo test --test engine_legality_integration`.

## Wave 3 — Genericize observability formatting

**Goal:** `observability.rs` formats from a generic section vocabulary; no
hardcoded `twin_core/support/predator/mind_game/battery` branches.

Changes:
- Replace `format_validation_snapshot()` per-owner branches with iteration over
  the `ExtRegistries`-collected sections (ordered, label-from-section).
- Split `observability.rs` (543 LoC) into data-collection vs formatting submodule
  if it reduces size meaningfully without churning the snapshot contract.
- Keep snapshot **string output stable** where tests assert it; if section order
  must change, update `tests/validation_snapshot.rs` in the same wave.

Files:
- `src/combat/observability.rs`
- `tests/validation_snapshot.rs` (only if output ordering changes)

Verify: `cargo test --test validation_snapshot`
+ `cargo test --test predator_loop_kernel`.

## Wave 4 — Migrate legacy public paths

**Goal:** Remove `combat::battery_loop` / `combat::precision_mind_game` glob
re-export leakage and route consumers to true blueprint ownership.

Changes:
- Repoint test imports:
  - `tests/tentomon_blueprint.rs`, `tests/passive_reactive_canon.rs`,
    `tests/battery_loop_kernel.rs` → blueprint-owned `BatteryLoopState` path.
- Classify `enemy_counterplay` vs `counterplay` (both exist in `mod.rs`):
  decide the canonical typed-data seam, add `#[deprecated]` alias on the other,
  repoint runtime/UI consumers (`ui/combat_panel.rs`, `bin/combat_cli.rs`,
  `combat/bootstrap.rs`, `combat/action_query.rs`, `combat/follow_up.rs`,
  `combat/turn_system/mod.rs`).
- `combat/mod.rs`: drop legacy `pub mod battery_loop / precision_mind_game`
  (or reduce to deprecated shims for one wave).

Files: `src/combat/mod.rs`, the 6 runtime/UI consumers above, ~4 test files.

Verify: `cargo test` (full suite — this is the high-blast-radius wave).

## Wave 5 — Dead-code and residue removal

**Goal:** Clear warnings introduced by/exposed during earlier waves.

Changes:
- Remove unused glob re-exports flagged by build
  (`twin_core::*`, `predator_loop`, `holy_support`, `tentomon::*`, `renamon::*`
  unused-import warnings).
- Remove genuinely dead framework scaffolding only if confirmed unreferenced:
  `CastRng`/`CastIdGen`, dead `Registry` variants (`Any/All/Not/CombatEvent/Custom`),
  unused `units_ron` enums (`TwinCoreLine/Role/PersonalLabel`).
- Anything still `#[allow(dead_code)]` must have a named live consumer or be deleted.

Files: scattered — driven by `cargo build` warning list, not guesswork.

Verify: `cargo build` → **0 dead-code/unused warnings in touched modules**;
`cargo test` full suite green.

## Wave 6 (optional) — Reframe `combat/api` naming

**Goal:** Address the misleading `api` name (it is a gameplay/FSM runtime layer).

Changes:
- Rename `combat/api` → `combat/gameplay_runtime` (or `combat/fsm`) with a
  `pub use` shim at the old path for one release, OR document-only if rename
  risk outweighs value at that point.

Files: `src/combat/mod.rs`, `src/combat/api/*`, broad import updates.

Verify: `cargo check` + `cargo test` full suite.
Gate: only attempt if Waves 1–5 left the tree stable; otherwise document as debt.

## Docs (folded into the last touching wave, not a separate wave)

- `docs/combat_current.md`: update to reflect kernel-is-generic +
  blueprint-owns-composition after Wave 2/3 land.
- Keep `docs/future_design_draft/02-03_blueprint_plugin.md` as the target spec.

## Dependency order (hard constraints)

1 → 2 → 3 → 4 → 5, with 6 optional/last.
Rationale: kernel must stop owning bootstrap (1) before a single seam can own it
(2); observability genericization (3) depends on stable ext-registration from (2);
public-path migration (4) must follow compiling implementation; dead-code removal
(5) is only safe once consumers/docs moved.

## Risk

- Behavioral: medium. Hotspots: passive bootstrap ordering (W1–2), validation
  snapshot strings (W3), Tentomon battery-loop imports + `enemy_counterplay`
  consumers (W4).
- Compile: medium-high in W2 and W4 (cross-cutting import moves).
- Mitigation: per-wave commit + targeted tests before full-suite gates.

## Out of scope (per inventory)

- Full `ExtRegistries` removal / pure plugin-local registration.
- `CAP-7c065a44` part 2 asset-structure split.
- Unrelated broad test pruning.
- One-shot un-shimmed `combat/api` rename.
