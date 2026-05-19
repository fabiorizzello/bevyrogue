---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T01: Closed-enum schema extensions + atomic id/asset/test migration

In src/animation/anim_graph.rs add a required `id: AnimGraphId` field to AnimGraph (closed transparent newtype). Add `cues: Vec<FrameCue>` with #[serde(default)] to AnimNode. Add `FrameCue { at: u32, command: FrameCueCommand }` where FrameCueCommand is a CLOSED enum carrying either a presentation Command or ReleaseKernelCue. Add ReleaseKernelCue. Add KernelCue variant to the closed Predicate enum. Update all test files and RON assets atomically.

## Inputs

- None specified.

## Expected Output

- `cargo test passes headless with id required everywhere`
- `New round-trip seam test green: cues-absent graph loads, graph with cues parses, unknown variant rejected`

## Verification

cargo test --test anim_graph_parse --test anim_graph_asset --test anim_validation
