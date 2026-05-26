# Angle 1 — Split `src/windowed/render.rs` by presentation responsibility

## Question

How should `src/windowed/render.rs` be decomposed so the largest windowed file becomes reviewable and maintainable without breaking the current engine/content boundary?

## Evidence

### Size hotspots

`src/windowed/render.rs` is **2344 lines**. The biggest top-level sections are:

- `advance_digimon_presentation` — lines **951-1382** (**432 lines**)
- test module tail (`#[cfg(test)]`) — lines **2108-2344** (~**237 lines**)
- `sync_digimon_mode` — lines **1888-2018** (**131 lines**)
- `impl DigimonSprite` — lines **194-324** (**131 lines**)
- `apply_camera_shake` — lines **567-689** (**123 lines**)
- `impl Plugin for RenderPlugin` — lines **396-516** (**121 lines**)
- `build_digimon_atlases` — lines **765-864** (**100 lines**)
- `spawn_unit_sprites` — lines **865-950** (**86 lines**)
- `spawn_detonate_particles` — lines **1769-1838** (**70 lines**)

### Responsibility mix inside one file

The file currently mixes at least six distinct concerns:

1. **Playback state/types**
   - `DigimonSprite`, playback mode, release-frame dedupe, marker components
2. **Render/plugin setup**
   - `RenderPlugin`, `setup_camera`, animation clock setup
3. **Shared registries used by content modules**
   - `EnokiVfxRegistry`, `SkillStartNodeRegistry`, `SpritePresentationRegistry`, etc.
4. **Sprite/bootstrap pipeline**
   - `build_digimon_atlases`, `spawn_unit_sprites`
5. **Per-frame presentation/effects**
   - `advance_digimon_presentation`, effect spawning, projectile advancement
6. **Feedback/overlay systems**
   - camera shake, hurt/death reactions, damage numbers, fade-out

### Boundary constraint discovered during inspection

The per-Digimon modules import registry types from `crate::windowed::render`:

- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`

That means `render.rs` is not just “the render systems file”; it is also the **type hub** for data registration. This is the main reason a naive split (“move a few functions out”) will leave awkward coupling behind.

## Options considered

### Option A — Responsibility-first module split inside `windowed/render/`  **(recommended)**

Turn `src/windowed/render.rs` into a small module root and split by cohesive responsibilities.

Suggested shape:

- `src/windowed/render/mod.rs`
  - exports `RenderPlugin`
  - owns system ordering only
- `src/windowed/render/registries.rs`
  - `EnokiVfxRegistry`
  - `OnEnterEffectRegistry`
  - `SkillReleaseEffectRegistry`
  - `DetonateEffectRegistry`
  - `SkillStartNodeRegistry`
  - `SpritePresentationRegistry`
  - `SpritePresentationEntry`
  - `PresentationAtlasRegistry`
- `src/windowed/render/playback.rs`
  - `DigimonSprite`
  - `advance_digimon_presentation`
  - `sync_digimon_mode`
  - release helpers / barrier helpers
- `src/windowed/render/spawn.rs`
  - `build_digimon_atlases`
  - `spawn_unit_sprites`
  - presentation atlas helpers
- `src/windowed/render/effects.rs`
  - `spawn_effect_by_id`
  - `spawn_detonate_particles`
  - `advance_enoki_projectiles`
  - anchor helpers / effect lifecycle structs
- `src/windowed/render/feedback.rs`
  - camera shake
  - damage numbers
  - hurt/death/fade reactions
- `src/windowed/render/clock.rs` or `state.rs`
  - `AnimationClock`
  - `PendingAnimationTicks`
  - render-local constants/markers

#### Pros
- Matches actual responsibility seams already visible in the code.
- Removes the worst coupling first: **registry types no longer live in the same file as giant runtime systems**.
- Makes future per-Digimon additions less likely to reopen a monolith.
- Lets tests and imports target narrower surfaces.

#### Cons
- Highest initial file churn.
- Requires a careful visibility plan (`pub(in crate::windowed)` / `pub(super)`).
- `advance_digimon_presentation` still remains large and may need a second internal refactor later.

### Option B — Two-way split: `render_core.rs` + `render_effects.rs`

Create a coarse split only:
- core: plugin/setup/spawn/playback
- effects: enoki lifecycle, particle spawn, projectile logic

#### Pros
- Lower immediate churn.
- Quickest path to shrinking file size.

#### Cons
- Leaves `render_core` as another large mixed-responsibility file.
- Does not solve the “registry types are owned by runtime file” problem cleanly.
- Likely creates another follow-up refactor almost immediately.

### Option C — Keep file flat, improve section comments only

#### Pros
- Lowest risk.
- Minimal import churn.

#### Cons
- Does not address the main problem.
- Future contributors still edit the same 2k+ line file.
- Review and ownership remain poor.

## Findings

1. **The first extraction should be types/registries, not the giant system body.**
   The content modules already depend on those types. Moving them into `registries.rs` gives a cleaner foundation for every later split.

2. **`advance_digimon_presentation` is the main complexity sink.**
   Even after file splitting, this system will still deserve internal helper extraction (mode sync, transient feedback application, node-enter VFX spawn, release handling, idle-return/fade behavior).

3. **Plugin/system ordering should become declarative.**
   The current `RenderPlugin::build` is readable but long; once systems live in submodules, the plugin root can become a concise ordering map instead of a logic file.

## Recommendation for this angle

Use **Option A**: a responsibility-first split under `src/windowed/render/`, with **`registries.rs` extracted first** and `playback.rs` reserved for the large animation/barrier system.

## Confidence

**High.** The file’s current shape already exposes these seams, and the dependency from Digimon modules into `render.rs` makes the registries-first extraction especially well-supported by the evidence.
