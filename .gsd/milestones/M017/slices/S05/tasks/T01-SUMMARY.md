---
id: T01
parent: S05
milestone: M017
key_files:
  - tests/status_blessed_cleanse_immune.rs
key_decisions:
  - Two test cases: isolated Blessed and Blessed-plus-debuffs — covers both the trivial and mixed-bag scenarios
duration: 
verification_result: passed
completed_at: 2026-05-13T09:44:24.314Z
blocker_discovered: false
---

# T01: Added tests/status_blessed_cleanse_immune.rs with two regression tests confirming Blessed survives cleanse_debuffs()

**Added tests/status_blessed_cleanse_immune.rs with two regression tests confirming Blessed survives cleanse_debuffs()**

## What Happened

Read status_effect.rs to confirm cleanse_debuffs() retains BuffKind::Buff entries (Blessed). Checked for pre-existing file — none. Wrote tests/status_blessed_cleanse_immune.rs with two focused tests: (1) Blessed-only bag → nothing removed, duration unchanged; (2) Blessed + debuffs → only debuffs removed, Blessed survives with correct duration. No src/ changes required.

## Verification

cargo test --test status_blessed_cleanse_immune — 2 tests passed, 0 failed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test status_blessed_cleanse_immune` | 0 | pass | 440ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/status_blessed_cleanse_immune.rs`
