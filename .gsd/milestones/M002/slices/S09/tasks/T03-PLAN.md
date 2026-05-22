---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Skill-graph mapping extensibility + stance-entry evidence test

Why: R005's 1:1 skill-id↔AnimGraph mapping and stance-return-to-idle must have an executable closeout proof; the registry already supports many ids and only the call-site constant is hardcoded, so the extensibility seam is a data/lookup change, not a rewrite (design-an-interface: deep SkillGraphRegistry, shallow hardcoded leak). Skills: tdd, design-an-interface. Do: add `tests/animation/skill_graph_mapping_extensibility.rs` (register in `tests/animation.rs` via `#[path]`), exercising the lib `bevyrogue::animation::registry` APIs: insert two or more distinct non-default `AnimGraphId`s into a `SkillGraphRegistry` with corresponding `AnimGraph` handles in an `Assets<AnimGraph>`, assert each `resolve_snapshot` returns its own graph with `source == Registry` and matching `requested_id` (proves 1:1, no hardcoded single-id constraint at the registry layer); assert `resolve_snapshot_or_instant_fallback` for an unregistered id returns `source == InstantFallback` and records a diagnostic; and for stance return, assert a stance-graph snapshot exposes a non-empty `graph().entry` (the node `return_to_idle` resets the player to), documenting that the binary-side `return_to_idle` consumes exactly this entry. Done-when: `cargo test --test animation skill_graph_mapping_extensibility` green. Note in a top comment that `return_to_idle` itself lives in the windowed binary crate (`src/windowed/render.rs`) and is therefore cited in the boundary map rather than directly callable from integration tests.

## Inputs

- `src/animation/registry.rs`
- `src/animation/anim_graph.rs`
- `src/windowed/render.rs`

## Expected Output

- `tests/animation/skill_graph_mapping_extensibility.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation skill_graph_mapping_extensibility

## Observability Impact

Asserts the InstantFallback diagnostic path is recorded on an unregistered id lookup.
