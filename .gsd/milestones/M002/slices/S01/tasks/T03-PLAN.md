---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T03: SkillGraphRegistry + StanceGraphRegistry (pure id->Handle resolution, R008)

Create src/animation/registry.rs with two Resource types each wrapping a map from AnimGraphId -> Handle<AnimGraph>, plus a pure `resolve(&self, id) -> Option<&Handle<AnimGraph>>` (no if-else dispatch; map lookup only). Classify by load-path provenance: skill graphs populate SkillGraphRegistry; stance graphs populate StanceGraphRegistry. Add a system that inserts entries keyed by the loaded AnimGraph.id once each handle resolves. Register both resources in AnimationAssetPlugin.

## Inputs

- None specified.

## Expected Output

- `Headless unit/integration test asserts hit (known id) and miss (unknown id -> None) for both registries`
- `cargo test green`

## Verification

cargo test --test anim_registry
