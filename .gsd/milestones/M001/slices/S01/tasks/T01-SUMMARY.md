---
id: T01
parent: S01
milestone: M001
key_files:
  - src/lib.rs
  - src/animation/mod.rs
  - src/animation/anim_graph.rs
  - tests/anim_graph_parse.rs
key_decisions:
  - Kept command, predicate, playback, and target-shape vocabularies closed with serde enums while leaving game/domain identifiers as string newtypes.
  - Used explicit `TransitionTarget::Node(...) | Exit` rather than an untagged target representation so invalid or ambiguous RON fails deterministically at parse time.
duration: 
verification_result: passed
completed_at: 2026-05-18T20:44:51.907Z
blocker_discovered: false
---

# T01: Added the public animation module seam plus a closed typed AnimGraph schema with passing parse-contract tests.

**Added the public animation module seam plus a closed typed AnimGraph schema with passing parse-contract tests.**

## What Happened

Added a new public `bevyrogue::animation` module and implemented a generic typed `AnimGraph` schema with serde-derived newtypes and closed enums for nodes, transitions, priorities, frame ranges, playback modifiers, commands, predicates, parameter references, and target shapes. The schema stays gameplay-agnostic by representing ids and keys as data strings/newtypes while rejecting out-of-vocabulary command, predicate, and target-shape variants at RON parse time. Added focused integration tests that parse a valid inline graph into typed variants and assert deterministic failure for unknown vocabulary.

## Verification

Verified the new schema by running the dedicated parse-contract integration test and a broader cargo check. `cargo test --test anim_graph_parse` passed with 5/5 tests, covering valid typed parsing plus unknown command, predicate, and target-shape rejection. `cargo check` passed to confirm the new public module compiles cleanly across the crate.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test anim_graph_parse` | 0 | ✅ pass | 3865ms |
| 2 | `cargo check` | 0 | ✅ pass | 3231ms |

## Deviations

Used an explicit `TransitionTarget::Node(NodeId) | Exit` enum instead of the draft’s implicit string-or-Exit target form because the untagged RON encoding was ambiguous in practice; the code comment in `src/animation/anim_graph.rs` documents this narrower closed shape.

## Known Issues

None.

## Files Created/Modified

- `src/lib.rs`
- `src/animation/mod.rs`
- `src/animation/anim_graph.rs`
- `tests/anim_graph_parse.rs`
