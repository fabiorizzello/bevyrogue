---
id: T03
parent: S01
milestone: M018
key_files:
  - tests/tempo_resistance.rs
key_decisions:
  - Floor invariant in tempo_resistance tests updated from -MIN_ACTION_THRESHOLD_AV (-15000) to 0 to match T01 apply_delay semantics
  - Advance ceiling in tests updated from MAX_AV to 2*MAX_AV to match T01 apply_advance semantics
  - No changes needed to pipeline.rs, skills.ron, combat_panel.rs, status_slowed_delay.rs — T02 had already migrated all those
duration: 
verification_result: passed
completed_at: 2026-05-13T15:46:05.963Z
blocker_discovered: false
---

# T03: Migrated tempo_resistance tests from old apply_av_change shims to apply_advance/apply_delay with floor-0 invariant; all other pipeline/skills.ron/UI/slowed targets already migrated by T02

**Migrated tempo_resistance tests from old apply_av_change shims to apply_advance/apply_delay with floor-0 invariant; all other pipeline/skills.ron/UI/slowed targets already migrated by T02**

## What Happened

T02 had already fully migrated: pipeline.rs (lines 359/368/675/684/776) uses AdvanceTurn/DelayTurn, skills.ron has AdvanceTurn(20), combat_panel.rs has LogEntry::AdvanceTurn/DelayTurn at lines 649/652, and status_slowed_delay.rs uses DelayTurn correctly. The only remaining work was tempo_resistance.rs pure-logic section: 7 tests still imported MIN_ACTION_THRESHOLD_AV/apply_av_change/compute_av_change (old signed shims). Updated imports to use apply_advance/apply_delay only. Replaced tests: three_consecutive_delays_show_diminishing_returns now uses apply_delay; compute_av_change_advance_bypasses_resistance → advance_bypasses_resistance_stack using apply_advance; compute_av_change_no_resistance_component → delay_without_resistance_full_strength; apply_av_change_records_hit_and_updates_av → apply_delay_records_hit_and_updates_av; apply_av_change_clamps_to_min_action_threshold → delay_clamps_to_floor_zero (asserts AV=0 not -15000); apply_av_change_clamps_without_resistance_too → delay_clamps_to_floor_zero_without_resistance; apply_av_change_advance_does_not_exceed_max_av → advance_does_not_exceed_2x_max_av (ceiling now 2*MAX_AV). Count unchanged: 14 tests, all green.

## Verification

cargo check && cargo check --features windowed both clean. cargo test --test status_slowed_delay (1/1 pass). cargo test --test tempo_resistance (14/14 pass).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 130ms |
| 2 | `cargo check --features windowed` | 0 | pass | 4250ms |
| 3 | `cargo test --test status_slowed_delay` | 0 | pass — 1/1 | 130ms |
| 4 | `cargo test --test tempo_resistance` | 0 | pass — 14/14 | 10ms |

## Deviations

Pipeline.rs, skills.ron, combat_panel.rs, and status_slowed_delay.rs were already fully migrated by T02; task plan described them as pending. Only tempo_resistance.rs pure-logic tests required changes.

## Known Issues

None.

## Files Created/Modified

- `tests/tempo_resistance.rs`
