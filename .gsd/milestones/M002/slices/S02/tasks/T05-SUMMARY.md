---
id: T05
parent: S02
milestone: M002
key_files:
  - src/ui/combat_panel/labels.rs
  - src/ui/combat_panel/mod.rs
  - src/ui/combat_panel/widgets.rs
  - tests/windowed_preview_cache.rs
key_decisions:
  - Kept the existing `src/windowed/render.rs` Sharp Claws playback bridge intact and focused T05 changes on unit-testable telegraph/preview proof surfaces because the animation->cue handshake behavior was already implemented.
  - Exposed telegraph chip text/tooltip as pure helpers so future agents can verify barrier wording and diagnostics without bringing up egui or a display server.
duration: 
verification_result: mixed
completed_at: 2026-05-20T15:09:03.438Z
blocker_discovered: false
---

# T05: Added unit-testable Sharp Claws telegraph helpers and feature-gated windowed tests covering Basic preview damage plus cue-chip hide/show behavior.

**Added unit-testable Sharp Claws telegraph helpers and feature-gated windowed tests covering Basic preview damage plus cue-chip hide/show behavior.**

## What Happened

Verified the existing `src/windowed/` presentation bridge already wires `TimelineClock(Clock::Windowed)`, Agumon Sharp Claws playback, cue-frame release detection, duplicate-release guarding, and idle fallback/return behavior expected by the task. I tightened the UI proof surface instead of rewriting that bridge: `src/ui/combat_panel/labels.rs` now exposes pure `telegraph_chip_text` and `telegraph_chip_tooltip` helpers that `cue_barrier_chip()` composes, so the Sharp Claws telegraph wording and diagnostics can be tested without rendering egui. `src/ui/combat_panel/mod.rs` re-exports those helpers, `tests/windowed_preview_cache.rs` now covers three cases — shared preview cache parity, Basic->Sharp Claws preview damage, and telegraph chip show/hide diagnostics for awaiting versus released barriers — and `src/ui/combat_panel/widgets.rs` got a small `1.0_f32` literal fix to remove the future-incompatible float fallback warning in the chip frame styling.

## Verification

`cargo test --features windowed --test windowed_preview_cache` passed with all three feature-gated tests green. `cargo check --features windowed` passed, confirming the windowed build still compiles. A best-effort one-second validation smoke was attempted with `BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=1 cargo run --features windowed --bin bevyrogue`, but this host has no Linux display session (`DISPLAY`/`WAYLAND_DISPLAY` unset), so winit exited before window creation; this is recorded as an environment limitation rather than a code failure.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_preview_cache` | 0 | ✅ pass | 4503ms |
| 2 | `cargo check --features windowed` | 0 | ✅ pass | 5771ms |
| 3 | `BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=1 cargo run --features windowed --bin bevyrogue` | 101 | ⚠️ environment-limited (winit display unavailable: no DISPLAY/WAYLAND session) | 65105ms |

## Deviations

The task plan referenced `src/windowed.rs`, but the actual windowed entrypoint lives under `src/windowed/mod.rs` and `src/windowed/render.rs`; no functional rewrite was needed there because the playback bridge was already present and aligned with the slice contract.

## Known Issues

Best-effort windowed runtime smoke cannot complete on this host because winit reports `neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.` A display-capable environment is still required to observe the one-second soak interactively.

## Files Created/Modified

- `src/ui/combat_panel/labels.rs`
- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/widgets.rs`
- `tests/windowed_preview_cache.rs`
