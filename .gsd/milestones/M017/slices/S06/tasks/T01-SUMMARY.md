---
id: T01
parent: S06
milestone: M017
key_files:
  - src/combat/observability.rs
  - tests/validation_snapshot.rs
key_decisions:
  - Sort statuses by explicit canonical ordinal match rather than repr(u8) cast — enum lacks #[repr(u8)], manual match is safer and covers reserved variants
  - Additive formatter token: existing exact-match tests updated with statuses=[] rather than converting to contains-only checks, keeping full contract coverage
duration: 
verification_result: passed
completed_at: 2026-05-13T10:48:12.250Z
blocker_discovered: false
---

# T01: Added ValidationStatusSnapshot type + ValidationUnitSnapshot.statuses field; formatter prints statuses=[Kind(dur),...] token per unit; new deterministic fixture test covers all 5 canon kinds.

**Added ValidationStatusSnapshot type + ValidationUnitSnapshot.statuses field; formatter prints statuses=[Kind(dur),...] token per unit; new deterministic fixture test covers all 5 canon kinds.**

## What Happened

Added `ValidationStatusSnapshot { kind: StatusEffectKind, duration_remaining: u32 }` struct to observability.rs. Extended `ValidationUnitSnapshot` with `pub statuses: Vec<ValidationStatusSnapshot>`. Updated the units query to include `Option<&StatusBag>`, mapping bag entries into snapshots sorted by canonical kind order (Heated=0, Chilled=1, Paralyzed=2, Slowed=3, Blessed=4, Burn=5, Shock=6) via `status_kind_ord` helper. Updated `format_unit` to append `,statuses=[Kind(dur),...]` token using new `format_statuses` helper. Updated 3 existing exact-match tests in validation_snapshot.rs to include `,statuses=[]` per unit. Added new test `per_unit_statuses_populated_deterministically` that applies all 5 canon kinds in scrambled order and asserts both the struct-level vector (sorted, deterministic) and the formatted substring.

## Verification

cargo check (exit 0), cargo test --test validation_snapshot (6/6 pass), cargo test full suite (all pass)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 1810ms |
| 2 | `cargo test --test validation_snapshot` | 0 | 6/6 pass | 3900ms |
| 3 | `cargo test` | 0 | all pass | 0ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/observability.rs`
- `tests/validation_snapshot.rs`
