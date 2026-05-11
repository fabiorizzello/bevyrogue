---
id: T01
parent: S07
milestone: M011
key_files:
  - src/combat/toughness.rs
  - src/combat/round_flags.rs
  - src/combat/mod.rs
  - src/combat/resolution.rs
key_decisions:
  - ToughnessCategory uses #[derive(Default)] on the Standard variant so Toughness::new() stays backward-compatible without any constructor signature change
  - Shielded uses saturating_sub(...).max(0) to clamp current cleanly without overflow risk
  - Armored rounding: (amount + 1) / 2 gives ceiling division — e.g. 1 raw → 1 effective, 2 → 1, 3 → 2
  - break_sealed short-circuits before category dispatch to keep the noop path trivial for all categories
duration: 
verification_result: passed
completed_at: 2026-04-28T08:52:35.086Z
blocker_discovered: false
---

# T01: Added ToughnessCategory enum (Standard/Armored/Shielded), RoundFlags component with break_sealed field, and updated Toughness::apply_hit to enforce category and seal logic

**Added ToughnessCategory enum (Standard/Armored/Shielded), RoundFlags component with break_sealed field, and updated Toughness::apply_hit to enforce category and seal logic**

## What Happened

Introduced `ToughnessCategory` enum (Standard, Armored, Shielded) in `toughness.rs` as a `#[derive(Default)]` enum so Standard remains the zero-cost default. Added `category` field to the `Toughness` component with `Default = Standard` to preserve backward compatibility for all existing `Toughness::new(max, weaknesses)` call sites. Added `Toughness::with_category(max, weaknesses, category)` for explicit construction.

Updated `Toughness::apply_hit` to accept `break_sealed: bool` as a third parameter and a match on `category`:
- `break_sealed=true` short-circuits immediately, returning false without mutating `current`.
- `Shielded`: clamps `current` at 0 (using `saturating_sub(...).max(0)`) but never sets `broken=true`.
- `Armored`: halves incoming damage rounded up via `(amount + 1) / 2` before applying standard break logic.
- `Standard`: exact prior semantics preserved.

Created `src/combat/round_flags.rs` with `RoundFlags { break_sealed: bool }` (Bevy Component, Default = false). Registered the module in `mod.rs` with `pub use round_flags::RoundFlags` and `pub use toughness::ToughnessCategory` re-exports. Updated the one call site in `resolution.rs` (`apply_effects`) to pass `false` as the `break_sealed` placeholder so the codebase compiles; T02 will thread the real `RoundFlags` query there.

Updated existing unit tests in `toughness.rs` to use the new 3-argument signature (`false` as the seal flag) and added three new tests: `shielded_never_breaks_from_toughness_hit`, `armored_requires_double_damage_to_break`, `apply_hit_is_noop_when_break_sealed`.

## Verification

Ran `cargo test --lib combat::toughness` — 9 tests passed (6 original + 3 new), 0 failed. Ran `cargo check` — finished with 0 errors (only pre-existing warnings about dead code).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib combat::toughness 2>&1 | grep -E 'test result'` | 0 | ✅ pass — 9 passed, 0 failed | 3200ms |
| 2 | `cargo check 2>&1 | tail -5` | 0 | ✅ pass — Finished dev profile, 0 errors | 2030ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/toughness.rs`
- `src/combat/round_flags.rs`
- `src/combat/mod.rs`
- `src/combat/resolution.rs`
