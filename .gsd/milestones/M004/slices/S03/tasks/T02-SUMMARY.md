---
id: T02
parent: S03
milestone: M004
key_files:
  - assets/digimon/agumon/vfx.ron
  - tests/animation/vfx_asset_load.rs
key_decisions:
  - RON-only reuse path: baby_burner.detonate reuses fan_out + static verbs already registered — no new placement verb, no register_agumon_ext change
  - on_expire chain: detonate → flash, terminal flash has no further chain
  - Visual K001 sign-off deferred to manual cargo winx user review
duration: 
verification_result: passed
completed_at: 2026-05-25T14:49:15.716Z
blocker_discovered: false
---

# T02: Enriched baby_burner.detonate into data-driven fan-out burst + flash via RON-only reuse of existing verbs

**Enriched baby_burner.detonate into data-driven fan-out burst + flash via RON-only reuse of existing verbs**

## What Happened

The vfx.ron already carried the full enriched baby_burner.detonate (fan_out, 8 shards, spread 64px, ease-out scale, alpha-fade color, on_expire → baby_burner.flash) and the baby_burner.flash (static at TargetCenter, ttl 2, size 26, alpha-fade). Tests in vfx_asset_load.rs were also already authored (baby_burner_detonate_is_fan_out_burst_chaining_flash, baby_burner_flash_is_static_and_fades, baby_burner_detonate_curves_match_authored_values, baby_burner_flash_curves_match_authored_values). T02 was code-complete but the verification run had never been recorded — the T02-VERIFY.json carried passed:false with zero checks, which blocked S03 completion in auto-mode. Fresh verification confirms all tests pass.

## Verification

cargo test --test animation (110 tests, 0 failed); cargo build --features windowed (clean); cargo test --features windowed --test windowed_only (32 tests, 0 failed). All T02 done-when criteria satisfied. Visual K001 sign-off (cargo winx) remains a manual user step.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation` | 0 | 110 passed, 0 failed — includes all baby_burner_* assertions | 2000ms |
| 2 | `cargo build --features windowed` | 0 | Finished dev profile, 0 errors | 310ms |
| 3 | `cargo test --features windowed --test windowed_only` | 0 | 32 passed, 0 failed — windowed_only spawn contract holds | 1800ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_asset_load.rs`
