---
name: bevy-enoki-vfx
description: >
  Anime cel-shading VFX (Digimon Survive / Honkai Star Rail look) authored on bevy_enoki. Use
  when working on ANY particle effect, .particle.ron, charge/projectile/impact/detonate bursts,
  or the visual verbs glow/spark/flash/aura/ember/star-burst/shockwave/trail/beam/dissolve — and
  when deciding whether an effect needs a hand-authored asset or just bevy_enoki primitives.
  Encodes the effect-agnostic L0-L4 asset-vs-primitive decision rule (keyed on the visual verb,
  not the creature) and a procedural-first art direction. Engine-generic: it knows bevy_enoki's
  capabilities, not any one project's assets — it tells you to search the host project for
  existing assets/conventions rather than assuming them.
---

<objective>
Make the agent decide and author bevy_enoki VFX for an anime cel-shading look: procedural-first,
the lowest cost level that still reads. The skill keys every decision on the *visual verb* of the
effect (glow, spark, aura, star-burst, trail, beam, dissolve…), never on a specific creature — so
it applies equally to an effect already on disk and one not yet authored. It is **engine-generic**:
it carries bevy_enoki's real capabilities and the art techniques, but **not** a catalogue of any
project's assets — when it needs to know whether an asset or a spawn-wiring exists, it tells you to
search the host project. It produces: a chosen authoring level (L0–L4), a concrete `.particle.ron`
(or a justification for an asset/shader), and a self-check against the anime-cel reading rules.
</objective>

<context>
Core finding this skill is built on: enoki is a *backend*, not a cinematic language — the look
lives in the layer above it (placement/rotation/turbulence + timing + HDR + value contrast),
which a host project wraps around the raw particle effects.

Five knowledge layers compose here, kept separate on purpose:
- **Target catalog** → `references/target-catalog.md`. *Read this when building a specific
  effect* (fire orb, charge aura, spark, explosion, dissolve, slash). Decomposes the reference
  clip family into per-verb recipes, the two technique families (particles vs
  mesh+shader), the "hero look is a GLOW not a shader" correction, and the explicit
  **when-to-request-an-asset** trigger table.
- **Art techniques** → `references/soft-particle-and-layering.md`. *Read this first when an
  effect "doesn't read"* — soft particles vs the default flat-square material, layering, the
  grayscale value test, density, and the cross-engine (Niagara/Unity) → enoki conversion table.
- **Anime-cel self-check** → `references/anime-cel-principles.md` (value contrast, impact frame,
  HDR core, shape language; the done checklist).
- **Backend capabilities** → `references/enoki-cookbook.md` (real `.particle.ron` fields incl.
  the `spawn_rate`-is-an-interval gotcha, the `scale_curve`-overrides-`scale` gotcha, the
  material-is-chosen-by-spawn-code reality, lifecycle, anchors, and the typical
  placement/rotation wrapper layer a host project adds).
- **Decision rule** → `references/decision-rule.md` (asset vs primitive, L0→L4).

(Older notes reference a generic `vfx-realtime` skill; it is **not installed** — the references
above are self-contained and do not depend on it.)

Load this skill when touching particle VFX in any form. It is executable on its own; read the
references on demand for the dense tables and real examples.
</context>

<process>

## Step 0: Decompose the reference clip — VFX vs NOT-VFX
A reference clip (a gameplay capture, a special-move recording, any source video)
is **not** a VFX. It is a creature performing a move, filmed by a camera, over a background, under
a HUD — and *somewhere in there* the particle VFX. enoki authors **only the particle VFX**. Before
naming any verb, split the clip into four buckets and scope three of them OUT:

| Bucket | Examples in a clip | enoki? |
|---|---|---|
| **Particle VFX** | flame body, embers, glow core, impact flash, shockwave, sparks, swirl, energy motes | ✅ **the only thing you author** |
| **Character / sprite body** | the Digimon, its attack pose/animation, a summoned creature, wings, a blade it holds | ❌ sprite/animation system — NOT enoki, NOT a particle "shape" |
| **Camera / scene** | zoom, shake, scene-cut to another shot, the arena/background, letterbox, light spill on the scene | ❌ camera/scene/`Bloom` — NOT authored as a layer |
| **HUD / UI** | damage numbers, health bars, skill icons, watermark | ❌ ignore entirely |

**The trap (confirmed by baseline test):** the impressive part of a clip is often the *creature*
(a fire dragon, an angel with wings, a mech with a glowing axe), not the particles. Do not try to
reproduce the creature, its silhouette, or its animation in enoki — enoki has no creature, no
skeletal sprite, no camera. A whole recognizable creature/weapon body is **character art**, not an
L2/L3 particle silhouette. Author the particle VFX that *dresses* it (the fire on the dragon, the
sparks off the axe, the feather-motes around the wings) and explicitly hand the body, camera, and
HUD back to their own systems. State the split before proceeding.

If, after removing buckets 2–4, the remaining VFX is **empty** (the move is "an animated creature
+ a camera move" with no particle layer), say so — there is nothing for enoki to author.

## Step 1: Name the visual verb, not the Digimon
State what the effect *is visually*: glow core, spark, flash, shockwave, persistent aura,
rising ember, rotating star-burst, swirl/vortex, projectile, trail, beam-core, recognizable
silhouette (flame-tongue / blade / claw / petal / lightning bolt), evolving shape, or a
material signature (dissolve / distortion / rim). The verb — not the kit — decides the level.
If the verb matches a reference effect (fire orb, charge aura, spark, explosion, dissolve,
slash), open `references/target-catalog.md` for the worked recipe and its asset trigger.

**Family gate first:** is this **particles + glow** (orb, aura, spark, explosion, swirl,
dissolve — enoki-native) or **mesh/ribbon + scrolling-texture shader** (slash, beam, ribbon
trail — *not* an enoki primitive)? A slash/beam is Family B: flag it as out-of-enoki and request
a mesh+shader subsystem rather than faking a trail enoki cannot produce.

**Glow-first rule (counters the urge to over-engineer):** the project's hero combat VFX (fire
orb, explosion, charge aura) are **glows**, not cel-edged shapes — `wgsl-hero.md` is explicit
that "a glow / a burst / a swirl" is L0/L1, **not** L4 and not a flipbook. The gap between a dull
procedural fire and the spectacular reference is calibration of three knobs you already have, not
new tech: **HDR core pushed hard (`color_curve` channels ~4–6, not ~1.0), density/solid core,
and bloom strength.** Max those before considering any asset or shader.

## Step 2: Pick the lowest authoring level that reads (the decision rule)
Read `references/decision-rule.md` and apply it. Default bias: **the cheapest level that
holds the look at the real scale (14–34px, 12fps).** Summary:

| Level | What | Asset? | Use for (verb) |
|---|---|---|---|
| L0 | Pure primitive: color/scale + curves + HDR bloom | No | glow, spark, flash, shockwave, abstract motes |
| L1 | + `PlacementParams`/`RotationParams`/`Turbulence` | No | aura, converging ember, **rotating star-burst (HSR look)**, fan-out, swirl — covers most kits |
| L2 | 1 minimal cel sprite, canonically oriented, reused | Yes (1 atom, N effects) | a *recognizable silhouette* a primitive can't give |
| L3 | Animated sprite-sheet / atlas | Yes | the shape must *evolve over time* — hero moment |
| L4 | Custom `Particle2dMaterial` (WGSL) | Shader | dissolve / distortion / rim / gradient-map signature |

Escalate only on a concrete trigger: "doesn't read at 14–34px" → consider L2; "shape must
evolve" → L3; "needs a material look flat color can't give" → L4. Otherwise stay low.

## Step 3: Author it
- **L0/L1** → write a `.particle.ron`. Read `references/enoki-cookbook.md` for the full field
  list (the RON loader has **no field defaults** — list every field), the
  charge→release→impact→residue lifecycle pattern, semantic anchors, and the project's
  rotation/placement verbs. **Search the repo for existing `.particle.ron` files and read them as
  living examples** — as *examples*, not a template to clone.
- **L2** → add one reusable cel atom (luminance→alpha, single element, canonically oriented);
  rotate/scale it across effects. Place reusable atoms in the project's shared VFX asset directory
  (search for where existing atoms live).
- **L3** (flipbook) → read the flipbook recipe + per-effect material seam in
  `references/soft-particle-and-layering.md` (bottom-up frame order, warm-white sheet, ≤4×4, route
  onto shape layers only). **L4** (WGSL) → read `references/wgsl-hero.md` before committing. Both
  are hero-only and carry real maintenance cost.

## Step 3b: Decide whether to request an asset (and check the code prerequisite)
Default is **no asset** — a radial soft-particle blob carries every Family-A *body* (glow, orb,
explosion, aura, ember, dissolve). **First search the project** for an existing reusable atom (a
soft/radial/streak `*.png` in the asset dir, plus any existing `.particle.ron` to see what's in
use); request a *new* asset only when the search finds nothing fitting **and** a concrete verb
trigger applies, per the table in `references/target-catalog.md`:
- **non-radial silhouette** that must read at 14–34px (sharp spark spike, poison spine, blade
  streak, leaf, claw, petal) → request **1 cel atom** (L2, canonically oriented, luminance→alpha);
- **shape that evolves over time** (keyframed explosion flash, flickering body) → request a
  **≤4×4 sprite-sheet** (L3, hero);
- **dissolve / distortion / rim** material look → **WGSL material** (L4, hero);
- **slash / beam / ribbon** → a **mesh+shader subsystem outside enoki** (Family B).

When requesting: name the verb, why a primitive can't carry it at scale, the exact atom
(dimensions, orientation, what reads in grayscale), and note a deterministic generator script can
produce it if hand-art isn't available.

**Hard prerequisite for any L2/L3 asset:** many bevy_enoki setups spawn *every* effect through
one shared material, so all effects use the same texture. A shaped/flipbook texture does nothing
until per-effect material assignment exists. **Find the spawn site** (search for `ParticleSpawner`)
and confirm whether it threads a per-effect material handle; if not, that wiring is a code change
that must land first. Surface it as a prerequisite before authoring, don't discover it
mid-implementation.

## Step 4: Self-check against the reading rules
First the foundational check from `references/soft-particle-and-layering.md`: **are the
particles soft blobs or flat squares?** The default `ColorParticle2dMaterial` paints solid
squares — the top reason an effect "doesn't read as fire". If the verb needs a glowing body,
the soft-sprite material + layering is the fix (and it's a windowed *code* change, not an asset
edit). Then read `references/anime-cel-principles.md` and confirm: value contrast in grayscale
before color; density enough to read as a mass; intentional silhouette; a clear impact frame;
anticipation (charge) and follow-through (residue) if multi-stage; HDR white-hot core via bloom;
star-burst/shatter shape language where it fits. The look is *material + composition + timing +
shape + value*, not a single rendering trick.

## Step 5: Respect the host project's conventions (discover them, don't assume)
Read the project's spawn/registry code before authoring and follow what you find: semantic
anchors (e.g. a mouth / caster-center / target-center placement enum rather than pixel coords),
expiry/impact chaining for multi-stage effects, and presentation data (effect ids, anchors,
lifecycles) kept in per-kit registries consumed by a kit-agnostic engine — don't hardcode effect
ids in engine control flow. If the project distinguishes a canonical authored VFX schema from the
raw enoki `Particle2dEffect` backend, author against the canonical one. Visual signoff requires
the windowed build — request manual UAT (do not auto-run the windowed binary if project policy
forbids it).

</process>

<anti_patterns>
- **Skipping Step 0 — feeding the whole clip to enoki.** A reference clip is creature + camera +
  HUD + VFX. Author *only* the particle VFX; the animated creature/weapon body is character art,
  the zoom/shake/scene-cut/background is camera+scene, the numbers/bars are HUD. None of those
  three are particles, none are an L2/L3 "silhouette" — scope them out explicitly first.
- **Trying to rebuild a recognizable creature or weapon as a particle silhouette.** enoki has no
  creature/skeletal sprite. A fire dragon, a winged angel, a glowing blade *is the character*;
  enoki only authors the fire/sparks/motes dressing it. If the "VFX" left after Step 0 is empty,
  say so rather than inventing particle work.
- **Treating one kit's existing effects as the template.** The rule is keyed on the visual verb;
  on-disk effects are *applied examples*, not the model. Read them for shape, then author fresh.
- **Expecting cinematic HSR 1:1 from enoki alone.** enoki is the backend; the look is in the
  layer above (placement/rotation/turbulence + timing + HDR + value contrast).
- **Reaching for L3/L4 by fashion.** Sprite-sheets and WGSL are hero-only. Most of any kit
  lives at L0/L1 at this scale.
- **Authoring trail/ribbon/beam-mesh, sub-emitters, or screen-space compositing as if native** —
  they are not enoki primitives.
- **Tuning curves to fix a flat-square effect.** If particles render as solid squares
  (`ColorParticle2dMaterial`), no `color_curve`/`scale_curve`/HDR value fixes it — the fix is
  the soft-sprite material + layering (`soft-particle-and-layering.md`).
- **Trusting the web editor / headless capture for the *look*.** It renders with its own
  material and no bloom, so it shows flat dots regardless of the asset. Authoritative only for
  particle count / motion / logic; aesthetic signoff is windowed-only (manual UAT).
- **Reading `spawn_rate` as particles/sec.** It is the interval in seconds between emissions
  (`1/rate`); a small value is dense. See `enoki-cookbook.md`.
- **Shipping a single emitter for a body effect.** Fire/water/auras need 2–4 layered systems
  (core/flames/embers/smoke) — the glow is emergent from overlap, not one layer.
- **Reaching for a flipbook or WGSL shader to make a glow "more spectacular".** The hero fire/
  explosion is a glow (`wgsl-hero.md`): the fix for "dull" is HDR core ~4–6 + density + bloom,
  not new tech. Escalate only after those three are maxed and it still doesn't read.
- **Authoring a curve as a normalized 0→1 multiplier.** `scale_curve` is the ABSOLUTE size in
  world units and OVERRIDES `scale` — a curve peaking at `1.0` renders a 1px dot. Author peaks as
  real pixel sizes (see the `scale_curve` gotcha in `enoki-cookbook.md`).
- **Treating a slash/beam/ribbon as a particle trail.** It is a mesh ribbon + scrolling texture
  (Family B), not enoki — flag it and request a subsystem, don't fake a trail enoki can't make.
- **Promising a shaped/flipbook texture without checking the material wiring.** If the project
  spawns every effect through one shared material, an L2/L3 asset is inert until the spawn path
  can assign a per-effect texture. Verify the spawn site first — that wiring is the prerequisite.
- **Omitting `.particle.ron` fields.** The RON loader has no defaults — every field must be present.
</anti_patterns>

<success_criteria>
- [ ] If the input was a reference clip, Step 0 was done: the particle VFX was separated from the
      character/sprite body, camera/scene, and HUD, and only the particle VFX was authored.
- [ ] The effect was classified by its visual verb and **technique family** (particles+glow vs
      mesh+shader); Family-B verbs (slash/beam/ribbon) were flagged as out-of-enoki, not faked.
- [ ] It was assigned the lowest L0–L4 level that reads; for a glow/burst/aura, HDR core +
      density + bloom were exhausted before any asset/shader was considered.
- [ ] An asset was requested only on a stated trigger (non-radial silhouette / evolving shape /
      material signature), with the per-effect-material code prerequisite surfaced.
- [ ] If a `.particle.ron` was written, all fields are present (curves as absolute pixel sizes)
      and it follows the cookbook + target-catalog recipes.
- [ ] The result passes the anime-cel self-check (value contrast, impact frame, HDR core).
- [ ] The host project's conventions were discovered by reading its code and respected (anchors,
      chaining, no hardcoded ids, canonical schema, windowed-only manual visual signoff).
</success_criteria>
