---
id: T02
parent: S02
milestone: M005
key_files:
  - src/windowed/render.rs
key_decisions:
  - DEATH_FADE_TICKS = 8 (a few ticks at the 12fps clock) — long enough to read as a fade, short enough not to clutter the field.
  - FadeOut is seeded in advance_agumon_presentation's death-exit branch (not at UnitDied read time) so the authored death frames play fully first, preserving M002 post-KO overshoot observability.
  - advance_death_fade chained after advance_agumon_presentation on the shared PendingAnimationTicks clock, so a same-frame FadeOut insert begins fading the next tick.
duration: 
verification_result: passed
completed_at: 2026-05-26T08:42:08.513Z
blocker_discovered: false
---

# T02: Added the FadeOut component + advance_death_fade system so a KO'd sprite fades off the field after its death node completes, then despawns.

**Added the FadeOut component + advance_death_fade system so a KO'd sprite fades off the field after its death node completes, then despawns.**

## What Happened

Completed the off-field exit half of S02, entirely in `src/windowed/render.rs` (binary crate, windowed-gated; zero lib symbols, preserving R002/R005).

1. Added `FadeOut { remaining_ticks, total_ticks }` (`#[derive(Component, Debug, Clone, Copy, PartialEq)]`) and `const DEATH_FADE_TICKS: u32 = 8;` — a few ticks at the 12fps animation clock — placed next to the T01 `DeathExiting` marker.

2. Extended the p0 query in `advance_agumon_presentation` to `(Entity, &mut AgumonSprite, &mut Sprite, &Transform, Option<&DeathExiting>, Option<&FadeOut>)`. In the `if advance.exited` death branch (where T01 previously just held the final frame), it now inserts `FadeOut { DEATH_FADE_TICKS, DEATH_FADE_TICKS }` via `commands.entity(entity)` — guarded by `fade_out.is_none()` so the once-only death-node exit cannot double-seed. The trace was updated to record the fade seeding instead of a static hold.

3. Added pure helper `fade_alpha(remaining_ticks, total_ticks) -> f32 = (remaining as f32 / total.max(1) as f32).clamp(0.0, 1.0)`, mirroring the `advance_vfx_particles` progress math, with the `.max(1)` divide-by-zero guard (Q5). Unit-tested via `fade_alpha_lerps_full_to_zero` (full=1.0, half≈0.5, zero=0.0, total=0 saturates without panic) following the `decrement_vfx_ttl_saturates_at_zero` pattern.

4. Added `advance_death_fade(commands, pending_ticks, faders: Query<(Entity, &mut FadeOut, &mut Sprite)>)`: for each of `pending_ticks.0` ticks it saturating-decrements `remaining_ticks`, reads the live color via `sprite.color.to_linear()` and rebuilds it with `Color::linear_rgba(r, g, b, fade_alpha(...))` (same alpha-rebuild idiom as `advance_vfx_particles`), and despawns the entity when `remaining_ticks` hits 0, emitting a `trace!(target: "windowed.agumon_playback", ...)` on despawn.

5. Registered `advance_death_fade` as the third member of the `(advance_vfx_particles, advance_agumon_presentation, advance_death_fade).chain()` group, so it runs strictly after presentation on the same `PendingAnimationTicks` clock — a sprite seeded with `FadeOut` in this frame's exit branch begins fading next tick.

Determinism (R004): the fader writes only `Sprite.color`/despawn, strictly downstream of presentation, never feeding the kernel. Failure modes (Q5): an entity despawned by another path mid-fade simply stops being yielded by the query (no panic); `total_ticks == 0` saturates to alpha 1.0 via `.max(1)`.

## Verification

Ran the slice-level verification commands via gsd_exec (pipefail). `cargo build --features windowed` exit 0 (3.09s). `cargo test --features windowed --bins fade_alpha` -> 1 passed, 0 failed. `cargo test --features windowed` full suite exit 0. `cargo build` (headless) exit 0 (7.71s). No lib file touched. Visible death+fade (K001) is left for manual `cargo winx` sign-off — auto-mode never runs the windowed binary.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 3090ms |
| 2 | `cargo test --features windowed --bins fade_alpha` | 0 | pass (1 passed) | 1000ms |
| 3 | `cargo test --features windowed` | 0 | pass | 15000ms |
| 4 | `cargo build` | 0 | pass | 7710ms |

## Deviations

Added Option<&FadeOut> to the p0 query (beyond the plan's Entity-only addition) to support the defensive once-only FadeOut insert guard. No behavioral change to the non-death path.

## Known Issues

none

## Files Created/Modified

- `src/windowed/render.rs`
