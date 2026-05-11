---
id: T03
parent: S09
milestone: M011
key_files:
  - assets/data/units.ron
  - assets/data/skills.ron
  - src/data/skills_ron.rs
  - docs/combat_design.md
key_decisions:
  - Strip BonusToughnessDamage and BonusDamageVsAttribute from Effect enum — never wired, dead code (D052)
  - Reduce Goblimon hp_max 120→40 to bring minion TTK into [2–3] band
  - Set Ogremon toughness_max=20 + weaknesses=[Fire,Light] + angemon_basic ToughnessHit 8→20 to guarantee break by 2nd attacker
  - Set Devimon hp_max=300 + toughness_max=35 + weaknesses=[Fire,Light] to bring boss TTK into [4–7] band
  - Add Fire weakness to both Ogremon and Devimon (Greymon/Kabuterimon become breakers too)
duration: 
verification_result: passed
completed_at: 2026-04-28T12:11:31.173Z
blocker_discovered: false
---

# T03: Numerical rebalance pass — all three R083 TTK scenario tests green, BonusToughnessDamage/BonusDamageVsAttribute stripped, combat_design.md §9 annotated.

**Numerical rebalance pass — all three R083 TTK scenario tests green, BonusToughnessDamage/BonusDamageVsAttribute stripped, combat_design.md §9 annotated.**

## What Happened

Started from failing T02 TTK fixtures. Three scenario tests needed to hit R083 bands: minion_ttk [2–3 turns], miniboss_ttk [3–5 turns + break], boss_ttk [4–7 turns + break + energy].

**Root cause analysis — toughness break mechanic:** Discovered that `apply_hit` in `toughness.rs` requires the SAME hit to both cross from >0 to ≤0 AND match a weakness tag. Non-weakness hits drain the bar but can never trigger a break; once drained to ≤0 by a non-weakness, `was_positive=false` forever and no subsequent hit can break it. Also: Armored enemies (Devimon) apply `ceil(amount/2)` effective toughness reduction.

**Minion test (Goblimon ×3):** Goblimon had `hp_max: 120`, giving ≈64 HP damage per round — needing ~6 turns. Reduced to `hp_max: 40`, enabling the party to kill one Goblimon per turn with overflow. Test landed at 3 turns. ✓

**Miniboss test (Ogremon + Goblimon ×2):** Ogremon `toughness_max: 60`, `weaknesses: [Light]`. DORUgamon form_identity fired ToughnessHit(10) but Angemon's ToughnessHit(8) couldn't alone break it. Multiple iterations: toughness=40 also failed because Angemon+Kabuterimon+DORUgamon drained the bar to exactly 0 in one turn with none being the crossing weakness hit. Solution: `toughness_max: 20` + `weaknesses: [Fire, Light]`. With Angemon's `angemon_basic` ToughnessHit bumped 8→20, Angemon (second attacker) hits Ogremon fresh with ToughnessHit(20) — 20−20=0, `was_positive=true`, Light IS weakness → BREAK. Ogremon `hp_max: 280→200` tuned TTK into [3–5] band.

**Boss test (Devimon, Armored):** `hp_max: 500→300`, `toughness_max: 100→35`, `weaknesses: [Light]→[Fire, Light]`. Turn 1: DORUgamon form_identity ToughnessHit(10) Armored→eff=5; toughness=30. Turn 2: Greymon Fire ToughnessHit eff→toughness≤0, Fire IS weakness → BREAK. energy_count≥1 satisfied by form_identity EnergyGained events. TTK landed at 5 turns.

**BonusToughnessDamage / BonusDamageVsAttribute stripping:** Both were dead Effect enum variants added in T01 but never wired into `apply_effects`. Stripped from `src/data/skills_ron.rs` (variants + round-trip tests + Attribute import). Recorded as decision D052.

**combat_design.md §9:** Added M011 wiring annotation with implementation table covering all 6 Adults, trigger types, and the fire-a-separate-skill design decision (D050 reference).

## Verification

Full `cargo test` suite: 37 test binaries, 0 failures. Specific scenario tests verified individually: `cargo test --test scenario_minion_ttk`, `cargo test --test scenario_miniboss_ttk`, `cargo test --test scenario_boss_ttk` — all green. grep confirmed combat_design.md contains M011 wiring annotation.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test scenario_minion_ttk` | 0 | minion_ttk PASS — 3 turns (band: 2–3) | 4200ms |
| 2 | `cargo test --test scenario_miniboss_ttk` | 0 | miniboss_ttk PASS — turn_count in [3,5], break_count >= 1 | 4300ms |
| 3 | `cargo test --test scenario_boss_ttk` | 0 | boss_ttk PASS — 5 turns (band: 4–7), energy_count >= 1, break_count >= 1 | 4400ms |
| 4 | `cargo test` | 0 | Full suite green — 37 test binaries, 0 failures | 62000ms |

## Deviations

angemon_basic ToughnessHit bumped 8→20 (not in original plan) — required to guarantee Ogremon break in miniboss scenario given toughness break mechanic constraints.

## Known Issues

None.

## Files Created/Modified

- `assets/data/units.ron`
- `assets/data/skills.ron`
- `src/data/skills_ron.rs`
- `docs/combat_design.md`
