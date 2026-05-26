---
id: T03
parent: S01
milestone: M005
key_files:
  - src/animation/reaction.rs
  - src/windowed/render.rs
  - tests/animation/stance_reaction_mapping.rs
key_decisions:
  - Verification-only task: no source changes were needed since all three contract commands were already green.
  - Confirmed no-leak by direct grep in addition to the green default cargo test.
duration: 
verification_result: passed
completed_at: 2026-05-26T08:19:12.672Z
blocker_discovered: false
---

# T03: Verified headless suite, windowed suite, and windowed build all green after the reaction wiring, with no windowed dep leak into the headless lib

**Verified headless suite, windowed suite, and windowed build all green after the reaction wiring, with no windowed dep leak into the headless lib**

## What Happened

Regression sweep over the three milestone-contract commands after the S01 reaction wiring (T01 lib mapping + T02 windowed bridge). Ran cargo test (headless): exit 0, full integration suite green including the four new stance_reaction_mapping tests (death_maps_to_death_node, death_takes_precedence_over_hurt_in_batch, hit_maps_to_hurt_node, non_reaction_kinds_and_empty_batch_map_to_none) confirmed present and passing within the 119-test animation scope. Ran cargo test --features windowed: exit 0, all 23 test binaries ok (0 failed). Ran cargo build --features windowed: exit 0. Per K001 the windowed binary was built but never executed. Confirmed the no-leak requirement (R002/R005): src/animation/reaction.rs contains only a doc-comment mention of windowed/bevy, no cfg(feature windowed) gates, no winit/wgpu/egui symbols, and the green default cargo test proves it compiles into the headless lib that tests/ link against. No regressions; new tests left intact (not weakened).

## Verification

Three commands run via gsd_exec, all exit 0: cargo test (headless suite green, 4 new stance_reaction_mapping tests passing), cargo test --features windowed (all 23 binaries ok, 0 failed), cargo build --features windowed (Finished). Direct grep of reaction.rs confirmed no windowed/render symbol dependency (only a doc comment). Windowed binary not executed (K001).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | pass | 7112ms |
| 2 | `cargo test --test animation` | 0 | pass | 185ms |
| 3 | `cargo test --features windowed` | 0 | pass | 10149ms |
| 4 | `cargo build --features windowed` | 0 | pass | 209ms |

## Deviations

None.

## Known Issues

Pre-existing non-fatal compiler warning under --features windowed: unused import BeatEdge. Not introduced by this task and does not affect build/test exit codes.

## Files Created/Modified

- `src/animation/reaction.rs`
- `src/windowed/render.rs`
- `tests/animation/stance_reaction_mapping.rs`
