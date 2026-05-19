# S02 — Research

**Date:** 2026-05-18

## Summary

S02 is a lower-risk companion to S01: the core schema is straightforward typed geometry, but the acceptance criterion is strict lossless parity with `assets/digimon/agumon_atlas.json`. The existing atlas JSON has `meta.frame_size` `{w:557,h:561}`, `columns:10`, `rows:10`, `total_frames:95`, and 8 animations: `attack` 0-8, `block` 9-13, `death` 14-22, `heavy_attack` 23-46, `hurt` 47-53, `idle` 54-59, `skill` 60-77, `victory` 78-94.

Implement `Clip` in the same generic `src/animation/` boundary recommended for S01. `clip.ron` should be pure animation geometry, not gameplay or Digimon-specific logic. Register it through the same Bevy RON asset plugin path as other typed data, and prove parity with a headless test that parses both the generated RON and source JSON.

## Recommendation

Define `Clip`, `ClipMeta`, `FrameSize`, and `ClipRange` as serde/asset types with a deterministic map such as `BTreeMap<String, ClipRange>`. Include optional loader-side/authored convenience fields only if needed (`fps`, `loop`, `texture_path`); do not let those optional presentation defaults affect the geometry parity assertion. The parity test should compare frame size, columns, rows, total frames, and every animation's inclusive `start`/`end` range, while optionally checking `count == end - start + 1` from the JSON.

Generate or manually author `assets/digimon/agumon/clip.ron` from `assets/digimon/agumon_atlas.json` without changing names or ranges. Prefer a small test helper that deserializes the JSON into a local test-only struct rather than depending on string matching; this makes drift obvious and keeps the proof independent of formatting.

## Implementation Landscape

### Key Files

- `src/animation/mod.rs` — new generic module owner for animation schemas and plugin exports; keeps R001's cohesive module boundary intact.
- `src/animation/clip.rs` — natural home for `Clip`, `ClipMeta`, `FrameSize`, `ClipRange`, and clip-specific helpers/tests.
- `src/animation/plugin.rs` or `src/animation/mod.rs` — register `RonAssetPlugin::<Clip>::new(&["ron"])` beside `AnimGraph` registration when both slices converge.
- `src/lib.rs` — add `pub mod animation;` so integration tests and later S03 validator can import `bevyrogue::animation::clip::Clip`.
- `src/data/mod.rs` — reference pattern for `RonAssetPlugin`, `AssetServer::load`, handle resources, and `AssetEvent::LoadedWithDependencies` / `Modified` handling.
- `assets/digimon/agumon_atlas.json` — authoritative source for Agumon frame geometry parity.
- `assets/digimon/agumon/clip.ron` — new asset to create for S02; directory may not currently exist because existing sprite files are flat under `assets/digimon/`.
- `docs/future_design_draft/02-02_animation_manifest.md` — canonical `clip.ron` rationale and shape; note that `clipmontage` is superseded but `clip.ron` remains unchanged.
- `docs/M022/slices/S02/S02-PLAN.md` — prior slice plan; adapt code paths away from old `src/combat/blueprints/anim_graph/clip.rs` if using the new `src/animation` seam.
- `tests/clip_parse.rs` or `tests/clip_geometry_parity.rs` — focused integration test for typed parse and lossless geometry parity.

### Build Order

1. Add the `Clip` schema under `src/animation/clip.rs` with serde derives and Bevy asset derive as needed by `RonAssetPlugin`.
2. Write a pure parse test with inline RON before creating the real asset. This catches serde field naming/tuple-vs-map shape decisions cheaply.
3. Create `assets/digimon/agumon/clip.ron` from `agumon_atlas.json` preserving exact animation names and inclusive ranges.
4. Add the parity test: deserialize `clip.ron` as `Clip`, deserialize `agumon_atlas.json` with `serde_json`, and assert exact `frame_size`, `columns`, `rows`, `total_frames`, and all named clip ranges. Check JSON `count` matches `end_index - start_index + 1` so bad source data fails loudly.
5. Register `RonAssetPlugin::<Clip>` in the animation plugin and add a headless Bevy asset-load smoke test only if needed for slice acceptance. The geometry parity test is the core proof; Bevy lifecycle can share the plugin path with S01/S03.

### Verification Approach

- `cargo test --test clip_geometry_parity` or `cargo test clip_geometry_parity` depending on test placement.
- `cargo test` to ensure no regression in existing data and combat tests.
- `cargo check` to catch feature leakage. S02 must remain headless-first and should not require `windowed`.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Typed RON asset loading | `bevy_common_assets::ron::RonAssetPlugin` | Same loader/hot-reload behavior as existing `UnitRoster`, `SkillBook`, and `PartyConfig`. |
| JSON parity parsing | `serde_json` already in dependencies | Avoid brittle textual comparisons; compare typed geometry values. |
| Ordered deterministic maps | `BTreeMap` already used in data types | Stable test/debug output and deterministic fixture comparisons. |

## Constraints

- R003: `clip.ron` must load as a typed Bevy asset and prove geometry parity against Agumon atlas source data.
- R008: tests must be headless; no winit/wgpu/egui or windowed-only code in clip schema/loading.
- `clip.ron` is animation geometry only. Do not add gameplay commands, QTE, damage, or status references to the clip schema.
- Frame ranges in the atlas are inclusive (`start_index`, `end_index`, `count`), and the generated `ClipRange` should preserve that interpretation so S03 frame-range validation can use it directly.
- Current atlas files are in `assets/digimon/<name>_atlas.json`, while current RON combat data is in `assets/data/digimon/<name>/...`; S02's milestone context expects animation assets under `assets/digimon/<name>/clip.ron`.

## Common Pitfalls

- **Losing count semantics** — the RON schema does not need to store `count`, but the parity test should verify source `count` equals the inclusive range length for every animation.
- **Default fields breaking parity** — if `fps`, `loop`, or `texture_path` are defaulted, keep parity assertions focused on geometry and assert defaults separately.
- **Using HashMap in snapshots** — unordered maps make test output noisier. Prefer `BTreeMap<String, ClipRange>`.
- **Changing atlas names** — do not normalize `heavy_attack` or other clip keys; S03/S04 will depend on exact catalog names.
- **Putting code under an Agumon module** — only the asset content is Agumon-specific. The Rust schema and loader must stay generic for S04 roster support.

## Open Risks

- The final shared plugin shape depends on S01. If slices are implemented independently, both should converge on a single `AnimationAssetPlugin` rather than duplicate plugin registrations in separate modules.
- Bevy typed asset derive/API details must match Bevy 0.18 and `bevy_common_assets 0.16`; executor should mirror existing data asset derives in `src/data/*_ron.rs` rather than external examples.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Bevy | `bevy` | installed in available skills; relevant for typed assets/plugin lifecycle |
| Rust | `rust-best-practices`, `rust-testing` | installed in available skills; relevant for serde types and parity tests |
| Observability | `observability` | installed in available skills; relevant for clear asset-load diagnostics |

## Sources

- Existing typed RON loader pattern: `src/data/mod.rs`.
- Current dependency versions and headless/windowed feature split: `Cargo.toml`.
- Canonical clip schema rationale: `docs/future_design_draft/02-02_animation_manifest.md`.
- M022 historical clip plan: `docs/M022/M022-ROADMAP.md`, `docs/M022/slices/S02/S02-PLAN.md`.
- Agumon authoritative geometry: `assets/digimon/agumon_atlas.json`.
