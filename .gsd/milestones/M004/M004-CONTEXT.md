# M004: Per-Digimon data-driven VFX (owned, extension-first)

**Gathered:** 2026-05-25
**Status:** Ready for planning

## Project Description

Port **all** of Agumon's combat VFX onto the owned, data-driven `vfx.ron` seam decided in D033/D034 — and make it look genuinely good, not placeholder. Three skills are in scope: **Sharp Claws** (basic, needs brand-new claw VFX — nothing renders today), **Baby Flame** (skill — move the hardcoded charge / ember / projectile / impact / shard-fan polish off `src/windowed/render.rs` onto data), and **Baby Burner** (ult — move the detonate flash onto data). Alongside the data port, this milestone adds the rendering tech needed to escape the "orange squares" look: an HDR `Camera2d` + `Bloom` + an additive-blend particle material in the windowed layer.

## Why This Milestone

Today's particles are flat alpha quads on a bare `Camera2d` (no HDR, no bloom, no additive blending) — the user explicitly calls the current result a placeholder that "is not good at all." The current Baby Flame polish is also hand-written in the windowed binary (the `VfxParticleKind` enum, `kind_from_name` string-match, per-skill offset fns), so each new skill or roster Digimon means touching the engine enum + match. We fix both before the M005+ roster multiplies the hardcoding: replace the closed enum with the extension-first `Registry<E>` verb seam, and replace flat quads with real glow. The biggest quality lever is rendering tech, not art — flat alpha quads can never read as fire no matter how good the texture.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Launch `cargo winx`, run an encounter, and see Agumon's **Sharp Claws** spawn a claw/slash VFX that did not exist before.
- See **Baby Flame** (charge ember-swirl → fast launch → impact shard-fan) and **Baby Burner** (detonate flash) render through the data-driven path with real glow (HDR + bloom + additive), looking visibly better than today's placeholder.
- Confirm (via grep / code read) that `VfxParticleKind` and `kind_from_name` no longer exist in `render.rs`, and that all Agumon effects are expressed in `assets/digimon/agumon/vfx.ron`.

### Entry point / environment

- Entry point: `cargo winx` (== `cargo run --features windowed`) for the visual surface; `cargo test` / nextest `agent` profile for the headless contract.
- Environment: local dev (windowed egui/winit for visuals; headless for CI-provable math).
- Live dependencies involved: none (no network, no DB). Particle textures, if any, are local asset files.

## Completion Class

- **Contract complete means:** headless tests load `assets/digimon/agumon/vfx.ron` into the typed `VfxAsset`, evaluate placement/appearance/curve verb math deterministically (R004), and prove variant selection maps a `VfxContext` → effect-tree variant — all without the windowed feature.
- **Integration complete means:** the existing FSM cue/barrier release path (D031/D032) still drives spawns; the generic Registry id dispatcher replaces only kind resolution, not the release seam; all three Agumon skills render end-to-end in `cargo winx`.
- **Operational complete means:** none beyond the above — this is a presentation/data milestone with no unattended lifecycle.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Headless: `vfx.ron` round-trips into typed verb-param structs and all verb math is deterministic and tested (R004), with zero hardcoded VFX-kind paths left in `render.rs`.
- Visual (manual, K001): the user reviews Sharp Claws, Baby Flame, and Baby Burner in `cargo winx` and signs off that they look good — iterated until satisfied. The user explicitly asked to be asked to review.
- What cannot be simulated: visual quality. It is not CI-assertable; the user's manual sign-off in the running windowed binary is the only valid proof of the "looks good" bar.

## Architectural Decisions

### Rendering tech is in scope (HDR + Bloom + additive blending)

**Decision:** Add an HDR `Camera2d` + `Bloom` post-process + an additive-blend material for VFX particles in the windowed layer (windowed-only, K001 manual-verified).

**Rationale:** The root cause of the placeholder look is that the windowed camera is a bare `Camera2d` rendering flat alpha quads — no amount of texture or curve tuning makes flat alpha read as fire. Additive + HDR + bloom is the single biggest quality jump and the precondition for everything else.

**Alternatives Considered:**
- Keep the flat material, push quality only through curves/textures — rejected; cannot produce glow.

### Texture path: spike all three, keep the best

**Decision:** Try all three particle-texture approaches and pick whichever reads best by eye: (a) a free CC0 pixel-VFX pack, (b) hand-authored / procedurally-generated grayscale PNG masks tinted via the schema, (c) fully procedural soft-circle additive meshes (no image files).

**Rationale:** The user wants to compare on-screen results rather than commit blind. All three are cheap to prototype on top of the additive material; the winner becomes the M004 path and others can be revisited later.

**Alternatives Considered:**
- Commit to one path up front — rejected; quality is subjective and best judged visually.
- Stable Diffusion / character asset generation — rejected; not needed for VFX. The `sprite-iterate` (3D→pixel-art character) pipeline is the wrong tool here.

### Skill-tree variation: prove the seam only

**Decision:** Prove the variant-selection seam is extensible (a `VfxContext` → selected effect-tree variant, headless-tested) but do NOT wire any real gameplay skill-tree unlock.

**Rationale:** There is no real skill-tree/unlock system in the codebase (only `Predicate::Unlock` for anim_graph node-gating). Shipping a real unlock is far too broad for M004. The variant-selection seam (D033 graft 5: selection keyed by `(skill_id, variant_key)`) is what must survive future extension; proving it deterministically is sufficient.

**Alternatives Considered:**
- Wire a real skill-tree unlock end-to-end — rejected by the user as out of scope, too broad.

> See `.gsd/DECISIONS.md` (D031–D034) for the full architectural register this milestone executes against.

## Error Handling Strategy

Load-time validation closes the `vfx.ron` ↔ anim_graph coupling: the `OffsetCurve.emitter_ticks` range and referenced verb ids / named appearances are validated when the asset loads, failing loudly with a contextual error (which Digimon, which effect, which verb/appearance) rather than silently mis-rendering. Verb resolution misses (an id not in the `Registry<E>`) are surfaced as explicit load/validation errors, not panics deep in the render loop. Pure verb math has no fallible IO; the headless tests are the regression net. Rendering failures are visual-only and caught by the manual `cargo winx` review.

## Risks and Unknowns

- **Scope is materially larger than the existing S01–S03 roadmap.** — The current roadmap was scoped to the Baby Flame + Baby Burner *port only*. The interview added: rendering tech (HDR/bloom/additive), brand-new Sharp Claws VFX, and a 3-way texture spike. The roadmap almost certainly needs reassessment at planning time before execution proceeds.
- **Visual quality is not CI-assertable (K001).** — The "looks good" bar lives entirely in manual review; auto-mode can never run the windowed binary, so the milestone cannot self-certify the visual half.
- **Texture-path spike is open-ended.** — "Try all three and pick the best" has no objective stop condition; needs a bounded comparison or it can absorb unbounded time.
- **HDR/bloom on a 2D pixel-art look may over-glow or wash out sprites.** — Bloom tuning interacts with the existing atlas sprites, not just particles; needs visual balancing.
- **`OffsetCurve.emitter_ticks` ↔ anim_graph frame-range coupling.** — Baby Flame's authored node ranges (e.g. the recover tail [60,77] overshooting the coarse `skill` clip label) mean placement timing must track the union of node ranges, not the clip label.

## Existing Codebase / Prior Art

- `src/windowed/render.rs` — holds the hardcoded `VfxParticleKind` enum, `kind_from_name` string-match, per-skill offset fns, and the bare `Camera2d`; the thing being replaced/upgraded. Only hardcoded skill-id reference lives here.
- `assets/digimon/agumon/` — destination for the new `vfx.ron`; already holds `anim_graph.ron`, `clip.ron`, `stance.ron`.
- `src/combat/.../Registry<E>` (D031) + `src/animation/registry.rs` — the verb-registration seam (`register("ns/name", fn)`, `Registry::iter()`) the VFX verbs plug into; same idiom as SelectorExt/PredicateExt/blueprints.
- `src/animation/anim_graph.rs` — existing typed Serialize/Deserialize asset pattern the typed `VfxAsset` mirrors; the FSM cue/barrier release path (D031/D032) that must keep driving spawns.
- `src/combat/blueprints/agumon/` — where a novel Agumon motion verb would be registered if RON-only reuse is insufficient.
- `.gsd/milestones/M004/M004-RESEARCH.md` — the `design-an-interface` four-design deliberation behind D033/D034.

## Relevant Requirements

- **R002 (headless-first)** — all verb math runs without `windowed`; egui/winit gated behind `#[cfg(feature = "windowed")]`.
- **R004 (determinism)** — placement/appearance/curve/variant-selection math is pure, seeded, and snapshot-stable.
- **R005 (dep gating)** — no winit/wgpu/egui deps outside `windowed`; any particle/bloom deps stay windowed-gated.

## Scope

### In Scope

- Owned typed `VfxAsset` schema (Serialize + Deserialize + Reflect, editor-ready per D034) loaded from `assets/digimon/agumon/vfx.ron`.
- Placement / appearance / variation verb math in `Registry<E>`, pure and headless-tested.
- Porting Baby Flame (charge/ember/launch/impact/shard-fan) and Baby Burner (detonate) off hardcoded `render.rs` onto the data path.
- Brand-new **Sharp Claws** claw/slash VFX (does not exist today).
- Rendering tech: HDR `Camera2d` + Bloom + additive-blend particle material (windowed-only).
- Texture spike: prototype all three texture approaches and select the best.
- Variant-selection seam proven extensible via headless `VfxContext` → variant test.
- Removal of `VfxParticleKind` and `kind_from_name` from `render.rs`.

### Out of Scope / Non-Goals

- Any real gameplay skill-tree / unlock system (seam only).
- Other Digimon's VFX (Renamon, etc.) — Agumon only this milestone.
- The GUI editor itself (M004 only adopts the editor-ready param shape; the editor is a later milestone).
- Adopting `bevy_enoki` or any external particle backend (deferred appearance adapter per D033).
- Stable Diffusion / character asset generation.
- Target hurt/flinch reaction animation (known unbuilt work, unrelated).
- Expression-tree placement evaluator and runtime modding (D033 deferred-additive).

## Technical Constraints

- K001: the windowed binary must never be run from auto-mode; visual verification is a manual user step.
- R002/R004/R005 as above: headless-first, deterministic, windowed-gated deps.
- The generic Registry dispatcher replaces only kind resolution — it must not alter the FSM cue/barrier release seam (D031/D032).
- `vfx.ron` schema must be editor-ready from S01: typed per-verb param structs deriving Serialize + Deserialize + Reflect, not a stringly-typed param map (D034).
- Load-time validation for the `emitter_ticks` ↔ anim_graph frame-range coupling.

## Integration Points

- Windowed render layer (`src/windowed/render.rs`) — consumes resolved verbs + curves to spawn/animate particle entities; gains the HDR camera + bloom + additive material.
- `Registry<E>` verb registry (D031) — VFX placement/appearance/variation verbs registered here; render becomes a generic id dispatcher.
- FSM cue/barrier release (D031/D032) — continues to trigger spawns at the right animation frames; unchanged by this milestone.
- Bevy asset loader — loads and validates `vfx.ron` into the typed `VfxAsset`.

## Testing Requirements

Headless (CI-provable, R004): `vfx.ron` round-trips into typed verb-param structs; `eval_placement` / `eval_appearance` / `eval_curve_*` produce deterministic, snapshot-stable output; `on_expire` chaining (charge→launch→impact) resolves from data; variant selection maps a synthetic `VfxContext` → effect-tree variant deterministically; a static grep asserts `VfxParticleKind` and `kind_from_name` are gone from `render.rs`. Tests live under the appropriate `tests/<scope>/` harness (R003). Visual (manual, K001): user sign-off in `cargo winx` for Sharp Claws, Baby Flame, Baby Burner — iterated until the user is satisfied with the look.

## Acceptance Criteria

Per-slice criteria to be (re)confirmed at planning/reassessment. At the milestone level:

- Zero hardcoded VFX-kind paths remain in `render.rs` (`VfxParticleKind` + `kind_from_name` removed).
- Every Agumon effect (Sharp Claws claw VFX, Baby Flame charge/launch/impact, Baby Burner detonate) is expressed in `assets/digimon/agumon/vfx.ron`.
- Adding an effect that reuses existing verbs is RON-only; a novel motion is one `register("ns/name", fn)` in the Agumon blueprint with no core change.
- All placement/appearance/variation verb math is headless-tested and deterministic.
- The `vfx.ron` schema is editor-ready (typed, introspectable, Reflect-deriving verb params).
- HDR + bloom + additive rendering is in place and the user signs off that all three skills look good in `cargo winx`.

## Open Questions

- Exact slice breakdown — current S01–S03 roadmap predates the scope expansion (rendering tech + Sharp Claws + texture spike); needs reassessment before execution. Current thinking: rendering tech likely earns its own early slice since it gates the visual bar.
- Which texture path wins (CC0 pack / PNG masks / procedural meshes) — deliberately deferred to the spike; resolved by visual comparison, not up front.
