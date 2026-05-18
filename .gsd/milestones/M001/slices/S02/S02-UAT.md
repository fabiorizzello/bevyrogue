# S02: S02 — UAT

**Milestone:** M001
**Written:** 2026-05-18T21:01:07.068Z

# S02: S02 — UAT

**Milestone:** M001
**Written:** 2025-02-14

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: This slice is a headless-first asset-schema and loader change whose success criteria are fully observable through deterministic test fixtures and typed Bevy asset-loading behavior.

## Preconditions

- Repository is at the S02 closeout state.
- Rust toolchain and project dependencies are installed.
- `assets/digimon/agumon_atlas.json` and `assets/digimon/agumon/clip.ron` are present.

## Smoke Test

Run `cargo test --test clip_asset` and confirm `agumon_clip_loads_as_typed_asset_before_ready_flips` passes.

## Test Cases

### 1. Strict clip schema parsing

1. Run `cargo test --test clip_parse`.
2. Observe the valid-schema test.
3. Observe the malformed-range and unknown-field rejection tests.
4. **Expected:** All three tests pass, proving valid `clip.ron` parses into the typed schema and malformed/unknown fields fail loudly.

### 2. Geometry parity against source atlas

1. Run `cargo test --test clip_geometry_parity`.
2. Let the test load `assets/digimon/agumon/clip.ron` and `assets/digimon/agumon_atlas.json`.
3. **Expected:** The test passes only if frame size, grid dimensions, total frames, all eight animation ranges, and inclusive frame counts exactly match the source atlas data.

### 3. Typed Bevy asset loading and readiness gating

1. Run `cargo test --test clip_asset`.
2. Let Bevy load the Agumon clip through `AnimationAssetPlugin`.
3. **Expected:** The test passes only if the clip becomes readable as a typed `Clip` asset before load readiness flips to true.

## Edge Cases

### Unknown or malformed clip fields

1. Run `cargo test --test clip_parse`.
2. **Expected:** Inputs with unknown fields or malformed range geometry fail parse tests instead of being silently accepted.

### Inclusive atlas count drift

1. Run `cargo test --test clip_geometry_parity` after any atlas or clip range edit.
2. **Expected:** If JSON range counts no longer equal `end_index - start_index + 1`, the test fails immediately and surfaces the off-by-one drift.

## Failure Signals

- Any failure in `clip_parse`, `clip_geometry_parity`, or `clip_asset`.
- `cargo test` regressions in existing animation graph tests.
- Readiness becoming true before the clip handle resolves to a typed `Assets<Clip>` entry.

## Not Proven By This UAT

- Runtime playback correctness or visual animation behavior.
- Cross-asset validation between graphs and clips, roster-wide assets, or windowed hot reload; those belong to later slices.

## Notes for Tester

This slice intentionally uses artifact-driven headless proof only. The authoritative geometry source remains `assets/digimon/agumon_atlas.json`, and the parity test is the fastest place to check suspected authored-data drift.
