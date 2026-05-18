---
id: T01
parent: S02
milestone: M001
key_files:
  - src/animation/clip.rs
  - src/animation/mod.rs
  - tests/clip_parse.rs
key_decisions:
  - Use `BTreeMap<String, ClipRange>` for authored clip ranges and apply `serde(deny_unknown_fields)` to clip schema structs to keep parse failures deterministic and loud.
duration: 
verification_result: passed
completed_at: 2026-05-18T20:54:40.496Z
blocker_discovered: false
---

# T01: Added a typed `Clip` animation-geometry schema with strict RON parse tests and `bevyrogue::animation` exports.

**Added a typed `Clip` animation-geometry schema with strict RON parse tests and `bevyrogue::animation` exports.**

## What Happened

Created `src/animation/clip.rs` with generic Bevy asset types `Clip`, `ClipMeta`, `FrameSize`, and `ClipRange`, using `serde(deny_unknown_fields)` on authored structs so schema drift fails at parse time. Stored named clip ranges in `BTreeMap<String, ClipRange>` for deterministic debug/test output and kept inclusive range semantics via `start`/`end`, plus small `len()` and `contains()` helpers that are already exercised by tests. Updated `src/animation/mod.rs` to export the new module beside the existing animation graph surface. Added `tests/clip_parse.rs` with one valid inline RON fixture and two negative cases that reject unknown metadata fields and malformed range shapes while proving the types are reachable through `bevyrogue::animation`.

## Verification

Ran `cargo test --test clip_parse`, which compiled the crate and passed all three clip parse tests: valid schema parsing, unknown-field rejection, and malformed-range rejection.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test clip_parse` | 0 | ✅ pass | 5210ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/animation/clip.rs`
- `src/animation/mod.rs`
- `tests/clip_parse.rs`
