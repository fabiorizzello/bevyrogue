---
estimated_steps: 14
estimated_files: 6
skills_used: []
---

# T02: Add Renamon presentation module and animation assets

Expected executor skills/frontmatter: rust-development, rust-skills, bevy, tdd, write-docs, verify-before-complete.

Why: Once the engine consumes generic registries, Renamon should be added the same way Agumon is now owned: via its own presentation module and authored assets, not by adding Renamon branches to engine code.

Do:
1. Create src/windowed/digimon/renamon/mod.rs modeled on the Agumon module but scoped to Renamon. Register SpritePresentationRegistry data for UnitId(7), presentation id, stance graph id renamon_stance, skill graph id renamon_skill, atlas path digimon/renamon_atlas.png, and clip index 1. Register SkillStartNodeRegistry mapping diamond_storm to diamond_storm_cast.
2. Mutate bevyrogue::animation::AnimationStancePaths during renamon::register(app) so digimon/renamon/stance.ron is loaded before Startup graph loading. Do this at app-build registration time rather than as a Startup system.
3. Register Renamon with the generic windowed demo registry from T01 so cargo winx has at least one Renamon combatant in the applied composition. Keep demo selection data in the Renamon module, not in engine files.
4. Add assets/digimon/renamon/stance.ron with idle, hurt, death, and victory stance nodes using the Renamon clip ranges: idle 35-42 loop, hurt 28-34 returning to idle, death 15-18 exiting, victory 55-67 exiting.
5. Add an all range to assets/digimon/renamon/clip.ron covering 0-67 so the stance graph can reference idle/hurt/death/victory frames through one graph clip.
6. Add a ReleaseKernel cue to assets/digimon/renamon/anim_graph.ron, likely on diamond_storm_impact at local frame 1, so bridging diamond_storm does not stall the SuspendedTimelineState barrier. Leave Renamon particles unregistered unless a real .particle.ron asset is introduced; a missing diamond_storm_leaf mapping should no-op rather than invent a fake effect.
7. Update src/windowed/digimon/mod.rs with only the aggregator-level module declaration/call for Renamon.

Done when: Renamon owns its windowed presentation data, the stance/skill graph assets can be loaded by the normal animation registries, and no engine/core file contains Renamon-specific ids or diamond_storm branches.

Failure Modes (Q5): If AnimationStancePaths is updated too late, Renamon sprites will never resolve renamon_stance; register it at app-build time. If ReleaseKernel is missing, diamond_storm will hang the combat barrier; the source contract in T03 must pin this. If the atlas path is wrong, windowed diagnostics from T01 should warn by path.

Load Profile (Q6): Adds one atlas handle/layout and one stance graph to startup; runtime cost is one additional presentation entry and at most one additional demo sprite, trivial at current scale.

Negative Tests (Q7): Contract tests should fail for missing stance.ron, missing all clip range, missing ReleaseKernel, missing renamon::register(app), or Renamon tokens in engine files.

## Inputs

- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/mod.rs`
- `src/windowed/render.rs`
- `src/windowed/demo.rs`
- `assets/digimon/renamon/anim_graph.ron`
- `assets/digimon/renamon/clip.ron`
- `assets/digimon/agumon/stance.ron`
- `assets/digimon/renamon_atlas.png`
- `src/animation/registry.rs`
- `src/animation/plugin.rs`
- `src/combat/blueprints/renamon/mod.rs`
- `assets/data/digimon/renamon/unit.ron`
- `tests/windowed_only/renamon_extension_contract.rs`

## Expected Output

- `src/windowed/digimon/renamon/mod.rs`
- `src/windowed/digimon/mod.rs`
- `assets/digimon/renamon/stance.ron`
- `assets/digimon/renamon/clip.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/windowed_only/renamon_extension_contract.rs`

## Verification

cargo test --features windowed --test windowed_only renamon_extension_contract -- --nocapture

## Observability Impact

Renamon asset/registry failures should surface through the generic diagnostics added in T01; the task should not add silent unwraps or panics around graph, clip, or atlas lookup.
