---
id: T01
parent: S10
milestone: M006
key_files:
  - /home/fabio/dev/bevyrogue/src/windowed/render.rs
  - /home/fabio/dev/bevyrogue/src/windowed/render/clock.rs
  - /home/fabio/dev/bevyrogue/src/windowed/render/spawn.rs
  - /home/fabio/dev/bevyrogue/src/windowed/render/effects.rs
  - /home/fabio/dev/bevyrogue/src/windowed/render/feedback.rs
  - /home/fabio/dev/bevyrogue/src/windowed/render/playback.rs
  - /home/fabio/dev/bevyrogue/tests/windowed_only/vfx_windowed_contracts.rs
  - /home/fabio/dev/bevyrogue/tests/windowed_only/enoki_impact_render.rs
  - /home/fabio/dev/bevyrogue/tests/windowed_only/digimon_sprite_cue_dispatch.rs
  - /home/fabio/dev/bevyrogue/tests/windowed_only/renamon_extension_contract.rs
key_decisions:
  - Kept all shared type/component/enum/const definitions in render.rs (module root) to avoid complex cross-submodule visibility chains — only system functions moved to submodules
  - Used specific `use super::ItemName` imports in each submodule rather than `use super::*` for clarity and to avoid ambiguity
  - Kept `advance_digimon_presentation` intact in playback.rs as required by task (T02 will decompose it)
  - Updated 4 source-contract test files to include submodule source alongside render.rs for pattern matching — preserved behavioral assertions, adapted file paths only
  - Re-exported `should_auto_release_unbridged` from render.rs with `#[allow(unused_imports)]` since it is only used in #[cfg(test)] code in the agumon module
duration: 
verification_result: passed
completed_at: 2026-05-27T12:15:17.877Z
blocker_discovered: false
---

# T01: Split render.rs (~2390 lines) into five per-concern submodules under src/windowed/render/: clock, spawn, effects, feedback, playback — behavior-preserving decomposition, clean build, all tests green.

**Split render.rs (~2390 lines) into five per-concern submodules under src/windowed/render/: clock, spawn, effects, feedback, playback — behavior-preserving decomposition, clean build, all tests green.**

## What Happened

## What happened

`src/windowed/render.rs` was a ~2390-line monolith owning the entire windowed presentation layer. The task called for a behavior-preserving split into focused submodules under `src/windowed/render/` (which already had `registries.rs` from S09).

### Module structure decision

The repo uses the modern Rust module style: `render.rs` is the module root, `render/` is the sibling directory. Creating a `mod.rs` would conflict, so `render.rs` kept its role as orchestrator.

**Key structural decision**: All shared type/component/enum definitions and constants were kept in `render.rs` to avoid complex cross-submodule visibility chains. Only system functions (and the clock types + impl) moved to submodules. Each submodule imports specific items from `super::` or sibling submodules.

### Submodule breakdown

- **`clock.rs`**: `AnimationClock`, `PendingAnimationTicks`, `DEFAULT_ANIM_FPS`, `MAX_CATCHUP_TICKS`, `parse_anim_fps`, `sample_animation_ticks`, clock tests
- **`spawn.rs`**: `setup_camera`, `init_soft_particle_material`, `build_digimon_atlases`, `spawn_unit_sprites`
- **`effects.rs`**: `diagnose_enoki_vfx_load`, `spawn_effect_by_id`, `spawn_detonate_particles`, `advance_enoki_projectiles`, helper functions (`mouth_anchor_xy`, `anchor_base_xy`, `should_spawn_node_vfx`)
- **`feedback.rs`**: `observe_camera_shake`, `apply_camera_shake`, `drive_hurt_reactions`, `drive_death_reactions`, `is_death_reaction`, `spawn_canvas_damage_numbers`, `advance_canvas_damage_numbers`, `advance_death_fade`, `fade_alpha`, feedback tests
- **`playback.rs`**: `advance_digimon_presentation`, `sync_digimon_mode`, `classify_same_skill_sync`, `should_auto_release_unbridged` (re-exported with `pub(in crate::windowed)` for agumon test access), all FSM/barrier helpers, `slot_offset_y`, `nearest_opposing_target_xy`, `find_sprite_xy`, `SameSkillSync`, playback tests

### External surface preservation

`render::RenderPlugin` stays in render.rs. `render::should_auto_release_unbridged` used by agumon module tests is re-exported via `#[allow(unused_imports)] pub(in crate::windowed) use playback::should_auto_release_unbridged`. `render::registries` module is unchanged.

### Test suite adaptation

Eight windowed_only source-contract tests use `include_str!` on `render.rs` and pattern-match for function names/tokens. Since functions moved to submodules, these tests were updated to also include the relevant submodule source files, preserving the behavioral contracts (the assertions themselves) while adapting to the new file layout. A pre-existing `BeatEdge` unused-import warning in `tests/timeline/timeline_loop_hop_cue_parity.rs` was not introduced by this change (verified via git stash).

### Preserved invariants

- `AnimationClock::tick` catch-up cap (`MAX_CATCHUP_TICKS`) unchanged
- Barrier-release-on-tick semantics in `advance_digimon_presentation` unchanged
- All warn-once diagnostics (`cast_cue_spawn_miss_warned`, atlas/graph load warnings) unchanged
- All `#[cfg(feature = "windowed")]` gating preserved (entire module is windowed-only)
- `advance_digimon_presentation` left intact (not decomposed — that is T02's scope)

## Verification

Ran `RUSTFLAGS='-D warnings' cargo build --features windowed` (exit 0, clean) and `cargo test --features windowed --test windowed_only` (exit 0, 66/66 tests green). Pre-existing BeatEdge warning in timeline test confirmed pre-existing via git stash check.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS='-D warnings' cargo build --features windowed` | 0 | PASS — clean build, zero warnings | 220ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | PASS — 66 tests green, 0 failed | 309ms |

## Deviations

Tests in render.rs's `#[cfg(test)] mod tests` were duplicated into both render.rs (for the integration test file contracts) and the submodule files (for the unit-level functions). The render.rs test module delegates to submodule functions via explicit `use super::clock::*` etc. imports. Some tests that referenced render.rs content by source pattern needed test file updates — this was anticipated by the task description which noted these are source-contract tests.

## Known Issues

None.

## Files Created/Modified

- `/home/fabio/dev/bevyrogue/src/windowed/render.rs`
- `/home/fabio/dev/bevyrogue/src/windowed/render/clock.rs`
- `/home/fabio/dev/bevyrogue/src/windowed/render/spawn.rs`
- `/home/fabio/dev/bevyrogue/src/windowed/render/effects.rs`
- `/home/fabio/dev/bevyrogue/src/windowed/render/feedback.rs`
- `/home/fabio/dev/bevyrogue/src/windowed/render/playback.rs`
- `/home/fabio/dev/bevyrogue/tests/windowed_only/vfx_windowed_contracts.rs`
- `/home/fabio/dev/bevyrogue/tests/windowed_only/enoki_impact_render.rs`
- `/home/fabio/dev/bevyrogue/tests/windowed_only/digimon_sprite_cue_dispatch.rs`
- `/home/fabio/dev/bevyrogue/tests/windowed_only/renamon_extension_contract.rs`
