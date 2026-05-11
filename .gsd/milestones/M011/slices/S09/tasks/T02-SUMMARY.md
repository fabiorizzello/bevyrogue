---
id: T02
parent: S09
milestone: M011
key_files:
  - tests/scenario_minion_ttk.rs
  - tests/scenario_miniboss_ttk.rs
  - tests/scenario_boss_ttk.rs
key_decisions:
  - Drain CombatEvent messages after each app.update() (not batched) because Messages<T> clears per frame
  - Boss EnergyGained assertion passes (FI works) — only turn_count and break_count need T03 rebalance
  - Ogremon break also fails with current numbers — both boss and miniboss need toughness rebalance in T03
duration: 
verification_result: mixed
completed_at: 2026-04-28T11:42:54.803Z
blocker_discovered: false
---

# T02: Created three deterministic TTK scenario fixtures (minion/miniboss/boss) that lock R083 rebalance targets for T03

**Created three deterministic TTK scenario fixtures (minion/miniboss/boss) that lock R083 rebalance targets for T03**

## What Happened

Created three integration test binaries that load real RON data, spawn encounters via bootstrap_encounter, and run scripted ally action sequences:

- tests/scenario_minion_ttk.rs: MinionWave (3× Goblimon), Victory assert passes, TTK assert fails (6 turns vs target 2–3)
- tests/scenario_miniboss_ttk.rs: MiniBossEncounter (Ogremon + 2× Goblimon), Victory passes, break_count assert fails (Ogremon never breaks with current numbers)
- tests/scenario_boss_ttk.rs: BossEncounter (Devimon solo, Armored), Victory passes, energy_count passes (Form Identity grants energy correctly), break_count fails (Devimon Armored + wrong final hit tag prevents break)

Key bug discovered and fixed: Messages<CombatEvent> are cleared after each app.update() frame. The original code drained events after 4 app.update() calls — all events were gone by then. Fix: drain events inside the update loop, once per update call.

All existing 245+ tests continue to pass. The three new tests fail at their labeled R083 assertions, correctly defining the rebalance targets for T03.

## Verification

Ran each test individually and confirmed: (1) no panic at Victory assertion, (2) panic at expected R083 assertion with descriptive message. Also ran full cargo test to confirm zero regressions in existing suite.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test scenario_minion_ttk -- --nocapture` | 101 | R083: MinionWave TTK out of target range — expected 2–3 turns, actual 6 | 500ms |
| 2 | `cargo test --test scenario_miniboss_ttk -- --nocapture` | 101 | R083: MiniBossEncounter expected at least 1 OnBreak (Ogremon bar), got 0 | 500ms |
| 3 | `cargo test --test scenario_boss_ttk -- --nocapture` | 101 | R083: BossEncounter expected at least 1 OnBreak (Devimon bar), got 0 (energy_count >= 1 passed) | 500ms |

## Deviations

energy_count >= 1 in boss test now PASSES (Form Identity correctly grants energy). Original spec listed it as 'expected to FAIL' but that was written before S08 shipped FI. The assertion correctly validates FI works — the test-first intent is preserved for break_count and turn_count.

## Known Issues

None.

## Files Created/Modified

- `tests/scenario_minion_ttk.rs`
- `tests/scenario_miniboss_ttk.rs`
- `tests/scenario_boss_ttk.rs`
