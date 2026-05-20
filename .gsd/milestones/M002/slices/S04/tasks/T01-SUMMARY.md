---
id: T01
parent: S04
milestone: M002
key_files:
  - src/combat/runtime/post_action.rs
  - src/combat/runtime/registry.rs
  - src/combat/runtime/mod.rs
  - src/combat/turn_system/pipeline/paths/single_target.rs
  - src/combat/turn_system/pipeline/application.rs
  - src/combat/turn_system/resolve.rs
  - src/combat/mechanics/follow_up/resolve.rs
  - tests/registry_internals.rs
key_decisions:
  - Added `combat::runtime::post_action` as the owner-neutral seam surface, with explicit `PostActionContext`, optional `PostActionUnitDied`, and deterministic roster snapshots.
  - Extended `ExtRegistries` with a `post_action_reactions` axis and dispatched it from the legacy single-target path after primary damage/KO is known but before the action resolves.
duration: 
verification_result: passed
completed_at: 2026-05-20T21:38:04.558Z
blocker_discovered: false
---

# T01: Added a public runtime post-action seam that carries KO context into legacy single-target dispatch and blueprint reaction registration.

**Added a public runtime post-action seam that carries KO context into legacy single-target dispatch and blueprint reaction registration.**

## What Happened

Created `src/combat/runtime/post_action.rs` with a public, owner-neutral post-action API: `PostActionContext`, `PostActionUnitDied`, `PostActionUnitSnapshot`, `PostActionQueue`, and `dispatch_post_action_reactions`. Extended `ExtRegistries` with a `post_action_reactions` registry axis and re-exported the new seam from `bevyrogue::combat::runtime`. Wired the legacy single-target path to capture the primary target's `UnitDied` payload plus a deterministic roster snapshot, dispatch registered post-action reactions after normal damage/KO is known, and emit any returned generic blueprint transitions before the action resolves. Updated dispatcher plumbing so `ExtRegistries` is available to both root actions and follow-up actions without breaking existing pipeline structure. Added registry tests covering empty defaults, deterministic no-op dispatch, and output collection for queued damage plus a blueprint flash transition.

## Verification

`cargo test --test registry_internals --test unit_died_payload --test timeline_cue_barrier_pipeline` passed, covering the new registry seam, preservation of the `UnitDied` payload contract, and unchanged timeline cue barrier behavior.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test registry_internals --test unit_died_payload --test timeline_cue_barrier_pipeline` | 0 | ✅ pass | 3036ms |

## Deviations

Threaded `ExtRegistries` into the legacy/follow-up action dispatcher via bundled `SystemParam` structs in `src/combat/turn_system/resolve.rs` and `src/combat/mechanics/follow_up/resolve.rs` so the new seam could stay wired without exceeding Bevy's system-function arity limits.

## Known Issues

Legacy single-target execution now emits post-action blueprint transitions immediately, but arbitrary queued runtime `Intent`s returned by post-action handlers are only collected by the seam/test surface and are not yet executed inline on that path.

## Files Created/Modified

- `src/combat/runtime/post_action.rs`
- `src/combat/runtime/registry.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/paths/single_target.rs`
- `src/combat/turn_system/pipeline/application.rs`
- `src/combat/turn_system/resolve.rs`
- `src/combat/mechanics/follow_up/resolve.rs`
- `tests/registry_internals.rs`
