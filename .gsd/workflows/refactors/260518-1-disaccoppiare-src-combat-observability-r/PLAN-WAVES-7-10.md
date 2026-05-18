# Plan — Waves 7–10 (continuation)

Date: 2026-05-18
Branch: `gsd/refactor/disaccoppiare-src-combat-observability-r`
Source: `GAP-ANALYSIS.md` (this directory)
Predecessor: `PLAN.md` (Waves 1–5 — all complete)

## Revised scope

The original gap analysis identified 6 remaining items (A–F). After verifying
current code, items A (#2 applier decoupling) and D (#8 enemy_counterplay merge)
were already resolved by earlier waves. Item B is partially resolved (kernel.rs
already split). This leaves 4 actionable items grouped into 4 waves.

## Wave 7 — Split oversized files into submodules

**Problem:** applier.rs (870 LoC) and observability.rs (452 LoC) are still
monolithic. Constraint #6 requires modular splits for agent-friendly navigation.

**Approach:**
- `api/applier.rs` → `api/applier/` module:
  - `mod.rs` — re-exports + `apply_resolved_action` orchestrator
  - `damage.rs` — damage computation, DR application, pre-damage reactions
  - `effects.rs` — follow-up processing, status effects, stun/energy/SP mutations
- `observability.rs` → `observability/` module:
  - `mod.rs` — re-exports + `ValidationSnapshot` struct
  - `collect.rs` — data collection from ECS world
  - `format.rs` — text formatting and section rendering

Pure structural move — zero behavior change.

**Files:** `src/combat/api/applier.rs` → `src/combat/api/applier/`, `src/combat/observability.rs` → `src/combat/observability/`
**Verify:** `cargo check` + `cargo test` (full suite — structural, zero behavior change)

## Wave 8 — Document combat/api role + naming debt

**Problem:** `combat/api` name is misleading (constraint #7). It is a
gameplay-ability execution runtime (FSM runner, timeline evaluation, intent
application), not an external API. The existing doc comment in `api/mod.rs`
says "Framework primitives for M021" which is accurate but doesn't address the
naming issue.

**Approach:**
- Extend the module-level doc comment in `src/combat/api/mod.rs` to explicitly
  note that "api" is a historical name and the module is a gameplay-ability
  runtime, not an external API surface. Record the rename as architectural debt.
- No actual rename — risk outweighs value at this point with 16+ files in the
  module and dozens of consumer imports.

**Files:** `src/combat/api/mod.rs`
**Verify:** `cargo check` (doc comment only)

## Wave 9 — Update docs/combat_current.md

**Problem:** `docs/combat_current.md` still references M016 as latest baseline.
It doesn't reflect the 5-wave refactor: kernel is now generic, blueprints own
composition, observability is blueprint-agnostic, kernel/ is a module directory,
counterplay is consolidated, etc.

**Approach:** Rewrite `docs/combat_current.md` to reflect post-refactor state:
- Kernel is generic FSM — `kernel/mod.rs` + `kernel/primitives.rs`
- Blueprint composition is centralized in `blueprints/mod.rs`
- Observability formats from generic section vocabulary (ExtRegistries)
- `counterplay.rs` is the single typed-data seam for enemy counterplay
- applier uses `ExtRegistries.pre_damage_reactions` for block reactions
- Legacy battery_loop/precision_mind_game exports removed
- All tests import from blueprint-owned paths

**Files:** `docs/combat_current.md`
**Verify:** `cargo check` (docs only — content accuracy verified by reading code)

## Wave 10 (optional) — Asset structure split (CAP-7c065a44 pt.2)

**Problem:** `assets/data/units.ron` and `assets/data/skills.ron` are monolithic
blobs, not per-digimon.

**Approach:** Split into per-digimon directories:
```
assets/data/digimon/{agumon,gabumon,dorumon,patamon,renamon,tentomon}/
  unit.ron
  skills.ron
assets/data/enemies/
  <enemy-specific RON>
assets/data/shared/
  party.ron (or keep at assets/data/party.ron)
```

Update RON loaders in `src/data/units_ron.rs` and `src/data/skills_ron.rs` to
discover per-directory files. Update `party.ron` references if needed.

**Files:** `assets/data/` restructure, `src/data/units_ron.rs`, `src/data/skills_ron.rs`, `src/data/party_ron.rs`
**Verify:** `cargo test` (full suite — loader behavior change)

## Dependency order

7 → 8 → 9 (→ 10 optional).
Wave 7 is structural code change. Wave 8 is doc comment. Wave 9 is documentation
update (benefits from 7 landing first so docs reflect final structure). Wave 10
is independent but listed last as it was flagged "non necessariamente da eseguire ora".

## Risk

- **Wave 7:** Medium compile risk (import moves across ~30 consumer files for applier).
  Zero behavioral risk — pure structural.
- **Wave 8:** Zero risk — doc comment only.
- **Wave 9:** Zero code risk — documentation only.
- **Wave 10:** Medium behavioral risk — loader change. All tests must pass.
