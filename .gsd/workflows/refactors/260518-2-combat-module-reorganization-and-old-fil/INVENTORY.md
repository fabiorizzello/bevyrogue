# Inventory — Combat module reorganization and old files cleanup

Date: 2026-05-18
Branch: `gsd/refactor/combat-module-reorganization-and-old-fil`
Predecessor: `260518-1-disaccoppiare-src-combat-observability-r` (waves 1-10 complete)

## Goal

Reorganize the 34 flat files in `src/combat/` into scoped submodules, remove shared demo skills that are no longer needed (each digimon has unique skills), clean up old .ron remnants, and update captures.

## Scope — 3 work streams

### A. Combat module reorganization

`src/combat/` has 34 `.rs` files at root level with only 5 submodules (`api/`, `blueprints/`, `kernel/`, `observability/`, `turn_system/`). The `mod.rs` already groups them into 6 logical buckets via comments. This reorganization makes those buckets into actual submodules.

**Current root files → proposed submodules:**

| Submodule | Files to move | LoC |
|-----------|--------------|-----|
| `mechanics/` | buffs.rs, damage.rs, energy.rs, follow_up.rs, modifiers.rs, round_flags.rs, sp.rs, status_effect.rs, stun.rs, toughness.rs, ultimate.rs | ~3,090 |
| `turn_system/` (merge into existing) | av.rs, speed.rs, turn_order.rs, resistance.rs | ~280 |
| `encounter/` | bootstrap.rs, counterplay.rs, enemy_ai.rs | ~680 |
| `observability/` (merge into existing) | events.rs, floating.rs, jsonl_logger.rs, log.rs | ~290 |
| Root keeps | types.rs, unit.rs, team.rs, kit.rs, state.rs, rng.rs, bevy_types.rs, plugin.rs, mod.rs, action_query.rs, preview.rs, resolution.rs | ~4,400 |

**Impact on imports:** Every `use crate::combat::damage` becomes `use crate::combat::mechanics::damage`, etc. This ripples into ~40 test files and ~15 src files.

**Re-export strategy:** Each new submodule's `mod.rs` will `pub use` its children. The combat `mod.rs` will re-export the submodule contents flat so that most external imports (`use crate::combat::damage::*`) continue to work via `pub use mechanics::damage`. This minimizes test churn.

### B. Shared skills cleanup

`assets/data/shared/skills.ron` contains 6 demo skills: rally, first_aid, taunt, brave_tri_strike, nova_burst, dark_flood.

Each digimon now has unique skills under `assets/data/digimon/{name}/skills.ron`. The shared skills are only used by:

| Consumer | Skills used | Action needed |
|----------|------------|---------------|
| `src/combat/bootstrap.rs` L227-233 | rally, first_aid, taunt, brave_tri_strike | Replace with a proper enemy skill set or inline constants |
| `src/combat/resolution.rs` L1507 | brave_tri_strike | Part of grant_free_skill_def — review if still needed |
| `src/bin/combat_cli.rs` L938-1003 | nova_burst, dark_flood | Demo CLI — remove references or inline |
| `src/data/mod.rs` L380,403 | loads shared/skills.ron | Remove from aggregate_skill_book paths |

No tests reference these skill names.

### C. Captures update

| Capture | Current status | Should be |
|---------|---------------|-----------|
| CAP-8d133d1a | resolved/defer | **DONE** — kernel cleaned of digimon-specific logic, orphan code pruned in waves 1-5 |
| CAP-af4db4ca | resolved/defer | **DONE** — counterplay merged, kept as typed data seam not kernel policy |
| CAP-7c065a44 | resolved/defer | **Partially done** — per-digimon skill/unit .ron files exist, but shared/skills.ron demo remnants remain. Split into: pt.1 done (per-digimon .ron), pt.2 this workflow (shared cleanup) |
| CAP-159d33b5 | resolved/defer | No change — M023 future work |
| CAP-b892da3a | resolved/defer | No change — future ergonomics work |

## Dependency order

1. **Shared skills cleanup (B)** can proceed independently — no structural dependencies
2. **Module reorganization (A)** is the main work; must preserve compile + test green at each step
3. **Captures update (C)** is a doc-only pass, can run last or in parallel

## Files touched

### Source (estimated)
- `src/combat/mod.rs` — heavy rewrite (all pub mod declarations change)
- ~15 files moved into new submodule directories
- ~4 files in `src/combat/` that import sibling modules (path adjustments)
- `src/data/mod.rs` — remove shared skill paths
- `src/combat/bootstrap.rs` — replace shared skill references
- `src/combat/resolution.rs` — review brave_tri_strike usage
- `src/bin/combat_cli.rs` — update or remove shared skill references

### Tests (~40 files)
Most use `use crate::combat::{damage, follow_up, ...}` style. If re-exports are maintained at `combat::` level, test changes are minimal. If not, ~40 files need import updates.

### Assets
- `assets/data/shared/skills.ron` — delete or empty
- `assets/data/shared/` — may become empty dir (delete)

### Docs
- `.gsd/CAPTURES.md` — update statuses

## Risk estimate

- **Behavioral risk:** low (pure structural moves + demo skill removal)
- **Compile risk:** medium (import path changes across many files)
- **Regression hotspots:** aggregate_skill_book() loading, bootstrap enemy setup, CLI demo
