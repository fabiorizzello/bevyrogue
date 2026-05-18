---
id: T02
parent: S02
milestone: M001
key_files:
  - assets/digimon/agumon/clip.ron
  - tests/clip_geometry_parity.rs
key_decisions:
  - Keep `assets/digimon/agumon/clip.ron` as the typed authored asset and prove parity against `assets/digimon/agumon_atlas.json` in a drift-detection integration test using local JSON-only structs.
duration: 
verification_result: passed
completed_at: 2026-05-18T20:56:42.435Z
blocker_discovered: false
---

# T02: Added the authored Agumon `clip.ron` asset and a geometry parity test that proves it stays lossless against the source atlas JSON.

**Added the authored Agumon `clip.ron` asset and a geometry parity test that proves it stays lossless against the source atlas JSON.**

## What Happened

Authored `assets/digimon/agumon/clip.ron` with the exact Agumon sprite-sheet geometry and inclusive clip ranges from the authoritative atlas JSON: frame size 557x561, 10 columns, 10 rows, 95 total frames, plus the eight named ranges from `attack` through `victory`. Added `tests/clip_geometry_parity.rs`, which deserializes the new RON asset as `bevyrogue::animation::Clip` and deserializes `assets/digimon/agumon_atlas.json` into local test-only serde structs. The test asserts exact metadata parity, exact named inclusive ranges, equal clip counts across both sources, and that every JSON `count` remains consistent with `end_index - start_index + 1`, so off-by-one bugs or atlas drift fail with a targeted message.

## Verification

Ran `cargo test --test clip_geometry_parity`, which loaded the new Agumon `clip.ron` fixture through the typed `Clip` schema, parsed the authoritative `assets/digimon/agumon_atlas.json`, and passed the exact geometry, range, clip-count, and inclusive-count consistency assertions.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test clip_geometry_parity` | 0 | ✅ pass | 584ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `assets/digimon/agumon/clip.ron`
- `tests/clip_geometry_parity.rs`
