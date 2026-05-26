# S05 Research: Second Digimon (Renamon) with zero engine edits

## Summary

S05 is **not a simple copy of `src/windowed/digimon/agumon/`**. The S04 registry seam exists, but current engine code still assumes exactly one sprite presentation entry and one shared atlas: `build_digimon_atlas()` and `spawn_unit_sprites()` read `presentation.entries.first()`, so a second `SpritePresentationEntry` is ignored. Also, `windowed_bootstrap_system()` still hardcodes `EncounterPreset::AgumonTrainingDummy`, so `cargo winx` never spawns UnitId(7) Renamon today. Strictly interpreted, “add Renamon with zero engine edits from current S04 state” is blocked.

There are also asset/presentation gaps: Renamon has `assets/digimon/renamon/anim_graph.ron` and `clip.ron`, plus `renamon_atlas.png/json`, but no `stance.ron`, no `all` clip range, no `ReleaseKernel` cue in `diamond_storm_impact`, and no Renamon particle assets. The skill graph can parse, but a bridged windowed skill would stall the kernel barrier unless a `ReleaseKernel` cue is authored or the skill is left unbridged. For working idle/hurt/death, a Renamon stance graph and a way to load it are required.

Recommendation: planner should treat S05 as two gates. First, prove/fix the remaining engine-generalization seams (multi-presentation atlas selection + demo roster selection) or explicitly replan because the strict “no engine edits” acceptance cannot pass. Second, add the Renamon presentation module/assets and the single aggregator registration, then prove no further engine/core edits were needed.

## Requirements / Constraints Owned or Supported

- No formal `REQUIREMENTS.md` entries were preloaded.
- Supports milestone-local extension-first presentation constraint: each Digimon owns presentation registration in `src/windowed/digimon/<name>/`; engine consumes registries only.
- Supports R002/R005 headless/dep-gating: Renamon presentation must stay in windowed binary code/assets; no `bevy_enoki` or windowed symbol should leak into headless lib paths.
- K001 manual sign-off remains required for live `cargo winx`; auto-mode can only prove source contracts, builds, and tests.

## Skills Discovered

- Directly relevant installed skills already present in prompt: `rust-development`, `rust-skills`, `rust-testing`, `bevy`, `decompose-into-slices`, `grill-me`, `observability`, `write-docs`.
- `bevy_enoki` has no installed professional skill found (`npx skills find "bevy_enoki"` returned “No skills found”). Skip external docs unless executors need to author new `.particle.ron`; current S05 can avoid Renamon particles.
- Applied skill guidance conceptually:
  - decompose into thin vertical proof-first slices rather than one broad Renamon dump.
  - grill/stress-test acceptance: current S04 seam still has hidden single-entry and hardcoded-demo blockers.
  - observability: preserve/extend source-contract tests and startup warnings; missing cue/effect should no-op visibly, not panic.
  - write-docs: keep source-contract tests and comments fresh-reader oriented because `src/windowed/` is binary-only and not integration-testable.

## Implementation Landscape

### Existing extension seam

- `src/windowed/digimon/mod.rs`
  - Currently declares `pub(in crate::windowed) mod agumon;` and `register_all(app)` calls `agumon::register(app)`.
  - S05’s nominal “registration call” is adding `mod renamon;` and `renamon::register(app);` here.

- `src/windowed/digimon/agumon/mod.rs`
  - Template for Renamon module shape.
  - Registers `CueRegistry`, `EnokiVfxRegistry`, `OnEnterEffectRegistry`, `SkillReleaseEffectRegistry`, `DetonateEffectRegistry`, `SkillStartNodeRegistry`, and `SpritePresentationRegistry` via Startup systems.
  - Has pure helper specs and module-local tests; copy this testing style.

### Engine registries and blockers

- `src/windowed/render.rs`
  - Registries exist: `EnokiVfxRegistry`, `OnEnterEffectRegistry`, `SkillReleaseEffectRegistry`, `DetonateEffectRegistry`, `SkillStartNodeRegistry`, `SpritePresentationRegistry`.
  - `SpritePresentationEntry` only has `stance_graph_id`, `skill_graph_id`, `atlas_image_path`, `clip_index`; no species/unit selector.
  - `SpritePresentationRegistry` is `Vec<SpritePresentationEntry>`, but engine calls `presentation.entries.first()` in both atlas construction and sprite spawning. This blocks multiple Digimon.
  - `AgumonAtlas` is still a single global atlas resource; both actors share it. A true multi-Digimon presentation likely needs a keyed atlas map/resource or per-entry atlas handles/layouts.
  - `sync_agumon_mode()` reads the skill graph id from each `DigimonSprite`; once sprites carry Renamon graph ids, the skill playback path is mostly generic.
  - `should_auto_release_unbridged()` auto-releases only absent skills. If `diamond_storm` is inserted into `SkillStartNodeRegistry`, Renamon’s graph needs a `ReleaseKernel` cue or combat will wait forever.

- `src/windowed/mod.rs`
  - `windowed_bootstrap_system()` always calls `bootstrap_encounter(..., EncounterPreset::AgumonTrainingDummy)`. That composition clones Agumon into an enemy dummy. Renamon will not appear in `cargo winx` without a demo-selection change or some other clean data-driven selection seam.
  - Existing baseline gotcha: per-Digimon registration runs from `UiPlugin::build`, while `RenderPlugin` is skipped for validation baseline. Avoid unconditional Startup systems that require render-only resources unless they are initialized in baseline too or gated.

### Animation and assets

- `assets/digimon/renamon/anim_graph.ron`
  - Existing skill graph id: `renamon_skill`.
  - Entry: `diamond_storm_cast`.
  - Nodes: `diamond_storm_cast`, `diamond_storm_impact`, `diamond_storm_recover`.
  - On-enter particle name: `diamond_storm_leaf`.
  - **Missing:** any `ReleaseKernel` cue. A bridged skill start node (`diamond_storm -> diamond_storm_cast`) requires adding a release cue, likely on `diamond_storm_impact` at local frame 1.

- `assets/digimon/renamon/clip.ron`
  - Has ranges `attack`, `block`, `death`, `heavy_attack`, `hurt`, `idle`, `skill`, `victory`; no `all` range.
  - A single stance graph containing idle/hurt/death cannot validate against `clip: "idle"` because hurt/death frames are outside that range. The Agumon stance pattern uses `clip: "all"`; Renamon likely needs an `all: (start: 0, end: 67)` range added to `clip.ron`.

- `assets/digimon/renamon/stance.ron`
  - Does not exist. Needed for working idle/hurt/death if Renamon gets its own stance graph.
  - Template from `assets/digimon/agumon/stance.ron`, adjusted to Renamon frames:
    - `id: "renamon_stance"`
    - `clip: "all"` (after adding `all` range)
    - `idle: (35, 42)` loop
    - `hurt: (28, 34)` -> idle
    - `death: (15, 18)` -> exit
    - `victory: (55, 67)` -> exit

- `src/animation/plugin.rs`
  - Already loads Renamon skill graph and clip in `DEFAULT_ANIM_GRAPH_PATHS` / `DEFAULT_ANIM_CLIP_PATHS`.
  - Does **not** load a Renamon stance graph; `DEFAULT_ANIM_STANCE_PATHS` in `src/animation/registry.rs` is only `["digimon/agumon/stance.ron"]`.
  - To preserve zero core edits, a Renamon presentation module can probably mutate `bevyrogue::animation::AnimationStancePaths` at app-build time before Startup `load_animation_graphs()` runs. Do this directly in `register(app)`, not as a Startup system, because `load_animation_graphs` consumes the resource during Startup.

### Combat logic

- `src/combat/blueprints/renamon/` already exists and is registered in `src/combat/blueprints/mod.rs`.
- Renamon data exists in `assets/data/digimon/renamon/unit.ron` (UnitId(7), name Renamon, basic `diamond_storm`, ultimate `renamon_ult`) and `skills.ron`.
- Headless tests already cover Renamon precision/passive/follow-up behavior; S05 should not need new combat logic.

### VFX/cues

- Only Agumon `.particle.ron` assets exist. There are no Renamon particle assets.
- Renamon skill graph’s `diamond_storm_leaf` on-enter currently maps to no effect id and will no-op. This is acceptable only if S05 acceptance does not require Renamon particles; roadmap says Renamon needs cue-driven flash/shake, not bespoke particles.
- `hit_flash`, `hit_shake`, and `camera_impact` are generic cue ids but are currently registered by Agumon module. Re-registering identical definitions from Renamon is idempotent in `CueRegistry`; conflicting definitions panic. Prefer either:
  1. Move generic hit/camera cue defaults to shared windowed setup (engine edit), or
  2. Have Renamon register the exact same defs and rely on idempotency.

## Natural Seams / Task Candidates

1. **Acceptance blocker proof / source contract first**
   - Add or update a windowed-only source-contract test that fails on current single-entry assumptions: `presentation.entries.first()`, singular `AgumonAtlas`, and hardcoded `AgumonTrainingDummy` if the slice requires Renamon in `cargo winx`.
   - This is the highest-risk first proof because it decides whether S05 is blocked/replanned or allowed to include engine hardening.

2. **If engine hardening is allowed: multi-Digimon presentation selection**
   - Change `SpritePresentationRegistry` from anonymous first-entry vec to keyed lookup (likely by `UnitId` for the current roster, or by a future presentation/species key if adding one to spawned units is acceptable).
   - Replace singular `AgumonAtlas` with a keyed atlas cache so Agumon and Renamon can have distinct image/layout/geometry.
   - Update `build_digimon_atlas()` to build all registered entries and `spawn_unit_sprites()` to choose the entry matching each `Unit`.
   - Rename remaining generic engine identifiers/log targets if desired, but avoid broad churn unless tests require it.

3. **If demo roster change is allowed: make Renamon actually spawn**
   - Current `EncounterPreset::AgumonTrainingDummy` blocks live Renamon appearance.
   - Options: introduce a data/env selected windowed demo preset, change the preset to include Renamon, or add a clean encounter-selection seam. Avoid having `src/windowed/digimon/renamon/` spawn combat units; that crosses presentation into gameplay bootstrap.

4. **Renamon presentation module**
   - Create `src/windowed/digimon/renamon/mod.rs` based on Agumon’s module, but minimal:
     - constants: `RENAMON_STANCE_GRAPH_ID = "renamon_stance"`, `RENAMON_SKILL_GRAPH_ID = "renamon_skill"`, `DIAMOND_STORM_SKILL_ID = "diamond_storm"`, `DIAMOND_STORM_CAST_NODE = "diamond_storm_cast"`, `RENAMON_ATLAS_IMAGE_PATH = "digimon/renamon_atlas.png"`, `RENAMON_CLIP_INDEX = 1`.
     - register exact same generic cue defs (`hit_flash`, `hit_shake`, `camera_impact`) or leave cue registration shared if engine hardening moves them.
     - register `diamond_storm -> diamond_storm_cast` in `SkillStartNodeRegistry` only after the asset has a release cue.
     - register sprite presentation entry.
     - mutate `AnimationStancePaths` to append `digimon/renamon/stance.ron` before Startup load.
   - Add module-local tests for start-node map, sprite entry, and idempotent cue registration.

5. **Renamon animation assets**
   - Add `assets/digimon/renamon/stance.ron` and add `all` range to `assets/digimon/renamon/clip.ron` if validation requires it.
   - Add `ReleaseKernel` cue to `diamond_storm_impact` in `assets/digimon/renamon/anim_graph.ron`, otherwise a bridged `diamond_storm` presentation can stall.
   - Consider adding/expanding `tests/animation/anim_stance_asset.rs` and `tests/animation/anim_graph_asset.rs` for Renamon stance and release-cue expectations.

6. **S05 structural contract**
   - Add `tests/windowed_only/renamon_extension_first.rs` (or extend S04 contract) to assert:
     - `src/windowed/digimon/mod.rs` declares/calls Renamon registration.
     - `src/windowed/digimon/renamon/mod.rs` owns `renamon_skill`, `renamon_stance`, `diamond_storm`, `digimon/renamon_atlas.png`, and registry tokens.
     - engine files do not gain Renamon-specific literals/helpers from S05 (target at least `src/windowed/render.rs`, `src/windowed/mod.rs`; consider `src/animation/plugin.rs`/`registry.rs` if using module-driven stance path registration).

## First Proof

Run a static proof before implementing Renamon. The current tree should expose these blockers:

```bash
rg -n "presentation\.entries\.first\(|AgumonTrainingDummy|DEFAULT_ANIM_STANCE_PATHS|ReleaseKernel" \
  src/windowed/render.rs src/windowed/mod.rs src/animation/registry.rs assets/digimon/renamon/anim_graph.ron
```

Expected current findings:
- `presentation.entries.first()` in `build_digimon_atlas()` and `spawn_unit_sprites()`.
- `EncounterPreset::AgumonTrainingDummy` in `windowed_bootstrap_system()`.
- `DEFAULT_ANIM_STANCE_PATHS` only contains Agumon stance.
- Renamon skill graph has no `ReleaseKernel` cue.

If the product owner insists on **no engine/core edits at all in S05**, stop and replan: current S04 output does not provide enough generic engine behavior for the promised Renamon addition. If engine-hardening edits are accepted as remediation, do them first, then make the final Renamon addition module-only and prove no further engine edits.

## Verification Plan

Automated gates after each executable task:

```bash
cargo test --features windowed --test windowed_only renamon_extension_first -- --nocapture
cargo test --features windowed --test windowed_only
cargo test --test dependency_gating
RUSTFLAGS='-D warnings' cargo build --features windowed
cargo test
```

Structural/diff proof for the final Renamon addition step:

```bash
git diff --name-only -- src/windowed src/combat assets tests | sort
rg -n "Renamon|renamon|diamond_storm|renamon_atlas|renamon_stance|RENAMON_" \
  src/windowed/render.rs src/windowed/mod.rs
```

Interpretation:
- New Renamon-specific tokens should live in `src/windowed/digimon/renamon/` and Renamon assets/tests, plus only the aggregator registration in `src/windowed/digimon/mod.rs`.
- If `src/windowed/render.rs` or `src/windowed/mod.rs` gain Renamon-specific tokens, extension-first proof fails.
- K001 manual: run `cargo winx`, verify Renamon appears, idles, plays Diamond Storm, flinches on hit, dies/fades, and flash/shake/camera-shake still fire.

## Risks / Watch-outs

- **Strict acceptance conflict:** S05’s promised “zero engine edits” conflicts with current engine first-entry/hardcoded-demo assumptions. This is the main planning decision.
- **Cue registration collision:** `CueRegistry::register` is idempotent only for equal defs. If Renamon wants different hit feedback params under `hit_flash`/`hit_shake`, it will panic at startup. Use identical defs or unique cue ids plus engine selection support.
- **Baseline validation gotcha (MEM119):** Startup systems that touch render-only registries can break kernel-only baseline if those resources are not initialized. Keep render registry mutations behind the same resource availability assumptions as Agumon, or split app-build resource mutation from Startup registry population.
- **Asset validation:** A Renamon stance graph with `clip: "all"` requires adding `all` to `renamon/clip.ron`. A stance graph with `clip: "idle"` cannot include hurt/death frames without validation errors.
- **Barrier release:** Registering `diamond_storm` as bridged without adding `ReleaseKernel` likely stalls the combat timeline. Add the cue first or intentionally leave it unbridged and accept no skill presentation (not acceptable for S05).
- **Particles:** No Renamon `.particle.ron` exists. Do not author enoki assets unless acceptance expands to require Renamon VFX; `.particle.ron` has strict field requirements and will add avoidable risk.

## Sources

- Memory: MEM123/MEM120 registry seam, MEM119 baseline resource gotcha, MEM109 Renamon zero-engine-edit acceptance.
- Code files inspected:
  - `src/windowed/digimon/mod.rs`
  - `src/windowed/digimon/agumon/mod.rs`
  - `src/windowed/render.rs`
  - `src/windowed/mod.rs`
  - `src/combat/encounter/bootstrap.rs`
  - `src/combat/blueprints/renamon/*`
  - `src/combat/blueprints/mod.rs`
  - `src/animation/plugin.rs`
  - `src/animation/registry.rs`
  - `src/ui/cues.rs`
  - `assets/digimon/renamon/anim_graph.ron`
  - `assets/digimon/renamon/clip.ron`
  - `assets/digimon/agumon/stance.ron`
  - `assets/data/digimon/renamon/unit.ron`
  - `assets/data/digimon/renamon/skills.ron`
  - `tests/windowed_only/agumon_module_extraction.rs`
  - `tests/windowed_only/digimon_sprite_cue_dispatch.rs`
  - `tests/animation/anim_stance_asset.rs`
- Research execs:
  - `.gsd/exec/cdd67a66-1321-4852-9a2c-58f1fd294ea8.stdout` broad inventory.
  - `.gsd/exec/bac7540e-564b-43aa-83bd-6251fa85f0d3.stdout` windowed spawning/Agumon-token scan.
  - `.gsd/exec/ea0b05fc-4954-4ed4-8204-805c8714f82e.stdout` stance path scan.
  - `.gsd/exec/81adbca5-bbce-47bb-843a-bcb069b283ff.stdout` particles/assets/Renamon references.
  - `.gsd/exec/132210e1-a7b0-4573-b7a1-eda109a99f81.stdout` skill discovery for `bevy_enoki`.
