---
name: bevy-enoki-vfx
description: >
  Anime cel-shading VFX (Digimon Survive / Honkai Star Rail look) authored on bevy_enoki for
  this project. Use when working on ANY particle effect, .particle.ron, VfxAsset,
  EnokiLifecycle, charge/projectile/impact/detonate bursts, or the visual verbs
  glow/spark/flash/aura/ember/star-burst/shockwave/trail/beam/dissolve — and when deciding
  whether an effect needs a hand-authored asset or just bevy_enoki primitives. Encodes the
  effect-agnostic L0-L4 asset-vs-primitive decision rule (keyed on the visual verb, not the
  Digimon) and the repo's procedural-first art direction. Builds on the generic `vfx-realtime`
  skill — link to it for principles, do not duplicate them here.
---

<objective>
Make the agent decide and author bevy_enoki VFX the way this project wants them: anime
cel-shading look, procedural-first, the lowest cost level that still reads. The skill keys
every decision on the *visual verb* of the effect (glow, spark, aura, star-burst, trail,
beam, dissolve…), never on a specific Digimon — so it applies equally to an effect already on
disk and one not yet authored. It produces: a chosen authoring level (L0–L4), a concrete
`.particle.ron` (or a justification for an asset/shader), and a self-check against the
anime-cel reading rules.
</objective>

<context>
Derived from `.gsd/workflows/spikes/260526-5-ricerchiamo-il-modo-per-rendere-coding-a/RECOMMENDATION.md`
(dated 2026-05-26), which folded in spike 3's finding that enoki is a *backend*, not a
cinematic language — the look lives in the layer above it (placement/rotation/turbulence +
timing + HDR + value contrast).

Four knowledge layers compose here, kept separate on purpose:
- **Art techniques** → `references/soft-particle-and-layering.md`. *Read this first when an
  effect "doesn't read"* — soft particles vs the default flat-square material, layering, the
  grayscale value test, density, and the cross-engine (Niagara/Unity) → enoki conversion table.
- **Anime-cel self-check** → `references/anime-cel-principles.md` (value contrast, impact frame,
  HDR core, shape language; the done checklist).
- **Backend capabilities** → `references/enoki-cookbook.md` (real `.particle.ron` fields incl.
  the `spawn_rate`-is-an-interval gotcha, the material-is-chosen-by-spawn-code reality,
  lifecycle, anchors, the repo's `PlacementParams`/`RotationParams` layer).
- **Decision rule** → `references/decision-rule.md` (asset vs primitive, L0→L4).

(Older notes reference a generic `vfx-realtime` skill; it is **not installed** — the references
above are self-contained and do not depend on it.)

Load this skill when touching particle VFX in any form. It is executable on its own; read the
references on demand for the dense tables and real examples.
</context>

<process>

## Step 1: Name the visual verb, not the Digimon
State what the effect *is visually*: glow core, spark, flash, shockwave, persistent aura,
rising ember, rotating star-burst, swirl/vortex, projectile, trail, beam-core, recognizable
silhouette (flame-tongue / blade / claw / petal / lightning bolt), evolving shape, or a
material signature (dissolve / distortion / rim). The verb — not the kit — decides the level.

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
  list (the RON loader has **no field defaults** — list all fields, MEM098), the
  charge→release→impact→residue lifecycle pattern, semantic anchors, and the repo's
  rotation/placement verbs. Use real on-disk effects as living examples (currently only
  `assets/digimon/agumon/*.particle.ron`) — as *examples*, not a template to clone.
- **L2** → add one reusable cel atom (luminance→alpha, single element, canonically oriented);
  rotate/scale it across effects. Place reusable atoms in `assets/vfx/`.
- **L3/L4** → read `references/wgsl-hero.md` before committing; these are hero-only and carry
  real maintenance cost.

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

## Step 5: Respect repo conventions
Semantic anchors (`Mouth`/`CasterCenter`/`TargetCenter`), `on_expire`/`ImpactSpawnPlan`
chaining for multi-stage, no hardcoded effect ids in engine control flow (presentation data
lives in the per-Digimon registries under `src/windowed/digimon/<name>/`, consumed by a
species-agnostic engine — see D049/D052). `VfxAsset` is the canonical authored schema; enoki's
`Particle2dEffect` is the rendering backend (D052). Never run the windowed binary from
auto-mode (K001) — request manual UAT for visual signoff.

</process>

<anti_patterns>
- **Treating one Digimon's effects as the template.** The rule is keyed on the visual verb.
  Baby Flame is the only kit currently on disk; it is an *applied example*, not the model.
- **Expecting cinematic HSR 1:1 from enoki alone.** enoki is the backend; the look is in the
  layer above (placement/rotation/turbulence + timing + HDR + value contrast).
- **Reaching for L3/L4 by fashion.** Sprite-sheets and WGSL are hero-only. Most of any kit
  lives at L0/L1 at this scale.
- **Authoring trail/ribbon/beam-mesh, sub-emitters, or screen-space compositing as if native** —
  they are not enoki primitives (confirmed spike 3).
- **Tuning curves to fix a flat-square effect.** If particles render as solid squares
  (`ColorParticle2dMaterial`), no `color_curve`/`scale_curve`/HDR value fixes it — the fix is
  the soft-sprite material + layering (`soft-particle-and-layering.md`).
- **Trusting the web editor / headless capture for the *look*.** It renders with its own
  material and no bloom, so it shows flat dots regardless of the asset. Authoritative only for
  particle count / motion / logic; aesthetic signoff is windowed-only (K001, manual UAT).
- **Reading `spawn_rate` as particles/sec.** It is the interval in seconds between emissions
  (`1/rate`); a small value is dense. See `enoki-cookbook.md`.
- **Shipping a single emitter for a body effect.** Fire/water/auras need 2–4 layered systems
  (core/flames/embers/smoke) — the glow is emergent from overlap, not one layer.
- **Omitting `.particle.ron` fields.** The RON loader has no defaults — every field must be present.
</anti_patterns>

<success_criteria>
- [ ] The effect was classified by its visual verb, then assigned the lowest L0–L4 level that reads.
- [ ] An asset/shader was introduced only on a stated escalation trigger, not by default.
- [ ] If a `.particle.ron` was written, all fields are present and it follows the cookbook patterns.
- [ ] The result passes the anime-cel self-check (value contrast, impact frame, HDR core).
- [ ] Repo conventions respected (anchors, chaining, no hardcoded ids, VfxAsset-canonical, K001).
</success_criteria>
