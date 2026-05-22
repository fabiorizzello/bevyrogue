---
id: S01
parent: M003
milestone: M003
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - Identity frame→atlas-index map: atlas_index(frame) returns Some(frame) iff frame < total_frames — clip frame index equals atlas tile index, so no mapping table is needed (proven by clip_atlas_parity)
  - AtlasGeometry kept Bevy-free (pure descriptor in lib); real TextureAtlasLayout is built windowed-side only — keeps the headless/windowed split clean
  - Stored AtlasGeometry inside AgumonAtlas resource so advance_agumon_presentation can drive the tile index per tick without re-reading the Clip asset
  - warn! fires only on genuine failures (clip ready-but-missing, image LoadState::Failed), not on the transient loading state — avoids startup log spam while preserving failure-visibility
  - Impact-frame test resolves the ReleaseKernel cue's local 'at' from the loaded graph (not a hardcoded frame number) and computes clip_frame as start()+at honoring reverse — seam mirrors render.rs local_frame_for exactly
patterns_established:
  - Bevy-free geometry seam pattern: pure descriptor struct in lib, real Bevy types constructed windowed-side — usable for S02/S03 when additional atlas ranges need different layout sources
  - Idempotent windowed resource build pattern: system early-returns if resource already present, gates downstream spawn on resource presence — scalable for other asset-backed resources
  - Impact-frame invariant test pattern: scan cues from loaded RON graph, compute clip frame via documented formula, assert both range membership and atlas_index identity — reusable for Baby Flame/Baby Burner cue invariants in S02
observability_surfaces:
  - info! on atlas build: frame_w/frame_h/columns/rows/total_frames (one-time, on successful AgumonAtlas insert)
  - warn! on clip load state ready but asset missing (one-time genuine failure)
  - warn! on atlas image LoadState::Failed (one-time genuine failure)
  - trace! per tick in advance_agumon_presentation: extended with resolved atlas_index alongside existing node/clip_frame/local_frame
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-22T10:30:01.988Z
blocker_discovered: false
---

# S01: S01

**Established a Bevy-free AtlasGeometry seam (lib), proved idle+Sharp Claws player-frame→atlas-index identity and the impact-frame-on-rendered-frame invariant headless, and wired the real Handle<Image>+TextureAtlas onto the on-screen Sprites with per-tick index drive from AnimGraphPlayer.**

## What Happened

S01 shipped in three clean tasks with no deviations beyond a minor observability refinement.

**T01 — Bevy-free AtlasGeometry seam**
Created `src/animation/atlas.rs` with `AtlasGeometry { frame_size, columns, rows, total_frames }`, a `from_clip_meta(&ClipMeta)` constructor (copies four geometry fields), and `atlas_index(frame) -> Option<u32>` returning `Some(frame)` when `frame < total_frames` else `None` (identity map). Doc comment records the identity rationale and that the real `TextureAtlasLayout` is built windowed-side. No `bevy/2d` imports anywhere in the file. Re-exported via `pub use atlas::*` in `src/animation/mod.rs`. Headless tests in `tests/animation/atlas_binding.rs` confirmed: geometry == 512×512 / cols 10 / rows 10 / total 93; atlas_index(0)==Some(0), atlas_index(92)==Some(92), atlas_index(93)==None, atlas_index(u32::MAX)==None. 54 animation tests pass.

**T02 — Headless parity + impact-frame invariant**
Extended `tests/animation/atlas_binding.rs` with three tests:
(a) `idle_player_frames_map_identity_within_idle_range`: builds `AnimGraphPlayer` on the parsed stance graph, advances 24 ticks crossing the Loop boundary, asserts every frame stays in `[53,58]` (IDLE_RANGE const sourced from clip.ron) and `atlas_index(frame)==Some(frame)`.
(b) `sharp_claws_player_frames_map_identity_within_attack_range`: drives windup(0–2)→strike(3–5)→recover(6–8) firing `fire_kernel_cue()` once to satisfy the KernelCue gate; asserts every frame stays in `[0,8]` (ATTACK_RANGE) and maps identity.
(c) `sharp_claws_release_cue_resolves_to_in_range_impact_atlas_tile`: scans the loaded anim_graph.ron for a `FrameCueCommand::ReleaseKernel` cue in sharp_claws* nodes, computes `clip_frame = start() + at` (honoring reverse, exact inverse of render.rs `local_frame_for`), and asserts the resolved frame (4) lies in `[0,8]` and `atlas_index(4)==Some(4)`. No hardcoded frame numbers. 57 animation tests pass.

**T03 — Windowed atlas binding**
Added `AgumonAtlas { image, layout, geometry }` resource and `build_agumon_atlas` system (idempotent, runs `.before(spawn_unit_sprites)`) in `src/windowed/render.rs`. Once the agumon `Clip` is readable, builds `AtlasGeometry::from_clip_meta`, constructs `TextureAtlasLayout::from_grid`, loads `digimon/agumon_atlas.png`, and inserts the resource once with a one-time `info!` (frame_w/h/cols/rows/total). `spawn_unit_sprites` gated on `AgumonAtlas` presence; spawned `Sprite` carries `image: atlas.image.clone()` and `texture_atlas: Some(TextureAtlas { layout, index: 0 })`, preserving `flip_x`. `advance_agumon_presentation` query extended to `(&mut AgumonSprite, &mut Sprite)`; per tick sets `texture_atlas.index` via `atlas.geometry.atlas_index(frame)` (identity), leaving index unchanged on `None`. Per-tick `trace!` extended with resolved `atlas_index`. `warn!` fires once on genuine failure (clip ready-but-missing or image LoadState::Failed), not on transient loading. `cargo build --features windowed` and full `cargo test` both green. Visual confirmation (idle loop + Sharp Claws impact frame) deferred to manual `cargo winx` per K001.

## Verification

Verification commands run at slice closeout:
1. `cargo test --test animation` — exit 0, 57 passed 0 failed. All six atlas_binding tests present and green: agumon_atlas_geometry_matches_clip_meta, atlas_index_is_identity_within_range, atlas_index_rejects_out_of_range_frames, idle_player_frames_map_identity_within_idle_range, sharp_claws_player_frames_map_identity_within_attack_range, sharp_claws_release_cue_resolves_to_in_range_impact_atlas_tile.
2. `cargo build --features windowed` — exit 0, Finished dev profile, no errors (cached; incremental build confirms no source regressions since T03 completed).
All three task verification_result fields: passed. No blockers discovered.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

Visual confirmation (idle loop + Sharp Claws impact frame on screen) is deferred to a manual user run of `cargo winx` per K001 — auto-mode must not launch the windowed binary. The headless parity/invariant tests from T01/T02 cover the logic contract. Baby Flame and Baby Burner windowed cue bridges (currently auto-released) are left for S02.

## Follow-ups

S02 should: (1) add windowed cue bridges for Baby Flame (skill range) and Baby Burner (heavy_attack range) so they release on the rendered impact frame rather than auto-releasing; (2) extend the impact-frame invariant tests to cover Baby Flame and Baby Burner ReleaseKernel cues using the same pattern established in TC-6.

## Files Created/Modified

- `src/animation/atlas.rs` — 
- `src/animation/mod.rs` — 
- `tests/animation/atlas_binding.rs` — 
- `tests/animation.rs` — 
- `src/windowed/render.rs` — 
