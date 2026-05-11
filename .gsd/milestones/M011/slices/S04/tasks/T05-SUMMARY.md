---
id: T05
parent: S04
milestone: M011
key_files:
  - src/bin/combat_cli.rs
key_decisions:
  - Loaded units.ron synchronously via std::fs::read_to_string + ron::from_str before App::run() so the selection prompt has roster data without requiring the Bevy asset system.
  - Removed PartyConfigHandle dependency from bootstrap_system — CLI harness now fully drives party selection through SelectedAllies resource, making party.ron irrelevant for CLI runs.
duration: 
verification_result: passed
completed_at: 2026-04-27T19:00:11.665Z
blocker_discovered: false
---

# T05: Added interactive MultiSelect roster selection at startup so players pick exactly 4 allies before combat begins, with non-interactive fallback using the first 4 allies.

**Added interactive MultiSelect roster selection at startup so players pick exactly 4 allies before combat begins, with non-interactive fallback using the first 4 allies.**

## What Happened

Replaced the `Text::new("Press Enter to start...")` prompt with a full roster selection phase. `load_ally_roster()` reads `assets/data/units.ron` synchronously via `ron::from_str` before `App::run()`, filters to ally-team units, and presents them via `inquire::MultiSelect`. The loop enforces exactly 4 selections, re-prompting on wrong count and falling back to first 4 on cancellation. Selected `UnitId`s are injected as a `SelectedAllies` resource. `bootstrap_system` was updated to consume `SelectedAllies` directly instead of `PartyConfigHandle`/`PartyConfig`, eliminating the party.ron dependency for the CLI harness. The `tamer_id` in the `PartySelected` event was set to `UnitId(0)` (no tamer in CLI mode). Non-interactive CI mode automatically selects the first 4 allies from the roster.

## Verification

Ran `echo "" | cargo run --bin combat_cli` (non-interactive). Output confirmed: roster loaded, first 4 ally IDs selected, bootstrap successful with those IDs, combat dashboard rendered. `cargo check --bin combat_cli` passed with no errors or warnings.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --bin combat_cli` | 0 | ✅ pass | 160ms |
| 2 | `echo "" | timeout 10 cargo run --bin combat_cli 2>&1 | head -10` | 0 | ✅ pass — non-interactive selects first 4 allies and bootstraps correctly | 5000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/bin/combat_cli.rs`
