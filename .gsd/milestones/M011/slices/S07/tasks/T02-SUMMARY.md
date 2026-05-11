---
id: T02
parent: S07
milestone: M011
key_files:
  - src/combat/toughness.rs
  - src/data/units_ron.rs
  - assets/data/units.ron
  - src/combat/bootstrap.rs
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/resolution.rs
  - src/combat/follow_up.rs
key_decisions:
  - Used Default::default() in test struct literals instead of ToughnessCategory::Standard path to avoid adding imports to ~9 test files; ToughnessCategory derives Default so this is identical at runtime
  - follow_up.rs defines its own local ResolveActorsQuery type alias (not re-using the one from turn_system/mod.rs) — both must be kept structurally in sync since Bevy queries are typed nominally; updated both in parallel
duration: 
verification_result: passed
completed_at: 2026-04-28T09:04:34.173Z
blocker_discovered: false
---

# T02: Wired ToughnessCategory through UnitDef/units.ron/bootstrap, threaded RoundFlags break_sealed into resolve pipeline, and reset seal on defender's turn advance

**Wired ToughnessCategory through UnitDef/units.ron/bootstrap, threaded RoundFlags break_sealed into resolve pipeline, and reset seal on defender's turn advance**

## What Happened

Six files updated to thread the T01 primitives end-to-end:

1. **`src/combat/toughness.rs`**: Added `serde::Serialize, serde::Deserialize` to `ToughnessCategory` so the field can round-trip through RON in `UnitDef`.

2. **`src/data/units_ron.rs`**: Added `toughness_category: ToughnessCategory` field with `#[serde(default)]` to `UnitDef`; added `use crate::combat::toughness::ToughnessCategory`; updated the `round_trip_unit_def` test fixture.

3. **`assets/data/units.ron`**: Set `toughness_category: Armored` on Devimon (id 101) — the integration test fixture for T03.

4. **`src/combat/bootstrap.rs`**: Imported `ToughnessCategory` and `RoundFlags`; changed `Toughness::new(...)` to `Toughness::with_category(def.toughness_max, def.weaknesses.clone(), def.toughness_category)`; added `RoundFlags::default()` to spawn bundle; added `toughness_category: ToughnessCategory::Standard` to `taichi_def()` literal.

5. **`src/combat/turn_system/mod.rs`**: Added `RoundFlags` to imports; extended `ResolveActorsQuery` with `Option<&'static mut RoundFlags>` as element 12; extended the `advance_turn_system` query with `Option<&mut RoundFlags>`; updated all four destructuring patterns (snapshots iterator, TurnAdvanced loop `get_mut`, AV iterator, entity_ready `get_mut`); added seal-reset logic at the start of each TurnAdvanced iteration.

6. **`src/combat/turn_system/pipeline.rs`**: Updated `step_declaration` iterator pattern from 11 to 12 elements; updated `step_app` `get_many_mut` destructuring to capture `defender_round_flags`; read `defender_break_sealed` before calling `apply_effects`; added `defender_break_sealed` argument to the call; added post-call seal-set block (`if outcome.broke { flags.break_sealed = true }`).

7. **`src/combat/resolution.rs`**: Added `defender_break_sealed: bool` parameter after `defender_is_commander`; removed the T01 placeholder comment; threaded the parameter into `defender_tough.apply_hit(...)`.

8. **`src/combat/follow_up.rs`**: Updated the local `ResolveActorsQuery` type alias to include `Option<&'static mut RoundFlags>` to match the turn_system definition (Bevy queries are structural).

9. **Test files** (`resolution_tests.rs`, `bootstrap_spawn_composition.rs`, `roster_smoke.rs`, `tempo_resistance.rs`, `follow_up_reentrancy.rs`, `pipeline_dispatch.rs`, `follow_up_triggers.rs`, `enemy_ai.rs`, `validation_snapshot.rs`): Updated all `apply_effects` call sites with `false` for the new `defender_break_sealed` parameter; added `toughness_category: Default::default()` to `UnitDef` literals; added `category: Default::default()` to `Toughness` struct literals.

## Verification

Full `cargo test` suite run — all tests pass, zero failures. Slice-level check: `cargo test 2>&1 | grep -E 'test result'` shows only `ok` lines. The `defender_break_sealed` path is now live: `resolution.rs` reads the flag from the pipeline instead of the hardcoded `false` placeholder. `RoundFlags` is spawned with every unit via `bootstrap.rs`, meaning the query will always find the component and `Option<Mut<RoundFlags>>` resolves to `Some`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test 2>&1 | grep -E 'test result' | tail -5` | 0 | ✅ pass | 28000ms |

## Deviations

follow_up.rs has a private ResolveActorsQuery type alias that mirrors the one in turn_system/mod.rs — the plan did not mention this second copy, but it must be kept in sync. Updated both without escalating since it is a straightforward structural change.

## Known Issues

none

## Files Created/Modified

- `src/combat/toughness.rs`
- `src/data/units_ron.rs`
- `assets/data/units.ron`
- `src/combat/bootstrap.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `src/combat/follow_up.rs`
