---
id: S02
parent: M017
milestone: M017
provides:
  - StatusBag multi-instance storage
  - refresh_max_dur on re-apply
  - BuffKind-classified cleanse
requires:
  []
affects:
  []
key_files:
  - src/combat/status_effect.rs
  - src/combat/turn_system.rs
  - src/combat/resolution.rs
  - tests/status_refresh_max_dur.rs
  - tests/status_multi_kind_coexist.rs
  - tests/status_cleanse_policy.rs
key_decisions:
  - StatusBag stores Vec<StatusEffect> per entity; multi-instance semantics are the default
  - refresh_max_dur: on re-apply of same kind keep max(existing_dur, new_dur)
  - BuffKind enum on StatusEffect drives cleanse classification (Debuff removed, Buff immune)
patterns_established:
  - StatusBag as component replaces flat Vec<StatusEffect> on Unit
  - BuffKind cleanse: iterate StatusBag, drain Debuff entries only
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-13T08:52:33.246Z
blocker_discovered: false
---

# S02: §H.1 StatusBag multi-instance + refresh_max_dur + BuffKind cleanse

**StatusBag multi-instance storage per (target,kind), refresh_max_dur on re-apply, BuffKind-classified cleanse — all DoD tests green, grep guard clean, smoke exit 0**

## What Happened

S02 delivered §H.1 status policy in 6 tasks. T01 introduced StatusBag (multi-instance Vec storage keyed per entity) and BuffKind enum (Buff/Debuff/Neutral). T02 migrated the apply pipeline to StatusBag with refresh_max_dur semantics. T03 migrated tick and expiration logic. T04 migrated follow_up reactions and in-tree tests. T05 verified all 5 DoD integration tests green: status_refresh_max_dur, status_multi_kind_coexist, status_cleanse_policy, status_accuracy, combat_coherence. T06 confirmed smoke exit 0, grep guard clean (no Vec<StatusEffect> remaining in src/ or tests/), and full suite 0 failed / 0 ignored.

## Verification

cargo run exit 0. grep Vec<StatusEffect> src/ tests/ → CLEAN. cargo test → 0 failed / 0 ignored. DoD scenarios: status_refresh_max_dur (re-apply keeps max dur), status_multi_kind_coexist (different kinds stack independently), status_cleanse_policy (Debuff removed, Buff survives cleanse), status_accuracy (fresh apply correct), combat_coherence (migrated, passing).

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

None.
