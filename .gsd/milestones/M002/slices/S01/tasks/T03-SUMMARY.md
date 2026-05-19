---
id: T03
parent: S01
milestone: M002
key_files:
  - src/animation/registry.rs
  - src/animation/plugin.rs
  - src/animation/mod.rs
  - tests/anim_registry.rs
key_decisions:
  - Registry lookup stays pure `AnimGraphId -> Handle<AnimGraph>` map access; no if/else dispatch is allowed at call sites.
  - Graph provenance is determined from the configured load paths so skill and stance graphs populate separate resources.
duration: 
verification_result: passed
completed_at: 2026-05-19T19:32:07.909Z
blocker_discovered: false
---

# T03: Added pure id-to-handle skill and stance graph registries and wired them into the asset-loading pipeline.

**Added pure id-to-handle skill and stance graph registries and wired them into the asset-loading pipeline.**

## What Happened

Added `SkillGraphRegistry` and `StanceGraphRegistry` as resource-backed `HashMap<AnimGraphId, Handle<AnimGraph>>` registries with pure `resolve()` lookups. Introduced `populate_graph_registries` so loaded graph handles are classified by configured asset-path provenance and inserted once their assets resolve. Registered the path resources and both registries in `AnimationAssetPlugin`, then covered hit, miss, and independence cases with integration tests.

## Verification

Fresh `cargo nextest run --profile agent` passed after the final updates, including the `anim_registry` coverage for registry hit/miss behavior and registry independence.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo nextest run --profile agent` | 0 | ✅ pass | 7700ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/animation/registry.rs`
- `src/animation/plugin.rs`
- `src/animation/mod.rs`
- `tests/anim_registry.rs`
