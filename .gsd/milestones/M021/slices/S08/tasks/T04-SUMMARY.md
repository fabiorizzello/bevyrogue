---
id: T04
parent: S08
milestone: M021
key_files:
  - tests/bouncing_fire_off_baseline.rs
key_decisions:
  - Used Attribute::Free for enemy units to get neutral triangle modifier so OnDamageDealt amounts are deterministic (18 primary, 9 bounce) without hard-coding post-modifier values.
  - Registered 'agumon'/'apply_heated' in SignalTaxonomy before firing the skill — omitting this causes a debug_assert panic in apply_blueprint_signal when the signal is unrecognized.
  - Asserted by target UnitId rather than damage amount to avoid fragility against future modifier changes while still distinguishing primary vs bounce hits.
  - Pre-existing combat_coherence failure (break_follow_up_and_ult_timing_trace) noted but not fixed — out of scope for T04.
duration: 
verification_result: passed
completed_at: 2026-05-16T21:58:17.641Z
blocker_discovered: false
---

# T04: Added deterministic bouncing_fire_off_baseline.rs with OFF=baseline (rank 0 → single DamageDealt, no bounce) and ON rank-1 tests (primary + exactly 1 bounce hop); twin_core tests verified green.

**Added deterministic bouncing_fire_off_baseline.rs with OFF=baseline (rank 0 → single DamageDealt, no bounce) and ON rank-1 tests (primary + exactly 1 bounce hop); twin_core tests verified green.**

## What Happened

T01–T03 had already migrated the codebase to the Blueprint { owner: "twin_core", ... } kernel transition path. T04's job was to write end-to-end tests that exercise baby_flame through the timeline runner and prove the bouncing fire gate behaves correctly.

Reviewed existing twin_core_integration.rs and twin_core_mechanics.rs — both already import from blueprints::twin_core using the Blueprint transition path (T01 had updated their imports). Both pass without changes.

For the new bouncing_fire_off_baseline.rs tests I modeled the app setup after compiled_timeline_petit_thunder.rs: canonical skills.ron compiled with register_kernel_builtins + register_all_blueprint_exts, TimelineLibrary populated from compile_skill_book_timelines. The key additions over petit_thunder's setup:
- init_resource::<TalentRanks>() — required so has_bouncing_fire can read rank from the world
- SignalTaxonomy::register("agumon", "apply_heated") — required because apply_blueprint_signal panics (debug_assert!) if the signal is not in the taxonomy before pushing to the bus

Used Attribute::Free for enemies to get neutral triangle modifier so damage amounts are predictable (no need to hard-code post-modifier values). Enemy toughness set to 8 with Fire weakness so baby_flame's BreakToughness(10) triggers OnBreak.

Test 1 (OFF=baseline, rank 0): fires baby_flame at a single enemy, asserts exactly 1 OnDamageDealt event (target=ENEMY_A_ID) and the apply_heated Blueprint transition is present. No bounce event.

Test 2 (ON, rank 1): sets TalentRanks "agumon::bouncing_fire" = 1, spawns two enemies. baby_flame fires, bounce_pick_next selector picks ENEMY_B_ID (not in cast_hit_set), on_bounce_hop enqueues DealDamage(9). After the first body pass both enemies are in cast_hit_set, bounce_exit returns true, loop terminates. Asserts exactly 2 OnDamageDealt events targeting ENEMY_A_ID and ENEMY_B_ID respectively, plus apply_heated signal.

The only failure in cargo test is break_follow_up_and_ult_timing_trace in combat_coherence, confirmed pre-existing (git stash found no local tracked changes, test already fails on HEAD before this task's file was written).

## Verification

cargo test --test bouncing_fire_off_baseline: 2 passed. cargo test twin_core: 3 passed (2 integration + 1 mechanics). cargo test full suite: only pre-existing combat_coherence failure (break_follow_up_and_ult_timing_trace), all other tests green.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test bouncing_fire_off_baseline` | 0 | pass | 1240ms |
| 2 | `cargo test twin_core` | 0 | pass | 450ms |
| 3 | `cargo test 2>&1 | grep -E 'FAILED|test result:' | grep -v '0 failed'` | 0 | pass — only pre-existing combat_coherence failure, unrelated to T04 | 45000ms |

## Deviations

none — task plan executed as written. twin_core_integration.rs and twin_core_mechanics.rs required no changes (T01 had already updated their imports to the Blueprint path).

## Known Issues

break_follow_up_and_ult_timing_trace in combat_coherence.rs is a pre-existing failure (confirmed: fails on HEAD before T04 file was created). Not introduced by this task.

## Files Created/Modified

- `tests/bouncing_fire_off_baseline.rs`
