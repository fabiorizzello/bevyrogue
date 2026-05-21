# S04: S04

**Goal:** Deliver a deterministic Rust-only Baby Burner reactive detonate proof: when Agumon's `agumon_ult` kills a Heated primary target, adjacent alive enemies take deterministic detonate damage exactly once and a feature-gated windowed flash indicator projects the generic combat transition without mutating combat state.
**Demo:** Baby Burner reactive detonate with a flash VFX (Rust code, no RON/editor); zero non-determinism, R004 intact, headless tests unchanged.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Owner-neutral post-KO reaction seam wired**
  - Files: `src/combat/runtime/post_action.rs`, `src/combat/runtime/registry.rs`, `src/combat/runtime/mod.rs`, `src/combat/turn_system/pipeline/paths/single_target.rs`, `tests/registry_internals.rs`
  - Verify: cargo test --test unit_died_payload --test timeline_cue_barrier_pipeline

- [x] **T02: Register Agumon Baby Burner detonate with headless tests**
  - Files: `src/combat/blueprints/agumon/mod.rs`, `src/combat/blueprints/agumon/baby_burner.rs`, `tests/agumon_baby_burner_reactive.rs`, `tests/common/app.rs`
  - Verify: cargo test --test agumon_baby_burner_reactive --test unit_died_payload

- [x] **T03: Project detonate transitions into a windowed flash indicator**
  - Files: `src/ui/combat_panel/mod.rs`, `src/ui/combat_panel/labels.rs`, `src/ui/combat_panel/render.rs`, `tests/windowed_preview_cache.rs`
  - Verify: cargo test --features windowed --test windowed_preview_cache

- [x] **T04: Run S04 regression matrix and document live-smoke limits**
  - Verify: cargo test --test agumon_baby_burner_reactive --test unit_died_payload --test timeline_cue_barrier_pipeline --test timeline_two_clock_parity

## Files Likely Touched

- src/combat/runtime/post_action.rs
- src/combat/runtime/registry.rs
- src/combat/runtime/mod.rs
- src/combat/turn_system/pipeline/paths/single_target.rs
- tests/registry_internals.rs
- src/combat/blueprints/agumon/mod.rs
- src/combat/blueprints/agumon/baby_burner.rs
- tests/agumon_baby_burner_reactive.rs
- tests/common/app.rs
- src/ui/combat_panel/mod.rs
- src/ui/combat_panel/labels.rs
- src/ui/combat_panel/render.rs
- tests/windowed_preview_cache.rs
