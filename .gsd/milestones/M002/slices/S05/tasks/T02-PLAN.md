---
estimated_steps: 3
estimated_files: 5
skills_used: []
---

# T02: Sprite-anchored HP bar + damage-number HUD

Why: CONTEXT requires HP visibly depleting via a minimal HUD anchored near each sprite; today HP is only textual in the roster panel and damage numbers render only in egui overlays decoupled from sprite position.

Do: (1) Add a windowed-only render system that draws a per-unit HP bar as a Bevy 2D primitive (filled rect + frame) positioned above each combatant's sprite Transform, width scaled by `hp_current / hp_max`. Update reactively from the `Unit` component (do not mutate `Unit`). (2) Re-use the existing `FloatingDamage` component (`src/combat/observability/floating.rs`) and `FdDisplay` (`src/ui/combat_panel/display.rs`); add a feature-gated render path that positions damage numbers above the matching unit's sprite world position with a deterministic frame-counter lifetime (do not use `Instant::now`; if a new spawn site is needed, use a frame-counter; otherwise leave the existing `time.elapsed_secs()` lifetime untouched and only read it for visibility). (3) Keep all of this behind `cfg(feature = "windowed")`. The combat panel egui overlay path is unchanged.

Done-when: a new feature-gated test in `tests/windowed_preview_cache.rs` (or a sibling test file `tests/windowed_hud_hp_bar.rs`) builds a synthetic `Unit` + `FloatingDamage` world, runs the HUD systems one tick, and asserts the derived display state (HP percentage, damage-number text + anchor unit id). `cargo test --features windowed --test windowed_preview_cache` and `cargo build --features windowed` pass; `cargo test --lib` and `cargo build --no-default-features` still pass.

## Inputs

- `src/windowed/mod.rs`
- `src/windowed/render.rs`
- `src/ui/combat_panel/display.rs`
- `src/ui/combat_panel/render.rs`
- `src/combat/observability/floating.rs`
- `src/combat/turn_system/pipeline/paths/single_target.rs`
- `src/combat/unit.rs`
- `tests/encounter_bootstrap_windowed.rs`

## Expected Output

- `src/windowed/render.rs`
- `src/ui/combat_panel/display.rs`
- `src/ui/combat_panel/render.rs`
- `tests/windowed_preview_cache.rs`
- `tests/windowed_hud_hp_bar.rs`

## Verification

cargo test --features windowed --test windowed_preview_cache --test windowed_hud_hp_bar
