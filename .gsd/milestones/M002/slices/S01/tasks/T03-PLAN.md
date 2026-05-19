---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T03: SkillGraphRegistry + StanceGraphRegistry (pure id->Handle resolution, R008)

Why: R008/D004/MEM024 require id->graph resolution with zero if-else; the player (T05) resolves the stance graph through StanceGraphRegistry and S02+ resolve skills via SkillGraphRegistry by the shared skill-id (CompiledTimeline.id = skill_id, skill_timeline.rs:73). Do: Create src/animation/registry.rs with two Resource types each wrapping a map from AnimGraphId -> Handle<AnimGraph>, plus a pure `resolve(&self, id) -> Option<&Handle<AnimGraph>>` (no if-else dispatch; map lookup only). Classify by load-path provenance per D042: graphs loaded via AnimationGraphPaths populate SkillGraphRegistry; graphs loaded via the new AnimationStancePaths (added in T04) populate StanceGraphRegistry. Add a system (or extend the existing track_*_loads path in plugin.rs) that inserts entries keyed by the loaded AnimGraph.id once each handle resolves. Register both resources in AnimationAssetPlugin. Keep registry.rs under the 500-LOC cap. Done when: a headless unit/integration test registers graphs and asserts hit (known id) and miss (unknown id -> None) for both registries; cargo test green. Decisions: D042. Q7: unknown id resolves to None (not panic); duplicate id is last-write-wins or rejected — pick and assert one.

## Inputs

- `src/animation/anim_graph.rs`
- `src/animation/plugin.rs`
- `src/animation/mod.rs`
- `src/data/skill_timeline.rs`

## Expected Output

- `src/animation/registry.rs`
- `src/animation/mod.rs`
- `src/animation/plugin.rs`
- `tests/anim_registry.rs`

## Verification

cargo test --test anim_registry
