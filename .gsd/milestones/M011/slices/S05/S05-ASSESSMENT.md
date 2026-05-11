---
sliceId: S05
uatType: artifact-driven
verdict: PASS
date: 2026-04-28T09:46:30.000Z
---

# UAT Result — S05

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| TC1: `child_discount_after_two_basics` integration test | runtime | PASS | `cargo test --test resource_caps -- child_discount_after_two_basics --nocapture` → 1 passed, 0 failed. Child streak reaches 2, SP drops from 5 to 3 (effective cost 2), then full cost 3 fires next Skill (SP→0). Streak resets to 0 after discount. |
| TC2: `sp_non_basic_cap_enforced` integration test | runtime | PASS | `cargo test --test resource_caps -- sp_non_basic_cap_enforced --nocapture` → 1 passed, 0 failed. +1 × 3 yields only +2 (cap), third gain blocked. reset() restores full budget. Pool = 4 after second batch. |
| TC3: Adult unit no discount — `adult_gets_no_discount_after_consecutive_basics` | runtime | PASS | `cargo test -- adult_gets_no_discount_after_consecutive_basics` → ok. Adult with 5 consecutive Basics pays full sp_cost with no -1 reduction. Note: UAT named this `test_adult_no_discount_after_basics`; actual name differs but intent is identical. |
| TC3 (supplemental): `adult_5_consecutive_basics_no_discount` | runtime | PASS | Additional adult negative-case test; 5 basics, no discount on Skill. |
| TC4: Child 1 Basic insufficient — `child_1_basic_not_enough_for_discount` | runtime | PASS | `cargo test -- child_1_basic_not_enough_for_discount` → ok. Child with streak=1 does not qualify for -1 discount; full cost applied. Note: UAT named this `test_child_one_basic_no_discount`. |
| TC5: Energy secondary cap (10) — `secondary_cap_at_10` | runtime | PASS | `cargo test -- secondary_cap_at_10` → ok. try_gain(Secondary, 15) yields 10; second call yields 0. Matches UAT contract. Note: UAT named this `test_secondary_cap`. |
| TC5: Energy external cap (30) — `external_cap_at_30` | runtime | PASS | `cargo test -- external_cap_at_30` → ok. try_gain(External, 50) yields 30; second call yields 0. |
| TC5: Energy caps independent — `caps_are_independent` | runtime | PASS | `cargo test -- caps_are_independent` → ok. Secondary and External budgets do not interfere. Note: UAT named this `test_caps_independent`. |
| TC5: Energy reset restores budget — `reset_restores_full_budget` | runtime | PASS | `cargo test -- reset_restores_full_budget` → ok. After exhausting both budgets, reset() zeros both trackers. Note: UAT named this `test_reset_restores_budget`. |
| Artifact: SpPool::default().max == 5 | artifact | PASS | `grep 'max: 5' src/combat/sp.rs` → line 53 confirms `max: 5`. Inline unit test `sp_pool_default_max_is_5` also passes. |
| Artifact: EvoStage field on Unit component | artifact | PASS | `grep 'evo_stage' src/combat/unit.rs` → line 14 confirms `pub evo_stage: EvoStage`. |
| Artifact: BasicStreak component defined in unit.rs | artifact | PASS | `grep 'BasicStreak' src/combat/unit.rs` → struct defined at line 18. |
| Artifact: Energy + RoundEnergyTracker spawned on all units | artifact | PASS | `grep 'Energy\|RoundEnergyTracker' src/combat/bootstrap.rs` → lines 131-132 confirm both spawned in `spawn_unit_from_def`. |

## Overall Verdict

PASS — all 13 automatable artifact-driven checks passed; both integration tests and all unit tests (energy caps, discount logic, negative cases) confirm the S05 contract is fully implemented and correct.

## Notes

**Test name discrepancies:** The UAT document referenced test names with a `test_` prefix (e.g., `test_secondary_cap`, `test_adult_no_discount_after_basics`) that do not match actual test function names (`secondary_cap_at_10`, `adult_gets_no_discount_after_consecutive_basics`). This is a documentation drift issue only — the functional coverage is complete and equivalent. No remediation required.

**Test location:** All tests run from the M011 worktree (`/home/fabio/dev/bevyrogue/.gsd/worktrees/M011`), not the main project directory. The `resource_caps.rs` test file and additional integration tests (tempo_resistance.rs, etc.) exist only in the worktree branch.

**Evidence commands run:**
- `cargo test --test resource_caps -- child_discount_after_two_basics --nocapture` → exit 0
- `cargo test --test resource_caps -- sp_non_basic_cap_enforced --nocapture` → exit 0
- `cargo test -- secondary_cap_at_10 external_cap_at_30 caps_are_independent reset_restores_full_budget adult_gets_no_discount_after_consecutive_basics child_1_basic_not_enough_for_discount adult_5_consecutive_basics_no_discount child_gets_minus1_sp_after_2_consecutive_basics --nocapture` → 9 passed, 0 failed
