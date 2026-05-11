---
id: T04
parent: S02
milestone: M011
key_files:
  - src/combat/damage.rs
  - src/combat/events.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/damage_tests.rs
  - src/combat/ultimate.rs
  - tests/damage_breakdown_log.rs
  - tests/commander_flow.rs
  - tests/ultimate_meter.rs
  - tests/follow_up_triggers.rs
  - tests/combat_coherence.rs
  - tests/follow_up_reentrancy.rs
key_decisions:
  - calculate_damage returns DamageBreakdown struct instead of i32 — allows callers to propagate tag_mod_pct and triangle_mod_pct without re-computing them; the struct is defined inline in damage.rs alongside the function
  - Existing exhaustive matchers in test helpers (format strings for OnDamageDealt) use `..` rather than adding the new fields, keeping test output format stable while compiling cleanly
  - Neutral fixtures in test helpers (commander_flow, ultimate_meter) use explicit (100, 100) values rather than `..` to keep struct construction exhaustive and catch future field additions at compile time
duration: 
verification_result: passed
completed_at: 2026-04-27T14:31:23.122Z
blocker_discovered: false
---

# T04: feat(combat): expose tag_mod_pct + triangle_mod_pct on OnDamageDealt and add Greymon-vs-Devimon JSONL breakdown scenario test

**feat(combat): expose tag_mod_pct + triangle_mod_pct on OnDamageDealt and add Greymon-vs-Devimon JSONL breakdown scenario test**

## What Happened

Extended `CombatEventKind::OnDamageDealt` with two new integer-percentage fields: `tag_mod_pct` (125/75/100 for weak/resist/neutral) and `triangle_mod_pct` (111/87/100 for attacker-wins/loses/neutral).

`calculate_damage` in `src/combat/damage.rs` was refactored to return `DamageBreakdown { final_damage, tag_mod_pct, triangle_mod_pct }` instead of a bare `i32`. The call site in `src/combat/resolution.rs` destructures the breakdown and threads the two pct fields into the `OnDamageDealt` event construction.

All exhaustive match patterns on `OnDamageDealt` across the codebase were updated:
- `src/combat/turn_system/pipeline.rs:153` — added `..` to the existing destructure
- `src/combat/ultimate.rs:228` — added explicit neutral values (100, 100) to the `damage_event` fixture
- `src/combat/damage_tests.rs` — all 18 matrix tests updated to assert `.final_damage` on the returned `DamageBreakdown`; multi-line assertion fixed manually after sed regex missed it
- `tests/commander_flow.rs` (×2), `tests/ultimate_meter.rs` (×2) — explicit neutral values (100, 100) added
- `tests/follow_up_triggers.rs`, `tests/combat_coherence.rs`, `tests/follow_up_reentrancy.rs` — added `..` to destructures that only need `amount` and `kind`

Created `tests/damage_breakdown_log.rs` with two deterministic scenarios:
- **Scenario A**: Greymon (Vaccine) attacks Devimon (Virus, resists Fire) → `tag_mod_pct=75`, `triangle_mod_pct=111`, `amount=83` — the canonical S02 roadmap case
- **Scenario B**: Greymon (Vaccine) attacks Devimon (Virus, weak Fire) → `tag_mod_pct=125`, `triangle_mod_pct=111`, `amount=139` — covers the weak-tag path

Note on task plan math: the plan description labeled "Virus attacker losing → 111" but the v5.3 triangle model gives `dmg_modifier=0.87` when attacker loses (Virus vs Vaccine). The correct scenario for `triangle_mod_pct=111` is Vaccine attacker vs Virus defender (attacker wins). The test fixtures reflect the mathematically correct values matching the existing `damage_tests.rs` matrix.

## Verification

cargo test --test damage_breakdown_log: 2/2 pass. Full suite: 28 test binaries, 0 failures across all prior tests including the 18-test damage matrix, status_accuracy (3 tests), pipeline_dispatch, follow_up_triggers, combat_coherence, ultimate_meter, commander_flow, and all others.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test damage_breakdown_log --no-fail-fast` | 0 | ✅ pass | 2360ms |
| 2 | `cargo test --no-fail-fast 2>&1 | grep -E 'FAILED|^test result'` | 0 | ✅ pass — 0 failures across all 28 test binaries | 3100ms |

## Deviations

Task plan expected Devimon (Virus) as attacker with `triangle_mod_pct=111`. Per the v5.3 model, Virus attacking Vaccine gives `dmg_modifier=0.87` (attacker loses), not 1.11. The plan had an inversion: the correct setup for `triangle_mod_pct=111` is Vaccine attacker vs Virus defender (attacker wins). Scenario A was implemented with Greymon (Vaccine) attacking Devimon (Virus), which produces the amount=83, tag_mod_pct=75, triangle_mod_pct=111 values the slice roadmap requires.

## Known Issues

none

## Files Created/Modified

- `src/combat/damage.rs`
- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/damage_tests.rs`
- `src/combat/ultimate.rs`
- `tests/damage_breakdown_log.rs`
- `tests/commander_flow.rs`
- `tests/ultimate_meter.rs`
- `tests/follow_up_triggers.rs`
- `tests/combat_coherence.rs`
- `tests/follow_up_reentrancy.rs`
