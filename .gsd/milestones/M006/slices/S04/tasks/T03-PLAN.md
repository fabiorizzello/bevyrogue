---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T03: Convert skill graph/start-node consts and sprite/atlas wiring to registries

Why: the remaining AGUMON_ consts (AGUMON_STANCE_GRAPH_ID, AGUMON_SKILL_GRAPH_ID, AGUMON_ULT_SKILL_ID) and the Agumon skill/node consts plus the skill_start_node closed match and the hardcoded atlas path/clip-index keep the engine coupled to Agumon; converting them to registry data completes the S04 grep gate and makes spawn_unit_sprites/build_atlas data-driven for S05. Do: (1) Define engine resources SkillStartNodeRegistry (HashMap<String,String>, skill_id->entry node) and SpritePresentationRegistry (entry { stance_graph_id, skill_graph_id, atlas_image_path, clip_index }); init-empty in RenderPlugin::build. (2) Move the AGUMON_STANCE_GRAPH_ID/AGUMON_SKILL_GRAPH_ID/AGUMON_ULT_SKILL_ID consts and the SHARP_CLAWS_*/BABY_FLAME_*/BABY_BURNER_* skill+node consts (src/windowed/mod.rs:38-56) into the agumon module; remove the `use super::{...}` re-export in render.rs:34-36. (3) Move the skill_start_node match (render.rs:1930-1937) into a SkillStartNodeRegistry populated by agumon::register; re-point the lookup at render.rs:1840 and should_auto_release_unbridged (render.rs:2027) to consult the registry (None = auto-release fallback, preserved). (4) Populate SpritePresentationRegistry from agumon (atlas_image_path="digimon/agumon_atlas.png", clip_index=0, stance/skill graph ids); re-point build_agumon_atlas (render.rs:761, rename to build_digimon_atlas) to read atlas_image_path + clip_index from the registry instead of the hardcoded string and `handles.0.first()`; re-point spawn_unit_sprites (render.rs:838) to seed DigimonSprite::idle_for from the registry entry instead of AGUMON_STANCE_GRAPH_ID/AGUMON_SKILL_GRAPH_ID (the resolve_snapshot AnimGraphId at render.rs:847 also reads the registry's stance id). (5) Move the skill_start_node / should_auto_release / classify_same_skill_sync unit tests that reference the Agumon consts (render.rs:2176-2295 region) into the agumon module's tests, importing the moved consts there. For S04 SpritePresentationRegistry holds the single agumon entry; AgumonAtlas resource may stay single. Done when: cargo build --features windowed green zero warnings; no `AGUMON_` const and no `fn skill_start_node` in src/windowed/render.rs or src/windowed/mod.rs; `grep -rq "digimon/agumon_atlas.png" src/windowed/render.rs` finds nothing; windowed_only green.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`

## Expected Output

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`

## Verification

cargo test --features windowed --test windowed_only
