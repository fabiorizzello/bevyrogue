---
id: S02
parent: M005
milestone: M005
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - DeathExiting is a separate marker component, not a new AgumonPlaybackMode variant — keeps mode match arms closed and the existing reconciliation branches minimal.
  - Death-precedence enforced via system ordering (.after(drive_hurt_reactions)) rather than per-event priority logic — simpler and impossible to get wrong at the call site.
  - drive_death_reactions drives the death node un-gated by playback mode (death always interrupts), unlike the idle-gated hurt driver.
  - DEATH_FADE_TICKS = 8 at 12fps — long enough to read as a fade (~0.67s), short enough not to clutter the field.
  - FadeOut is seeded in the advance.exited death branch (not at UnitDied read time) so the authored death frames play fully first — preserves M002 post-KO overshoot observability.
  - advance_death_fade chained as the third member of the (advance_vfx_particles, advance_agumon_presentation, advance_death_fade) chain — a FadeOut seeded this frame begins fading next tick on the shared PendingAnimationTicks clock.
patterns_established:
  - Terminal sprite exit pattern: marker component (DeathExiting) + reconciliation guard in advance_agumon_presentation + chained fade system (advance_death_fade) — reusable for any future entity-despawn-after-animation need.
  - Pure helper + unit test for alpha math: fade_alpha(remaining, total) mirrors advance_vfx_particles progress math; both helpers co-located in render.rs #[cfg(test)] mod tests.
observability_surfaces:
  - trace!(target: "windowed.agumon_playback") fires on death node seed (unit_id, reaction, node, prior mode)
  - trace!(target: "windowed.agumon_playback") fires when DeathExiting reconciliation guard suppresses sync_agumon_mode (mid-skill interrupt evidence)
  - trace!(target: "windowed.agumon_playback") fires on FadeOut seed in advance.exited death branch
  - trace!(target: "windowed.agumon_playback") fires on entity despawn when fade_alpha reaches 0
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-26T08:46:38.010Z
blocker_discovered: false
---

# S02: Death reaction and field exit

**Death event driver + FadeOut system wired in the windowed crate: a KO'd unit now plays its authored death frames then fades off the field; all builds and tests green, dep-gating confirmed.**

## What Happened

S02 implemented the full death-reaction-and-field-exit pipeline in `src/windowed/render.rs` (binary crate, windowed-gated) with zero new lib symbols — preserving the R002/R005 dep-gating contract established in previous milestones.

**T01 — Death event driver + mode-reconciliation guard**

Added `DeathExiting` as a terminal marker component (not a new `AgumonPlaybackMode` variant, keeping match arms closed). Added `drive_death_reactions` which reads `CombatEvent`, filters via `is_death_reaction` (`UnitDied → StanceReaction::Death`), deduplicates targets per-frame, and calls `drive_stance_reaction(death_node, ...)` *without* the idle-gate (death interrupts in-flight skills). Inserts `DeathExiting` via commands and emits a `trace!(target: "windowed.agumon_playback", ...)`. Registered `.after(drive_hurt_reactions)` to enforce death-precedence. Added a reconciliation guard in `advance_agumon_presentation`: when `DeathExiting` is present, `sync_agumon_mode` is skipped (prevents a still-active barrier from re-seeding the skill node) and the `advance.exited` `return_to_idle` branch is suppressed (sprite rests on its final death frame). Pure helper `is_death_reaction` + negative unit test `is_death_reaction_only_matches_unit_died` added.

**T02 — FadeOut component + advance_death_fade system**

Added `FadeOut { remaining_ticks, total_ticks }` with `DEATH_FADE_TICKS = 8`. Extended the p0 query to include `Option<&FadeOut>`. In the `advance.exited` death branch, `FadeOut` is seeded via `commands.entity(entity)` guarded by `fade_out.is_none()` (once-only). Added `advance_death_fade` which saturating-decrements `remaining_ticks` each tick, rebuilds `Sprite.color` alpha via `fade_alpha(remaining, total)` (mirrors the `advance_vfx_particles` idiom), despawns at 0 with a trace. Chained as the third member of `(advance_vfx_particles, advance_agumon_presentation, advance_death_fade)` so fading begins the tick after the death-node exit. Pure helper `fade_alpha` + unit test `fade_alpha_lerps_full_to_zero` (full=1.0, half≈0.5, zero=0.0, total=0 saturates without panic) added. The fade writes only `Sprite.color`/despawn — strictly downstream of presentation, never feeding the deterministic kernel (R004).

**T03 — Regression sweep + dep-gating closeout**

Verification-only. All four cargo commands green. S02 commits (0870c7a, d90296c, 34d5e85) each touched only `src/windowed/render.rs`; the lib death-mapping changes belong to S01 commit b1c3428 and are consumed read-only by S02. `grep -nE "bevy::render|wgpu|winit|egui|bevy_render" src/windowed/render.rs` returned no matches — no banned dep leaked into the windowed crate.

K001 (visible death-frames-then-fade in `cargo winx`) remains a manual human sign-off. Auto-mode stops at the build/test boundary and makes no visual PASS claim.

## Verification

Fresh slice-level verification run on current HEAD (all exit 0, no failures):
- `cargo test --lib` (headless): 21 passed, 0 failed
- `cargo test --features windowed --lib`: 29 passed, 0 failed — includes new helpers `is_death_reaction_only_matches_unit_died` and `fade_alpha_lerps_full_to_zero`
- `cargo test --features windowed --bins`: 22 + 2 passed, 0 failed
- Integration tests headless: all suites pass (21, 1, 2, 44, 119, 46, 18, 16, 52, 72, 14, 10, 9, 7, 16, 52, 50, 30, 58, 53, 51 passed)
- Integration tests windowed: same suites + 33 windowed-only tests — all pass
- `cargo build --features windowed`: exit 0 (0.19s, fully incremental)
- `cargo build` (headless): exit 0 (0.13s)
- dep-leak grep (`bevy::render|wgpu|winit|egui|bevy_render` in render.rs): no matches — clean
- K001 (visible death+fade in cargo winx): manual human sign-off, not claimed by auto-mode

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

K001 (visible death-frames-then-fade in cargo winx) requires manual human sign-off — auto-mode stops at the build/test boundary and makes no visual PASS claim.

## Follow-ups

K001 sign-off: human must run cargo winx and confirm death frames play then sprite fades off field before S02 can be considered fully accepted.

## Files Created/Modified

- `src/windowed/render.rs` — 
