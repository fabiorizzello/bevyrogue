---
estimated_steps: 4
estimated_files: 4
skills_used: []
---

# T01: Add windowed-gated lib hit-feedback projection (flash/shake state + kinematics) with headless tests

Why: src/windowed/render.rs is part of the BINARY crate (declared via src/main.rs `mod windowed`), so tests/ cannot reach its private items. The only honest headless-testable seam for flash/shake/damage-number logic is LIB code, exactly like TargetHurtState/observe_target_hurt/tick_target_hurt_state in src/ui/combat_panel/mod.rs (windowed-gated lib, tested with a bare App in tests/windowed_only/windowed_target_hurt.rs). This task owns ALL testable projection logic so T02/T03 only apply it to ECS visuals.

Do: Create src/ui/hit_feedback.rs gated `#[cfg(feature = "windowed")]` and register it in src/ui/mod.rs as `#[cfg(feature = "windowed")] pub mod hit_feedback;`. Provide: (1) `HitFlashState` and `HitShakeState` Resources, each a HashMap<UnitId,u32> of remaining ticks, with consts FLASH_TICKS and SHAKE_TICKS (~6-10, sized for ~12fps anim clock); methods `arm(target)` (insert/reset to full — idempotent so same-window multi-hit arms once) and `decay_by(n)` (saturating_sub, despawn/remove entries that hit 0). (2) An `observe_hit_feedback` Bevy system reading MessageReader<CombatEvent>, arming both states for `event.target` on every OnHitTaken (own cursor per MEM065); dedup is automatic via the map reset. (3) Pure fns: `flash_tint(remaining: u32, total: u32) -> Color` (lerp Color::WHITE at remaining=0 to a bright tint at remaining=total; WHITE when not flashing), `shake_offset(remaining: u32, total: u32) -> Vec2` (deterministic decaying jitter — NO rng; e.g. sinusoid scaled by remaining/total; zero at remaining=0), `damage_number_kinematics(age_ticks: u32, total_ticks: u32) -> (f32, f32)` returning (rise_px, alpha) where alpha→0 and rise grows as age→total, and `hit_damage_amount(kind: &CombatEventKind) -> Option<i32>` returning Some(amount) only for OnHitTaken (None otherwise). Keep decay tick-budget-driven by the caller (render.rs passes PendingAnimationTicks count to decay_by) — do NOT add a frame-driven tick system, so there is a single decay source of truth. Then add tests/windowed_only/windowed_hit_feedback.rs (`#![cfg(feature = "windowed")]`, model build_app + write_on_hit_taken on tests/windowed_only/windowed_target_hurt.rs) and register it in tests/windowed_only.rs via the existing `#[path = ...] mod` pattern.

Tests (Q7 negatives included): (a) OnHitTaken via observe_hit_feedback arms HitFlashState and HitShakeState for the target to FLASH_TICKS/SHAKE_TICKS; (b) decay_by drains to 0 with NO underflow when called past the budget; (c) two OnHitTaken for the same target in one update arm once to full (dedup); (d) a non-hit event (e.g. OnDamageDealt or OnRevive) does NOT arm any state and hit_damage_amount returns None for it while returning Some for OnHitTaken; (e) damage_number_kinematics: at age 0 alpha≈1.0 and rise≈0, at age==total alpha≈0 and rise>0, monotonic; (f) flash_tint(total,total) != Color::WHITE and flash_tint(0,total) == Color::WHITE; (g) shake_offset(0,total) == Vec2::ZERO.

Done when: cargo test --features windowed passes including the new windowed_hit_feedback tests, and a headless `cargo test` still compiles/passes (the module is gated out of the headless build, so no Text2d/render dep can leak — R002/R005/R016). skills_used: tdd, bevy-ecs-expert, rust-development.

## Inputs

- `src/combat/observability/events.rs`
- `src/ui/combat_panel/mod.rs`
- `tests/windowed_only/windowed_target_hurt.rs`
- `src/ui/mod.rs`
- `tests/windowed_only.rs`
- `src/combat/observability/floating.rs`

## Expected Output

- `src/ui/hit_feedback.rs`
- `src/ui/mod.rs`
- `tests/windowed_only/windowed_hit_feedback.rs`
- `tests/windowed_only.rs`

## Verification

cargo test --features windowed

## Observability Impact

No runtime signal added in this task (pure lib + tests); the trace seams land in T02/T03 where the systems are registered.
