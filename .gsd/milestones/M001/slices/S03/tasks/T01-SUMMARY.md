---
id: T01
parent: S03
milestone: M001
key_files:
  - src/animation/validation.rs
  - src/animation/mod.rs
  - tests/anim_validation.rs
key_decisions:
  - Used a report-first validator API that accumulates typed diagnostics instead of short-circuiting on the first failure.
  - Kept validation catalogs adapter-owned and deterministic via `BTreeSet` surfaces so `src/animation` stays domain-agnostic.
duration: 
verification_result: passed
completed_at: 2026-05-18T21:12:24.788Z
blocker_discovered: false
---

# T01: Added a generic animation graph validator with typed diagnostics, report/blocking APIs, and focused integration coverage for valid and broken graph+clip combinations.

**Added a generic animation graph validator with typed diagnostics, report/blocking APIs, and focused integration coverage for valid and broken graph+clip combinations.**

## What Happened

Added `src/animation/validation.rs` as a pure animation-domain seam that validates typed `AnimGraph` + `Clip` pairs against adapter-supplied catalogs without importing combat or Digimon internals. The new surface includes deterministic `AnimationValidationCatalogs`, typed severity/check/reason/context diagnostics, an aggregate `AnimationValidationReport`, a blocking wrapper, and a headless-friendly `AnimationValidationState` resource that future loader/runtime work can inspect for ready vs failed outcomes. The validator now checks graph clip-range presence, entry-node existence, node frame ordering and bounds, transition source/target references, recursive predicate references, and command status/particle/param references. Added `tests/anim_validation.rs` with in-memory fixtures that prove valid graphs pass and broken graphs return multiple typed diagnostics rather than panicking or collapsing into string-only errors.

## Verification

Verified the new public validator surface with `cargo test --test anim_validation`, covering a valid in-memory graph plus negative cases for missing clip ranges, missing entry/transition nodes, bad frame bounds, and unresolved param/status/particle references. The focused test target passed cleanly after tightening one fixture to actually exceed `total_frames`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test anim_validation` | 0 | ✅ pass | 410ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/animation/validation.rs`
- `src/animation/mod.rs`
- `tests/anim_validation.rs`
