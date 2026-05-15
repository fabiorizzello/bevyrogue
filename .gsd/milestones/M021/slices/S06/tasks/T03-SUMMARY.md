---
id: T03
parent: S06
milestone: M021
key_files:
  - src/combat/state.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - tests/compiled_timeline_runtime_dispatch.rs
  - tests/patamon_revive.rs
key_decisions:
  - Store timeline dispatch metadata on InFlightAction instead of ResolvedAction.
  - Keep runtime verification focused on compiled-timeline fixtures instead of legacy effect-resolver assertions.
duration: 
verification_result: passed
completed_at: 2026-05-15T20:56:20.189Z
blocker_discovered: false
---

# T03: Moved action dispatch metadata off ResolvedAction and converted the runtime dispatch tests to compiled-timeline fixtures for timeline-backed combat flow.

**Moved action dispatch metadata off ResolvedAction and converted the runtime dispatch tests to compiled-timeline fixtures for timeline-backed combat flow.**

## What Happened

Removed the `timeline_backed` flag from `ResolvedAction` and carried that dispatch metadata on `InFlightAction` instead, with `step_declaration` deriving it from the skill's authored timeline. The timeline-backed runtime test now exercises two compiled-timeline skills only, including a damage-only variant, and the revive fixture was updated to serialize with an explicit `timeline: None` so its SkillDef is complete. The verification pass confirmed the timeline-backed combat flow still emits the expected beat/event ordering, while bounce and revive semantics remain intact under the current production pipeline.

## Verification

Verified with `cargo test --test compiled_timeline_runtime_dispatch --test target_shape_bounce_chain --test patamon_revive` and `cargo test --test compiled_timeline_runtime_dispatch`. Both passed after the edits, confirming compiled-timeline dispatch, bounce ordering, and revive/support behavior remain green.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test compiled_timeline_runtime_dispatch --test target_shape_bounce_chain --test patamon_revive` | 0 | ✅ pass | 165ms |
| 2 | `cargo test --test compiled_timeline_runtime_dispatch` | 0 | ✅ pass | 164ms |

## Deviations

Adjusted the dispatch-routing flag placement from ResolvedAction to InFlightAction to avoid widening the existing public action constructor surface.

## Known Issues

None.

## Files Created/Modified

- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `tests/compiled_timeline_runtime_dispatch.rs`
- `tests/patamon_revive.rs`
