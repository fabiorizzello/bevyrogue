# M004 Research — VFX schema design (design-an-interface)

Captures the deliberation behind D033 (VFX architecture) and D034 (editor-ready
constraint). Source: a `design-an-interface` pass (four designs) plus a focused
review of `bevy_enoki` 0.6 and the existing `Registry<E>` (D031), `anim_graph.rs`,
and the hardcoded polish in `src/windowed/render.rs`.

## Problem

The three-phase Baby Flame polish (ember swirl on charge → fast launch → shard
fan-out on impact) was hand-written in the windowed binary: const tuning block,
a `VfxParticleKind` enum, a `kind_from_name` string-match, and per-skill offset
fns. Adding the next skill's VFX (or the M005+ roster) means touching the engine
enum + match — the hardcoding to remove. Goal: an extension-first seam where a
Digimon brings its own data (and, rarely, its own registered verb) without core
changes, and which a future single GUI editor (anim_graph + stance + vfx) can drive.

## The decomposition that unlocked the decision: three orthogonal axes

The repeated oscillation came from treating VFX as one problem. It is three:

1. **PLACEMENT** — where the emitter is this frame (mouth-follow windup curve,
   homing/arc-to-target, converge-inward). This is emitter-Transform motion and
   is **host-owned by architecture**, whether or not an external particle lib is used.
2. **APPEARANCE** — what a puff looks like (spread, lifetime, scale/color keyframe
   curves, burst/OneShot). Buy-vs-build candidate.
3. **VARIATION** — how the effect changes with game state (skill-tree unlocks:
   stronger, recolored, multiple flames).

`bevy_enoki` 0.6 (Bevy ^0.18, CPU 2D, `.particle.ron`, hot-reload, web editor)
covers only APPEARANCE. It has no point-attractor and no homing, so PLACEMENT
stays ours regardless. Adopting it buys a second uncontrolled schema that chases
Bevy, a separate web editor, and appearance escaping the headless test net — for
~40-50 saved lines. Rejected as foundation; left as a possible deferred APPEARANCE
adapter (the Design-D port) if appearance iteration ever becomes painful.

## The four designs

- **A — Flat owned single schema.** One `vfx.ron`, flat `ParticleDef` struct, one
  `tick_particle` fn. Simplest on paper but a false simplicity: `GrowPulse` is a
  type-lie vs `scale_curve`; flat non-composable verbs (no keyframe curves) force
  duplicate particles; placement and appearance mixed in one object. Overfits Baby Flame.
- **B — enoki binding + thin layer.** enoki owns appearance; we own only an
  `EmitterPlacement` enum + binding. Cancels appearance math, free hot-reload +
  editor. But two schemas (one not ours) that fragment the single-editor vision;
  appearance out of headless tests (R004); two durations to hand-sync.
- **C — Owned single schema, two explicit axes.** One `vfx.ron` where an effect =
  `(Placement) × (Appearance)`, `OffsetCurve` keyframed (solves the moving mouth in
  windup), `scale_curve`/`color_curve` as `CurveStop<T>` keyframe lists, reusable
  named appearance palette, pure `eval_placement`/`eval_appearance`/`eval_curve_*`.
  Covers 3/3 phases in one owned, headless-testable, round-trippable format. Cost:
  we write ~50 lines of curve eval; two-axis overhead for trivial effects; the
  `OffsetCurve.emitter_ticks` ↔ anim_graph frame-range coupling (closeable with
  load-time validation).
- **D — Ports & adapters.** Backend-agnostic schema + `ParticleBackend` trait with
  a pure adapter and an enoki adapter. Max flexibility but double abstraction cost,
  schema degrades to lowest-common-denominator, fragile `tick()` asymmetry, and the
  enoki editor still emits its own schema (no closed loop). YAGNI for one Digimon.

## Decision (D033): C as the spine, with grafts

Chosen because C alone satisfies the user's deciding lenses — a single editor on
one owned format, and R004 headless determinism — with the two ortho axes explicit
rather than confused. Grafts onto C:

1. `on_expire` chaining in **data** (from A/D): charge→launch→impact is data, not
   string-match in the engine.
2. enoki kept as a **deferred** appearance adapter (C's `Appearance` keyframes map
   1:1 onto enoki `MultiCurve`), not built now.
3. Reusable named appearance palette + inline fallback.
4. **Verbs live in `Registry<E>` (D031), not a closed enum match.** This is what
   makes the engine extension-first: a new Digimon writes `vfx.ron` (content
   extension, no recompile); a novel motion is `register("ns/name", fn)` in its
   blueprint (behavior extension, no core change) — same idiom as SelectorExt/
   PredicateExt and blueprints (D031/D032). Render becomes a generic id dispatcher;
   `kind_from_name` disappears.
5. **Variation by variant SELECTION** keyed by `(skill_id, variant_key)` (anim_graph
   idiom), with parametric `modifiers` (count/tint/size_mul/spread_mul) bound to the
   existing `PredicateExt` as sugar over selection. Selection (not field-patching)
   is required so a skill-tree node can swap in a wholly different `on_expire` chain
   (e.g. one flame → three separately-homing chains) — structural, not just scaled.

Deferred-additive (NOT in M004, designed so each is a later addition not a rewrite):
an expression-tree evaluator as one extra registered placement verb (data-driven
motion math without a turing-complete DSL), and third-party runtime modding
(scripting/WASM atop the registry by relaxing `&'static str` → `String`).

## Editor-ready refinement (D034)

A GUI editor needs two things at runtime: (a) which verbs exist — already free via
`Registry::iter()` (registry.rs:59); (b) which params each verb wants — the gap. A
stringly-typed `params: { "radius_px": 58.0 }` map hides param names/types/ranges
inside the verb fn body. Fix adopted from M004 S01: **typed per-verb param structs**
(e.g. `ConvergeInward { radius_px: f32, omega: f32 }`) deriving
`Serialize + Deserialize` (round-trip, already free in anim_graph.rs) **plus
`#[derive(Reflect)]`** so an egui editor generates forms by reflection. Chosen over a
registry-carried param manifest because typed structs are compiler-validated and are
the idiomatic match for Bevy reflection-driven UIs. Note: a single editor that also
edits anim_graph will require adding `#[derive(Reflect)]` to the existing anim_graph
types (currently Serialize/Deserialize only) — additive, scheduled as editor-milestone
work, not M004. The editor itself is a separate milestone after M004; M004 only adopts
the editor-ready param shape so the editor never forces a schema refactor.

## Constraints held throughout

- All verb math pure and headless-testable (R004); only rendering windowed-gated
  (R002/R005) — same boundary as atlas (D030) and anim_graph.
- No external particle dependency in the foundation (enoki deferred).
- The existing FSM cue/barrier release path (D031/D032) must keep driving spawns;
  the generic dispatcher replaces only the kind resolution, not the release seam.

## Example (Baby Flame, C + grafts)

`assets/digimon/agumon/vfx.ron` defines a reusable `appearances` palette and three
`effects` (charge/launch/impact). `baby_flame_charge` uses
`placement: { verb: "core/converge_inward", anchor: { verb: "core/offset_curve", keys: [...] }, params: {...} }`,
`emit: (count: 7, distribution: Ring)`, `appearance: Named("flame_puff")`, and
`on_expire: [ Spawn("baby_flame_launch") ]`. Launch chains to impact via `on_expire`.
The engine never knows the names "launch"/"impact"; it resolves verbs by id from the
Registry and evaluates pure curves. Under D034 the `params` become typed structs
rather than the stringly-typed map shown in early sketches.
