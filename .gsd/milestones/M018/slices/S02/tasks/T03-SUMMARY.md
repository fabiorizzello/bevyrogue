---
id: T03
parent: S02
milestone: M018
key_files:
  - src/data/skills_ron.rs
  - src/combat/resolution.rs
  - src/combat/action_query.rs
key_decisions:
  - Allowlist is Single|Blast|AllEnemies at all three sites — kept identical to satisfy greppable invariant from task plan
  - Updated stale test assertion (contains 'TargetShape::Single') to contain 'Row' — Row is the shape under test and still correctly rejected; the old string was testing the error message, not the behavior
duration: 
verification_result: passed
completed_at: 2026-05-13T16:21:30.293Z
blocker_discovered: false
---

# T03: Widened all three validation gates (skills_ron, resolution, action_query) to accept Blast and AllEnemies; Row/SelfOnly remain deferred

**Widened all three validation gates (skills_ron, resolution, action_query) to accept Blast and AllEnemies; Row/SelfOnly remain deferred**

## What Happened

Three sites gated non-Single shapes behind UnimplementedTargetShape. Each widened with a consistent matches!() allowlist (Single | Blast | AllEnemies):

1. src/data/skills_ron.rs:279 — validate_skill_def allowlist changed from `shape != Single` to `!matches!(shape, Single | Blast | AllEnemies)`. Error message updated to list all allowed shapes.
2. src/combat/resolution.rs:242 — target_shape_is_executable_now() extended to include Blast and AllEnemies.
3. src/combat/action_query.rs:485 — target_status_for_unit() deferral gate extended with same allowlist.

One existing test (validate_rejects_implemented_non_single_shape) used Row (still correctly rejected) but asserted the old error message text "TargetShape::Single". Updated assertion to match the new message which contains "Row" — behavior invariant preserved, message text modernized.

## Verification

cargo test: all 162+ unit tests green across all crates. Greppable invariant confirmed: rg lists Blast|AllEnemies at all three gate sites. cargo check not run separately (full test suite compile-verifies headless path).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test 2>&1 | grep -E '^test result'` | 0 | pass — all suites green, no failures | 800ms |
| 2 | `rg -n 'TargetShape::(Blast|AllEnemies)' src/data/skills_ron.rs src/combat/resolution.rs src/combat/action_query.rs` | 0 | pass — all three gate sites listed | 50ms |

## Deviations

One test assertion text updated: validate_rejects_implemented_non_single_shape previously asserted err.detail.contains(\"TargetShape::Single\") — that string was the old error message fragment, not a semantic check. Updated to contains(\"Row\") to match the shape actually under test.

## Known Issues

none

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`
- `src/combat/action_query.rs`
