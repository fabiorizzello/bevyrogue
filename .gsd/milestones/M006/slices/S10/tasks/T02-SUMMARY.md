---
id: T02
parent: S10
milestone: M006
key_files:
  - src/windowed/render/playback.rs
key_decisions:
  - Kept all helpers as private (fn, not pub/pub(super)) since they are orchestration details of advance_digimon_presentation only
  - Extracted atlas_index_for as a small pure helper to avoid duplicating the atlas lookup between advance_playback_atlas and trace_playback_tick
  - Captured mode_skill_id as Option<String> (owned) before the execute_barrier_release call to resolve a Rust borrow-check conflict between the immutable borrow of sprite.mode and &mut sprite
duration: 
verification_result: passed
completed_at: 2026-05-27T12:20:54.848Z
blocker_discovered: false
---

# T02: Decomposed advance_digimon_presentation into 7 named per-concern private helpers within playback.rs

**Decomposed advance_digimon_presentation into 7 named per-concern private helpers within playback.rs**

## What Happened

Found `advance_digimon_presentation` in `src/windowed/render/playback.rs` as expected (T01 had already relocated it there). Read the full ~420-line function and identified 9 distinct logical phases. Extracted these into private named helpers:

1. `tick_clock_auto_release_unbridged` — Phase 1: checks for an unbridged skill barrier and auto-releases it, returning `true` so the caller early-returns. Preserves the exact debug log and early-return semantics.

2. `tick_clock_decay_feedback` — Phase 2: decays hit_flash, hit_shake, and camera_shake windows on the animation-tick clock (single source of truth, R010). Preserves the trace-before-decay for freshly-armed units and the FLASH_TICKS/SHAKE_TICKS guard.

3. `atlas_index_for` (helper for Step B) — resolves atlas index for a clip frame; extracted to avoid duplicating the lookup between `advance_playback_atlas` and `trace_playback_tick`.

4. `advance_playback_atlas` — Step B: drives the rendered sprite tile from the player frame via the atlas-index map.

5. `apply_hit_feedback` — Step C: applies parametric flash tint and shake translation. Preserves the death-fade colour ownership guard (D031/D032) and the absolute-offset shake semantics.

6. `trace_playback_tick` — Step E: emits the per-tick TRACE log with all existing fields.

7. `compute_pending_release` — Step F: determines whether the current frame is a ReleaseKernel frame, applying the dedup guard. Returns `Option<(String, u32)>`.

8. `try_spawn_node_vfx` — Step G: spawns on-enter VFX for newly-entered nodes via the engine-generic registry, including the warn-once spawn-miss diagnostic (S06/S08 pattern).

9. `execute_barrier_release` — Step H: executes the full release sequence: request_release, despawn charge/ember emitters, spawn projectile effect, fire kernel cue, record ReleaseFrameKey dedup key.

10. `handle_node_exit` — Step I: seeds death fade-out for dying sprites or restores idle via stance graph for living sprites. Preserves the preserve_missing fallback logic (InstantFallback path).

The public `advance_digimon_presentation` function retains its exact signature and orchestrates the helpers with labeled phase comments. One minor borrow-check issue arose: `mode_skill_id` (borrowed from `sprite.mode`) conflicted with `&mut sprite` passed to `execute_barrier_release`. Fixed by capturing it as `Option<String>` (owned) before the call. All behavior, ordering, barrier-release semantics, and warn-once diagnostics are preserved exactly.

## Verification

Verified via: (1) `RUSTFLAGS='-D warnings' cargo build --features windowed` — clean build, zero warnings. (2) `cargo test --features windowed --test windowed_only` — all 66 windowed integration tests pass. (3) `cargo test` — all headless test suites pass (24 unit tests + 700+ integration tests across all test files, zero failures).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS='-D warnings' cargo build --features windowed` | 0 | PASS — clean build, no warnings | 3590ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | PASS — 66/66 tests pass | 3670ms |
| 3 | `cargo test` | 0 | PASS — all test suites pass (zero failures across all suites) | 7090ms |

## Deviations

None. The refactor is purely structural. No behavior, system ordering, or public API changed.

## Known Issues

None.

## Files Created/Modified

- `src/windowed/render/playback.rs`
