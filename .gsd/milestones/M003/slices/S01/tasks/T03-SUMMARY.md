---
id: T03
parent: S01
milestone: M003
key_files:
  - src/windowed/render.rs
key_decisions:
  - Stored AtlasGeometry inside the AgumonAtlas resource so advance_agumon_presentation drives the tile index per tick without re-reading the Clip asset
  - Built the real TextureAtlasLayout windowed-side from AtlasGeometry::from_clip_meta, keeping all bevy/2d types confined to src/windowed/render.rs (R005)
  - Made warn!s fire only on genuine failures (clip ready-but-missing, image LoadState::Failed) rather than the transient loading state, to avoid startup log spam
  - Added #[allow(clippy::too_many_arguments)] on build_agumon_atlas matching the codebase's existing Bevy-system convention
duration: 
verification_result: passed
completed_at: 2026-05-22T10:27:57.820Z
blocker_discovered: false
---

# T03: Bound the agumon atlas image + TextureAtlasLayout onto the on-screen Sprite and drove tile index from the AnimGraphPlayer frame each tick

**Bound the agumon atlas image + TextureAtlasLayout onto the on-screen Sprite and drove tile index from the AnimGraphPlayer frame each tick**

## What Happened

The on-screen Agumon Sprites previously spawned with `Sprite { flip_x, ..default() }` (no texture), so both the ally and the mirrored dummy rendered blank. T03 closes that gap windowed-side, behind the `windowed` feature, entirely within `src/windowed/render.rs`.

Added an `AgumonAtlas { image: Handle<Image>, layout: Handle<TextureAtlasLayout>, geometry: AtlasGeometry }` resource and a `build_agumon_atlas` system registered in `RenderPlugin` (Update, `.before(spawn_unit_sprites)`). The system is idempotent: it early-returns once the resource exists. Once the agumon `Clip` is readable (`AnimationClipHandles[0]` resolvable via `Assets<Clip>`), it builds `AtlasGeometry::from_clip_meta(&clip.meta)` (the Bevy-free seam from T01), constructs the real `TextureAtlasLayout::from_grid(UVec2::new(w, h), columns, rows, None, None)`, adds it to `Assets<TextureAtlasLayout>`, loads `digimon/agumon_atlas.png` via the asset server, and inserts the resource once. It also stores the geometry so the per-tick index map needs no clip re-read.

`spawn_unit_sprites` now takes `Option<Res<AgumonAtlas>>` and is gated on it (in addition to the stance graph); the spawned `Sprite` carries `image: atlas.image.clone()` and `texture_atlas: Some(TextureAtlas { layout: atlas.layout.clone(), index: 0 })`, preserving `flip_x`. `advance_agumon_presentation`'s query changed from `Query<&mut AgumonSprite>` to `Query<(&mut AgumonSprite, &mut Sprite)>`; after computing `advance.frame` it resolves `atlas.geometry.atlas_index(advance.frame)` (identity map) and sets `texture_atlas.index`, leaving the index unchanged on `None` (out-of-range).

Observability per the slice verification: the existing per-tick `trace!` was extended with the resolved `atlas_index`; a one-time `info!` logs `frame_w/frame_h/columns/rows/total_frames` when the atlas is built; one-time `warn!`s cover the genuine-failure cases (clip load state ready but asset missing, or atlas image `LoadState::Failed`).

Per K001 the windowed binary was not executed — the visual idle-loop / Sharp Claws demo is left for manual user verification (`cargo winx`). The headless contract (image+texture_atlas binding, identity frame->index parity, impact-frame invariant) is already covered by the T01/T02 headless tests, which remain green.

## Verification

Ran `cargo build --features windowed` (the plan's verification command) — compiles clean with no warnings. Ran the full headless `cargo test` suite — all binaries green, 0 failures. Ran `cargo clippy --features windowed` and confirmed the new `build_agumon_atlas` function introduces no new lint (added `#[allow(clippy::too_many_arguments)]` matching the codebase's existing Bevy-system convention; remaining collapsible-if warnings are pre-existing and untouched). Confirmed the spawned Sprite no longer uses `..default()` for image/texture_atlas and that the index is set from the player frame each tick. Per K001, did not run the windowed binary; visual confirmation deferred to manual user run.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 6040ms |
| 2 | `cargo test` | 0 | pass | 30000ms |
| 3 | `cargo clippy --features windowed` | 0 | pass (no new warnings from this task) | 5480ms |

## Deviations

Plan said to warn! "if the clip is not yet ready"; warning every frame during normal asset loading would spam the log, so the warn! fires once only when the clip load state reports ready but the asset is still missing (a real failure). The transient loading state silently early-returns. This preserves the intended failure-visibility surface without noise.

## Known Issues

Visual confirmation (idle loop + Sharp Claws impact frame on screen) is deferred to a manual user run of `cargo winx` per K001 — auto-mode must not launch the windowed binary. The headless parity/invariant tests from T01/T02 cover the logic side.

## Files Created/Modified

- `src/windowed/render.rs`
