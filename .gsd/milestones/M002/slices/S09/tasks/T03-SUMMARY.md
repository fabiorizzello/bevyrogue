---
id: T03
parent: S09
milestone: M002
key_files:
  - tests/animation/skill_graph_mapping_extensibility.rs
  - tests/animation.rs
key_decisions:
  - Used inline RON fixture helper functions rather than loading asset files, keeping the test hermetic and fast.
  - Documented return_to_idle boundary via a stance entry assertion rather than calling the binary-crate function directly, consistent with the integration test layer boundary.
  - Three focused tests: multi-id 1:1 mapping, InstantFallback diagnostic path, stance entry surface — each proving one requirement from the task plan.
duration: 
verification_result: passed
completed_at: 2026-05-22T08:09:34.580Z
blocker_discovered: false
---

# T03: Added skill_graph_mapping_extensibility integration test proving 1:1 registry mapping, InstantFallback diagnostic path, and stance-entry surface for return_to_idle boundary.

**Added skill_graph_mapping_extensibility integration test proving 1:1 registry mapping, InstantFallback diagnostic path, and stance-entry surface for return_to_idle boundary.**

## What Happened

Read src/animation/registry.rs and src/animation/anim_graph.rs to understand the public API surface: SkillGraphRegistry and StanceGraphRegistry both use a plain HashMap<AnimGraphId, Handle<AnimGraph>>, resolve_snapshot returns Option<ResolvedAnimGraph> with source=Registry, and resolve_snapshot_or_instant_fallback always returns with source=InstantFallback when the id is missing, also recording a MissingGraphDiagnostic. Read existing animation tests for patterns (imports, Assets::default(), inline RON fixtures). Created tests/animation/skill_graph_mapping_extensibility.rs with three tests: (1) skill_registry_supports_multiple_distinct_graph_ids — inserts three distinct AnimGraphIds and asserts each resolve_snapshot returns source=Registry with its own entry node, proving 1:1 no-collision mapping; (2) unregistered_skill_id_returns_instant_fallback_with_diagnostic — asserts source=InstantFallback, fallback entry=MISSING_GRAPH_FALLBACK_NODE_ID, and that a structured MissingGraphDiagnostic is recorded with registry='skill' and message containing 'missing_or_unloaded'; (3) stance_graph_snapshot_entry_is_non_empty_for_return_to_idle_boundary — inserts a stance graph and asserts graph().entry is non-empty and equals 'idle', documenting that the binary-side return_to_idle in src/windowed/render.rs clones exactly this field. Registered the module in tests/animation.rs via #[path]. All 3 tests passed green in 0.00s after 0.46s compile.

## Verification

cargo test --test animation skill_graph_mapping_extensibility — all 3 tests pass, exit code 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation skill_graph_mapping_extensibility` | 0 | PASS — 3 passed, 0 failed, 44 filtered out | 515ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `tests/animation/skill_graph_mapping_extensibility.rs`
- `tests/animation.rs`
