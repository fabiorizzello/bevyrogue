# S03: Section 9 phase strip live (event-driven) — UAT

**Milestone:** M002
**Written:** 2026-05-20T21:02:24.506Z

## UAT Type

- UAT mode: automated verification with optional visual follow-up
- Why this mode is sufficient: The slice contract is an event-to-UI projection plus a structural non-mutation guarantee, both of which are proven by focused tests and the real windowed compile path without requiring a display server.

## Preconditions

- Repository is at the verified S03 state.
- Rust/Cargo toolchain is installed.
- The `windowed` feature can be compiled in the environment.

## Smoke Test

Run the three automated slice gates in order:

1. `cargo test`
2. `cargo test --test phase_strip_readonly --features windowed`
3. `cargo build --features windowed`

**Expected:** All three commands succeed. The focused windowed test proves the phase-strip ingress is read-only and ignores non-beat / empty updates; the windowed build proves the real `UiPlugin` wiring compiles.

## Test Cases

### 1. Headless-first path stays clean

1. Run `cargo test`.
2. Observe the default, non-windowed test suite result.
3. **Expected:** Tests pass without requiring egui/winit/wgpu at runtime, proving no windowed-only dependencies leaked into headless execution.

### 2. Phase-strip ingress remains combat-read-only

1. Run `cargo test --test phase_strip_readonly --features windowed`.
2. Observe the three focused regression cases.
3. **Expected:** `combat_event_reader_seam_is_read_only`, `phase_strip_projects_latest_beat_without_mutating_combat_state`, and `phase_strip_ignores_non_beat_events_and_empty_updates` all pass.

### 3. Real windowed wiring still compiles

1. Run `cargo build --features windowed`.
2. Observe the build result.
3. **Expected:** Build succeeds, proving `UiPlugin` initializes `PhaseStripDisplay` and schedules the observe/render chain in `EguiPrimaryContextPass`.

## Edge Cases

### No combat-beat events arrive

1. Exercise the focused regression suite.
2. **Expected:** The display remains inactive/stale, but combat state is unchanged and the system does not panic.

### Non-beat combat events arrive

1. Exercise the focused regression suite with a non-`OnCombatBeat` event.
2. **Expected:** The current display beat remains unchanged and `CombatState` is not modified.

### Multiple beat events arrive in one update

1. Exercise the focused regression suite that feeds multiple beat messages before `App::update()`.
2. **Expected:** The latest beat wins for display only; combat state remains unchanged.

## Failure Signals

- `cargo test` fails, indicating headless/windowed cfg leakage or unrelated regression.
- `phase_strip_readonly` fails, indicating the UI seam stopped being read-only or stopped handling non-beat/empty updates correctly.
- `cargo build --features windowed` fails, indicating egui/plugin wiring drift in the real windowed path.

## Not Proven By This UAT

- Visual polish of the banner on a live desktop display.
- Long-running runtime behavior in extended combat sessions beyond the covered regression cases.
- Future downstream UI enrichments beyond the current beat-to-label strip.

## Notes for Tester

If a manual desktop smoke run is desired later, treat missing `DISPLAY` / `WAYLAND_DISPLAY` as an environment limitation, not a slice failure, as long as the three automated checks above remain green.
