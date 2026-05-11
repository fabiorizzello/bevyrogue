# S07: Toughness 3 categorie (Standard/Armored/Shielded) + Break Seal

**Goal:** Differentiate enemy defensive archetypes via three Toughness categories (Standard, Armored, Shielded) and introduce a Break Seal that prevents repeated Toughness breaks on the same defender within the same round, satisfying R079.
**Demo:** scenario CLI: rompere un Armored mostra 2 colpi richiesti; Break Seal blocca successivi tentativi nel turno

## Must-Haves

- CLI scenario: breaking an Armored enemy requires roughly 2x the toughness damage of a Standard one; once broken, further break attempts on that defender within the same round emit no new OnBreak event. Shielded enemies never break from a normal ToughnessHit. Integration test `tests/toughness_categories.rs` covers all three categories and the seal lifecycle.

## Proof Level

- This slice proves: - This slice proves: integration
- Real runtime required: no
- Human/UAT required: no

## Integration Closure

- Upstream surfaces consumed: `src/combat/toughness.rs` (Toughness::apply_hit), `src/combat/resolution.rs::apply_effects` (defender_tough mutation), `src/combat/turn_system/pipeline.rs::step_app` (call site), `src/combat/turn_system/mod.rs` (TurnAdvanced handling for round-flag reset), `src/combat/bootstrap.rs::spawn_unit_from_def` (component wiring), `src/data/units_ron.rs::UnitDef`.
- New wiring introduced: `RoundFlags` component spawned on every unit; reset hook on TurnAdvanced; `category` field on Toughness threaded from RON through bootstrap into apply_hit.
- What remains before milestone is usable end-to-end: S08 (Form Identity) will reuse `RoundFlags` for once-per-round form triggers; S09 will rebalance numerical toughness values per category.

## Verification

- Runtime signals: existing CombatEventKind::OnBreak continues to be the single source of truth — when a break is suppressed by Shielded or by an active seal, no OnBreak fires (callers can detect "tried to break, didn't" by tracking ToughnessHit emissions vs. OnBreak count).
- Inspection surfaces: Toughness component (now carries `category`); RoundFlags component (now carries `break_sealed`) — both queryable in tests.
- Failure visibility: ActionLog already records Break entries via LogEntry::Break; absence of a new entry after a sealed attempt is the diagnostic signal.
- Redaction constraints: none.

## Tasks

- [x] **T01: Add ToughnessCategory enum + RoundFlags component; extend Toughness::apply_hit with category and seal logic** `est:1h`
  Introduce the foundational types for the slice. Define a `ToughnessCategory` enum (Standard, Armored, Shielded) in `src/combat/toughness.rs` and add it as a field on the `Toughness` component (default Standard for back-compat with existing constructors). Extend `Toughness::new` with a category parameter and add a convenience `Toughness::with_category(max, weaknesses, category)` constructor; keep the existing `new` signature working by defaulting to Standard inside the body. Modify `Toughness::apply_hit` to accept a `break_sealed: bool` flag and to behave as follows: (a) Shielded NEVER transitions to broken from a `ToughnessHit` and never decrements `current` past 0 (clamp at 0, return false); (b) Armored halves the incoming toughness damage (rounded up: `(amount + 1) / 2`) before applying — so an Armored unit needs roughly 2x the cumulative toughness damage to break; (c) Standard preserves the existing semantics. In all categories, if `break_sealed` is true the function returns false without mutating `current`. Define the `RoundFlags` component in a new file `src/combat/round_flags.rs` with a single `pub break_sealed: bool` field (Default = false), register the module in `src/combat/mod.rs`, and re-export the type. Update the existing in-file unit tests in `toughness.rs` to (i) keep passing for Standard via the default, (ii) add three new unit tests: `shielded_never_breaks_from_toughness_hit`, `armored_requires_double_damage_to_break`, `apply_hit_is_noop_when_break_sealed`. Do NOT yet wire the new component into bootstrap or threadthe new `apply_hit` signature into resolution — those happen in T02. To keep the codebase compiling at the end of T01, update the single call site in `src/combat/resolution.rs::apply_effects` to pass `false` as the new `break_sealed` argument (placeholder; T02 will replace it with the real query).
  - Files: `src/combat/toughness.rs`, `src/combat/round_flags.rs`, `src/combat/mod.rs`, `src/combat/resolution.rs`
  - Verify: cargo test --lib combat::toughness 2>&1 | grep -E 'test result' && cargo check 2>&1 | tail -5

- [x] **T02: Wire ToughnessCategory through UnitDef/units.ron and bootstrap; thread RoundFlags + break_seal into resolve pipeline; reset seal on defender's next turn** `est:1h30m`
  Wire the T01 primitives end-to-end. (1) In `src/data/units_ron.rs`, add `pub toughness_category: ToughnessCategory` to `UnitDef` with `#[serde(default)]` so existing units default to Standard; re-export `ToughnessCategory` from `src/combat/toughness.rs` and import it in `units_ron.rs`. Update the in-file `round_trip_unit_def` test fixture to set the field. (2) In `assets/data/units.ron`, set `toughness_category: Armored` on Devimon (id 101) so the integration test in T03 has a real Armored fixture; leave all other units defaulting to Standard. (3) In `src/combat/bootstrap.rs::spawn_unit_from_def`, construct `Toughness::with_category(def.toughness_max, def.weaknesses.clone(), def.toughness_category)` and add `RoundFlags::default()` to the spawn bundle. Import `RoundFlags` from `crate::combat::round_flags`. (4) Extend `ResolveActorsQuery` in `src/combat/turn_system/mod.rs` to include `Option<&'static mut RoundFlags>` as element 12 (mirroring the BasicStreak pattern at element 11). (5) In `src/combat/turn_system/pipeline.rs::step_app`, after the existing `actors.get_many_mut([...])` destructure, read the defender's `RoundFlags.break_sealed` (defaulting to false if absent), pass it as a new parameter to `apply_effects`, and after the call — if `outcome.broke == true` — set the defender's `RoundFlags.break_sealed = true`. Update the destructuring tuples accordingly. (6) In `src/combat/resolution.rs::apply_effects`, add a `defender_break_sealed: bool` parameter (placed after `defender_is_commander`), thread it into the `defender_tough.apply_hit(resolved.damage_tag, resolved.toughness_damage, defender_break_sealed)` call, and replace the T01 placeholder. (7) In `src/combat/turn_system/mod.rs::advance_turn_system`, in the per-`TurnAdvanced` loop (around the stunned/status_opt block), reset the active unit's `RoundFlags.break_sealed = false` at the start of its turn — this is the seal-clear point. Add `Option<&mut RoundFlags>` to the existing turn-system query and clear when present. (8) Sweep all integration tests in `tests/` that construct `Toughness` directly via `Toughness::new(...)` — keep them compiling (the existing `new` signature is preserved by T01); no test changes required unless a compile error surfaces. Document any test fixture that you do touch. Confirm `cargo test` (full suite) is green before T03.
  - Files: `src/data/units_ron.rs`, `assets/data/units.ron`, `src/combat/bootstrap.rs`, `src/combat/turn_system/mod.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/resolution.rs`
  - Verify: cargo test 2>&1 | grep -E 'test result' | tail -5

- [x] **T03: Add tests/toughness_categories.rs integration suite covering Armored/Shielded behavior and Break Seal lifecycle** `est:1h30m`
  Create `tests/toughness_categories.rs`, a headless integration test file that drives the full ECS pipeline. Build a minimal Bevy `App` (mirroring the setup pattern used in `tests/resource_caps.rs`) that registers the resolve_action_system, advance_turn_system, and the necessary resources (CombatState, SpPool, ActionLog, TurnOrder, CombatRng with a fixed seed). Spawn one Ally attacker (Agumon-like, Fire-tagged Basic, hp=100) plus three enemy defenders constructed via `spawn_unit_from_def` with explicit `UnitDef`s: a Standard enemy (toughness_max=20, weaknesses=[Fire]), an Armored enemy (toughness_max=20, weaknesses=[Fire], toughness_category=Armored), and a Shielded enemy (toughness_max=20, weaknesses=[Fire], toughness_category=Shielded). Use a skill that deals exactly `ToughnessHit(20)` plus a small `Damage` effect; either reuse an existing skill from skills.ron or inject a SkillBook fixture inline. Implement four tests: (1) `standard_breaks_in_one_full_hit` — fire one ToughnessHit(20) on Standard, assert OnBreak emitted and `Toughness.broken == true`. (2) `armored_requires_two_full_hits` — fire one ToughnessHit(20) on Armored, assert NO OnBreak and `current` reduced by ~10 (Armored halving); fire a second hit, assert OnBreak emitted. (3) `shielded_never_breaks` — fire three ToughnessHit(20) on Shielded, assert NO OnBreak across all three and `Toughness.broken == false`, `Toughness.current == 20` (clamped). (4) `break_seal_blocks_repeat_break_in_same_round_then_lifts_on_next_turn` — break a Standard enemy, assert OnBreak (1st); restore its toughness in-test by mutating the component back to max and broken=false to simulate a second attempt within the same round, then fire another ToughnessHit(20) and assert NO new OnBreak fired and the seal flag is set. Then synthesize a `TurnAdvanced { unit_id: defender_id, .. }` event for the defender, run `app.update()`, assert `RoundFlags.break_sealed == false`; fire another ToughnessHit(20) and assert OnBreak fires again. Use `MessageReader<CombatEvent>` to count OnBreak occurrences between actions. Keep all setup deterministic — fixed RNG seed, no wall-clock dependence. Ensure the file does NOT read from `.gitignore`d paths (use inline fixtures or `assets/data/skills.ron` which is git-tracked).
  - Files: `tests/toughness_categories.rs`
  - Verify: cargo test --test toughness_categories 2>&1 | tail -20 && cargo test 2>&1 | grep -E 'test result' | tail -5

## Files Likely Touched

- src/combat/toughness.rs
- src/combat/round_flags.rs
- src/combat/mod.rs
- src/combat/resolution.rs
- src/data/units_ron.rs
- assets/data/units.ron
- src/combat/bootstrap.rs
- src/combat/turn_system/mod.rs
- src/combat/turn_system/pipeline.rs
- tests/toughness_categories.rs
