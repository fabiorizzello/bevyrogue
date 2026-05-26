# S04 Research: Extract Agumon presentation into its own module

**Lane:** research · **Depth:** targeted (known codebase, established registry pattern, but real interface-design work in the registry shape) · **Risk:** medium

## Summary

S03 already did the hard structural lift: `AgumonSprite`→`DigimonSprite` with `stance_graph_id`/`skill_graph_id` as data, and flash/shake/camera-shake routed through `CueRegistry` + parametric math. **S04 is the extraction-and-seam slice:** pull every remaining Agumon-specific *datum* (consts, the enoki effect map + loader, the two closed `match` statements, the cue registration, the hardcoded atlas image path) out of the engine files (`src/windowed/render.rs`, `src/windowed/mod.rs`) and into a new `src/windowed/digimon/agumon/` module that registers all of it via a `register(app)` entry point. The engine systems stay (they are generic players/state-machines), but they must be re-pointed to **read from registries** instead of reading consts and matching on Agumon strings.

The slice contract (roadmap "After this"): grep of the windowed engine files shows **no `AGUMON_*` const, no closed `on_enter_effect_ids` match, no `load_agumon_enoki_vfx`** — all of it lives in and is registered by `src/windowed/digimon/agumon/`. Windowed build/test green; Agumon behaves identically (K001 manual).

The non-obvious part: this is **not** a pure move. Three engine seams currently encode Agumon knowledge *as control flow* (two `match` statements + one `match effect_id` lifecycle dispatch). Removing them requires turning that control flow into **data carried in registry entries**. That data design is the real work; the rest is mechanical relocation.

## Active Requirements This Slice Owns/Supports

- **Extension-first presentation** (local constraint): S04 builds the registry seams that make S05's "zero engine edits" gate achievable. If S04 leaves any Agumon datum as engine control flow, S05 fails.
- **R002/R005 (headless-first + dep gating):** new module is windowed-only (binary crate, under `#[cfg(feature = "windowed")]` reachability); `cargo test --test dependency_gating` must still prove no enoki/windowed leak into headless. The new module imports `bevy_enoki` so it is correctly inside the binary crate.
- **R004 (determinism):** no new wall-clock/RNG; registry lookups are pure.

## Extraction Inventory (exact targets, with file:line)

### In `src/windowed/mod.rs`
- **`AGUMON_*` consts** — `38-56`: `AGUMON_STANCE_GRAPH_ID`, `AGUMON_SKILL_GRAPH_ID`, `SHARP_CLAWS_SKILL_ID`, `SHARP_CLAWS_WINDUP_NODE`, `BABY_FLAME_SKILL_ID`, `AGUMON_ULT_SKILL_ID`, `BABY_FLAME_CAST_NODE`, `BABY_BURNER_CHARGE_NODE`, + four `#[allow(dead_code)]` node consts (`50-56`). Re-exported into render.rs via `use super::{...}` at `render.rs:34-36`.
- **`register_agumon_cues`** — `136-163` (registers `hit_flash` Flash, `hit_shake` SpriteShake, `camera_impact` CameraShake). Wired at `90-91` (`init_resource::<CueRegistry>` + `add_systems(Startup, register_agumon_cues)`).

### In `src/windowed/render.rs`
- **Effect-id + enoki path consts** — `580-613`: `AGUMON_*_EFFECT_ID` (6), `AGUMON_PROJECTILE_FLIGHT_TICKS`, `AGUMON_ENOKI_*_PATH` (6).
- **`AgumonEnokiVfx` resource + entry struct** — `615-636`; **`load_agumon_enoki_vfx`** loader — `637-696`; **`diagnose_agumon_enoki_vfx_load`** — `704-731`; **`effect_path`** id→path map — `733-746`. Wired in `RenderPlugin::build` at `407` (Startup) and `410` (Update).
- **`on_enter_effect_ids` closed match** — `1500-1508`: particle_name→`&[effect_id]`. Called at `1149`.
- **`skill_start_node` closed match** — `1930-1937`: skill_id→entry node. Called at `1934`-area via `skill_start_node(...)`.
- **`spawn_effect_by_id` lifecycle dispatch** — `1571-1588`: `match effect_id { CHARGE|EMBER => persistent marker, PROJECTILE => ProjectileFlight{ticks_total: AGUMON_PROJECTILE_FLIGHT_TICKS}, _ => OneShot::Despawn }`. **This is Agumon control flow that must become data.**
- **`build_agumon_atlas`** — `761-832`: hardcodes `"digimon/agumon_atlas.png"` (`804`) and assumes the Agumon clip is index 0 of `DEFAULT_ANIM_CLIP_PATHS` (`779-781`). `AgumonAtlas` resource consumed by `spawn_unit_sprites` (`842`) and `advance_agumon_presentation` (`902`).
- **`spawn_unit_sprites`** — `838-892`: reads `AGUMON_STANCE_GRAPH_ID` (`847`) and seeds `DigimonSprite::idle_for(... AGUMON_STANCE_GRAPH_ID.into(), AGUMON_SKILL_GRAPH_ID.into())` (`869-874`).

### Secondary "agumon" grep hits (judgment-call, NOT required by the S05 gate)
- **System/struct names**: `advance_agumon_presentation`, `sync_agumon_mode`, `build_agumon_atlas`, `diagnose_agumon_enoki_vfx_load`, `AgumonAtlas`, `AgumonEnokiVfx`, `ChargeEmberEnokiMarker`. These are **generic engine machinery** misnamed "agumon"; they stay in the engine.
- **Trace target string** `"windowed.agumon_playback"` (~40 sites).
- These are string/name hits, not `AGUMON_*` consts. **They do not block S05** (adding Renamon registers a new module; it never edits these). Renaming them to `digimon_*` is grep-cleanliness polish — see Open Question 3. Recommend renaming the *engine-resident generic systems/resources* to `digimon_*` (coherent once they're registry-driven) but treating the trace-target string as out-of-scope to avoid churn.

## The Three Control-Flow → Data Conversions (the actual design)

These are why S04 is "medium" not "low":

1. **`on_enter_effect_ids` match → binding registry.** `particle_name → Vec<effect_id>`. Engine site (`render.rs:1149`) iterates the result. Replace the `&'static [&'static str]` return with a registry lookup returning the bound ids (empty = no-op, matching current `_ => &[]`). Note: there is a unit test at `render.rs:2104-2129` asserting this map's contents — it moves to the agumon module's tests.

2. **`skill_start_node` match → start-node registry.** `skill_id → Option<entry_node>`. Engine consults it to seed the player at a bridged skill's FSM entry. `None` = auto-release fallback (preserve).

3. **`spawn_effect_by_id` lifecycle `match effect_id` → `EnokiLifecycle` data field.** Each enoki registry entry must carry a lifecycle enum: `PersistentEmitter` (charge/ember — cleared by marker at launch), `Projectile { flight_ticks }` (caster→target, chains impact), `OneShot` (fire-and-forget `OneShot::Despawn`). The engine then matches on the *enum*, not on Agumon effect-id strings. `ChargeEmberEnokiMarker` and `ProjectileFlight` are already generic component types — keep them; only the *selection* of which to attach moves to data. `AGUMON_PROJECTILE_FLIGHT_TICKS` becomes the `Projectile{flight_ticks}` value.

## Recommended Architecture (designed-twice: see Alternatives)

**Mirror the established small-registry + module-`register()` convention** (MEM018, MEM106, MEM109; same shape as `SkillGraphRegistry`/`StanceGraphRegistry`/`CueRegistry` and the `blueprints/<name>/register_*` pattern). Concretely, generalize/add these windowed-side registry resources and have the agumon module populate them at Startup:

- **`EnokiVfxRegistry`** (rename of `AgumonEnokiVfx`): `effect_id → { handle, anchor, lifecycle: EnokiLifecycle }`. Absorbs the path consts, the loader, the lifecycle dispatch, and `effect_path`/`diagnose`.
- **`OnEnterEffectRegistry`**: `particle_name → Vec<effect_id>` (replaces match #1).
- **`SkillStartNodeRegistry`**: `skill_id → entry_node` (replaces match #2).
- **`SpritePresentationRegistry`** (or extend a roster entry): `digimon_key → { stance_graph_id, skill_graph_id, atlas_image_path, clip_index/handle }`. Feeds `spawn_unit_sprites` + `build_*_atlas`. For S04 it holds one entry (agumon); S05 adds renamon.
- **`CueRegistry`**: already exists — just move the `register_agumon_cues` body into the agumon module.

**`src/windowed/digimon/agumon/mod.rs`** exposes `pub(in crate::windowed) fn register(app: &mut App)` that adds the Startup systems inserting all the above entries. `RenderPlugin::build` and `UiPlugin::build` call `crate::windowed::digimon::agumon::register(app)` (one line each, or a single `digimon::register_all(app)` aggregator mirroring `blueprints::register_blueprints`). Add `mod digimon;` to `src/windowed/mod.rs` and `pub(super) mod agumon;` under it.

**Why this shape:** matches every existing extension seam in the repo; allocation-free static wiring; each registry is independently testable; adding Renamon (S05) = a second `register()` call + entries, zero engine edits. The aggregator (`digimon::register_all`) is the single seam S05 appends to — confirm whether *that* counts as an "engine edit" (see Open Q1).

### Alternatives considered (design-it-twice)
- **(A) One unified `WindowedDigimonRegistry` struct** (`digimon_key → DigimonPresentation { all fields }`). Fewer types, one insert per digimon. *Rejected:* a new god-struct abstraction the codebase doesn't otherwise use; the engine systems each need only one slice of it, so small registries match access patterns better and mirror `SkillGraphRegistry`/`CueRegistry`.
- **(B) Move the systems themselves into the agumon module.** *Rejected:* breaks the generic-engine principle — Renamon would re-implement the player. The systems are generic; only data is per-digimon.
- **(C) Keep matches but feature-flag/`cfg` per digimon.** *Rejected:* still engine edits per digimon; defeats the milestone.

Recommendation: **small registries + module `register()` (the convention)**, with the three control-flow→data conversions above.

## Natural Seams (task decomposition guidance)

Roughly independent units; build the registries first (no behavior change), then re-point the engine, then move the data:

- **T0 — module skeleton + aggregator wiring.** Create `src/windowed/digimon/mod.rs` (+ `register_all`) and `agumon/mod.rs` with an empty `register(app)`; wire the one-line call into `RenderPlugin`/`UiPlugin`. Compiles, no behavior change.
- **T1 — Cues.** Move `register_agumon_cues` body into `agumon::register`; remove from `mod.rs`. (Lowest risk, fully isolated — good first proof.)
- **T2 — Enoki VFX registry + lifecycle-as-data.** Generalize `AgumonEnokiVfx`→`EnokiVfxRegistry` with `EnokiLifecycle`; move loader/paths/`diagnose`/`effect_path` to agumon module; convert `spawn_effect_by_id`'s `match effect_id` to match on `entry.lifecycle`. (Highest risk — touches the spawn path. See First Proof.)
- **T3 — on_enter + skill_start_node registries.** Replace both closed matches with registry lookups; move bindings to agumon module; relocate the `on_enter_effect_ids` unit tests (`render.rs:2104-2129`).
- **T4 — Sprite/atlas presentation registry.** Generalize `build_agumon_atlas` + `spawn_unit_sprites` to read graph ids + atlas image path + clip from `SpritePresentationRegistry`; move agumon entry data to module. (Enables S05; resolve Open Q2 first.)
- **T5 — consts cleanup + rename engine generics.** Delete `AGUMON_*` consts from `mod.rs`/`render.rs`; rename `*_agumon_*` engine systems/resources to `digimon_*` (mechanical). Update `use super::{...}` import block (`render.rs:34-36`).
- **T6 — source-contract tests.** Extend `tests/windowed_only/digimon_sprite_cue_dispatch.rs` (or add a new file) with `include_str!` assertions: engine files (`render.rs`, `mod.rs`) contain **zero** `AGUMON_`, no `fn on_enter_effect_ids`, no `fn load_agumon_enoki_vfx`; the agumon module file contains them. This is the structural proof of the slice contract (MEM030/MEM101 pattern).

## First Proof / Highest Risk

**T2 (enoki lifecycle-as-data) is the highest-risk unblocker.** It is the only conversion that changes the *spawn path* control flow, and VFX correctness is K001-manual (not headless-testable post-D043). Do it early and verify with `cargo build --features windowed` + the existing enoki contract tests (`tests/windowed_only/enoki_*`). If the lifecycle enum models charge/ember/projectile/oneshot cleanly, the rest is mechanical. T1 (cues) is the cheapest *first* compile-green proof that the module/`register()` seam works end-to-end before tackling T2.

## Verification

- `cargo build --features windowed` — exit 0, zero warnings (run after each task; S03 baseline was clean).
- `cargo test --features windowed --test windowed_only` — must stay green (currently 59 passed; T6 adds extraction-contract tests).
- `cargo test --test dependency_gating` — 2/2 green (no enoki/windowed leak into headless).
- `cargo test` (full headless) — green.
- **Structural grep (slice contract):** `rg 'AGUMON_' src/windowed/render.rs src/windowed/mod.rs` → 0 hits; `rg 'fn on_enter_effect_ids|fn load_agumon_enoki_vfx' src/windowed/` → only under `digimon/agumon/`. Encode as include_str! tests (T6).
- **K001 manual** (`cargo winx`): Agumon idle/skill/hurt/death + all VFX + flash/shake/camera-shake behave **identically** to pre-S04. Auto-mode cannot run the binary; observability seam is the existing `trace!(target: "windowed.agumon_playback", ...)` arming logs.

## Open Questions (for planner / grill before exec)

1. **Aggregator-edit semantics for S05.** If S04 introduces `digimon::register_all(app)` calling `agumon::register`, does S05 adding `renamon::register` to that aggregator count as an "engine edit" and fail the zero-edit gate? Resolve now. Likely answer: the aggregator is the *registration manifest* (analogous to `blueprints/mod.rs::BLUEPRINTS`/`register_blueprints`, which is edited per blueprint and accepted) — so editing it is allowed, OR use an inventory/linkme-style auto-collect to truly avoid edits. **Recommend:** treat the per-digimon `mod` + `register` call line as the sanctioned manifest edit (mirrors the lib blueprint convention) and scope the "zero engine edits" gate to `render.rs`/the generic systems — confirm with user.
2. **Unit → digimon-presentation resolution.** `Unit` (`src/combat/unit.rs:5-14`) has `name`/`evo_stage` but **no species/blueprint key**. `spawn_unit_sprites` currently gives *every* unit the Agumon atlas + graphs. For S04 (Agumon-only) the registry can hold one entry and the spawn site picks it; but S05 needs per-unit resolution. Decide the key now (Unit gains a `species`/`blueprint_id` field? map via `name`? via the existing combat blueprint owner?) so T4's registry is keyed correctly and S05 doesn't force an engine edit.
3. **Scope of "zero Agumon identifiers."** Confirm whether renaming engine systems (`advance_agumon_presentation`→`advance_digimon_presentation`, etc.) and the trace target `"windowed.agumon_playback"` are in-scope. The S05 gate (grep/diff on engine edits when adding Renamon) does **not** require it. Recommend: rename the generic systems/resources (coherence), leave the trace-target string (churn vs. value).
4. **`DEFAULT_ANIM_CLIP_PATHS` clip-index-0 assumption** (`render.rs:779-781`, `animation/plugin.rs:24/560`). The atlas builder assumes Agumon is clip index 0. The presentation registry entry should carry the clip handle/index per digimon rather than hardcoding `.first()`. Confirm the clip-loading path exposes per-digimon clips for S05 (Renamon has `assets/digimon/renamon/clip.ron`).

## Existing Codebase / Prior Art

- `src/combat/blueprints/mod.rs` — the **exact convention to mirror**: per-name modules, a `BLUEPRINTS` manifest, `register_blueprints(app)` aggregator calling each `<name>::register_*`. Windowed `digimon/` should look like this.
- `src/windowed/render.rs:42-58` (`DigimonSprite`) — already data-carrying (S03); spawn site is the only remaining const consumer.
- `src/ui/cues` (`CueRegistry`/`CueDef`, MEM112) — register() is idempotent-identical, panics on conflicting def; safe for multi-module registration.
- `tests/windowed_only/digimon_sprite_cue_dispatch.rs` — S03 source-contract test to extend (MEM030/MEM101 include_str! pattern; binary-crate seams invisible to normal tests).
- `assets/digimon/agumon/` (full VFX set) vs `assets/digimon/renamon/` (only `anim_graph.ron` + `clip.ron`) — Renamon has no `.particle.ron`; S05 VFX is minimal/reused, out of S04 scope.

## Skills Discovered

No new external skills installed. Relevant installed skills: `bevy-ecs-expert` (registry resources + Startup system ordering), `rust-skills`/`rust-development` (module boundaries, enum-as-data over match-on-string). Applied the *design-it-twice* principle (3 registry shapes compared above) and *decompose-into-slices* (thin, dependency-ordered tasks T0–T6) per the requested skill activations.

## Sources

Local code only (no web search needed — established local pattern). Key files: `src/windowed/render.rs`, `src/windowed/mod.rs`, `src/combat/blueprints/mod.rs`, `src/combat/unit.rs`, S03-SUMMARY.
