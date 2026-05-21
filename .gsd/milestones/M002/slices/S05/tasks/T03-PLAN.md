---
estimated_steps: 3
estimated_files: 6
skills_used: []
---

# T03: OnHitTaken → frame-counted target blink/hurt projection

Why: the milestone demo requires targets to visibly react to hits; today there is no consumer of `CombatEventKind::OnHitTaken` in the render/animation layer.

Do: (1) Add a windowed-only resource `TargetHurtState { entries: HashMap<UnitId, u32> }` in `src/ui/combat_panel/mod.rs` (mirroring `BabyBurnerFlashState`). Initialize it in `UiPlugin::build`. (2) Add an observer system that reads `MessageReader<CombatEvent>` and, for every `OnHitTaken { amount }` event, sets `entries[target] = HURT_FRAMES` (a small const, e.g. 12 frames). (3) Add a tick system in the same `chain()` that decrements all entries by 1 per frame and removes zeroed entries — deterministic, frame-counter, no wall clock. (4) Add a render-side system in `src/windowed/render.rs` that tints the matching sprite's color (e.g. shifts toward red or sets alpha pulse) while the unit appears in `TargetHurtState`. (5) Keep `CombatState` immutable: do not write to anything but the new resource and the sprite's render color/material. Register the new systems alongside the existing flash state systems.

Done-when: extend `tests/windowed_preview_cache.rs` (or add `tests/windowed_target_hurt.rs`) with cases proving: (a) an `OnHitTaken` event seeds the resource with the configured frame count; (b) repeated hits on the same frame collapse into one max-countdown entry (no underflow / no panic); (c) the countdown decrements each frame and clears at zero; (d) no `CombatState` mutation occurred. `cargo test --features windowed --test windowed_preview_cache` and `cargo build --features windowed` pass; headless suite is untouched.

## Inputs

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/labels.rs`
- `src/windowed/mod.rs`
- `src/windowed/render.rs`
- `src/combat/observability/events.rs`
- `tests/windowed_preview_cache.rs`
- `tests/encounter_bootstrap_windowed.rs`

## Expected Output

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/labels.rs`
- `src/windowed/mod.rs`
- `src/windowed/render.rs`
- `tests/windowed_preview_cache.rs`
- `tests/windowed_target_hurt.rs`

## Verification

cargo test --features windowed --test windowed_preview_cache --test windowed_target_hurt

## Observability Impact

Adds `TargetHurtState` (windowed-only resource): a future agent can dump per-unit hurt countdowns to verify that hit projection is firing without inspecting wall-clock state.
