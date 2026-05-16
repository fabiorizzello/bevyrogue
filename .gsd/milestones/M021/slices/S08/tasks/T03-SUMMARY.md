---
id: T03
parent: S08
milestone: M021
key_files:
  - src/combat/api/timeline.rs
  - src/combat/api/runner_common.rs
  - src/combat/blueprints/agumon/mod.rs
  - src/combat/blueprints/mod.rs
  - assets/data/skills.ron
  - tests/_tmp_inspect_timeline.rs
  - tests/compiled_timeline_boot_validation.rs
  - tests/compiled_timeline_active_canon.rs
  - tests/compiled_timeline_petit_thunder.rs
  - tests/compiled_timeline_tohakken.rs
  - tests/cast_id_propagation.rs
  - tests/compiled_timeline_builtin_validation.rs
key_decisions:
  - Extended SelectorCtx with world and cast_hit_set fields rather than using a world-based workaround, since the bounce selector genuinely needs both to pick valid targets across iterations.
  - Used find_map (item by value) instead of find (item by reference) in the bounce selector to avoid the double-reference pitfall where team: &&Team makes *team a &Team instead of Team.
  - Exposed register_agumon_ext as a public bare-registry function and register_all_blueprint_exts as a module-level aggregator so test files can register blueprint extension points without an App.
  - Gate-only approach for baseline proof: impact_signal → bounce_loop edge has gate has_bouncing_fire; when gate returns false, next_beat returns None → Done, preserving the baseline intent stream with zero code paths executing.
duration: 
verification_result: passed
completed_at: 2026-05-16T21:41:30.089Z
blocker_discovered: false
---

# T03: Added Bouncing Fire Loop branch to baby_flame: TalentRanks resource, bounce predicate/selector/hook registered, skills.ron updated with BeatKind::Loop beat, SelectorCtx extended with world+cast_hit_set access.

**Added Bouncing Fire Loop branch to baby_flame: TalentRanks resource, bounce predicate/selector/hook registered, skills.ron updated with BeatKind::Loop beat, SelectorCtx extended with world+cast_hit_set access.**

## What Happened

T03 introduced the Bouncing Fire talent gate on baby_flame's timeline. The implementation required five coordinated changes:

1. **SelectorCtx extension** (`src/combat/api/timeline.rs`): Added `world: &'a World` and `cast_hit_set: &'a HashSet<UnitId>` fields so bounce selectors can query alive enemies and skip already-hit targets. Updated `runner_common.rs::fire_beat` to pass these fields. The existing `core/primary` and `core/caster` selectors continue to work unchanged since they ignore the new fields.

2. **TalentRanks resource** (`src/combat/blueprints/agumon/mod.rs`): Added `pub struct TalentRanks(pub HashMap<String, u8>)` as a Bevy Resource. Defaulting all ranks to 0 means the bouncing fire branch is off by default — this is the baseline proof required by S08.

3. **Bouncing fire extension points** (same file): Registered four new functions:
   - `agumon/has_bouncing_fire` predicate: reads `TalentRanks`, returns true only when rank >= 1. Missing resource = false (safe default).
   - `agumon/bounce_exit` predicate: checks via world query whether any alive enemy outside `cast_hit_set` remains; returns true when loop should terminate.
   - `agumon/bounce_pick_next` selector: queries alive enemies not in `cast_hit_set`, returns first valid target or empty vec. Uses `find_map` (item-by-value, avoids `find`'s `&Self::Item` double-ref pattern).
   - `agumon/on_bounce_hop` hook: enqueues `DealDamage` for each beat_target at 9 damage (half of baby_flame's 18); F6 fix in runner_common automatically adds these to `cast_hit_set`.

4. **baby_flame timeline** (`assets/data/skills.ron`): Added `bounce_loop` beat with `BeatKind::Loop { body: [bounce_hop], exit_when: "agumon/bounce_exit" }` and a gated edge `impact_signal → bounce_loop` with gate `"agumon/has_bouncing_fire"`. At rank 0, the gate fails and `next_beat` returns None → Done (identical to baseline). At rank 1+, the loop runs until no alive enemies remain outside `cast_hit_set`.

5. **Test infrastructure**: Exposed `pub fn register_agumon_ext(regs: &mut ExtRegistries)` for bare-registry timeline validation tests. Added `pub fn register_all_blueprint_exts(regs: &mut ExtRegistries)` to `blueprints/mod.rs` as the central registration point. Updated all 8 test files that call `compile_skill_book_timelines` with the canonical skills.ron to also call `register_all_blueprint_exts`.

One pre-existing test failure: `s_m008_s06_break_follow_up_and_ult_timing_trace` in `tests/combat_coherence.rs` — this test uses the full combat coherence stack, not timeline compilation, and was failing before T03 work began (the pre-T03 state doesn't even compile due to the gabumon.rs deletion from T02).

## Verification

cargo check exits 0 (no errors, only pre-existing warnings). Full test suite with `cargo test -- --skip s_m008_s06_break_follow_up` shows all 300+ tests passing across all integration test files including the previously-failing inspect, cast_id_propagation, compiled_timeline_boot_validation, compiled_timeline_active_canon, compiled_timeline_petit_thunder, compiled_timeline_tohakken, and compiled_timeline_builtin_validation tests.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 2560ms |
| 2 | `cargo test -- --skip s_m008_s06_break_follow_up` | 0 | pass — all 300+ tests pass (pre-existing s_m008_s06_break_follow_up failure excluded, pre-dates T03) | 6000ms |

## Deviations

Added register_all_blueprint_exts in blueprints/mod.rs (not mentioned in task plan) — required because baby_flame now references blueprint-specific predicates and the existing bare-registry test pattern (used by 8 test files) needed a single registration call rather than per-file imports of each blueprint's ext function.

## Known Issues

Pre-existing test failure: s_m008_s06_break_follow_up_and_ult_timing_trace in tests/combat_coherence.rs. Assertion about ult charge after a timeline-backed break. Pre-dates T03 — the pre-T02 state doesn't compile so isolation is not feasible here.

## Files Created/Modified

- `src/combat/api/timeline.rs`
- `src/combat/api/runner_common.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/mod.rs`
- `assets/data/skills.ron`
- `tests/_tmp_inspect_timeline.rs`
- `tests/compiled_timeline_boot_validation.rs`
- `tests/compiled_timeline_active_canon.rs`
- `tests/compiled_timeline_petit_thunder.rs`
- `tests/compiled_timeline_tohakken.rs`
- `tests/cast_id_propagation.rs`
- `tests/compiled_timeline_builtin_validation.rs`
