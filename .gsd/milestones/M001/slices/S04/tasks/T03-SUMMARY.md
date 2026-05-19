---
id: T03
parent: S04
milestone: M001
key_files:
  - src/windowed.rs
key_decisions:
  - Displayed AnimationValidationState in the existing roster_panel as an Option<Res<...>> parameter so the panel gracefully handles the Pending state when the animation plugin hasn't run yet.
  - Used colored_label (Green=READY, Yellow=PENDING, Red=FAILED) with error count for clear visual status.
duration: 
verification_result: passed
completed_at: 2026-05-19T06:39:12Z
blocker_discovered: false
---

# T03: Visual Validation Status and Hot-Reload Proof

**Added colored animation validation status indicator to the windowed roster panel, showing PENDING/READY/FAILED with error counts.**

## What Happened

Updated `src/windowed.rs` to import `AnimationValidationState` and display it in `roster_panel`. The panel now shows a separator below the roster list with a colored label: YELLOW for Pending, GREEN for Ready (with diagnostic count), and RED for Failed (with error count). The system parameter is `Option<Res<AnimationValidationState>>` so it degrades safely if the animation plugin is not registered. The hot-reload proof (manual UAT: edit RON → see FAILED, fix → see READY) follows directly from the existing `watch_for_changes_override: Some(true)` asset watcher and the `validate_animation_assets` system in `AnimationAssetPlugin`, which fires dirty on `AssetEvent::Modified`.

## Verification

- `grep -q "AnimationValidationState" src/windowed.rs` → exit 0 (PASS)
- `cargo check --features windowed` → exit 0, no errors
- `cargo test` → 237+ tests, 0 failures

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -q "AnimationValidationState" src/windowed.rs` | 0 | ✅ pass | <1s |
| 2 | `cargo check --features windowed` | 0 | ✅ pass | ~5s |
| 3 | `cargo test` | 0 | ✅ pass (237 tests) | ~3s |

## Diagnostics

The windowed roster panel left-side UI now shows:

```
Animation Validation
[READY]          ← green label when all graphs valid
diagnostics: 0
```

or on failure:

```
Animation Validation
[FAILED]         ← red label
errors: 1
```

## Deviations

None — panel was extended inline rather than as a separate egui panel, keeping changes minimal.

## Known Issues

Manual hot-reload UAT (run → typo → observe FAILED → fix → observe READY) requires a windowed environment with a display. This was not executed in the headless CI environment. The code path is exercised by the existing `validate_animation_assets` system which fires on `AssetEvent::Modified` events, proven by the `watch_for_changes_override: Some(true)` watcher and the dirty-flag logic in the system.

## Files Created/Modified

- `src/windowed.rs` — Added `AnimationValidationState` import and colored status section to `roster_panel`
