---
id: S06
parent: M006
milestone: M006
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - Replace `return` with `continue` in populate_graph_registries event loop — each AssetEvent is now handled independently with no early exit; per-event error isolation preserved via `continue`
  - Warn-once uses Local<HashSet<AssetId<AnimGraph>>> to deduplicate across frames so a misconfigured asset logs once on load, not on every Modified event
patterns_established:
  - In Bevy event-reader for-loops, use `continue` for per-event skips — `return` exits the entire system and starves remaining events in the same batch
observability_surfaces:
  - warn! log per unbuildable graph handle (deduplicated by asset id via Local<HashSet>), includes graph_id and path for diagnosis
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-27T08:06:52.196Z
blocker_discovered: false
---

# S06: Graph registry processes all matching graph events so Renamon sprite spawns

**Fixed AnimationGraph batch starvation by replacing early `return` with `continue` in populate_graph_registries; added warn-once for unbuildable graph events.**

## What Happened

The root cause of Renamon's sprite never spawning was an early `return` inside the `populate_graph_registries` event-loop (src/animation/registry.rs). When two `AssetEvent::LoadedWithDependencies` messages arrived in the same Bevy system tick — one for a skill-path graph and one for a stance-path graph — the system would process the first matching event, insert it into `SkillGraphRegistry`, and then `return` immediately, leaving the second event unconsumed and `StanceGraphRegistry` empty.

**T01** reproduced the bug as a headless test (`registry_starvation::populate_graph_registries_starves_second_event_when_first_matches`). The test queues two graph load events in the same frame and asserts both registries are populated after a single `app.update()` call. The test was designed to fail against the buggy code, confirming the starvation path.

**T02** applied the fix: the `return` statements at the early-return branches inside the event loop were replaced with `continue`, giving each event independent processing. Per-event error isolation is preserved — a bad graph (handle not in `AnimationGraphHandles`, asset not loaded yet, or path matching neither list) logs and is skipped without aborting the rest of the batch. The T01 test now passes green, and the full headless suite (700+ tests) remains green.

**T03** added a warn-once observability surface: when a configured graph asset loads but its path matches neither `SkillGraphPaths` nor `StanceGraphPaths`, a single `warn!` is emitted per asset ID (deduplicated via `Local<HashSet<AssetId<AnimGraph>>>`). This makes missing-graph regressions visible in logs rather than silent spawn misses.

Manual windowed sign-off (K001) confirming Renamon's idle sprite is present alongside Agumon's is required by the slice plan but cannot be performed in auto-mode. All automatable verification passes.

## Verification

1. `cargo test --test animation -- registry`: 6/6 pass including `registry_starvation::populate_graph_registries_starves_second_event_when_first_matches` (exit 0, 4.9s).
2. `cargo test` (full headless suite): all test binaries pass, 0 failures across ~700 tests (exit 0, 6.6s).
3. Code inspection of src/animation/registry.rs:254-299 confirms the for-loop uses `continue` for all skip branches and `built`/`warned` for the warn-once path — no `return` inside the event loop.
4. Manual windowed sign-off (K001): NOT PERFORMED — auto-mode cannot launch windowed binary. Renamon idle sprite presence is pending human confirmation.

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

None.

## Known Limitations

Manual K001 windowed sign-off (Renamon idle sprite visible) was not performed — auto-mode cannot launch the windowed binary.

## Follow-ups

None.

## Files Created/Modified

- `src/animation/registry.rs` — 
- `tests/animation/registry_starvation.rs` — 
