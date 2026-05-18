# S02: Clip schema and lossless geometry loading

**Goal:** Add a generic typed clip.ron schema and Bevy asset-loading path, author Agumon clip geometry from the existing atlas JSON, and prove lossless parity between the RON asset and assets/digimon/agumon_atlas.json in headless tests.
**Demo:** `cargo test` loads Agumon `clip.ron` as a typed asset and proves geometry parity with the source atlas data.

## Must-Haves

- Must-haves:
- Clip schema lives under the existing generic src/animation boundary created by S01, not under Digimon/gameplay-specific code.
- assets/digimon/agumon/clip.ron preserves Agumon frame geometry exactly: frame size 557x561, 10 columns, 10 rows, 95 total frames, and all eight inclusive animation ranges from assets/digimon/agumon_atlas.json.
- Tests parse the schema directly, prove malformed/unknown clip fields fail, prove source JSON count semantics match inclusive ranges, and prove clip.ron loads as a typed Bevy asset before readiness flips.
- Requirement impact:
- Requirements touched: R003 directly; R008 as a supporting headless-first constraint.
- Re-verify: cargo test for clip parse, geometry parity, typed asset loading, and the broader test suite.
- Decisions revisited: none; plan follows D001 one cohesive animation module boundary and D004 boot/reload readiness semantics.
- Negative tests:
- Unknown RON fields or malformed geometry must fail during direct parse tests.
- The parity test must assert JSON count equals end_index - start_index + 1 so source drift or off-by-one errors fail loudly.
- The asset-load test must assert readiness does not become true before the Clip asset is readable from Assets<Clip>.

## Proof Level

- This slice proves: Contract plus headless integration proof. Real runtime required: no. Human/UAT required: no. Verification commands: cargo test --test clip_parse; cargo test --test clip_geometry_parity; cargo test --test clip_asset; cargo test.

## Integration Closure

Upstream surfaces consumed: existing S01 src/animation module, src/animation/plugin.rs AnimationAssetPlugin, assets/digimon/agumon_atlas.json, and assets/digimon/agumon/anim_graph.ron path conventions. New wiring introduced: AnimationAssetPlugin registers RonAssetPlugin<Clip>, loads configured clip paths, tracks typed load readiness, and exports Clip types for S03 validators. Remaining milestone work: S03 validator cross-asset checks and S04 roster-wide assets plus windowed hot-reload UAT.

## Verification

- Runtime signals: typed asset load requests, load/modify events, and ready-state logs for clip assets, mirroring S01 graph diagnostics. Inspection surfaces: headless tests inspect AnimationClipLoadState, AnimationClipHandles, and Assets<Clip>. Failure visibility: parse errors surface through RON/serde test failures; asset readiness tests localize premature ready flags or unreadable handles. Redaction constraints: none; assets contain geometry only.

## Tasks

- [x] **T01: Define generic Clip schema and direct parse tests** `est:1h`
  Expected executor skills: bevy, rust-best-practices, rust-testing, tdd, verify-before-complete.
  - Files: `src/animation/clip.rs`, `src/animation/mod.rs`, `tests/clip_parse.rs`
  - Verify: cargo test --test clip_parse

- [x] **T02: Author Agumon clip.ron and geometry parity test** `est:1h`
  Expected executor skills: rust-testing, tdd, verify-before-complete.
  - Files: `assets/digimon/agumon/clip.ron`, `tests/clip_geometry_parity.rs`
  - Verify: cargo test --test clip_geometry_parity

- [x] **T03: Wire Clip into AnimationAssetPlugin and asset-load smoke test** `est:1h 15m`
  Expected executor skills: bevy, rust-best-practices, rust-testing, verify-before-complete, observability.
  - Files: `src/animation/plugin.rs`, `tests/clip_asset.rs`
  - Verify: cargo test --test clip_asset

## Files Likely Touched

- src/animation/clip.rs
- src/animation/mod.rs
- tests/clip_parse.rs
- assets/digimon/agumon/clip.ron
- tests/clip_geometry_parity.rs
- src/animation/plugin.rs
- tests/clip_asset.rs
