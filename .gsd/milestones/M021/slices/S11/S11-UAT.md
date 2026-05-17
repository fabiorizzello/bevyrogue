# S11: UI and AI consumers via SkillCtx Preview — UAT

**Milestone:** M021
**Written:** 2026-05-17T08:15:13.642Z

# S11: UI and AI consumers via SkillCtx Preview — UAT

**Milestone:** M021
**Written:** 2026-05-17

## UAT Type

- UAT mode: mixed
- Why this mode is sufficient: The slice is validated by focused integration tests for preview semantics plus compile-time coverage for both headless and windowed entrypoints, which is enough to prove the shared preview contract without a manual gameplay walkthrough.

## Preconditions

- Repository is at the verified S11 state.
- Cargo toolchain is available.
- Windowed feature dependencies are present for the windowed compile/test pass.

## Smoke Test

Run the preview and AI test suites plus both `cargo check` modes.

## Test Cases

### 1. Shared preview stream stays non-mutating

1. Run `cargo test --test skill_preview -- --nocapture`.
2. Observe the preview-stream assertion results.
3. **Expected:** Preview intent streams match execute-mode shape, and combat state remains unchanged after preview execution.

### 2. Windowed damage preview uses the shared cache bridge

1. Run `cargo test --features windowed --test windowed_preview_cache -- --nocapture`.
2. Observe the cache refresh and mismatch behavior assertions.
3. **Expected:** The cached preview summary mirrors the shared preview-stream result, and the cache does not update when preview refresh is unavailable.

### 3. Enemy AI ranks by preview score and still falls back deterministically

1. Run `cargo test --test enemy_ai -- --nocapture`.
2. Run `cargo test --test enemy_ai_preview -- --nocapture`.
3. **Expected:** Preview-based scoring selects the best candidate when preview data exists, and the legacy fallback path still behaves deterministically when it does not.

## Edge Cases

### Missing or unavailable preview data

1. Exercise a candidate path with no usable preview data.
2. **Expected:** The system returns no mutating preview result and the AI falls back instead of panicking or inventing a score.

### Preview cache context mismatch

1. Change the active actor, skill kind, or target after a cache refresh.
2. **Expected:** The UI rejects the stale cached summary and waits for a matching refresh.

## Failure Signals

- Preview tests report changed HP, toughness, SP, ult charge, or intent queue state.
- Windowed preview cache diverges from the shared preview-stream summary.
- Enemy AI preview tests stop selecting the highest-preview candidate or lose deterministic fallback behavior.
- `cargo check` fails in either headless or windowed mode.

## Not Proven By This UAT

- Full human-driven gameplay feel in the windowed client.
- Long-run performance characteristics under extended combat sessions.
- Any future preview consumers beyond the current UI and enemy AI wiring.

## Notes for Tester

This slice is intended to prove that the same preview stream now drives both consumer surfaces. If anything breaks, first inspect the shared preview seam, then the windowed cache bridge, then the enemy-AI request/resolver bridge.
