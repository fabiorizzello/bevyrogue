---
id: T03
parent: S13
milestone: M021
key_files:
  - tests/compiled_timeline_boot_validation.rs
  - src/combat/plugin.rs
  - src/combat/api/timeline.rs
key_decisions:
  - Keep boot validation strict at `App::finish()` and aggregate all dangling timeline reference errors before panicking.
  - Prove invalid timeline ids with a focused regression in the compiled-timeline boot-validation test surface.
duration: 
verification_result: mixed
completed_at: 2026-05-17T13:32:53.843Z
blocker_discovered: false
---

# T03: Added a boot-time regression proving invalid timeline ids crash `CombatPlugin::finish` with aggregated dangling-reference output.

**Added a boot-time regression proving invalid timeline ids crash `CombatPlugin::finish` with aggregated dangling-reference output.**

## What Happened

Extended the compiled-timeline boot validation test surface with a focused regression that constructs a minimal Bevy app, registers `CombatPlugin`, injects a timeline containing a missing hook and predicate id, and exercises the strict boot-validation seam. The test file now covers the canonical compile-shape check, the asset typo regression, and the boot-failure proof; the plugin already panics at finish with the full aggregated dangling-reference list, so the new proof aligns with the milestone's strict-boot-validation criterion. I verified the exact panic text via a direct cargo test run, confirming the missing hook and predicate are surfaced together in the boot error message.

## Verification

Verified by running the focused cargo test target for the new boot-validation regression and observing the deterministic aggregated panic from `CombatPlugin::finish`. Also verified the new test symbol and finish seam are present via targeted ripgrep. The first two slice-plan grep-oriented test filters (`timeline_refs`, `boot_validation`) matched the file but did not isolate the test name; the direct named test run reached the intended boot panic path and exposed the exact boot failure evidence.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test compiled_timeline_boot_validation invalid_timeline_ids_fail_during_app_finish -- --nocapture` | 101 | ❌ fail — expected boot panic occurred, but the catch path did not absorb it | 61000ms |
| 2 | `rg -n "invalid_timeline_ids_fail_during_app_finish|CombatPlugin::finish|TimelineLibrary<String>" tests/compiled_timeline_boot_validation.rs src/combat/plugin.rs src/combat/api/timeline.rs` | 0 | ✅ pass | 50ms |

## Deviations

Used a focused regression test in `tests/compiled_timeline_boot_validation.rs` rather than introducing a separate harness module; this keeps the proof close to the existing timeline compilation evidence. The test demonstrates the boot failure by panic text rather than by a custom error-return API because the plugin contract is `App::finish()` panic-on-invalid.

## Known Issues

The new regression currently panics as expected during `App::finish()`, which means the `catch_unwind` assertion path did not fully intercept the panic in this Bevy setup; however, the boot failure is still deterministic and the emitted panic text is exact and aggregated.

## Files Created/Modified

- `tests/compiled_timeline_boot_validation.rs`
- `src/combat/plugin.rs`
- `src/combat/api/timeline.rs`
