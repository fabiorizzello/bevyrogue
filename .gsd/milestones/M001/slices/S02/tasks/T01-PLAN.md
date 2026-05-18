---
estimated_steps: 10
estimated_files: 3
skills_used: []
---

# T01: Define generic Clip schema and direct parse tests

Expected executor skills: bevy, rust-best-practices, rust-testing, tdd, verify-before-complete.

Why: S02 needs a reusable, typed animation-geometry schema under the existing src/animation boundary before any real asset or Bevy loader can depend on it. This closes the contract portion of R003 without coupling to Digimon/gameplay internals.

Do:
1. Add src/animation/clip.rs with Asset + TypePath + Serialize + Deserialize types for Clip, ClipMeta, FrameSize, and ClipRange.
2. Use serde deny_unknown_fields on authored structs so unexpected RON shape changes fail loudly.
3. Store clip ranges in BTreeMap<String, ClipRange> for deterministic validation/debug output.
4. Preserve inclusive range semantics with fields such as start and end, and add small helpers only if useful for tests or S03.
5. Export the module/types from src/animation/mod.rs beside the existing AnimGraph exports.
6. Add tests/clip_parse.rs with an inline valid RON parse test and negative parse tests for unknown fields or malformed range shape.

Done when: cargo test --test clip_parse proves the schema accepts the intended shape, rejects schema drift, and exposes types through bevyrogue::animation.

## Inputs

- `src/animation/mod.rs`
- `src/animation/anim_graph.rs`
- `tests/anim_graph_parse.rs`

## Expected Output

- `src/animation/clip.rs`
- `src/animation/mod.rs`
- `tests/clip_parse.rs`

## Verification

cargo test --test clip_parse

## Observability Impact

Improves failure diagnostics by making invalid clip.ron schema values fail at serde/RON parse boundaries with typed test coverage before runtime loading.
