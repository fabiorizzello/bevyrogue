---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T03: Extract a generic reusable warn-once dedup util

Promote S06's inline warn-once dedup (currently a Local<HashSet<AssetId<AnimGraph>>> in src/animation/registry.rs, specific to AssetId<AnimGraph>) into a generic dedup util keyed by an arbitrary id type, exported from the lib so both the animation and windowed consumers reuse it instead of re-implementing. Identical dedup behavior; repoint the existing registry.rs call site to the new util. This is the shared helper S08/S11/S12/S13/S14 refer to.

## Inputs

- `src/animation/registry.rs`

## Expected Output

- `A generic warn-once dedup util in the lib keyed by an arbitrary id type`
- `registry.rs repointed to the new util with identical behavior`
- `downstream slices reuse the util rather than re-implementing the pattern`

## Verification

cargo test (headless green); cargo test --features windowed --test windowed_only (green)

## Observability Impact

Centralizes warn-once dedup so all spawn/cue/verb-miss diagnostics share one consistent, testable surface.
