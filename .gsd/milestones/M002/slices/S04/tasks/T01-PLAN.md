---
estimated_steps: 11
estimated_files: 5
skills_used: []
---

# T01: Wire an owner-neutral post-KO reaction seam

---
estimated_steps: 7
estimated_files: 5
skills_used:
  - rust-best-practices
  - tdd
  - bevy
---
Why: Baby Burner needs skill/cast/KO context immediately after the primary hit is committed, but shared combat code must stay Digimon-free and existing timeline/AnimGraph KernelEvent branching is not viable for S04.

Do: Add a generic post-application/post-KO reaction seam to the combat runtime, preferably as a new `post_action` runtime module plus `PostActionReactionExt` registry axis. The context must include at minimum `skill_id`, `source`, `primary_target`, `cast_id`, `follow_up_depth`, and the `UnitDied` payload (`status_remaining`, `heated_remaining`). Invoke the seam from the legacy single-target path after normal Baby Burner damage/KO is known and before the action is considered fully resolved. Registered functions should return or enqueue generic `Intent`s and/or generic blueprint signals; shared runtime/turn-system code must not mention Agumon, Baby Burner, or `agumon_ult`. Keep the timeline path and S02 suspended-cue semantics unchanged.

Done when: an empty registry is a no-op, existing UnitDied payload behavior is preserved, timeline cue barrier tests remain green, and the new seam is publicly reachable through `bevyrogue::combat::runtime` for blueprint tests/registration.

## Inputs

- `src/combat/runtime/registry.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/paths/single_target.rs`
- `src/combat/state.rs`
- `src/combat/observability/events.rs`
- `tests/unit_died_payload.rs`
- `tests/timeline_cue_barrier_pipeline.rs`

## Expected Output

- `src/combat/runtime/post_action.rs`
- `src/combat/runtime/registry.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/paths/single_target.rs`
- `tests/registry_internals.rs`

## Verification

cargo test --test unit_died_payload --test timeline_cue_barrier_pipeline

## Observability Impact

Creates a named generic seam for future agents to inspect when reactive post-KO behavior fails, with cast/skill/target context preserved instead of inferred from later events.
