---
sliceId: S02
uatType: artifact-driven
verdict: PASS
date: 2026-05-26T11:30:00.000Z
---

# UAT Result — S02

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| `cargo build --features windowed` succeeds | runtime | PASS | `Finished dev profile` — exit 0 |
| `src/ui/cues.rs` present and registered in `src/ui/mod.rs` without `#[cfg(feature = "windowed")]` gate | artifact | PASS | `grep -n "cues" src/ui/mod.rs` → line 2: `pub mod cues;` — bare, no cfg gate |
| `tests/ui.rs` and `tests/ui/cue_registry.rs` present | artifact | PASS | Both files confirmed present |
| Dependency boundary — `bevy_enoki_absent_from_headless_graph` | runtime | PASS | `cargo test --no-default-features --features dev --test dependency_gating` → 2/2 pass; enoki absent headless, present windowed |
| Dependency boundary — `bevy_enoki_present_in_windowed_graph` | runtime | PASS | Same run — both tests ok |
| CueRegistry contract — `cue_registry_lookup_returns_registered_def` | runtime | PASS | `cargo test --no-default-features --features dev --test ui` → ok |
| CueRegistry contract — `cue_registry_unknown_id_returns_none` | runtime | PASS | ok |
| CueRegistry contract — `cue_registry_collision_panics_on_different_def` | runtime | PASS | ok (should panic — passed) |
| CueRegistry contract — `cue_registry_idempotent_reregister_same_def` | runtime | PASS | ok |
| Flash math — `flash_tint_parametric_zero_remaining_is_white` | runtime | PASS | ok |
| Flash math — `flash_tint_parametric_matches_legacy_at_peak` | runtime | PASS | ok |
| Shake math — `shake_offset_parametric_zero_remaining_is_zero` | runtime | PASS | ok |
| Shake math — `shake_offset_parametric_nonzero_at_peak` | runtime | PASS | ok |
| Shake math — `shake_offset_parametric_determinism` | runtime | PASS | ok |
| Shake math — `camera_shake_uses_same_shake_math` | runtime | PASS | ok |
| Windowed regression — 54/54 tests pass | runtime | PASS | `cargo test --features windowed` → `54 passed; 0 failed` — exit 0 |

## Overall Verdict

PASS — All 16 checks passed: 2/2 dependency_gating tests, 10/10 ui integration tests (headless), 54/54 windowed regression tests, build clean, and file preconditions confirmed.

## Notes

All five UAT command sequences exited 0:
1. `cargo build --features windowed` — Finished dev profile, exit 0
2. `cargo test --no-default-features --features dev --test dependency_gating` — 2/2 pass
3. `cargo test --no-default-features --features dev --test ui` — 10/10 pass
4. `cargo test --features windowed` — 54/54 pass (includes both ui harness tests and full windowed suite)

`src/ui/cues.rs` confirmed ungated: `pub mod cues;` at line 2 of `src/ui/mod.rs` with no `#[cfg(feature = "windowed")]` wrapper.

No regressions detected. The enoki-isolation seam holds: `bevy_enoki` absent from headless dependency graph, present in windowed graph.
