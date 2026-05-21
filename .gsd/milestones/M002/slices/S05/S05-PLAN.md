# S05: S05

**Goal:** Assemble Agumon vs Agumon dummy on screen at full kit: a windowed encounter bootstrap places two sprites left/right, per-unit sprite-anchored HP bars and damage numbers render, OnHitTaken drives target blink/hurt, the Baby Flame multi-hit loop visibly iterates exactly the kernel `hop_index` count via per-hop kernel cues, Baby Burner's primary timeline (windup → impact + Heated/Thermal + ToughnessHit → recovery) lands and (when target is Heated and lethal) chains S04's reactive detonate flash, Twin Core signal projects to a windowed synergy badge, and the dummy dies at 0 HP — all without windowed/UI code mutating `CombatState` and with headless tests unchanged.
**Demo:** Agumon vs Agumon dummy at full kit; multi-hit loop visibly = kernel hop count; target blink/hurt driven by CombatEvent.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Windowed Agumon-vs-Agumon-dummy encounter bootstrap with two on-screen sprites**
  - Files: `src/combat/encounter/bootstrap.rs`, `src/bin/combat_cli.rs`, `src/bin/combat_cli/config.rs`, `src/windowed/mod.rs`, `src/windowed/render.rs`, `tests/encounter_bootstrap_windowed.rs`
  - Verify: cargo test --test encounter_bootstrap_windowed --features windowed

- [x] **T02: Sprite-anchored HP bar + damage-number HUD**
  - Files: `src/windowed/render.rs`, `src/ui/combat_panel/display.rs`, `src/ui/combat_panel/render.rs`, `tests/windowed_preview_cache.rs`, `tests/windowed_hud_hp_bar.rs`
  - Verify: cargo test --features windowed --test windowed_preview_cache --test windowed_hud_hp_bar

- [x] **T03: OnHitTaken → frame-counted target blink/hurt projection**
  - Files: `src/ui/combat_panel/mod.rs`, `src/ui/combat_panel/labels.rs`, `src/windowed/mod.rs`, `src/windowed/render.rs`, `tests/windowed_preview_cache.rs`, `tests/windowed_target_hurt.rs`
  - Verify: cargo test --features windowed --test windowed_preview_cache --test windowed_target_hurt

- [x] **T04: Per-hop kernel cue: visible loop iterations = kernel hop_index**
  - Files: `src/combat/runtime/runner.rs`, `src/combat/runtime/cue_barrier.rs`, `src/combat/runtime/mod.rs`, `src/combat/turn_system/pipeline/timeline_exec.rs`, `src/animation/player.rs`, `assets/digimon/agumon/anim_graph.ron`, `tests/timeline_loop_hop_cue_parity.rs`
  - Verify: cargo test --test timeline_loop_hop_cue_parity --test timeline_two_clock_parity --test timeline_cue_barrier_pipeline --test anim_gameplay_command_forbidden --test anim_player_fsm --test bouncing_fire_off_baseline --test clip_atlas_parity

- [x] **T05: Baby Burner primary timeline + animation graph + thermal stack on impact**
  - Files: `assets/data/digimon/agumon/skills.ron`, `assets/digimon/agumon/anim_graph.ron`, `tests/agumon_baby_burner_primary.rs`
  - Verify: cargo test --test agumon_baby_burner_primary --test agumon_baby_burner_reactive --test data_skills_ron_validation --test data_skills_ron_roundtrip --test anim_graph_asset --test anim_player_fsm --test anim_gameplay_command_forbidden --test clip_atlas_parity

- [x] **T06: Twin Core synergy badge + slice verification matrix**
  - Files: `src/ui/combat_panel/mod.rs`, `src/ui/combat_panel/render.rs`, `src/ui/combat_panel/labels.rs`, `src/windowed/mod.rs`, `tests/windowed_twin_core_badge.rs`
  - Verify: cargo test --features windowed --test windowed_twin_core_badge --test windowed_preview_cache --test windowed_hud_hp_bar --test windowed_target_hurt

## Files Likely Touched

- src/combat/encounter/bootstrap.rs
- src/bin/combat_cli.rs
- src/bin/combat_cli/config.rs
- src/windowed/mod.rs
- src/windowed/render.rs
- tests/encounter_bootstrap_windowed.rs
- src/ui/combat_panel/display.rs
- src/ui/combat_panel/render.rs
- tests/windowed_preview_cache.rs
- tests/windowed_hud_hp_bar.rs
- src/ui/combat_panel/mod.rs
- src/ui/combat_panel/labels.rs
- tests/windowed_target_hurt.rs
- src/combat/runtime/runner.rs
- src/combat/runtime/cue_barrier.rs
- src/combat/runtime/mod.rs
- src/combat/turn_system/pipeline/timeline_exec.rs
- src/animation/player.rs
- assets/digimon/agumon/anim_graph.ron
- tests/timeline_loop_hop_cue_parity.rs
- assets/data/digimon/agumon/skills.ron
- tests/agumon_baby_burner_primary.rs
- tests/windowed_twin_core_badge.rs
