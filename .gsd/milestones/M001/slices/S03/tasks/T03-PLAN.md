---
estimated_steps: 8
estimated_files: 6
skills_used: []
---

# T03: Wire headless asset validation state into the animation plugin

Why: R004 says invalid animation assets at boot fail fast with typed diagnostics, not just that a pure function exists. This task composes S01/S02 asset readiness with the S03 validator in a headless Bevy path while keeping catalogs injectable by adapters. Expected executor skills: decompose-into-slices, tdd, bevy, rust-best-practices, verify-before-complete.

Do: Extend `src/animation/plugin.rs` with an `AnimationValidationState` resource and validation system that runs after both graph and clip assets are loaded/readable. Keep the catalog input as an injectable `AnimationValidationCatalogs` resource or similarly explicit adapter seam; the animation plugin must not aggregate skill data itself. The state should expose whether validation has run, whether it is ready/passed, and the typed diagnostics for the latest blocking failure. For initial boot proof, validate configured graph handles against configured clip handles using the graph's `clip` id and loaded clip data; for this milestone's default Agumon path, one graph and one clip is sufficient as long as the matching logic is not hardcoded to Agumon. Add `tests/anim_asset_validation.rs` and small committed fixtures under `assets/test/animation_validation/` for a valid graph+clip and a broken validation-passing-parse graph+clip. The tests should build a minimal Bevy app with `AnimationAssetPlugin`, insert test `AnimationGraphPaths`, `AnimationClipPaths`, and `AnimationValidationCatalogs`, tick until assets load, then assert valid fixtures set validation ready/pass and broken fixtures expose blocking typed diagnostics while not reporting validation success. Keep existing `anim_graph_asset` and `clip_asset` readiness behavior green.

Done when: `cargo test --test anim_asset_validation` proves the plugin-level validation state for valid and broken assets, and full `cargo test` remains green.

Q3 Threat Surface: committed asset fixtures simulate boot-time local data tampering; no network/user auth/secrets are involved.
Q4 Requirement Impact: validates R004 and supports R008; re-runs S01/S02 asset tests through full regression to protect existing loader semantics.
Q5 Failure Modes: unreadable assets keep validation pending; readable-but-invalid assets set failed diagnostics; missing catalogs fail through typed catalog diagnostics; malformed RON remains an asset-load/schema failure before validation.
Q6 Load Profile: boot validation should be one pass over configured loaded assets with O(graphs + clips + graph contents) behavior; diagnostics vectors should stay bounded by authored asset size.
Q7 Negative Tests: broken fixture with missing clip/node/catalog references fails validation after successful parse; malformed schema remains covered by existing parse tests.

## Inputs

- `src/animation/plugin.rs`
- `src/animation/validation.rs`
- `src/animation/anim_graph.rs`
- `src/animation/clip.rs`
- `tests/anim_graph_asset.rs`
- `tests/clip_asset.rs`

## Expected Output

- `src/animation/plugin.rs`
- `tests/anim_asset_validation.rs`
- `assets/test/animation_validation/valid_anim_graph.ron`
- `assets/test/animation_validation/valid_clip.ron`
- `assets/test/animation_validation/broken_anim_graph.ron`
- `assets/test/animation_validation/broken_clip.ron`

## Verification

cargo test --test anim_asset_validation

## Observability Impact

Adds the runtime/headless inspection surface for validation pass/fail state and latest typed diagnostics.
