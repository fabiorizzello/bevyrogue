---
id: S03
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
  - Caller-driven decay: render.rs passes PendingAnimationTicks to decay_by — single decay source of truth per the animation clock
  - arm() inserts to full unconditionally (vs combat_panel <FLASH_TICKS guard) — simpler, same-window dedup is behaviourally identical
  - Deterministic sinusoid for shake_offset — decaying phase-based jitter with no RNG, honours R004
  - SpriteRest { xy } component captured at spawn — hard-set shake restores to exact rest position, eliminates drift vs hardcoded layout constants
  - Flash tint guarded by death_exiting/fade_out presence — prevents competition with advance_death_fade's alpha lerp
  - CanvasDamageNumber.base_y captured at spawn — advance hard-sets translation.y = base_y + rise_px absolutely, mirrors SpriteRest discipline
  - TextFont::default() / embedded default font — bevy/2d->default_font means no assets/fonts/ dir needed (MEM095)
patterns_established:
  - Windowed-gated lib module (HitFlashState/HitShakeState + pure fns) with headless App tests mirrors the TargetHurtState pattern from src/ui/combat_panel/mod.rs — canonical seam for testable windowed projection logic
  - observe_* system registration ordering: .after(spawn_unit_sprites).after(resolve_action_system).before(advance_agumon_presentation).before(continue_suspended_timeline_system) — drives all arming systems before the decay/apply loop
  - Absolute-position discipline: capture rest/base coords at spawn, hard-set each tick from rest+offset — never accumulate deltas
observability_surfaces:
  - trace!(target: "windowed.agumon_playback", unit_id, "flash+shake armed") fires once per arming in advance_agumon_presentation
  - trace!(target: "windowed.agumon_playback", unit_id, amount, "spawned canvas damage number") fires per spawn in spawn_canvas_damage_numbers
  - debug! log in spawn_canvas_damage_numbers when find_sprite_xy resolves to None (unresolved target, no orphan spawned)
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-26T09:09:13.979Z
blocker_discovered: false
---

# S03: Hit feedback flash shake and canvas damage numbers

**Wired OnHitTaken to a color flash + positional shake on the struck AgumonSprite and a floating world-space Text2d damage number on the pixel canvas, all driven from a windowed-gated lib projection with 9 new headless tests.**

## What Happened

S03 adds three transient on-hit presentation effects to the windowed view with no combat-state mutation (R010).

**T01 — Windowed-gated lib hit-feedback projection with headless tests**

Created `src/ui/hit_feedback.rs` (gated `#[cfg(feature = "windowed")]`) and registered it in `src/ui/mod.rs`, mirroring the `TargetHurtState`/`observe_target_hurt` pattern from `src/ui/combat_panel/mod.rs`. The module provides:
- `HitFlashState` and `HitShakeState` Resources (`HashMap<UnitId, u32>` of remaining ticks with constants `FLASH_TICKS=8` / `SHAKE_TICKS=8`), both with `arm(target)` (idempotent reset-to-full) and `decay_by(n)` (saturating_sub, drops zeroed entries) accessors.
- `observe_hit_feedback` system: reads `MessageReader<CombatEvent>` with its own cursor (MEM065), arms both states on every `OnHitTaken`.
- Pure fns: `flash_tint(remaining, total)` (lerps WHITE → bright red-white tint, exact `Color::WHITE` when not flashing), `shake_offset(remaining, total)` (deterministic decaying sinusoid, no RNG per R004, `Vec2::ZERO` at remaining=0), `damage_number_kinematics(age, total)` → `(rise_px, alpha)` (monotone rise + alpha decay), `hit_damage_amount(kind)` → `Option<u32>`.

Key decisions: caller-driven decay (render.rs passes `PendingAnimationTicks` count to `decay_by`) keeps a single decay source of truth; `arm()` inserts unconditionally to full (simpler than the combat_panel `<FLASH_TICKS` guard, behaviourally equivalent for dedup); deterministic sinusoid for shake to honour R004.

Added `tests/windowed_only/windowed_hit_feedback.rs` with 9 tests covering arming, dedup, decay clamping, non-hit no-op, `damage_number_kinematics` endpoints + monotonicity, `flash_tint` boundary, `shake_offset` zero guard, and R010 no-mutation invariant. Registered in `tests/windowed_only.rs` via the `#[path = ...] mod` pattern.

**T02 — Flash tint + shake offset applied to the struck AgumonSprite**

In `RenderPlugin::build`: init'd `HitFlashState`/`HitShakeState` resources and registered `observe_hit_feedback` with the same ordering constraints as `drive_hurt_reactions` (`.after(spawn_unit_sprites).after(resolve_action_system).before(advance_agumon_presentation).before(continue_suspended_timeline_system)`).

Added binary-local `SpriteRest { xy: Vec2 }` component, captured at sprite spawn — so shake restores to exact rest position without drift (avoids hardcoded `±200` stale-layout risk). In `advance_agumon_presentation`, added `HitFlashState`/`HitShakeState` as `ResMut` params; once per frame (gated on `pending_ticks.0 > 0`), traces freshly-armed units (those still at `FLASH_TICKS` before decay), then decays both windows by `pending_ticks.0`. Per sprite: writes `render_sprite.color = flash_tint(...)` **guarded** by `death_exiting.is_none() && fade_out.is_none()` so it never competes with `advance_death_fade`'s alpha lerp; hard-sets `transform.translation` from `rest.xy + shake_offset(...)` absolutely each tick, eliminating drift.

**T03 — Spawn / float / fade / despawn world-space Text2d damage numbers**

Added binary-local `CanvasDamageNumber { age_ticks, total_ticks, base_y }` component (`base_y` added beyond the plan's illustrative struct to enable absolute Y positioning — no drift). `spawn_canvas_damage_numbers` reads `MessageReader<CombatEvent>` with its own cursor, resolves target sprite XY via `find_sprite_xy`, and spawns one `Text2d` entity per `OnHitTaken` at `(x, y + 40px)`, z=2.0 (above VFX_PARTICLE_Z=1.0), using Bevy's embedded default font (no asset file needed — `bevy/2d` transitively enables `default_font`, captured as MEM095). `advance_canvas_damage_numbers` runs per pending animation tick: ages each number, applies `damage_number_kinematics` for `(rise_px, alpha)`, hard-sets `translation.y = base_y + rise_px`, sets `TextColor` alpha, and despawns once `age_ticks >= total_ticks` — numbers cannot accumulate unbounded (Q6). Registered spawn bridge with the `drive_hurt_reactions` ordering pattern; advance system registered `.after(sample_animation_ticks)` as a standalone system on a disjoint component set. Observability: `trace!(target: "windowed.agumon_playback", unit_id, amount, "spawned canvas damage number")` fires per spawn.

## Verification

All slice-level checks run fresh via gsd_exec:

| # | Command | Exit Code | Verdict |
|---|---------|-----------|---------|
| 1 | `cargo test --no-run` | 0 | pass — headless build confirms no windowed dep leaks (R002/R005/R016) |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass — 42 tests (0 failed), including 9 new windowed_hit_feedback tests |
| 3 | `cargo build --features windowed` | 0 | pass — binary builds with flash/shake/Text2d wiring |
| 4 | `cargo fmt --check` | 0 | pass |
| 5 | `cargo clippy --features windowed` | 0 | pass — no new warnings in S03 code; pre-existing warnings only |

K001 (visible flash/shake/damage number appearance in `cargo winx`) is not auto-verifiable and requires manual user sign-off as stated in the slice plan.

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

K001 (visible flash/shake/number appearance and placement on the pixel canvas) was not exercised in auto-mode and requires manual `cargo winx` user sign-off. Damage number color is plain white from OnHitTaken.amount; kind-based coloring (from co-emitted OnDamageDealt) is explicitly out of scope per S03-RESEARCH.

## Follow-ups

None.

## Files Created/Modified

- `src/ui/hit_feedback.rs` — 
- `src/ui/mod.rs` — 
- `tests/windowed_only/windowed_hit_feedback.rs` — 
- `tests/windowed_only.rs` — 
- `src/windowed/render.rs` — 
