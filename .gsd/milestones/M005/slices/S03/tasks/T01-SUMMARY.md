---
id: T01
parent: S03
milestone: M005
key_files:
  - src/ui/hit_feedback.rs
  - src/ui/mod.rs
  - tests/windowed_only/windowed_hit_feedback.rs
  - tests/windowed_only.rs
key_decisions:
  - Caller-driven decay (decay_by takes the tick budget) instead of a frame-driven tick system, keeping a single decay source of truth that render.rs feeds from PendingAnimationTicks
  - arm() inserts to full unconditionally (vs combat_panel's <FLASH_TICKS guard) since both collapse same-window multi-hit to one full countdown; chose the simpler form
  - Deterministic sinusoid for shake_offset (phase = remaining ticks) to honour R004 no-RNG while still giving visible per-tick jitter
  - flash_tint returns exactly Color::WHITE on the remaining==0/total==0 guard so the headless equality assertion is exact
duration: 
verification_result: passed
completed_at: 2026-05-26T08:57:12.252Z
blocker_discovered: false
---

# T01: Added windowed-gated lib hit-feedback projection (flash/shake state + kinematics) with headless tests

**Added windowed-gated lib hit-feedback projection (flash/shake state + kinematics) with headless tests**

## What Happened

Created `src/ui/hit_feedback.rs` gated `#[cfg(feature = "windowed")]` and registered it in `src/ui/mod.rs`, mirroring the `TargetHurtState`/`observe_target_hurt` pattern in `src/ui/combat_panel/mod.rs`. The module owns ALL testable projection logic for S03 so T02/T03 only apply it to ECS visuals.

Provided: (1) `HitFlashState` and `HitShakeState` Resources — each a `HashMap<UnitId,u32>` of remaining ticks — with consts `FLASH_TICKS`/`SHAKE_TICKS` (8, sized for the ~12fps anim clock), `arm(target)` (insert/reset to full, idempotent so same-window multi-hit arms once), `decay_by(n)` (saturating_sub via `retain`, dropping entries at 0), and a `remaining(id)` accessor. (2) `observe_hit_feedback` Bevy system reading `MessageReader<CombatEvent>` with its own cursor (MEM065), arming both states for `event.target` on every `OnHitTaken`; dedup is automatic via the map reset. (3) Pure fns: `flash_tint(remaining,total)` (lerps WHITE→bright red-white tint, exactly `Color::WHITE` when not flashing), `shake_offset(remaining,total)` (deterministic decaying sinusoid jitter, NO rng per R004, `Vec2::ZERO` at remaining=0), `damage_number_kinematics(age,total)` returning `(rise_px, alpha)` with rise monotonic up from 0 and alpha monotonic down from 1.0, and `hit_damage_amount(kind)` returning `Some(amount)` only for `OnHitTaken`.

Decay is caller-driven (render.rs will pass the `PendingAnimationTicks` count to `decay_by`) — no frame-driven tick system was added, keeping a single decay source of truth.

Added `tests/windowed_only/windowed_hit_feedback.rs` (`#![cfg(feature = "windowed")]`) modelled on `windowed_target_hurt.rs` (build_app + write_on_hit_taken), registered in `tests/windowed_only.rs` via the `#[path = ...] mod` pattern. Covers all Q7 negatives.

No runtime signal added in this task (pure lib + tests); the trace seams land in T02/T03 where the systems are registered.

## Verification

Ran headless `cargo test --no-run` (exit 0): the gated module compiles out of the headless build with no Text2d/render dependency leak (R002/R005/R016). Ran `cargo test --features windowed --test windowed_only`: 42 passed, 0 failed, including the 9 new windowed_hit_feedback tests. Confirmed via targeted clippy grep that the new module produces zero clippy findings (the crate-wide clippy exit 101 is from pre-existing code, none referencing hit_feedback.rs).

Test coverage matches the plan: (a) OnHitTaken arms HitFlashState/HitShakeState to FLASH_TICKS/SHAKE_TICKS; (b) decay_by past budget clamps to 0 with no underflow and removes the entry; (c) two OnHitTaken same update dedup to one full entry; (d) a non-hit event (OnRevive) does not arm and hit_damage_amount returns None (also OnDamageDealt→None, OnHitTaken→Some); (e) damage_number_kinematics endpoints + monotonicity; (f) flash_tint(total,total)!=WHITE and flash_tint(0,total)==WHITE; (g) shake_offset(0,total)==Vec2::ZERO; plus R010 no-CombatState-mutation.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --no-run` | 0 | pass | 3755ms |
| 2 | `cargo test --features windowed --test windowed_only windowed_hit_feedback` | 0 | pass (9 passed) | 7596ms |
| 3 | `cargo test --features windowed --test windowed_only` | 0 | pass (42 passed, 0 failed) | 31950ms |

## Deviations

Used arm() unconditional insert-to-full rather than the combat_panel <FLASH_TICKS guard variant; behaviourally equivalent for dedup. Otherwise faithful to the plan.

## Known Issues

none

## Files Created/Modified

- `src/ui/hit_feedback.rs`
- `src/ui/mod.rs`
- `tests/windowed_only/windowed_hit_feedback.rs`
- `tests/windowed_only.rs`
