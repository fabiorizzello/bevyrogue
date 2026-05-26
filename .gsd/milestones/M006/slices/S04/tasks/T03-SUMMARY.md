---
id: T03
parent: S04
milestone: M006
key_files:
  - src/windowed/render.rs
  - src/windowed/digimon/agumon/mod.rs
  - src/windowed/mod.rs
key_decisions:
  - Inlined the thin skill_start_node registry-lookup helper at its two call sites (bridged-skill guard and should_auto_release_unbridged) rather than keeping it, to satisfy the literal S04 grep gate forbidding `fn skill_start_node` in the engine files; the EffectRegistries.skill_start_node field name is retained as it is a data field, not a function.
duration: 
verification_result: passed
completed_at: 2026-05-26T12:18:27.664Z
blocker_discovered: false
---

# T03: Converted Agumon skill-graph/start-node consts and sprite/atlas wiring to engine-generic registries, closing the S04 grep gate by inlining the residual skill_start_node helper out of render.rs

**Converted Agumon skill-graph/start-node consts and sprite/atlas wiring to engine-generic registries, closing the S04 grep gate by inlining the residual skill_start_node helper out of render.rs**

## What Happened

On entry, most of T03 was already implemented (consts SHARP_CLAWS_*/BABY_FLAME_*/BABY_BURNER_* and AGUMON_STANCE/SKILL/ULT graph ids living in src/windowed/digimon/agumon/mod.rs; SkillStartNodeRegistry and SpritePresentationRegistry defined and init-empty in RenderPlugin::build; build_agumon_atlas already renamed to build_digimon_atlas and reading atlas_image_path+clip_index from the registry; spawn_unit_sprites seeding DigimonSprite::idle_for from the registry entry; the skill-start-node and sprite-presentation register systems wired into agumon::register; unit tests moved into the agumon module). Verification revealed one outstanding done-criterion violation: render.rs still defined `fn skill_start_node` (a thin registry-lookup helper). I inlined it at both call sites — the bridged-skill guard in the skill-presentation system now reads `start_node_reg.map.get(...).map(String::as_str)` directly, and should_auto_release_unbridged now uses `!reg.map.contains_key(skill_id)` — then removed the function and repaired the two doc-comment intra-links that referenced it. No engine const, fn skill_start_node, or hardcoded atlas path remains in the engine files.

## Verification

cargo build --features windowed with RUSTFLAGS="-D warnings" exits 0 (zero warnings). cargo test --features windowed --test windowed_only: 59 passed, 0 failed. cargo test --features windowed --bins: 27 passed (incl. agumon module tests register_populates_the_skill_start_node_registry, auto_release_fallback_only_targets_unbridged_skills, register_populates_the_sprite_presentation_registry), 0 failed. Grep gates: no `const AGUMON_`, no `fn skill_start_node`, and no `digimon/agumon_atlas.png` literal in src/windowed/render.rs or src/windowed/mod.rs.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS="-D warnings" cargo build --features windowed` | 0 | pass | 5110ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass | 1485ms |
| 3 | `cargo test --features windowed --bins` | 0 | pass | 1266ms |
| 4 | `grep -rnE 'const AGUMON_|fn skill_start_node|digimon/agumon_atlas.png' src/windowed/render.rs src/windowed/mod.rs` | 1 | pass (no matches) | 5ms |

## Deviations

Most of the task plan's edits (const moves, registry definitions, build_digimon_atlas rename, registry-driven spawn_unit_sprites, moved tests) were already present from a prior partial execution of T03. The only net change this run was inlining and removing the residual `fn skill_start_node` helper plus fixing its two doc references — required to pass the done-criterion grep gate.

## Known Issues

none

## Files Created/Modified

- `src/windowed/render.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/mod.rs`
