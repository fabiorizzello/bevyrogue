---
sliceId: S02
uatType: artifact-driven
verdict: PASS
date: 2026-05-13T08:54:00.000Z
---

# UAT Result — S02

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| Scenario 1: refresh_max_dur — `status_refresh_max_dur` test | runtime | PASS | `tests/status_refresh_max_dur.rs`: `refresh_max_dur_keeps_longer_and_replaces_with_longer` ok (1/1) |
| Scenario 2: multi-kind coexistence — `status_multi_kind_coexist` test | runtime | PASS | `tests/status_multi_kind_coexist.rs`: `three_different_kinds_coexist_in_bag` ok (1/1) |
| Scenario 3: cleanse policy — `status_cleanse_policy` test | runtime | PASS | `tests/status_cleanse_policy.rs`: `cleanse_removes_all_debuffs_and_keeps_blessed` ok (1/1) |
| Scenario 4: fresh apply accuracy — `status_accuracy` test | runtime | PASS | `tests/status_accuracy.rs`: 3/3 ok (vaccine_vs_vaccine_neutral, vaccine_vs_data_miss, vaccine_vs_data_hit) |
| Scenario 5: combat coherence — `combat_coherence` test | runtime | PASS | `tests/combat_coherence.rs`: 3/3 ok (shared_sp_history, status_pressure_failed_action, break_follow_up_ult_timing) |
| Smoke: `cargo run --bin bevyrogue` headless exit 0 | runtime | PASS | Tick budget (120) reached, exiting cleanly — exit code 0 |
| Grep guard: no `Vec<StatusEffect>` in src/ or tests/ | artifact | PASS | `grep -rn "Vec<StatusEffect>" src/ tests/` → no matches (exit 1) |

## Overall Verdict

PASS — all 7 checks passed: 5 DoD integration tests green, smoke exit 0, grep guard clean.

## Notes

- Tests in `status_refresh_max_dur.rs`, `status_multi_kind_coexist.rs`, `status_cleanse_policy.rs`, `status_accuracy.rs`, and `combat_coherence.rs` all pass with 0 failures and 0 ignored.
- Full `cargo test` suite: 0 failed / 0 ignored across all integration test binaries.
- `cargo run --bin bevyrogue` (headless) exits cleanly after tick budget; no panics.
- Grep guard confirmed: `Vec<StatusEffect>` has been fully replaced by `StatusBag` in all of src/ and tests/.
