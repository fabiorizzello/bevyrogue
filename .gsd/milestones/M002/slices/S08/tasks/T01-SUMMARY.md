---
id: T01
parent: S08
milestone: M002
key_files:
  - src/animation/anim_graph.rs
  - src/animation/player.rs
  - tests/animation.rs
  - tests/animation/anim_graph_input_purity.rs
key_decisions:
  - Use a closed `AnimGraphRole` enum plus a read-only `AnimGraphInput` set as the explicit graph-input seam for R009.
  - Keep legacy player entrypoints as thin wrappers over `AnimGraphInput::default()` so existing callers retain behavior while purity-aware tests target the explicit API.
duration: 
verification_result: passed
completed_at: 2026-05-21T22:09:22.179Z
blocker_discovered: false
---

# T01: Added a closed `AnimGraphRole`/`AnimGraphInput` seam plus animation tests proving player evaluation stays typed, read-only, and world-free.

**Added a closed `AnimGraphRole`/`AnimGraphInput` seam plus animation tests proving player evaluation stays typed, read-only, and world-free.**

## What Happened

`src/animation/anim_graph.rs` now defines the closed `AnimGraphRole` vocabulary (`Caster`, `PrimaryTarget`, `AdjacentLeftTarget`, `AdjacentRightTarget`) and the small read-only `AnimGraphInput` surface as a `BTreeSet` wrapper with `new`/`contains` helpers. `src/animation/player.rs` keeps the FSM pure by threading `&AnimGraphInput` through `advance_with_input`, `advance_result_with_input`, and transition selection while preserving the legacy no-input entrypoints as default-input wrappers, so no world handle or mutable graph context is involved in evaluation. `tests/animation/anim_graph_input_purity.rs` provides the executable R009 proof: typed-role RON parsing succeeds for the closed vocabulary, stringly/custom/unknown roles fail deserialization, the player can advance from explicit typed input without mutating that input, and the legacy `advance` path remains behaviorally equivalent to the default typed-input path. This execution pass found the implementation already present in the working tree but missing its canonical GSD completion artifact, so the task was freshly verified and recorded here.

## Verification

Ran the task verification command `cargo test --test animation anim_graph_input_purity`. The focused animation harness passed all four purity tests: closed typed-role parsing, rejection of stringly/unknown roles, explicit read-only typed-input player evaluation, and legacy/default-input behavioral parity.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation anim_graph_input_purity` | 0 | ✅ pass | 165ms |

## Deviations

Implementation was already present in the working tree; this pass did not need code edits and instead supplied the missing canonical task completion artifact after fresh verification.

## Known Issues

Slice S08 still has other pending tasks, so the slice summary artifact is not created by this task completion alone.

## Files Created/Modified

- `src/animation/anim_graph.rs`
- `src/animation/player.rs`
- `tests/animation.rs`
- `tests/animation/anim_graph_input_purity.rs`
