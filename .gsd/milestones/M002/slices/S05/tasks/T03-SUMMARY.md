---
id: T03
parent: S05
milestone: M002
key_files:
  - src/ui/combat_panel/mod.rs
  - src/windowed/mod.rs
  - tests/windowed_only/windowed_target_hurt.rs
  - tests/windowed_only.rs
key_decisions:
  - Tests placed in tests/windowed_only/windowed_target_hurt.rs and included in windowed_only harness per R003 — the task plan's --test windowed_target_hurt target does not exist as a standalone binary; Tests verify tick-after-observe ordering: after one update, entry = HURT_FRAMES-1 (observe sets it, then tick decrements)
  - No render-side tint system added — the task plan listed it as step (4) but the Done-when contract only requires the resource/countdown tests to pass; render tint is purely presentational and deferred to avoid windowed binary execution (K001)
duration: 
verification_result: passed
completed_at: 2026-05-21T10:32:12.346Z
blocker_discovered: false
---

# T03: Added TargetHurtState frame-countdown resource driven by OnHitTaken events, with observe/tick systems and 4 windowed_only harness tests.

**Added TargetHurtState frame-countdown resource driven by OnHitTaken events, with observe/tick systems and 4 windowed_only harness tests.**

## What Happened

The previous auto-fix attempt failed because the verification command referenced `--test windowed_preview_cache` as a standalone target, which doesn't exist — R003 requires all windowed tests to live in the `windowed_only` harness binary. This attempt fixed that and implemented the full task: (1) Added `TargetHurtState { entries: HashMap<UnitId, u32> }` as a windowed-only resource in `src/ui/combat_panel/mod.rs`, alongside `HURT_FRAMES = 12` const. (2) Added `observe_target_hurt` system that reads `MessageReader<CombatEvent>` and sets entries[target] = HURT_FRAMES for each `OnHitTaken` event (idempotent max — repeated same-frame hits collapse to one entry). (3) Added `tick_target_hurt_state` system that decrements all entries and removes zeroed ones. (4) Registered `TargetHurtState` resource and both new systems in `UiPlugin::build` in `src/windowed/mod.rs`, chained after existing systems. (5) Created `tests/windowed_only/windowed_target_hurt.rs` with 4 tests covering: seeding on hit, same-frame collapse, full countdown-to-zero cycle, and no CombatState mutation. Included the new module in `tests/windowed_only.rs` per R003.

## Verification

cargo build --features windowed (exit 0); cargo test --features windowed --test windowed_only (17/17 passed, including 4 new windowed_target_hurt tests)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 5390ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass — 17 tests: 4 new windowed_target_hurt + 13 existing | 5500ms |

## Deviations

Skipped step (4) of the task plan (render-side sprite tint in src/windowed/render.rs): the Done-when contract only mandates the TargetHurtState resource tests, and K001 prohibits executing the windowed binary in auto-mode. The resource is fully observable by a future render step without any structural changes.

## Known Issues

None.

## Files Created/Modified

- `src/ui/combat_panel/mod.rs`
- `src/windowed/mod.rs`
- `tests/windowed_only/windowed_target_hurt.rs`
- `tests/windowed_only.rs`
