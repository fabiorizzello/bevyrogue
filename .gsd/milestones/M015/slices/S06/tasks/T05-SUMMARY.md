---
id: T05
parent: S06
milestone: M015
key_files:
  - tests/holy_support_mechanics.rs
  - docs/m015_failure_ledger.md
key_decisions:
  - Keep HolySupport system registration centralized in `register_combat_kernel_runtime()`; local test helpers must not add the same system again.
  - Treat full-suite failures as classification events first; this one was a test-helper duplication, not a combat-contract regression.
duration: 
verification_result: mixed
completed_at: 2026-05-08T22:29:05.997Z
blocker_discovered: false
---

# T05: Fixed the duplicate HolySupport system registration that broke the runtime sweep and restored the green broad-suite baseline.

**Fixed the duplicate HolySupport system registration that broke the runtime sweep and restored the green broad-suite baseline.**

## What Happened

I ran the full headless suite with `cargo test --no-fail-fast` and found a single runtime regression in `tests/holy_support_mechanics.rs::martyr_light_is_only_marked_and_consumed_once_per_cycle`. The failing assertion showed that the first `MarkMartyrLight` transition was being rejected as a duplicate, which traced to `app_with_holy_support()` registering `apply_holy_support_transitions_system` twice: once through `register_combat_kernel_runtime()` and again directly in the test helper. I removed the redundant local registration so the helper now relies on the runtime bootstrap as the single source of truth, then reran the focused mechanics test, the full `cargo test --no-fail-fast` sweep, and the plain `cargo test` suite. I also updated `docs/m015_failure_ledger.md` with the runtime classification and green evidence, and captured the duplicate-registration gotcha for future Bevy combat tests.

## Verification

Initial `cargo test --no-fail-fast` failed only in `tests/holy_support_mechanics.rs::martyr_light_is_only_marked_and_consumed_once_per_cycle`, where the first `MarkMartyrLight` transition was rejected as a duplicate because the HolySupport system had been registered twice. After removing the redundant registration from the test helper, `cargo test --test holy_support_mechanics` passed (6/6), `cargo test --no-fail-fast` passed, `cargo test` passed, and `lsp diagnostics tests/holy_support_mechanics.rs` reported no diagnostics.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --no-fail-fast` | 101 | ❌ fail | 1180ms |
| 2 | `cargo test --test holy_support_mechanics` | 0 | ✅ pass | 438ms |
| 3 | `cargo test --no-fail-fast` | 0 | ✅ pass | 1104ms |
| 4 | `cargo test` | 0 | ✅ pass | 1121ms |

## Deviations

Fixed the runtime failure in the test harness instead of `src/combat` because the implementation was correct and the helper was double-registering the HolySupport transition system.

## Known Issues

None.

## Files Created/Modified

- `tests/holy_support_mechanics.rs`
- `docs/m015_failure_ledger.md`
