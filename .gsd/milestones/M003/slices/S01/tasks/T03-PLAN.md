---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Bind Handle<Image> + TextureAtlas onto the on-screen Sprite and drive index from the player frame

Why: the on-screen Sprite currently spawns with Sprite{..default()} (no texture), so both actors render blank; this task makes idle + Sharp Claws visible by binding the atlas and driving the tile index from AnimGraphPlayer. skills_used: bevy-ecs-expert, verify-before-complete. Do (windowed-only, all behind feature `windowed`): in src/windowed/render.rs add a resource (e.g. AgumonAtlas { image: Handle<Image>, layout: Handle<TextureAtlasLayout> }) and a system in RenderPlugin (Update, idempotent, runs before spawn_unit_sprites) that, once the agumon Clip is readable (AnimationClipLoadState.ready and Assets<Clip>.get(AnimationClipHandles[0]) is Some), builds AtlasGeometry::from_clip_meta(&clip.meta), creates a TextureAtlasLayout via TextureAtlasLayout::from_grid(UVec2::new(geo.frame_size.w, geo.frame_size.h), geo.columns, geo.rows, None, None), adds it to ResMut<Assets<TextureAtlasLayout>>, loads the atlas via asset_server.load("digimon/agumon_atlas.png"), inserts AgumonAtlas once, and emits a one-time info! (frame_size/columns/rows/total_frames) plus a warn! if the clip is not yet ready or the image load_state is Failed. Gate spawn_unit_sprites on AgumonAtlas being present (in addition to the stance graph) and spawn Sprite with `image: atlas.image.clone()`, `texture_atlas: Some(TextureAtlas { layout: atlas.layout.clone(), index: 0 })`, keeping flip_x. Change advance_agumon_presentation's query to Query<(&mut AgumonSprite, &mut Sprite)> and, after computing advance.frame, set the Sprite's texture_atlas index via AtlasGeometry::atlas_index(advance.frame) (fall back to leaving the index unchanged on None); extend the existing trace! with the resolved atlas_index. Do NOT use bevy/2d types outside this windowed module. Done when: cargo build --features windowed compiles, the on-screen Sprite is constructed with bound image+texture_atlas (no ..default() for those fields), the index is set from the player frame each tick, and the headless suite (cargo test) remains green.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/animation/atlas.rs`
- `src/animation/plugin.rs`
- `assets/digimon/agumon/clip.ron`
- `assets/digimon/agumon_atlas.png`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed
