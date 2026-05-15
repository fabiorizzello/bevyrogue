---
id: T01
parent: S02
milestone: M021
key_files:
  - src/combat/api/timeline.rs
  - src/combat/api/mod.rs
  - tests/timeline_validate_typo.rs
key_decisions:
  - SelectorCtx/CueCtx use a generic `S` parameter (default `()`) to keep timeline.rs import-free from Bevy — the runner/skill_ctx will supply a concrete type in S02 T02+.
  - validate_timeline_refs collects all errors before returning (not fail-fast) to give complete diagnostics on a single pass.
duration: 
verification_result: passed
completed_at: 2026-05-15T07:38:18.702Z
blocker_discovered: false
---

# T01: Ported Timeline FSM data types + validate_timeline_refs from spike into src/combat/api/timeline.rs with 4 inline unit tests and 1 integration test — all green.

**Ported Timeline FSM data types + validate_timeline_refs from spike into src/combat/api/timeline.rs with 4 inline unit tests and 1 integration test — all green.**

## What Happened

Read the spike lib.rs and production registry.rs/skill_ctx.rs/intent.rs to understand the types. Created src/combat/api/timeline.rs with: BeatId type alias, Presentation, BeatKind (Cast/Phase/Impact/Aftermath/Loop{body,exit_when}), Beat, BeatEdge, CompiledTimeline, BeatEvent (carrying CastId + hop_index), generic SelectorCtx<'a,S> and CueCtx<'a,S> (no Bevy import in this file), ValidationError, and validate_timeline_refs which recursively validates hook/selector IDs on beats, exit_when predicates on Loop beats, and gate predicates on edges. Added pub mod timeline and re-exported all public types in src/combat/api/mod.rs. Added 4 inline #[cfg(test)] cases covering: clean timeline Ok, missing hook Err, missing edge gate Err with 'edge from->to' site, missing loop exit_when Err with 'beat id' site. Created tests/timeline_validate_typo.rs as the integration-level demo gate 2 fixture. cargo check clean; all 5 tests green.

## Verification

cargo check (clean, no new errors); cargo test --test timeline_validate_typo (1/1 pass); cargo test --lib combat::api::timeline (4/4 pass); rg 'pub fn validate_timeline_refs' src/combat/api/timeline.rs (found).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 4040ms |
| 2 | `cargo test --test timeline_validate_typo` | 0 | pass — 1 test | 4690ms |
| 3 | `cargo test --lib combat::api::timeline` | 0 | pass — 4 tests | 1670ms |
| 4 | `rg 'pub fn validate_timeline_refs' src/combat/api/timeline.rs` | 0 | pass | 50ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/api/timeline.rs`
- `src/combat/api/mod.rs`
- `tests/timeline_validate_typo.rs`
