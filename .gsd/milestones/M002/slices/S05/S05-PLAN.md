# S05: Full kit: Agumon vs Agumon dummy

**Goal:** Assemble Agumon vs Agumon dummy on screen at full kit: a windowed encounter bootstrap places two sprites left/right, per-unit sprite-anchored HP bars and damage numbers render, OnHitTaken drives target blink/hurt, the Baby Flame multi-hit loop visibly iterates exactly the kernel `hop_index` count via per-hop kernel cues, Baby Burner's primary timeline (windup → impact + Heated/Thermal + ToughnessHit → recovery) lands and (when target is Heated and lethal) chains S04's reactive detonate flash, Twin Core signal projects to a windowed synergy badge, and the dummy dies at 0 HP — all without windowed/UI code mutating `CombatState` and with headless tests unchanged.
**Demo:** Agumon vs Agumon dummy at full kit; multi-hit loop visibly = kernel hop count; target blink/hurt driven by CombatEvent.

## Must-Haves

- A `cargo run --features windowed --bin bevyrogue` session shows two Agumon sprites with HP bars; clicking Basic plays Sharp Claws (S02 path) and the dummy HP drops by ~5–6; clicking Skill plays Baby Flame and the dummy is hit a number of visible iterations equal to the kernel hop count (bouncing fire ON); clicking Ultimate plays Baby Burner with windup→impact→recovery beats, drops the dummy below zero on a Heated stack, and triggers the S04 detonate flash chip; OnHitTaken tints the target sprite for a deterministic frame window; the Twin Core badge appears after the Ultimate resolves. Headless test matrix is unchanged: `cargo test --test timeline_two_clock_parity --test timeline_cue_barrier_pipeline --test agumon_baby_burner_reactive --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity --test bouncing_fire_off_baseline` all pass; new tests cover loop-hop cue parity, Baby Burner primary timeline, the target-hurt projection, and the encounter bootstrap; `cargo test --features windowed --test windowed_preview_cache`, `cargo build --no-default-features`, and `cargo build --features windowed` all pass. The live windowed soak is environment-limited in `gsd_exec` (MEM053) and recorded as such.

## Proof Level

- This slice proves: final-assembly

## Integration Closure

Closes M002's user-facing loop. Upstream surfaces consumed: S02 two-clock cue barrier (`src/combat/runtime/cue_barrier.rs`, `runner.rs`) and Sharp Claws/anim handshake; S04 post-action reaction seam (`src/combat/runtime/post_action.rs`) and `OnKernelTransition::Blueprint` projection; existing `CombatEvent::OnHitTaken`/`OnDamageDealt`/`FloatingDamage` observability; encounter bootstrap (`src/combat/encounter/bootstrap.rs`, `apply_composition`); the egui combat-panel pending-action dispatch. New wiring introduced in this slice: a windowed-only `Startup` system in `src/windowed/mod.rs` that loads roster + applies an Agumon-vs-Agumon-dummy composition, emits `PartySelected`/`TurnOrderSeeded`, and seeds the demo Agumon's `agumon::bouncing_fire` talent rank; per-hop cue barrier in `runner.rs`/`cue_barrier.rs`; Baby Burner primary timeline in `assets/data/digimon/agumon/skills.ron` with matching graph nodes in `assets/digimon/agumon/anim_graph.ron`; sprite-anchored HUD primitives in `src/windowed/render.rs`; `TargetHurtState` + `TwinCoreBadgeState` resources in `src/ui/combat_panel/mod.rs` with render-only systems. After this slice the only remaining milestone work is S06 (end-to-end smoke + repomix architectural review).

## Verification

- Adds windowed-only deterministic frame-counter resources: `TargetHurtState` (per-`UnitId` countdown driven by `CombatEventKind::OnHitTaken`) and `TwinCoreBadgeState` (set on `BlueprintSignal` projection of Twin Core synergy).
- Extends the existing two-clock cue-barrier seam with a per-hop barrier on `BeatKind::Loop` body iterations so each loop hop is externally visible as a one-shot `KernelCue` release; the runtime continues to author observability (`OnKernelTransition` / hop_index) and presentation only projects from it.
- New test files create durable failure-localizing assertions: `tests/timeline_loop_hop_cue_parity.rs` (N hops → N cues → identical final state), `tests/agumon_baby_burner_primary.rs` (Ultimate timeline beats, Heated/ToughnessHit, thermal stack), `tests/encounter_bootstrap_windowed.rs` (Agumon vs Agumon dummy composition), plus extended `tests/windowed_preview_cache.rs` cases for HP bar/damage-number display state, target-hurt countdown, and Twin Core badge gating.
- Telegraph chip diagnostics from S02 are reused; combat-panel rendering keeps reading from `CombatEvent`s and never mutates `CombatState`.

## Tasks

- [x] **T01: Bootstrap a windowed Agumon-vs-Agumon-dummy encounter with two on-screen sprites** `est:2h`
  Why: today the windowed app starts with no units on screen and no party selection, so the slice cannot show any combat. Establish the smallest end-to-end pass that lights up the visible scene and gives every later task something to verify against.
  - Files: `src/combat/encounter/bootstrap.rs`, `src/bin/combat_cli.rs`, `src/bin/combat_cli/config.rs`, `src/windowed/mod.rs`, `src/windowed/render.rs`, `tests/encounter_bootstrap_windowed.rs`
  - Verify: cargo test --test encounter_bootstrap_windowed --features windowed

- [x] **T02: Sprite-anchored HP bar + damage-number HUD** `est:2h`
  Why: CONTEXT requires HP visibly depleting via a minimal HUD anchored near each sprite; today HP is only textual in the roster panel and damage numbers render only in egui overlays decoupled from sprite position.
  - Files: `src/windowed/render.rs`, `src/ui/combat_panel/display.rs`, `src/ui/combat_panel/render.rs`, `tests/windowed_preview_cache.rs`, `tests/windowed_hud_hp_bar.rs`
  - Verify: cargo test --features windowed --test windowed_preview_cache --test windowed_hud_hp_bar

- [x] **T03: OnHitTaken → frame-counted target blink/hurt projection** `est:1h30m`
  Why: the milestone demo requires targets to visibly react to hits; today there is no consumer of `CombatEventKind::OnHitTaken` in the render/animation layer.
  - Files: `src/ui/combat_panel/mod.rs`, `src/ui/combat_panel/labels.rs`, `src/windowed/mod.rs`, `src/windowed/render.rs`, `tests/windowed_preview_cache.rs`, `tests/windowed_target_hurt.rs`
  - Verify: cargo test --features windowed --test windowed_preview_cache --test windowed_target_hurt

- [x] **T04: Per-hop kernel cue: visible loop iterations = kernel hop_index** `est:4h`
  Why: Baby Flame's `BeatKind::Loop` body fires N hops via `BeatEvent { hop_index }`, but the AnimGraphPlayer today only knows `PlaybackModifier::Loop { count }`, and writing the kernel hop count into a `count: N` field on the animation graph would leak gameplay numbers into presentation (anti-DRY invariant guarded by `tests/anim_gameplay_command_forbidden.rs`). The runtime must drive each visible iteration via a kernel cue.
  - Files: `src/combat/runtime/runner.rs`, `src/combat/runtime/cue_barrier.rs`, `src/combat/runtime/mod.rs`, `src/combat/turn_system/pipeline/timeline_exec.rs`, `src/animation/player.rs`, `assets/digimon/agumon/anim_graph.ron`, `tests/timeline_loop_hop_cue_parity.rs`
  - Verify: cargo test --test timeline_loop_hop_cue_parity --test timeline_two_clock_parity --test timeline_cue_barrier_pipeline --test anim_gameplay_command_forbidden --test anim_player_fsm --test bouncing_fire_off_baseline --test clip_atlas_parity

- [x] **T05: Baby Burner primary timeline + animation graph + thermal stack on impact** `est:3h`
  Why: `agumon_ult` (`Baby Burner`) has `implementation: Implemented` but no `timeline` field and no animation graph nodes, so the Ultimate button in the egui combat panel produces no visible beats — the kit demo cannot land its Ultimate.
  - Files: `assets/data/digimon/agumon/skills.ron`, `assets/digimon/agumon/anim_graph.ron`, `tests/agumon_baby_burner_primary.rs`
  - Verify: cargo test --test agumon_baby_burner_primary --test agumon_baby_burner_reactive --test data_skills_ron_validation --test data_skills_ron_roundtrip --test anim_graph_asset --test anim_player_fsm --test anim_gameplay_command_forbidden --test clip_atlas_parity

- [x] **T06: Twin Core synergy badge + slice verification matrix** `est:2h`
  Why: Twin Core is the milestone's placeholder ally signal; the demo needs visible proof that the Ultimate resolves into the Twin Core synergy without actually spawning a second unit (M003+ territory). This task also runs the slice verification matrix and records environment limitations explicitly per MEM048/MEM053.
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
