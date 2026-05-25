---
id: T04
parent: S02
milestone: M004
key_files:
  - tests/animation/render_no_vfx_kind_guard.rs
  - tests/animation.rs
key_decisions:
  - Guard strips //-line-comment tails before asserting, so the test targets the deleted code construct (enum + string-match dispatch) rather than failing on T03's legitimate historical comments that mention VfxParticleKind to document the replacement.
duration: 
verification_result: passed
completed_at: 2026-05-25T11:09:33.632Z
blocker_discovered: false
---

# T04: Added a headless compile-time grep-guard test asserting VfxParticleKind/kind_from_name/vfx_particle_kind no longer appear in render.rs code

**Added a headless compile-time grep-guard test asserting VfxParticleKind/kind_from_name/vfx_particle_kind no longer appear in render.rs code**

## What Happened

Created tests/animation/render_no_vfx_kind_guard.rs and registered it in tests/animation.rs via a #[path] mod line. The test embeds src/windowed/render.rs at compile time with include_str! (feature-independent, runs in the headless `cargo test --test animation` lane) and asserts none of the three forbidden VFX-kind identifiers survive in the actual code. Key deviation from the literal plan: a raw substring check would have failed. T03 left five historical comments in render.rs (lines 319/688/741/855/867) that still mention VfxParticleKind to explain what replaced the deleted dispatch. The grep-guard's intent is to prove the code construct (the enum + string-match) is gone, not to forbid explanatory prose. So the test strips //-comment tails per line before asserting, targeting code only. On failure the assertion names exactly which identifier survived.

## Verification

Ran `cargo test --test animation`: 101 passed, 0 failed, including render_no_vfx_kind_guard::render_rs_has_no_vfx_kind_dispatch. This transitively certifies T03 removed the enum and string-match dispatch from render.rs code. Confirmed via grep that the only remaining textual occurrences of the forbidden identifiers are historical comments, which the test deliberately excludes.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation` | 0 | pass | 594ms |

## Deviations

Plan specified a plain substring assertion on the raw source. Because T03 left 5 historical comments referencing VfxParticleKind, a raw check would false-positive; the test strips line comments first to honor the actual intent (no code construct) without forcing prose churn.

## Known Issues

none

## Files Created/Modified

- `tests/animation/render_no_vfx_kind_guard.rs`
- `tests/animation.rs`
