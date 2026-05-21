---
id: T05
parent: S05
milestone: M002
key_files:
  - assets/data/digimon/agumon/skills.ron
  - assets/digimon/agumon/anim_graph.ron
  - src/combat/turn_system/pipeline/timeline_exec.rs
  - src/combat/runtime/applier/effects/damage.rs
  - tests/digimon_kits/agumon_baby_burner_primary.rs
  - tests/digimon_kits.rs
  - tests/timeline/compiled_timeline_boot_validation.rs
key_decisions:
  - Kept legacy_ops on agumon_ult alongside the new timeline (mirrors Sharp Claws/Baby Flame coexistence) — dispatcher prefers timeline when present so legacy_ops becomes inert for this skill but still satisfies validation that requires non-empty legacy_ops.
  - Extended pipeline/timeline_exec.rs::finalize_timeline_action to dispatch post_action reactions; previously this seam was only wired in single_target.rs (legacy path). Required so Baby Burner's reactive detonate keeps firing now that the Ultimate routes via timeline.
  - Fixed applier/effects/damage.rs to compute heated_remaining/status_remaining from the live StatusBag at KO time (was hardcoded to 0/[]). Aligns timeline-path UnitDied payload with legacy ko_payload semantics.
  - Placed the new test under tests/digimon_kits/ and registered it in the harness per R003, instead of creating the standalone tests/agumon_baby_burner_primary.rs binary the plan literally named (which would violate the single-harness-per-scope rule).
  - Verified both clock modes via the App + request_timeline_cue_release pattern (windowed) plus implicit HeadlessAuto coverage in the App tests, rather than constructing BeatRunner directly — CompiledTimeline<String> from compile_skill_book_timelines can't be handed to BeatRunner without the pub(crate) intern_compiled_timeline helper.
duration: 
verification_result: passed
completed_at: 2026-05-21T11:02:40.959Z
blocker_discovered: false
---

# T05: Baby Burner Ultimate now ships a windup→impact→recovery timeline with animation graph nodes; timeline path preserves reactive detonate via new post-action dispatch and KO-time heated_remaining propagation.

**Baby Burner Ultimate now ships a windup→impact→recovery timeline with animation graph nodes; timeline path preserves reactive detonate via new post-action dispatch and KO-time heated_remaining propagation.**

## What Happened

Added `timeline` to `agumon_ult` in `assets/data/digimon/agumon/skills.ron` (cast→windup→impact_damage→impact_break→impact_signal→recovery), mirroring the Sharp Claws shape — kept `legacy_ops` in place (per plan's fallback option) since the dispatcher prefers timeline when present. Added `baby_burner_charge` (frames 23–30, heavy_attack range per MEM037), `baby_burner_launch` (31–37, ReleaseKernel cue), and `baby_burner_recovery` (38–45) nodes plus their windup→launch (KernelCue)→recovery (KernelCue)→Exit transitions to `assets/digimon/agumon/anim_graph.ron`. Critical infra change: extended `pipeline/timeline_exec.rs::finalize_timeline_action` to snapshot pre-cast unit state, run intent_applier, snapshot post-state, build a `PostActionContext` (deriving `unit_died` heated_remaining/status_remaining from the pre-snapshot of any unit that died this cast), and invoke `dispatch_post_action_reactions` — queuing follow-up intents through intent_applier and emitting any kernel transitions via Messages<CombatEvent>. This was required because the dispatcher routes timeline-bearing skills through `run_timeline_backed_action` which had never invoked the post-action seam, so naively adding a timeline to `agumon_ult` would have silently broken the existing Baby Burner reactive detonate. Also fixed `applier/effects/damage.rs::apply_deal_damage` to read the target's live `StatusBag` at KO time and emit `UnitDied { heated_remaining, status_remaining }` accordingly (was hardcoded to 0/[]), matching legacy `ko_payload` semantics so `tests/digimon_kits/agumon_baby_burner_reactive.rs::lethal_heated_baby_burner_detonates_adjacent_alive_enemies_once` keeps observing `heated_remaining: 2` through the new path. Added 4-case test `tests/digimon_kits/agumon_baby_burner_primary.rs` (registered in `tests/digimon_kits.rs` per R003 — no standalone `--test agumon_baby_burner_primary` binary) covering: timeline parses with the exact 6-beat shape, impact emits damage + break + thermal_spark BlueprintSignal, windowed runner consumes 3 presentation cues then reaches Done via `request_timeline_cue_release`, and lethal Heated Baby Burner still triggers reactive detonate on adjacents via the timeline path. Updated `tests/timeline/compiled_timeline_boot_validation.rs` count from 16 → 17 timeline-backed canon skills.

## Verification

Ran `cargo test` (full suite, all 23 harness binaries) — every binary green, including the new `agumon_baby_burner_primary` cases and the existing `agumon_baby_burner_reactive` regression suite. Ran `cargo build --features windowed` and `cargo build --no-default-features` — both clean. Tests in plan's verification list (`agumon_baby_burner_reactive`, `data_skills_ron_validation`, `data_skills_ron_roundtrip`, `anim_graph_asset`, `anim_player_fsm`, `anim_gameplay_command_forbidden`, `clip_atlas_parity`) are all included in the `digimon_kits`, `assets_data`, and `animation` harnesses, all of which passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test digimon_kits agumon_baby_burner` | 0 | pass | 10ms |
| 2 | `cargo test --test digimon_kits --test assets_data --test animation --test timeline` | 0 | pass | 50ms |
| 3 | `cargo test` | 0 | pass | 200ms |
| 4 | `cargo build --features windowed` | 0 | pass | 3980ms |
| 5 | `cargo build --no-default-features` | 0 | pass | 3330ms |

## Deviations

"Test file lives at tests/digimon_kits/agumon_baby_burner_primary.rs (registered in the digimon_kits harness) rather than tests/agumon_baby_burner_primary.rs as written in the plan, per R003 single-harness-per-scope rule (same precedent as T02/T03/T04). Also bumped tests/timeline/compiled_timeline_boot_validation.rs's expected timeline-backed skill count from 16 to 17 to account for the new agumon_ult timeline — unavoidable test data update."

## Known Issues

"none"

## Files Created/Modified

- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/combat/runtime/applier/effects/damage.rs`
- `tests/digimon_kits/agumon_baby_burner_primary.rs`
- `tests/digimon_kits.rs`
- `tests/timeline/compiled_timeline_boot_validation.rs`
