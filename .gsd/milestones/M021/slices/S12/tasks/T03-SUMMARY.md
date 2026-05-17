---
id: T03
parent: S12
milestone: M021
key_files:
  - tests/holy_support_affordance.rs
  - tests/holy_support_mechanics.rs
  - tests/holy_support_resolution.rs
  - tests/presentation_metadata_boundary.rs
  - tests/combat_cli_shared_surface.rs
key_decisions:
  - Treat validation snapshots as owner-keyed sections only; no direct holy_support field on ValidationSnapshot.
  - Exercise the blueprint signal path in tests rather than the retired direct HolySupport transition path.
  - Use a minimal validation registry in unit tests when only section capture is under test.
duration: 
verification_result: passed
completed_at: 2026-05-17T09:08:12.482Z
blocker_discovered: false
---

# T03: Reworked the holy-support proof tests and CLI boundary checks to assert owner-keyed validation sections instead of retired shared snapshot fields.

**Reworked the holy-support proof tests and CLI boundary checks to assert owner-keyed validation sections instead of retired shared snapshot fields.**

## What Happened

Updated the holy-support affordance, mechanics, and resolution tests to read validation data through `ValidationSnapshot::section("support")` and `ValidationSection::field(...)` instead of the removed `snapshot.holy_support` field. Reworked the mechanics harness to exercise the generic blueprint signal path and registered only the validation extension needed for section capture, avoiding the full runtime side effects. Added a new presentation boundary test proving optional blueprint sections render as stable `support=none` tokens when the resource is absent, and tightened the CLI shared-surface proof to reject retired digimon-named snapshot-field references in source. The windowed/headless/CLI validation formatting call sites already matched the generic contract, so no production code changes were needed there.

## Verification

Focused proof suite passed: `cargo test --test combat_cli_shared_surface --test holy_support_affordance --test holy_support_mechanics --test holy_support_resolution --test presentation_metadata_boundary`. Both cargo-check modes passed: `cargo check` and `cargo check --features windowed`. Final structural greps confirmed the retired `.holy_support` field surface is gone from tests/source and the new owner-keyed `section("support")` assertions are in place.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test combat_cli_shared_surface --test holy_support_affordance --test holy_support_mechanics --test holy_support_resolution --test presentation_metadata_boundary --quiet` | 0 | ✅ pass | 919ms |
| 2 | `cargo check` | 0 | ✅ pass | 5063ms |
| 3 | `cargo check --features windowed` | 0 | ✅ pass | 10495ms |
| 4 | `rg -n "\.holy_support\b" tests src || true && rg -n "section\(\"support\"\)|support=none|support=grace=" tests/holy_support_affordance.rs tests/holy_support_mechanics.rs tests/holy_support_resolution.rs tests/presentation_metadata_boundary.rs tests/combat_cli_shared_surface.rs` | 0 | ✅ pass | 14ms |

## Deviations

Expanded the planned proof update to include a minimal validation-registry harness in the holy-support tests so the owner-keyed section contract is exercised without full runtime side effects.

## Known Issues

None.

## Files Created/Modified

- `tests/holy_support_affordance.rs`
- `tests/holy_support_mechanics.rs`
- `tests/holy_support_resolution.rs`
- `tests/presentation_metadata_boundary.rs`
- `tests/combat_cli_shared_surface.rs`
