---
id: T04
parent: S04
milestone: M002
key_files: []
key_decisions: []
duration: 
verification_result: mixed
completed_at: 2026-05-21T06:11:00.049Z
blocker_discovered: false
---

# T04: Ran the full S04 regression/build matrix green and documented that real windowed smoke is blocked only by the `gsd_exec` display/GPU sandbox.

**Ran the full S04 regression/build matrix green and documented that real windowed smoke is blocked only by the `gsd_exec` display/GPU sandbox.**

## What Happened

Ran the full S04 regression matrix on current HEAD without changing repo code or assets. All mandatory targeted tests, the windowed-only preview/flash regression, `cargo test --lib`, and both headless/windowed builds passed. Because the parent shell advertises `DISPLAY=:0` and `WAYLAND_DISPLAY=wayland-0`, I also inspected the built-in bounded windowed validation path (`BEVYROGUE_VALIDATION_WINDOWED`) in `src/main.rs` and `src/windowed/mod.rs` and attempted a real smoke using the explicit `bevyrogue` binary. Those probes established that the remaining gap is sandbox-only: `gsd_exec` initially lacked display variables, explicit Wayland hit `WaylandError(Connection(NoCompositor))`, and X11-only fallback reached Bevy's `Unable to find a GPU!` panic. No repo files, assets, atlas data, or animation clips were modified during this closeout.

## Verification

Verified the S04 closeout matrix on current HEAD: `cargo test --test agumon_baby_burner_reactive --test unit_died_payload --test timeline_cue_barrier_pipeline --test timeline_two_clock_parity`, `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity`, `cargo test --features windowed --test windowed_preview_cache`, `cargo test --lib`, `cargo build --no-default-features`, and `cargo build --features windowed` all passed. For optional live smoke, I used the built-in bounded validation mode via `cargo run --bin bevyrogue --features windowed` with `BEVYROGUE_VALIDATION_WINDOWED=1`; explicit Wayland failed with `NoCompositor` and X11-only fallback failed with `Unable to find a GPU!`, so the live-window gap is recorded as an execution-sandbox limitation rather than a product failure.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test agumon_baby_burner_reactive --test unit_died_payload --test timeline_cue_barrier_pipeline --test timeline_two_clock_parity` | 0 | ✅ pass | 792ms |
| 2 | `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity` | 0 | ✅ pass | 824ms |
| 3 | `cargo test --features windowed --test windowed_preview_cache` | 0 | ✅ pass | 921ms |
| 4 | `cargo test --lib` | 0 | ✅ pass | 795ms |
| 5 | `cargo build --no-default-features` | 0 | ✅ pass | 774ms |
| 6 | `cargo build --features windowed` | 0 | ✅ pass | 867ms |
| 7 | `DISPLAY=:0 WAYLAND_DISPLAY=wayland-0 BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=1 cargo run --bin bevyrogue --features windowed` | 101 | ⚠️ expected sandbox limitation: Wayland compositor unavailable (`NoCompositor`) | 297ms |
| 8 | `DISPLAY=:0; unset WAYLAND_DISPLAY; BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=1 cargo run --bin bevyrogue --features windowed` | 101 | ⚠️ expected sandbox limitation: Bevy could not find a GPU in X11 fallback | 498ms |

## Deviations

None.

## Known Issues

Optional real-window smoke could not complete inside the `gsd_exec` sandbox: after exporting display variables, Wayland failed with `NoCompositor`, and X11 fallback reached Bevy's `Unable to find a GPU!` panic. The parent shell does expose `DISPLAY=:0` and `WAYLAND_DISPLAY=wayland-0`, so this is an execution-environment limitation rather than a code regression.

## Files Created/Modified

None.
