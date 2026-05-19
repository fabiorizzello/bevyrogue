---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T01: Closed-enum schema extensions + atomic id/asset/test migration

Why: every later task and slice keys off AnimGraph.id (registry, D004/MEM024) and the inert cue seam (S02). Schema-first is lowest risk and unblocks all. Do: In src/animation/anim_graph.rs add a required `id: AnimGraphId` field to AnimGraph (closed transparent newtype mirroring ClipId/NodeId; keep #[serde(deny_unknown_fields)]). Add `cues: Vec<FrameCue>` with #[serde(default)] to AnimNode. Add `FrameCue { at: u32, command: FrameCueCommand }` where FrameCueCommand is a CLOSED enum carrying either a presentation Command or `ReleaseKernelCue` (no untagged, no Custom(String); follow the existing closed-enum convention exactly — MEM023/D003). Add `ReleaseKernelCue` (no id, no number — D003). Add a `KernelCue` variant to the closed Predicate enum (inert in S01, consumed S02). Re-export new public types where ClipId/NodeId are re-exported. Add `id` to assets/digimon/agumon/anim_graph.ron and assets/digimon/renamon/anim_graph.ron (do NOT yet remove EmitDamage — that is T02). Atomically update every test that constructs an AnimGraph literal or asserts graph structure to include the new id field: tests/anim_graph_asset.rs, tests/anim_graph_parse.rs, tests/anim_validation.rs. Add a round-trip parse test proving (a) a cues-absent graph still loads via #[serde(default)], (b) a graph with cues:[FrameCue(at:N, command: <ReleaseKernelCue and a presentation Command>)] and a transition with when: KernelCue parses through the closed enum with no untagged fallback, (c) an unknown cue/predicate variant is rejected. Keep each touched/new source file under the 500-LOC cap (source_file_loc_limit). Done when: cargo test passes headless with id required everywhere and the new round-trip seam test green. Decisions: D040 (id required, not serde default), D041/D042 context. Negative tests (Q7): unknown FrameCue command variant and unknown Predicate variant must fail to parse; cues-absent RON must still succeed.

## Inputs

- `src/animation/anim_graph.rs`
- `src/animation/mod.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/anim_graph_asset.rs`
- `tests/anim_graph_parse.rs`
- `tests/anim_validation.rs`

## Expected Output

- `src/animation/anim_graph.rs`
- `src/animation/mod.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/anim_graph_asset.rs`
- `tests/anim_graph_parse.rs`
- `tests/anim_validation.rs`

## Verification

cargo test --test anim_graph_parse --test anim_graph_asset --test anim_validation
