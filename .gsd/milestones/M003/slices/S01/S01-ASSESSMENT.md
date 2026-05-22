---
sliceId: S01
uatType: artifact-driven
verdict: PASS
date: 2026-05-22T00:00:00.000Z
---

# UAT Result — S01

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| Precondition: `assets/digimon/agumon_atlas.png` present | artifact | PASS | `file` reports PNG 5120×5120 (10×10 grid of 512px frames) |
| Precondition: `clip.ron` defines frame_size:(512,512), columns:10, rows:10, total_frames:93 | artifact | PASS | `cat assets/digimon/agumon/clip.ron` confirms all four geometry fields exactly |
| TC-1: `atlas_binding::agumon_atlas_geometry_matches_clip_meta` | runtime | PASS | `cargo test --test animation` — test ok |
| TC-2: `atlas_binding::atlas_index_is_identity_within_range` | runtime | PASS | `cargo test --test animation` — test ok |
| TC-3: `atlas_binding::atlas_index_rejects_out_of_range_frames` | runtime | PASS | `cargo test --test animation` — test ok |
| TC-4: `atlas_binding::idle_player_frames_map_identity_within_idle_range` | runtime | PASS | `cargo test --test animation` — test ok |
| TC-5: `atlas_binding::sharp_claws_player_frames_map_identity_within_attack_range` | runtime | PASS | `cargo test --test animation` — test ok |
| TC-6: `atlas_binding::sharp_claws_release_cue_resolves_to_in_range_impact_atlas_tile` | runtime | PASS | `cargo test --test animation` — test ok |
| TC-7: Windowed build compiles clean | runtime | PASS | `cargo build --features windowed` — exit 0, "Finished dev profile"; `render.rs` grep confirms `Handle<Image>` and `TextureAtlas { layout, index: 0 }` on spawned Sprites, not `..default()` |
| TC-8: Idle stance loops on both actors | human-follow-up | NEEDS-HUMAN | Visual check only — launch `cargo winx` and confirm both Agumon and dummy cycle visible idle frames |
| TC-9: Sharp Claws animation with damage on impact frame | human-follow-up | NEEDS-HUMAN | Visual check only — launch `cargo winx`, trigger basic attack, confirm windup→strike→recover atlas tiles visible and damage number appears on rendered impact frame |

**Full animation test suite:** `cargo test --test animation` — 57 passed, 0 failed, 0 ignored.

## Overall Verdict

PASS — All 7 automatable checks passed (5 headless unit tests, windowed build clean, atlas PNG and clip.ron preconditions verified); 2 visual checks deferred to manual `cargo winx` run per K001.

## Notes

- Atlas PNG resolves to 5120×5120 pixels (10 columns × 10 rows of 512px frames), confirming the geometry contract.
- `src/windowed/render.rs` `AgumonAtlas` resource carries `Handle<Image>`, `Handle<TextureAtlasLayout>`, and `AtlasGeometry`; spawned `Sprite` has explicit `image: atlas.image.clone()` and `texture_atlas: Some(TextureAtlas { layout, index: 0 })` — no `..default()` fallback.
- TC-8 and TC-9 require the windowed binary and human observation; auto-mode cannot launch the windowed binary (K001). Mark these NEEDS-HUMAN before closing S01 as fully verified.
