---
id: T03
parent: S05
milestone: M006
key_files:
  - tests/windowed_only/renamon_extension_contract.rs
key_decisions:
  - Kept the Renamon extension contract semantic rather than numeric/format-sensitive by asserting ownership tokens, forbidden engine tokens, and the multi-presentation lookup seam instead of exact frame values.
duration: 
verification_result: passed
completed_at: 2026-05-26T18:02:29.861Z
blocker_discovered: false
---

# T03: Hardened the Renamon windowed source-contract test to enforce zero-engine-edit ownership boundaries and multi-presentation registry seams.

**Hardened the Renamon windowed source-contract test to enforce zero-engine-edit ownership boundaries and multi-presentation registry seams.**

## What Happened

Updated `tests/windowed_only/renamon_extension_contract.rs` to assert the exact structural promises for S05 without depending on `.git`, `.gsd`, or brittle formatting. The contract now forbids Renamon-specific identifiers in `src/windowed/render.rs` and `src/windowed/mod.rs`, pins the generic multi-presentation seam in `render.rs` by requiring `SpritePresentationRegistry` + `presentation_entry_for_unit(...)` and forbidding single-entry lookups, requires `src/windowed/digimon/mod.rs` to declare and register `renamon`, and requires `src/windowed/digimon/renamon/mod.rs` to own stance-path registration, presentation/skill registry data, and Renamon asset ids. Asset assertions were tightened to semantic ownership checks for `stance.ron`, `clip.ron`, and `anim_graph.ron`, including the `ReleaseKernel(())` cue, while avoiding brittle frame/range pinning beyond the presence of the `all` clip range contract.

## Verification

Ran the full windowed-only integration harness with `cargo test --features windowed --test windowed_only -- --nocapture`. The entire target passed, including the hardened Renamon contract and all prior S01-S04 windowed-only source/data contracts.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only -- --nocapture` | 0 | ✅ pass | 3090ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `tests/windowed_only/renamon_extension_contract.rs`
