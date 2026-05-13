---
id: T01
parent: S02
milestone: M017
key_files:
  - src/combat/status_effect.rs
  - src/combat/mod.rs
key_decisions:
  - Added deprecated StatusEffect shim (still Component) to keep lib compilable — T02 removes it when migrating call sites in follow_up.rs, turn_system/mod.rs, turn_system/pipeline.rs, turn_system/tests.rs
  - apply doc-comment locks pipeline contract: roll_pct gate runs before apply, resisted re-apply never calls apply
duration: 
verification_result: passed
completed_at: 2026-05-13T06:55:29.153Z
blocker_discovered: false
---

# T01: Added StatusBag + BuffKind + classify_buff_kind with refresh_max_dur/cleanse policy; deprecated StatusEffect shim keeps lib compilable for T02 migration

**Added StatusBag + BuffKind + classify_buff_kind with refresh_max_dur/cleanse policy; deprecated StatusEffect shim keeps lib compilable for T02 migration**

## What Happened

Replaced the single-instance StatusEffect Component model with the multi-instance StatusBag architecture defined in §H.1. Added StatusInstance (non-Component, Serialize/Deserialize/Clone/PartialEq/Debug) as the element type; added BuffKind {Buff, Debuff} enum (Copy/Eq) and free fn classify_buff_kind returning Buff for Blessed and Debuff for the 6 remaining variants. StatusBag(Vec<StatusInstance>) derives Component/Default/Debug/Clone and exposes: apply (refresh_max_dur upsert), tick_all (decrement all, return+remove expired), cleanse_debuffs (drain Debuff instances, Blessed survives), has, get_dur, is_empty, iter. The apply doc-comment locks the pipeline contract: the roll_pct gate at pipeline.rs:725-729 runs before apply, so resisted re-apply emits OnStatusResisted without touching duration. Rewrote the inline test suite (7 RON round-trip tests removed, 8 policy tests added). Added a deprecated StatusEffect shim (same struct, still Component) in status_effect.rs and kept it in mod.rs re-exports alongside StatusBag so the rest of the lib compiles during T02-T04 migration. mod.rs line 61 updated to pub use status_effect::{StatusBag, StatusEffect, StatusEffectKind}.

## Verification

Ran `cargo test --lib combat::status_effect`. All 8 new tests passed: apply_refresh_max_dur_keeps_longer, apply_refresh_max_dur_replaces_with_longer, multi_kind_coexistence, classify_buff_kind_totality (all 7 variants), cleanse_debuffs_removes_debuffs_leaves_blessed, tick_all_returns_expired_and_removes_them, tick_all_multi_expire, is_empty_reflects_state. Lib compiles cleanly (warnings only, no errors).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib combat::status_effect` | 0 | 8/8 pass | 4280ms |

## Deviations

Added deprecated StatusEffect compat shim instead of hard-removing the struct. Task plan implied tree would not compile until T02-T04, but cargo test --lib requires a clean compile. Shim preserves Component behaviour for existing callers; marked deprecated with migration hint. T02 must remove it.

## Known Issues

57 deprecation/unused-import warnings across turn_system and follow_up files — expected; T02 migrates those call sites.

## Files Created/Modified

- `src/combat/status_effect.rs`
- `src/combat/mod.rs`
