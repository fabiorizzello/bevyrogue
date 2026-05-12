---
id: T05
parent: S01
milestone: M017
key_files:
  - tests/status_effect_apply.rs
  - tests/status_accuracy.rs
  - tests/form_identity.rs
  - tests/follow_up_chains.rs
  - tests/combat_coherence.rs
key_decisions:
  - Test fixture skill/unit names containing old taxonomy words renamed for grep-clean compliance: shock_bolt→para_bolt, Shock Bolt→Para Bolt, Shock Tester→Para Tester, Shock Strike→Para Strike
  - Baby Burner comment updated to 'Agumon Ult' — skill name in skills.ron is out of scope (not src/ or tests/)
  - 0 #[ignore] needed — no per-status semantic tests found in remaining files; all status tests only assert lifecycle (applied/resisted events), not behavior
duration: 
verification_result: passed
completed_at: 2026-05-12T16:46:33.608Z
blocker_discovered: false
---

# T05: Cascade rename tests/*: error message strings + comments updated from Burn/Freeze/Shock to Heated/Chilled/Paralyzed; test fixture names sanitized; full suite green, 0 ignored.

**Cascade rename tests/*: error message strings + comments updated from Burn/Freeze/Shock to Heated/Chilled/Paralyzed; test fixture names sanitized; full suite green, 0 ignored.**

## What Happened

T05 targeted 7 test files for legacy status taxonomy references. Enum variant call sites (StatusEffectKind::Burn/Freeze/Shock) were already migrated in T03. Remaining in tests/ were: error message strings in status_effect_apply.rs ("OnStatusApplied(Burn)") and status_accuracy.rs ("OnStatusResisted(Shock)", "OnStatusApplied(Shock)"); comments in form_identity.rs ("applies Freeze", "after Freeze application"); a comment in follow_up_chains.rs ("Baby Burner ToughnessHit"); and test fixture names in combat_coherence.rs ("Shock Bolt"/"shock_bolt" skill, "Shock Tester" unit) and status_accuracy.rs ("Shock Strike" skill name). All updated: Burn→Heated, Freeze/Chilled, Shock→Paralyzed. Fixture IDs: shock_bolt→para_bolt, Shock Bolt→Para Bolt, Shock Tester→Para Tester, Shock Strike→Para Strike. Baby Burner comment updated to 'Agumon Ult'. Post-edit grep on tests/ finds zero Burn/Freeze/Shock/DeepFreeze matches. cargo check and full cargo test --no-fail-fast green.

## Verification

grep -rn 'Burn|Freeze|Shock|DeepFreeze' tests/ → 0 matches. cargo check → Finished dev profile. cargo test --no-fail-fast → all test targets ok, 0 failed, 0 ignored.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -rn 'Burn\|Freeze\|Shock\|DeepFreeze' /home/fabio/dev/bevyrogue/tests/` | 1 | pass — zero matches | 50ms |
| 2 | `cargo check` | 0 | pass | 380ms |
| 3 | `cargo test --no-fail-fast` | 0 | pass — all suites ok, 0 failed, 0 ignored | 2400ms |

## Deviations

none

## Known Issues

None.

## Files Created/Modified

- `tests/status_effect_apply.rs`
- `tests/status_accuracy.rs`
- `tests/form_identity.rs`
- `tests/follow_up_chains.rs`
- `tests/combat_coherence.rs`
