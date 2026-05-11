---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Add tests/toughness_categories.rs integration suite covering Armored/Shielded behavior and Break Seal lifecycle

Create `tests/toughness_categories.rs`, a headless integration test file that drives the full ECS pipeline. Build a minimal Bevy `App` (mirroring the setup pattern used in `tests/resource_caps.rs`) that registers the resolve_action_system, advance_turn_system, and the necessary resources (CombatState, SpPool, ActionLog, TurnOrder, CombatRng with a fixed seed). Spawn one Ally attacker (Agumon-like, Fire-tagged Basic, hp=100) plus three enemy defenders constructed via `spawn_unit_from_def` with explicit `UnitDef`s: a Standard enemy (toughness_max=20, weaknesses=[Fire]), an Armored enemy (toughness_max=20, weaknesses=[Fire], toughness_category=Armored), and a Shielded enemy (toughness_max=20, weaknesses=[Fire], toughness_category=Shielded). Use a skill that deals exactly `ToughnessHit(20)` plus a small `Damage` effect; either reuse an existing skill from skills.ron or inject a SkillBook fixture inline. Implement four tests: (1) `standard_breaks_in_one_full_hit` — fire one ToughnessHit(20) on Standard, assert OnBreak emitted and `Toughness.broken == true`. (2) `armored_requires_two_full_hits` — fire one ToughnessHit(20) on Armored, assert NO OnBreak and `current` reduced by ~10 (Armored halving); fire a second hit, assert OnBreak emitted. (3) `shielded_never_breaks` — fire three ToughnessHit(20) on Shielded, assert NO OnBreak across all three and `Toughness.broken == false`, `Toughness.current == 20` (clamped). (4) `break_seal_blocks_repeat_break_in_same_round_then_lifts_on_next_turn` — break a Standard enemy, assert OnBreak (1st); restore its toughness in-test by mutating the component back to max and broken=false to simulate a second attempt within the same round, then fire another ToughnessHit(20) and assert NO new OnBreak fired and the seal flag is set. Then synthesize a `TurnAdvanced { unit_id: defender_id, .. }` event for the defender, run `app.update()`, assert `RoundFlags.break_sealed == false`; fire another ToughnessHit(20) and assert OnBreak fires again. Use `MessageReader<CombatEvent>` to count OnBreak occurrences between actions. Keep all setup deterministic — fixed RNG seed, no wall-clock dependence. Ensure the file does NOT read from `.gitignore`d paths (use inline fixtures or `assets/data/skills.ron` which is git-tracked).

## Inputs

- `src/combat/toughness.rs`
- `src/combat/round_flags.rs`
- `src/combat/bootstrap.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `src/data/units_ron.rs`
- `assets/data/skills.ron`
- `tests/resource_caps.rs`

## Expected Output

- `tests/toughness_categories.rs`

## Verification

cargo test --test toughness_categories 2>&1 | tail -20 && cargo test 2>&1 | grep -E 'test result' | tail -5

## Observability Impact

This test is the slice's objective stopping condition. It must observe `CombatEventKind::OnBreak` count on the bus (not just `Toughness.broken`) — both are inspection surfaces and must agree. Failure visibility: each assertion includes the OnBreak count and the current toughness value in its panic message so a future regression points directly at which mechanic broke.
