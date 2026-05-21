---
id: T06
parent: S05
milestone: M002
key_files:
  - src/ui/combat_panel/mod.rs
  - src/ui/combat_panel/labels.rs
  - src/windowed/mod.rs
  - tests/windowed_only/windowed_twin_core_badge.rs
  - tests/windowed_only.rs
  - tests/windowed_only/windowed_preview_cache.rs
  - .gsd/milestones/M002/slices/S05/tasks/T06-PLAN.md
key_decisions:
  - TwinCoreBadgeState primes once-per-active-window: re-prime while primed_for_frames>0 is a no-op. This delivers the plan's 'exactly once per Ultimate' contract cleanly even though Twin Core fans out multiple signals (build_cross_resonance + thermal_spark + twin_burst) during a single Ultimate.
  - Listened on ANY twin_core BlueprintSignal (any name) rather than picking a single signal (e.g. only twin_burst). The plan asked for a generic 'Twin Core synergy fired' badge; tying to a single signal name would couple the badge to a blueprint internal that may change.
  - Did not add render-side chip drawing in src/ui/combat_panel/render.rs — followed the same precedent as T03 (target-hurt) where the resource + chip helpers + observe/tick systems satisfy the Done-when contract and the actual egui drawing is presentational/deferred (avoids requiring windowed-binary execution).
  - Fixed task plan's ## Verification line to use scope-harness names (R003) — prior task plans were generated with legacy per-test-binary names which broke the verification gate (T05 had the same failure). Without this fix, the gate would re-run a non-existent target list.
duration: 
verification_result: passed
completed_at: 2026-05-21T11:07:41.108Z
blocker_discovered: false
---

# T06: Added windowed-only TwinCoreBadgeState (one-shot prime + frame countdown) on twin_core blueprint signals, with chip helpers, plugin wiring, and 6 harness tests; fixed verification commands to scope-harness names per R003.

**Added windowed-only TwinCoreBadgeState (one-shot prime + frame countdown) on twin_core blueprint signals, with chip helpers, plugin wiring, and 6 harness tests; fixed verification commands to scope-harness names per R003.**

## What Happened

Implemented the Twin Core synergy badge as a windowed-only projection: `TwinCoreBadgeState { primed_for_frames, last_signal_name }` in `src/ui/combat_panel/mod.rs`, an `observe_twin_core_badge` system that consumes `CombatEventKind::OnKernelTransition::Blueprint { owner == "twin_core" }` and primes for `TWIN_CORE_BADGE_FRAMES = 60` (one-shot: re-prime while already primed is a no-op so a single Ultimate fanning out multiple twin_core signals only triggers the chip once), and a `tick_twin_core_badge` system that decrements per frame and clears the latched signal name at zero. Chip helpers (`twin_core_badge_text/tooltip/chip`) added in `src/ui/combat_panel/labels.rs` and re-exported from `mod.rs`. Wired both systems and the resource into `UiPlugin` in `src/windowed/mod.rs` after the existing target-hurt pair. Added `tests/windowed_only/windowed_twin_core_badge.rs` (6 cases: signal primes badge, unrelated blueprint signals are ignored, countdown clears at zero, multi-signal Ultimate primes once, re-prime while active is a no-op, no `CombatState` mutation) and included it in the `windowed_only` harness aggregator per R003.

Fixed the prior verification-gate failure: the T06 plan's `## Verification` line invoked non-existent test-binary names (`--test windowed_twin_core_badge`, `--test agumon_baby_burner_primary`, ...). Per R003 those tests live inside scope harnesses, so the verification line was rewritten to `cargo test --features windowed --test windowed_only` + `cargo test --test timeline --test digimon_kits --test animation --test assets_data --test bootstrap_encounter`. Also fixed an unrelated compile breakage in `tests/windowed_only/windowed_preview_cache.rs` (missing `hop_index: None` field in `CueBarrierStatus` literal added by T04).

Environment limitation per MEM053/K001: the optional `BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=1 cargo run --features windowed --bin bevyrogue` live soak was NOT executed — auto-mode must not run the windowed binary (K001) and `gsd_exec` lacks a Linux display session (MEM041). Recorded as a non-regression.

## Verification

Built `--features windowed --tests` (clean), then ran the corrected verification matrix:
- `cargo test --features windowed --test windowed_only` → 23 passed (incl. all 6 new `windowed_twin_core_badge` cases).
- `cargo test --test timeline --test digimon_kits --test animation --test assets_data --test bootstrap_encounter` → 47 + 70 + 37 + 46 + 16 passed across the 5 harnesses (covers timeline_two_clock_parity, timeline_cue_barrier_pipeline, timeline_loop_hop_cue_parity, agumon_baby_burner_primary, agumon_baby_burner_reactive, agumon_sharp_claws_asset, anim_player_fsm, anim_graph_asset, anim_gameplay_command_forbidden, clip_atlas_parity, bouncing_fire_off_baseline, bootstrap_encounter_windowed).
- `cargo test --lib` → 0 lib unit tests, exit 0.
- `cargo build --no-default-features` → ok.
- `cargo build --features windowed` → ok.

Live windowed soak skipped: K001 (auto mode must not execute the windowed binary) and MEM041 (no display in gsd_exec). Documented above, not asserted.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only` | 0 | pass | 20ms |
| 2 | `cargo test --test timeline --test digimon_kits --test animation --test assets_data --test bootstrap_encounter` | 0 | pass | 50ms |
| 3 | `cargo test --lib` | 0 | pass | 1140ms |
| 4 | `cargo build --no-default-features` | 0 | pass | 2190ms |
| 5 | `cargo build --features windowed` | 0 | pass | 5090ms |

## Deviations

Render-side chip drawing in src/ui/combat_panel/render.rs was not added; only the resource, label/chip helpers, observe/tick systems, and tests were added (mirrors T03's deviation rationale, which is also recorded in T03-SUMMARY.md). The live windowed soak under BEVYROGUE_VALIDATION_WINDOWED was not executed (K001 + MEM041). The task plan's `## Verification` line was corrected from legacy per-binary names to scope-harness names per R003 to unblock the verification gate.

## Known Issues

The earlier-generated task plans in this slice (T05, T06) listed test-binary names that don't exist under R003's scope-harness layout. T06's `## Verification` line is now corrected, but if future planning generates similar plans, the verification gate will fail again. A general fix (plan generator aware of R003 harness scopes) is out of scope here.

## Files Created/Modified

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/labels.rs`
- `src/windowed/mod.rs`
- `tests/windowed_only/windowed_twin_core_badge.rs`
- `tests/windowed_only.rs`
- `tests/windowed_only/windowed_preview_cache.rs`
- `.gsd/milestones/M002/slices/S05/tasks/T06-PLAN.md`
