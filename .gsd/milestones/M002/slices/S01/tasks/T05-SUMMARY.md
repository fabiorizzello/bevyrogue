---
id: T05
parent: S01
milestone: M002
key_files:
  - src/animation/player.rs
  - src/windowed.rs
  - src/main.rs
  - tests/anim_player_fsm.rs
key_decisions:
  - Keep the runtime player feature-agnostic and headless-testable; windowed rendering only consumes frame indices from the player.
  - Split windowed responsibilities into `RenderPlugin` for sprite/camera work and `UiPlugin` for egui surfaces rather than keeping one mixed plugin.
duration: 
verification_result: passed
completed_at: 2026-05-19T19:33:05.397Z
blocker_discovered: false
---

# T05: Shipped a feature-agnostic AnimGraph player and split windowed rendering from UI, with the idle runtime now aligned to the canonical Agumon asset frames.

**Shipped a feature-agnostic AnimGraph player and split windowed rendering from UI, with the idle runtime now aligned to the canonical Agumon asset frames.**

## What Happened

Implemented `AnimGraphPlayer` as a feature-agnostic FSM core that tracks the active node and elapsed animation frames, derives sprite-sheet frame indices from frame ranges, honors loop/hold/speed/reverse playback modifiers, and evaluates only `TimeInNode` and `Always` transitions. Split `windowed.rs` into `RenderPlugin` and `UiPlugin`, then wired the render side to spawn an Agumon idle sprite driven by the stance registry and player. When T04 corrected the Agumon atlas contract, the idle-frame expectations in `tests/anim_player_fsm.rs` were refreshed to the canonical 54–59 range; the runtime code itself did not need logic changes.

## Verification

Fresh `cargo test --test anim_player_fsm`, `cargo nextest run --profile agent`, and `cargo build --features windowed` all passed after the final test update, proving the player logic, full suite, and windowed build remain green.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test anim_player_fsm` | 0 | ✅ pass | 7700ms |
| 2 | `cargo nextest run --profile agent` | 0 | ✅ pass | 7700ms |
| 3 | `cargo build --features windowed` | 0 | ✅ pass | 7700ms |

## Deviations

After T04 restored Agumon clip↔atlas parity, the idle-frame assertions in `tests/anim_player_fsm.rs` were updated from the stale 53–58 range to the canonical 54–59 range so the test reflects the real asset contract.

## Known Issues

None.

## Files Created/Modified

- `src/animation/player.rs`
- `src/windowed.rs`
- `src/main.rs`
- `tests/anim_player_fsm.rs`
