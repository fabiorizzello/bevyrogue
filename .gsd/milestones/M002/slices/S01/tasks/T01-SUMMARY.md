---
id: T01
parent: S01
milestone: M002
key_files:
  - src/animation/anim_graph.rs
  - src/animation/validation/predicate.rs
  - assets/digimon/agumon/anim_graph.ron
  - assets/digimon/renamon/anim_graph.ron
  - tests/anim_graph_parse.rs
  - tests/anim_graph_asset.rs
  - tests/anim_validation.rs
key_decisions:
  - Keep frame-cue vocabulary closed: presentation commands or ReleaseKernelCue only.
  - Model graph identity with a transparent `AnimGraphId` newtype rather than ad-hoc strings.
duration: 
verification_result: passed
completed_at: 2026-05-19T19:31:33.740Z
blocker_discovered: false
---

# T01: Extended the AnimGraph schema with closed ids, frame cues, and kernel-release predicates, with assets and parse/validation coverage kept green.

**Extended the AnimGraph schema with closed ids, frame cues, and kernel-release predicates, with assets and parse/validation coverage kept green.**

## What Happened

Added `AnimGraphId` as a required field on `AnimGraph`, introduced `FrameCue`, `FrameCueCommand`, and `ReleaseKernelCue`, and extended the closed `Predicate` enum with `KernelCue`. Updated the production Agumon and Renamon anim graph assets plus validation fixtures so the schema change remained atomic and typed. The final closeout rerun kept the schema parse and validation surfaces green after the later Agumon asset parity remediation.

## Verification

Fresh `cargo nextest run --profile agent` passed after the final asset/test updates, including the `anim_graph_parse`, `anim_graph_asset`, and `anim_validation` suites covered by this task.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo nextest run --profile agent` | 0 | ✅ pass | 7700ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/animation/anim_graph.rs`
- `src/animation/validation/predicate.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/anim_graph_parse.rs`
- `tests/anim_graph_asset.rs`
- `tests/anim_validation.rs`
