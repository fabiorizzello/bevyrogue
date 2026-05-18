---
estimated_steps: 10
estimated_files: 4
skills_used: []
---

# T01: Define closed AnimGraph schema and parse contracts

Skills expected in executor frontmatter: bevy, rust-best-practices, rust-testing, api-design, design-an-interface, tdd, verify-before-complete.

Why: S01's highest-risk boundary is the schema contract, not Bevy asset IO. The project needs a cohesive generic `animation` module with closed-vocabulary types so invalid graph data fails before later runtime/validator work can consume it.

Do:
1. Add `pub mod animation;` to `src/lib.rs`.
2. Create `src/animation/mod.rs` and `src/animation/anim_graph.rs`.
3. In `anim_graph.rs`, define serde-derived typed schema for `AnimGraph`, nodes, transitions/edges, priorities, frame ranges, playback modifiers, commands, predicates, parameter references, and target shapes. Keep vocabulary closed: do not add `Custom(String)`, free-form command strings, free-form predicate kinds, or untyped target shape maps.
4. Keep the module generic. It may represent skill ids, clip names, node ids, and parameter names as data strings/newtypes, but it must not import Digimon/gameplay modules or hardcode Agumon behavior.
5. Add focused tests in `tests/anim_graph_parse.rs` that parse an inline valid RON graph and assert important fields/variants, plus negative fixtures for an unknown command, unknown predicate, and unknown target shape.
6. If the final enum shape differs from the draft vocabulary, document that choice in code comments near the enum so later S03/S04 work can extend deliberately.

Done when: The schema compiles, public exports are reachable through `bevyrogue::animation`, valid inline RON parses into typed variants, and invalid vocabulary RON fails deterministically through `ron::from_str::<AnimGraph>`.

## Inputs

- `src/lib.rs`
- `Cargo.toml`
- `docs/future_design_draft/02-02b_animation_fsm.md`
- `docs/M022/slices/S01/S01-PLAN.md`
- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon_atlas.json`

## Expected Output

- `src/lib.rs`
- `src/animation/mod.rs`
- `src/animation/anim_graph.rs`
- `tests/anim_graph_parse.rs`

## Verification

cargo test --test anim_graph_parse

## Observability Impact

Negative-test observability: parse failures are surfaced as concrete RON/serde errors in a dedicated test. Failure modes covered: malformed/unknown enum values for commands, predicates, and target shapes.
