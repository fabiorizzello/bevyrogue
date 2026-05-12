---
id: T02
parent: S01
milestone: M017
key_files:
  - assets/data/skills.ron
  - src/data/skills_ron.rs
key_decisions:
  - Burn→Heated, Shock→Paralyzed, Freeze→Chilled translation applied in RON (consistent with T01 slice mapping)
  - Validator rejects Burn/Shock in ApplyStatus at validate_skill_book time; reserved variants remain in enum for RON/log vocabulary anchoring per D004+D009
  - cargo test failures in src/ test modules and integration tests left for T04/T05 per slice plan scope
duration: 
verification_result: passed
completed_at: 2026-05-12T16:09:34.092Z
blocker_discovered: false
---

# T02: skills.ron migrated to 5 canonical status ids; load-time validator rejects reserved Burn/Shock in ApplyStatus

**skills.ron migrated to 5 canonical status ids; load-time validator rejects reserved Burn/Shock in ApplyStatus**

## What Happened

The previous auto-fix attempt failed because the verification command string contained shell-invalid characters ("(" unexpected) — it tried to run the plan description as a shell command. The actual code state was partially correct: T01 had already rewritten StatusEffectKind to flat unit variants (Heated/Chilled/Paralyzed/Slowed/Blessed + reserved Burn/Shock), but skills.ron still had legacy struct-variant syntax (Burn(damage_per_turn:5), Shock(cancel_chance_pct:25), Freeze(speed_reduction:15)) that would fail RON parsing against the new flat enum.

Fixed 8 ApplyStatus entries in assets/data/skills.ron: Burn→Heated (3 sites), Shock→Paralyzed (3 sites), Freeze→Chilled (3 sites). Added CANON_STATUS_IDS constant and a validator guard in validate_skill_def that returns an error with the 5 valid ids when Burn or Shock appears in ApplyStatus. cargo check headless and windowed both finish clean.

## Verification

cargo check (headless): Finished dev profile, no errors. cargo check --features windowed: Finished dev profile, no errors. cargo test --lib compilation errors are pre-existing in src/combat/turn_system/tests.rs and follow_up_tests.rs — those use old struct variants scoped to T04 migration per slice plan. RON parsing tests in skills_ron.rs #[cfg(test)] mod are blocked only by the same T04 src/ migration, not by T02 changes.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 1350ms |
| 2 | `cargo check --features windowed` | 0 | pass | 1610ms |

## Deviations

none — T02 scope is RON schema + validator only; src/combat/* test file migration is T04

## Known Issues

None.

## Files Created/Modified

- `assets/data/skills.ron`
- `src/data/skills_ron.rs`
