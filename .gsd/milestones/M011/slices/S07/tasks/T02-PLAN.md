---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T02: Wire ToughnessCategory through UnitDef/units.ron and bootstrap; thread RoundFlags + break_seal into resolve pipeline; reset seal on defender's next turn

Wire the T01 primitives end-to-end. (1) In `src/data/units_ron.rs`, add `pub toughness_category: ToughnessCategory` to `UnitDef` with `#[serde(default)]` so existing units default to Standard; re-export `ToughnessCategory` from `src/combat/toughness.rs` and import it in `units_ron.rs`. Update the in-file `round_trip_unit_def` test fixture to set the field. (2) In `assets/data/units.ron`, set `toughness_category: Armored` on Devimon (id 101) so the integration test in T03 has a real Armored fixture; leave all other units defaulting to Standard. (3) In `src/combat/bootstrap.rs::spawn_unit_from_def`, construct `Toughness::with_category(def.toughness_max, def.weaknesses.clone(), def.toughness_category)` and add `RoundFlags::default()` to the spawn bundle. Import `RoundFlags` from `crate::combat::round_flags`. (4) Extend `ResolveActorsQuery` in `src/combat/turn_system/mod.rs` to include `Option<&'static mut RoundFlags>` as element 12 (mirroring the BasicStreak pattern at element 11). (5) In `src/combat/turn_system/pipeline.rs::step_app`, after the existing `actors.get_many_mut([...])` destructure, read the defender's `RoundFlags.break_sealed` (defaulting to false if absent), pass it as a new parameter to `apply_effects`, and after the call — if `outcome.broke == true` — set the defender's `RoundFlags.break_sealed = true`. Update the destructuring tuples accordingly. (6) In `src/combat/resolution.rs::apply_effects`, add a `defender_break_sealed: bool` parameter (placed after `defender_is_commander`), thread it into the `defender_tough.apply_hit(resolved.damage_tag, resolved.toughness_damage, defender_break_sealed)` call, and replace the T01 placeholder. (7) In `src/combat/turn_system/mod.rs::advance_turn_system`, in the per-`TurnAdvanced` loop (around the stunned/status_opt block), reset the active unit's `RoundFlags.break_sealed = false` at the start of its turn — this is the seal-clear point. Add `Option<&mut RoundFlags>` to the existing turn-system query and clear when present. (8) Sweep all integration tests in `tests/` that construct `Toughness` directly via `Toughness::new(...)` — keep them compiling (the existing `new` signature is preserved by T01); no test changes required unless a compile error surfaces. Document any test fixture that you do touch. Confirm `cargo test` (full suite) is green before T03.

## Inputs

- `src/data/units_ron.rs`
- `assets/data/units.ron`
- `src/combat/bootstrap.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `src/combat/toughness.rs`
- `src/combat/round_flags.rs`

## Expected Output

- `src/data/units_ron.rs`
- `assets/data/units.ron`
- `src/combat/bootstrap.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`

## Verification

cargo test 2>&1 | grep -E 'test result' | tail -5

## Observability Impact

Affects the action pipeline (step_app) and turn loop (advance_turn_system). When a break is suppressed by an active seal, no `CombatEventKind::OnBreak` is emitted and `outcome.broke == false` — call sites must not assume break-on-toughness-zero anymore. Reset on TurnAdvanced is the single point where the seal lifts; logging the reset is unnecessary but a `debug!` line in advance_turn_system is acceptable for diagnosis.
