---
id: T04
parent: S01
milestone: M017
key_files:
  - src/combat/speed.rs
  - src/combat/rng.rs
key_decisions:
  - battery_loop ShockTransfer/MissingPreExistingShock left unchanged — these are battery-loop domain names, not status taxonomy
  - turn_system reserved Burn/Shock match arms preserved per T01 design (reserved no-ops)
  - Only 2 comment-level edits required; no logic changes
duration: 
verification_result: passed
completed_at: 2026-05-12T16:39:58.313Z
blocker_discovered: false
---

# T04: Cascade rename complete: 2 legacy comment refs updated in speed.rs/rng.rs; battery_loop/kernel/observability Shock refs confirmed as battery-loop mechanic (not status taxonomy); turn_system reserved Burn/Shock variants preserved.

**Cascade rename complete: 2 legacy comment refs updated in speed.rs/rng.rs; battery_loop/kernel/observability Shock refs confirmed as battery-loop mechanic (not status taxonomy); turn_system reserved Burn/Shock variants preserved.**

## What Happened

Scanned all 7 target files for legacy tokens (Burn/Freeze/Shock/DeepFreeze). Found: (1) speed.rs line 7 comment: 'Freeze' → 'Chilled'; (2) rng.rs line 7 comment: 'Shock cancel rolls' → 'Paralyzed cancel rolls'. Battery_loop.rs ShockTransfer, kernel.rs MissingPreExistingShock, and observability.rs 'missing-shock' are BatteryLoop-mechanic names unrelated to the status taxonomy — left unchanged. turn_system/mod.rs StatusEffectKind::Burn|Shock are the reserved no-op match arms from T01 — left unchanged. tests.rs was already clean (T03 cleared it). cargo check headless and windowed both green.

## Verification

grep for non-reserved legacy tokens post-edit returned only status_effect.rs Burn/Shock reserved enum variants. cargo check (headless) and cargo check --features windowed both finished with 0 errors.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -rn 'Burn|Freeze|Shock|DeepFreeze' src/combat/ | grep -v reserved/battery/missing` | 0 | only reserved enum variants remain | 50ms |
| 2 | `cargo check` | 0 | pass | 1390ms |
| 3 | `cargo check --features windowed` | 0 | pass | 1680ms |

## Deviations

T04-PLAN listed 8 files / turn_system/tests.rs (11 occurrences) but tests.rs was already fully migrated by T03. Actual edits: 2 files (comment-only).

## Known Issues

None.

## Files Created/Modified

- `src/combat/speed.rs`
- `src/combat/rng.rs`
