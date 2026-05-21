---
id: S05
parent: M002
milestone: M002
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - src/combat/encounter/bootstrap.rs
  - src/windowed/mod.rs
  - src/windowed/render.rs
  - src/ui/combat_panel/mod.rs
  - src/ui/combat_panel/labels.rs
  - src/combat/runtime/runner.rs
  - src/combat/runtime/cue_barrier.rs
  - src/combat/turn_system/pipeline/timeline_exec.rs
  - src/combat/runtime/applier/effects/damage.rs
  - assets/data/digimon/agumon/skills.ron
  - assets/digimon/agumon/anim_graph.ron
  - tests/bootstrap_encounter/encounter_bootstrap_windowed.rs
  - tests/windowed_only/windowed_hud_hp_bar.rs
  - tests/windowed_only/windowed_target_hurt.rs
  - tests/windowed_only/windowed_twin_core_badge.rs
  - tests/timeline/timeline_loop_hop_cue_parity.rs
  - tests/digimon_kits/agumon_baby_burner_primary.rs
key_decisions:
  - Enemy dummy assembled by cloning ally Agumon def and overriding id/team/name — no unit.ron edits needed; AGUMON_DUMMY_ID = UnitId(9001) exported from bootstrap.rs for test stability
  - Loop body beats in BeatKind::Loop { body } are absent from timeline.beats so awaiting_presentation must be cached at latch time alongside awaiting_cue — find_beat cannot locate them post-latch
  - finalize_timeline_action must invoke dispatch_post_action_reactions: the post-action seam was only wired in the legacy single_target.rs path, so adding a timeline to agumon_ult silently dropped reactive detonate without this fix
  - UnitDied { heated_remaining, status_remaining } must be read from the live StatusBag at KO time, not hardcoded to 0/[] — fixed in applier/effects/damage.rs to align timeline path with legacy ko_payload semantics
  - TwinCoreBadgeState primes once-per-active-window: re-prime while primed_for_frames>0 is a no-op, so a single Ultimate fanning out multiple twin_core signals only triggers the badge once
  - Render-side sprite tint and Twin Core chip drawing deferred in T03/T06 — K001 prohibits windowed binary execution in auto-mode; resources and helpers are fully wired and harness-tested; presentational rendering is the only gap
  - All windowed tests live in the windowed_only scope harness (tests/windowed_only.rs) per R003 — task plan --test windowed_* names referenced non-existent standalone binaries; fixed in T06 verification gate
patterns_established:
  - Loop body beats (BeatKind::Loop { body }) must cache their Presentation at latch time (awaiting_presentation field on BeatRunner) because find_beat cannot locate them after the loop body executes — established in runner.rs/cue_barrier.rs
  - timeline_exec.rs::finalize_timeline_action is the canonical post-action dispatch point for timeline-backed skills — must call dispatch_post_action_reactions to preserve follow-up reactions (reactive detonate etc.) that were previously only reachable via the legacy single_target path
  - Windowed-only projection resources (TargetHurtState, TwinCoreBadgeState, HpBarView, FloatingDamageView) live in src/ui/combat_panel/mod.rs so they are importable by lib-crate tests via bevyrogue:: — matching BabyBurnerFlashState/PhaseStripDisplay precedent
observability_surfaces:
  - CueBarrierStatus.hop_index: Option<u32> — exposes current loop hop to windowed telegraph chip diagnostics
  - TargetHurtState entries HashMap<UnitId, u32> — per-unit frame countdown observable in tests and future render systems
  - TwinCoreBadgeState { primed_for_frames, last_signal_name } — observability surface for twin_core blueprint signal projection
  - HpBarView / FloatingDamageView — computed display resources projecting CombatState without mutating it
  - UnitDied { heated_remaining, status_remaining } — now populated from live StatusBag at KO time on timeline path, matching legacy path payload
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-21T11:12:23.409Z
blocker_discovered: false
---

# S05: Full kit: Agumon vs Agumon dummy

**Assembled Agumon vs Agumon dummy at full kit: windowed encounter bootstrap, sprite-anchored HP bars, per-hop Baby Flame cues, Baby Burner Ultimate timeline with reactive detonate, target hurt, and Twin Core badge — all headless tests green.**

## What Happened

S05 assembled the full Agumon-vs-Agumon-dummy encounter across six tasks, closing M002's user-facing loop.

**T01 — Windowed encounter bootstrap:** The windowed startup system in `src/windowed/mod.rs` was wired to load the roster, apply an Agumon-vs-Agumon-dummy composition via `src/combat/encounter/bootstrap.rs`, cap SP to 999, fire `PartySelected`+`TurnOrderSeeded`, and seed the demo talent rank. The enemy dummy is assembled by cloning the ally Agumon definition and overriding id/team/name — no `units.ron` edits needed. Stable constant `AGUMON_DUMMY_ID = UnitId(9001)` exported from `bootstrap.rs` for test assertions. Sprites placed ally at x=−200, enemy at x=+200 with `flip_x=true`. 16 bootstrap_encounter harness tests green (1 subprocess test ignored by design).

**T02 — Sprite-anchored HP bar + damage numbers:** Added `HpBarView` (HP pct computed from UnitStore, clamped 0–1) and `FloatingDamageView` (damage text with per-sprite anchor, expired entries pruned by lifetime) as windowed-only resources in `src/ui/combat_panel/mod.rs`, with corresponding observe systems registered in `UiPlugin`. Six new `windowed_hud_hp_bar` harness tests assert HP pct computation, overheal clamp, damage text formatting, anchor unit-id, and expired-entry exclusion — all without mutating CombatState.

**T03 — OnHitTaken → target hurt countdown:** Added `TargetHurtState { entries: HashMap<UnitId, u32> }` in `src/ui/combat_panel/mod.rs`, driven by `CombatEventKind::OnHitTaken` events via `observe_target_hurt` (idempotent max — same-frame repeated hits collapse to `HURT_FRAMES = 12`). `tick_target_hurt_state` decrements per frame and removes entries at zero. Both systems registered in `UiPlugin`. Four `windowed_target_hurt` harness tests cover seed-on-hit, same-frame collapse, full countdown cycle, and no-CombatState-mutation invariant. Render-side sprite tint deferred per K001 (windowed binary must not execute in auto-mode).

**T04 — Per-hop kernel cue for Baby Flame loop:** Extended `BeatRunner` with an `awaiting_presentation: Option<Presentation>` cache field, latched alongside `awaiting_cue`. This was required because loop body beats live inside `BeatKind::Loop { body }` and are absent from `timeline.beats`, so the existing `find_beat` call in `awaiting_cue_info()` would panic for loop hops. `resume_cue()` clears both fields. `hop_index: Option<u32>` added to `AwaitingCueInfo` (derived lazily from `loop_stack.last()`) and propagated to `CueBarrierStatus` for windowed telegraph-chip diagnostics. In `skills.ron`, the `bounce_hop` beat gained `presentation: Some((cue_id: "agumon/baby_flame/bounce_hop", anim: Some("baby_flame_impact"), vfx: None, sfx: None))`, making each hop a presentation barrier in Windowed mode (HeadlessAuto ignores presentation). In `anim_graph.ron`, a self-transition `baby_flame_impact → baby_flame_impact` on `KernelCue` (priority 10) was added so the impact animation re-triggers on each hop cue; the existing `TimeInNode → baby_flame_recover` transition fires when the loop ends. Three `timeline_loop_hop_cue_parity` tests prove HeadlessAuto/Windowed parity and hop_index correctness.

**T05 — Baby Burner Ultimate timeline + thermal stack:** Added a 6-beat timeline (cast→windup→impact_damage→impact_break→impact_signal→recovery) to `agumon_ult` in `skills.ron`, keeping `legacy_ops` inert (dispatcher prefers timeline when present). Three animation graph nodes (`baby_burner_charge` frames 23–30, `baby_burner_launch` frames 31–37 with ReleaseKernel cue, `baby_burner_recovery` frames 38–45) added to `anim_graph.ron` with windup→launch (KernelCue)→recovery (KernelCue)→Exit transitions. Critical infrastructure: extended `pipeline/timeline_exec.rs::finalize_timeline_action` to snapshot pre-cast unit state, run `intent_applier`, snapshot post-state, build a `PostActionContext`, and call `dispatch_post_action_reactions` — this seam was previously only wired in the legacy `single_target.rs` path, so routing the Ultimate via timeline would have silently dropped reactive detonate. Also fixed `applier/effects/damage.rs` to read the target's live `StatusBag` at KO time for `UnitDied { heated_remaining, status_remaining }` (was hardcoded to 0/[]), aligning the timeline path's KO payload with legacy semantics. Updated `compiled_timeline_boot_validation.rs` expected count from 16→17. Four `agumon_baby_burner_primary` harness tests (in `digimon_kits`) cover: timeline shape, impact effects, windowed cue handshake, and lethal-heated reactive detonate.

**T06 — Twin Core synergy badge + slice verification matrix:** Added `TwinCoreBadgeState { primed_for_frames, last_signal_name }` to `src/ui/combat_panel/mod.rs`, primed by any `twin_core` `OnKernelTransition::Blueprint` event for `TWIN_CORE_BADGE_FRAMES = 60` frames. Re-prime while already primed is a no-op, so a single Ultimate fanning out multiple twin_core signals (build_cross_resonance + thermal_spark + twin_burst) only triggers the badge once. Chip helpers (`twin_core_badge_text/tooltip/chip`) added in `labels.rs`. Fixed a compile break in `windowed_preview_cache.rs` (missing `hop_index: None` field on `CueBarrierStatus` struct literal added by T04). Six `windowed_twin_core_badge` harness tests cover signal priming, countdown, once-per-ultimate semantics, unrelated-signal exclusion, and no-CombatState-mutation.

**Environment limitation (K001/MEM041/MEM053):** The live windowed soak (`cargo run --features windowed --bin bevyrogue`) was not executed in auto-mode — K001 prohibits this and `gsd_exec` has no Linux display session. Both `cargo build --features windowed` and `cargo build --no-default-features` compile clean; all headless harnesses pass in full. The user must verify the live windowed session manually before S06.

## Verification

Full slice verification matrix — all checks pass:

| Harness | Command | Result |
|---------|---------|--------|
| timeline | `cargo test --test timeline` | 47 passed, 0 failed (incl. timeline_loop_hop_cue_parity, timeline_two_clock_parity, timeline_cue_barrier_pipeline) |
| assets_data | `cargo test --test assets_data` | 46 passed, 0 failed (incl. data_skills_ron_validation, data_skills_ron_roundtrip) |
| animation | `cargo test --test animation` | 37 passed, 0 failed (incl. anim_player_fsm, anim_gameplay_command_forbidden, clip_atlas_parity, anim_graph_asset) |
| windowed_only | `cargo test --features windowed --test windowed_only` | 23 passed, 0 failed (incl. windowed_preview_cache, windowed_hud_hp_bar, windowed_target_hurt, windowed_twin_core_badge, phase_strip_readonly) |
| bootstrap_encounter | `cargo test --features windowed --test bootstrap_encounter` | 16 passed, 1 ignored (subprocess, by design), 0 failed |
| digimon_kits | `cargo test --test digimon_kits` | 70 passed, 0 failed (incl. agumon_baby_burner_primary, agumon_baby_burner_reactive, bouncing_fire_off_baseline, twin_core) |
| build | `cargo build --no-default-features` | EXIT 0 |
| build | `cargo build --features windowed` | EXIT 0 |

Live windowed soak: environment-limited (K001 + MEM041) — not executed in auto-mode, documented as non-regression. User verification required before S06.

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

Render-side sprite tint (src/windowed/render.rs) not added in T03 — Done-when contract only requires TargetHurtState resource tests; tint is presentational and deferred to avoid windowed binary execution (K001). Twin Core chip drawing in render.rs not added in T06 — same rationale. Test files placed under scope harnesses per R003 rather than standalone binaries named in task plans (e.g. tests/digimon_kits/agumon_baby_burner_primary.rs vs tests/agumon_baby_burner_primary.rs). compiled_timeline_boot_validation.rs expected timeline-backed skill count bumped 16→17 for agumon_ult. T06 task plan Verification line corrected from legacy per-binary names to scope-harness names. Live windowed soak not executed: K001 + MEM041 (no display in gsd_exec); documented as environment limitation.

## Known Limitations

Live windowed session not verified in auto-mode (K001/MEM041/MEM053). Render-side sprite tint on hurt and Twin Core badge chip draw in egui are deferred — resources and helpers are fully wired and harness-tested but the visual rendering calls were not added. Future task plans generated by the planner may again use legacy per-binary test names that violate R003's scope-harness layout; a systemic fix (planner R003 awareness) is out of scope for this slice.

## Follow-ups

None.

## Files Created/Modified

None.
