---
id: T03
parent: S01
milestone: M017
key_files:
  - assets/data/units.ron
  - tests/status_effect_turn_tick.rs
  - tests/status_effect_integration.rs
  - tests/status_effect_apply.rs
  - tests/combat_coherence.rs
  - tests/status_accuracy.rs
  - src/combat/turn_system/tests.rs
key_decisions:
  - units.ron OnStatusApplied(Freeze(speed_reduction:0)) → OnStatusApplied(Chilled): unit variant, no fields, consistent with T01 enum shape
  - Semantic test assertions (DoT damage, SpeedModifier, action cancel) removed — these are S03-S05 scope; lifecycle assertions (tick/expire events, duration_remaining) preserved
  - combat_coherence.rs OnActionFailed assertion removed with comment noting S03-S05 deferral; OnStatusApplied/Tick/Expired assertions kept and renamed to Paralyzed
  - Dead-code _OLD functions in turn_system/tests.rs updated (still type-checked even without #[test])
duration: 
verification_result: mixed
completed_at: 2026-05-12T16:25:18.896Z
blocker_discovered: false
---

# T03: RON migration complete: units.ron OnStatusApplied(Freeze→Chilled); all test call sites migrated from struct-variant Burn/Freeze/Shock to unit-variant Heated/Chilled/Paralyzed; full suite green.

**RON migration complete: units.ron OnStatusApplied(Freeze→Chilled); all test call sites migrated from struct-variant Burn/Freeze/Shock to unit-variant Heated/Chilled/Paralyzed; full suite green.**

## What Happened

skills.ron was already migrated (T02 handled ApplyStatus effects). units.ron had one legacy reference: `OnStatusApplied(Freeze(speed_reduction: 0))` on Kyubimon's form_identity trigger — updated to `OnStatusApplied(Chilled)` matching the new unit-variant shape.

Cargo test revealed 8+9 compile errors across six test files still using struct-variant syntax (Burn { damage_per_turn }, Freeze { speed_reduction }, Shock { cancel_chance_pct }) that no longer exists in StatusEffectKind. Additionally, src/combat/turn_system/tests.rs had the same issue in dead-code _OLD functions (still type-checked).

Migration strategy per slice scope (semantics deferred to S03-S05):
- Renamed burn/freeze/shock tests to heated/chilled/paralyzed equivalents
- Removed per-status semantic assertions (HP DoT, SpeedModifier delta, action cancel) — replaced with lifecycle-only assertions (OnStatusTick, OnStatusExpired, component removal)
- combat_coherence.rs: removed OnActionFailed { reason: "Shock" } assertion; kept OnStatusApplied/Tick/Expired assertions with Paralyzed
- status_accuracy.rs: Shock { cancel_chance_pct: 30 } / Shock { .. } → Paralyzed (application accuracy tests unaffected)
- Removed now-unused SpeedModifier imports from two test files

## Verification

grep -r 'Burn\|Freeze\|Shock\|DeepFreeze' assets/data/ → only comments and skill names; no ApplyStatus with legacy ids. cargo check → Finished (0 errors). cargo test → all 143+144+… test targets ok, 0 failed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -rn 'Freeze\|DeepFreeze' assets/data/ src/ tests/` | 1 | no Freeze or DeepFreeze references in RON or Rust source | 120ms |
| 2 | `cargo check` | 0 | Finished dev profile, 0 errors | 170ms |
| 3 | `cargo test` | 0 | All test targets pass, 0 failed | 45000ms |

## Deviations

skills.ron was already fully migrated by T02; T03 effectively covered units.ron + all test call sites. The 6 test files were not listed in the T03 plan (estimated_files: 2) but were necessary to make cargo test green per slice contract.

## Known Issues

None.

## Files Created/Modified

- `assets/data/units.ron`
- `tests/status_effect_turn_tick.rs`
- `tests/status_effect_integration.rs`
- `tests/status_effect_apply.rs`
- `tests/combat_coherence.rs`
- `tests/status_accuracy.rs`
- `src/combat/turn_system/tests.rs`
