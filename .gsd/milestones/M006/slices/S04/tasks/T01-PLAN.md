---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Scaffold src/windowed/digimon/ module tree and move register_agumon_cues into it

Why: establish the per-digimon module + register(app) seam (mirroring blueprints/<name>/register_*; MEM018/MEM106/MEM109, D049) and validate the wiring end-to-end with the smallest extraction (the cue registration) before the larger effect-id/skill refactors. Do: (1) Create src/windowed/digimon/mod.rs exposing `pub(super) fn register_all(app: &mut App)` that calls `agumon::register(app)`, plus `pub(in crate::windowed) mod agumon;`. (2) Create src/windowed/digimon/agumon/mod.rs exposing `pub(in crate::windowed) fn register(app: &mut App)` that, for this task, adds the moved `register_agumon_cues` Startup system. (3) Add `mod digimon;` to src/windowed/mod.rs. (4) Move the body of register_agumon_cues (src/windowed/mod.rs:136-163, the hit_flash Flash + hit_shake SpriteShake + camera_impact CameraShake registrations) into the agumon module; delete it and its `.add_systems(Startup, register_agumon_cues)` line from mod.rs. (5) Keep `.init_resource::<bevyrogue::ui::cues::CueRegistry>()` in UiPlugin (engine owns the resource; agumon only populates it). (6) Call `crate::windowed::digimon::register_all(app)` exactly once from UiPlugin::build (after the CueRegistry init_resource). Ensure render.rs engine types the agumon module will later need are reachable (sibling-module visibility via `super::render::*`); for this task only CueRegistry/CueDef from the lib are needed. Done when: cargo build --features windowed is green with zero warnings; register_agumon_cues no longer appears in src/windowed/mod.rs; the cue registrations appear in src/windowed/digimon/agumon/mod.rs; windowed_only tests still pass (54+ baseline unchanged).

## Inputs

- `src/windowed/mod.rs`
- `src/windowed/render.rs`

## Expected Output

- `src/windowed/digimon/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/mod.rs`

## Verification

cargo test --features windowed --test windowed_only
