---
id: S09
parent: M011
milestone: M011
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["assets/data/units.ron", "assets/data/skills.ron", "src/combat/bootstrap.rs", "src/bin/combat_cli.rs", "tests/scenario_minion_ttk.rs", "tests/scenario_miniboss_ttk.rs", "tests/scenario_boss_ttk.rs", "src/data/skills_ron.rs", "docs/combat_design.md", ".gsd/milestones/M011/slices/S09/S09-UAT.md", ".gsd/milestones/M011/slices/S09/S09-ASSESSMENT.md"]
key_decisions:
  - ["EncounterPreset enum in bootstrap.rs alongside bootstrap_encounter — same file, zero indirection (MEM060)", "Ogremon toughness_max started at 60 (plan typo ~6 would be trivially breakable), converged to 20 in T03 rebalance", "Toughness break requires SAME hit to cross >0→≤0 AND match weakness — shaped all rebalance decisions (MEM058)", "angemon_basic ToughnessHit bumped 8→20 so Angemon single-handedly triggers OnBreak on Ogremon in one hit", "BonusToughnessDamage/BonusDamageVsAttribute stripped as dead Effect variants (D052) — fire-a-separate-skill is the canonical workaround (MEM061)", "UAT verdict left <awaiting human sign-off> — auto-mode cannot sign off on a 30-minute subjective playthrough"]
patterns_established:
  - ["Three-tier TTK scenario test pattern: synchronous RON pre-load + hardcoded ActionIntent script + MessageCursor drain per frame + turn-band assertion (tests/scenario_*_ttk.rs)", "EncounterPreset enum as lightweight preset registry in bootstrap.rs — additive, no engine changes required", "MEM055 dual-surface assertion: event count (MessageCursor) + component state per update tick"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T12:18:29.303Z
blocker_discovered: false
---

# S09: Numerical rebalance pass + UAT scenarios

**Extended the enemy roster (Goblimon/Ogremon), locked R083 TTK targets in three passing deterministic scenario tests, rebalanced numbers iteratively until all three bands are met, stripped dead Effect variants, annotated combat_design.md §9, and authored the UAT script and assessment scaffold for human sign-off.**

## What Happened

S09 is the M011 closure slice — it assembles the full playable encounter system and locks in verifiable balance targets.

**T01 — Enemy roster + EncounterPreset wiring**
Added Goblimon (UnitId 102: HP 120, toughness_max 0, Standard/Virus, no Form Identity) and Ogremon (UnitId 103: HP 280→200 after rebalance, toughness_max 60→20 after rebalance, Standard/Data) to assets/data/units.ron, plus 7 new skills (goblimon_basic/slash/ult, ogremon_basic/club/smash/ult). Introduced the `EncounterPreset` enum (MinionWave / MiniBossEncounter / BossEncounter) in bootstrap.rs, extended `bootstrap_encounter` to accept a preset parameter, added a `SelectedEncounter` resource, and wired an `inquire::Select` preset prompt into combat_cli with a non-interactive default of BossEncounter for CI safety. All call-site tests updated. Key decision: Ogremon toughness_max started at 60 (plan's ~6 was a typo-magnitude risk); T03 later converged it to 20. Catalog invariant tests narrowed tempo_resistant assertion to boss-tagged enemies only.

**T02 — Three deterministic scenario test fixtures**
Created tests/scenario_minion_ttk.rs, tests/scenario_miniboss_ttk.rs, and tests/scenario_boss_ttk.rs. Each binary loads RON synchronously (pre-App pattern from combat_cli.rs), spawns the canonical encounter via bootstrap_encounter with the appropriate preset, drives a hardcoded ActionIntent script, drains MessageCursor<CombatEvent> per frame, and asserts: VICTORY reached, turn count within R083 band, OnBreak ≥1 (boss/miniboss), EnergyGained ≥1 (boss). Tests were intentionally red post-T02 — they defined the rebalance target for T03.

**T03 — Numerical rebalance pass**
Root cause analysis revealed that the toughness break mechanic requires the SAME hit to cross toughness from >0 to ≤0 AND match a weakness tag — non-weakness hits drain the bar but can never trigger OnBreak. This constraint shaped all rebalance decisions:

- Goblimon hp_max: 120 → 40 (minion TTK 3 turns, band 2–3 ✓)
- Ogremon toughness_max: 60 → 20 + weaknesses [Fire, Light]; angemon_basic ToughnessHit: 8 → 20 so Angemon single-handedly crosses from 20 → 0 with a Light hit (was_positive=true, Light matches weakness → OnBreak). Ogremon hp_max: 280 → 200 for 3–5 band.
- Devimon hp_max: 500 → 300; toughness_max: 100 → 35; weaknesses: [Light] → [Fire, Light]. DORUgamon Form Identity ToughnessHit(10) Armored→eff=5 drains to 30; Greymon Fire ToughnessHit crosses to ≤0, Fire IS weakness → OnBreak. TTK 5 turns (band 4–7 ✓).
- Stripped BonusToughnessDamage and BonusDamageVsAttribute Effect enum variants — both were dead code never wired into apply_effects, removed cleanly (D052). fire-a-separate-skill workaround from S08 is the canonical approach for conditional toughness pressure.
- combat_design.md §9 annotated with M011 wiring table: all 6 Adults, trigger types, and D050 fire-a-separate-skill note.

All three TTK scenario tests green. Full cargo test suite: 37 binaries, 0 failures.

**T04 — UAT script + assessment scaffold**
Authored S09-UAT.md as a structured 30-minute playthrough script covering smoke test, Encounter 1 (MinionWave, 5 min), Encounter 2 (MiniBossEncounter, 10 min), Encounter 3 (BossEncounter/Devimon, 15 min), subjective rubric checklist, verdict slot, and tester notes. S09-ASSESSMENT.md records all T03 deviations (D052, angemon ToughnessHit bump, HP/toughness tuning), M012 follow-up triage (Tamer Gauge, DNA Chips, Enemy Counterplay Traits, multi-skill AI, windowed egui refresh, status dashboard, Break Seal visual indicator), and integration binary green count. UAT verdict is left as `<awaiting human sign-off>` — auto-mode cannot sign off on the subjective 30-minute play component.

## Verification

1. `cargo test --test scenario_minion_ttk` → exit 0, minion_wave_ttk_target_2_to_3_turns PASS (3 turns, band 2–3)
2. `cargo test --test scenario_miniboss_ttk` → exit 0, miniboss_encounter_ttk_target_3_to_5_turns PASS (in band, break_count ≥ 1)
3. `cargo test --test scenario_boss_ttk` → exit 0, boss_encounter_ttk_target_4_to_7_turns PASS (5 turns, break_count ≥ 1, energy_count ≥ 1)
4. `cargo test` → exit 0, 37 test binaries, 0 failures
5. S09-UAT.md and S09-ASSESSMENT.md present; UAT contains MinionWave/MiniBossEncounter/BossEncounter section headers
6. R083 updated to validated
7. docs/combat_design.md §9 contains M011 wiring annotation

## Requirements Advanced

None.

## Requirements Validated

- R083 — Three scenario tests green: minion_ttk 3 turns (band 2–3), miniboss_ttk in band with ≥1 OnBreak, boss_ttk 5 turns (band 4–7) with ≥1 OnBreak and ≥1 EnergyGained. Full cargo test 37 binaries, 0 failures.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

UAT verdict is awaiting human sign-off. M011 milestone closure is blocked until the product owner completes the 30-minute manual playthrough.

## Follow-ups

M012 scope confirmed: Tamer Gauge + 3 Commands (Data Scan / Emergency Guard / Retreat), Enemy Counterplay 4 traits (Type Trap / Reactive Armor / Break Seal nemico / Tempo Anchor), Charged Attacks with Danger Window, DNA Chips RON schema. UAT human sign-off pending — milestone closure blocked until product owner completes 30-minute playthrough and records verdict in S09-UAT.md and S09-ASSESSMENT.md.

## Files Created/Modified

None.
