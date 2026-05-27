# Target catalog — reference VFX → technique → recipe → asset trigger

This is the "how do I build *that* effect?" reference. It decomposes the **Digimon-Survive /
HSR-style** hero-VFX family (the look this art direction chases) into reproducible recipes on
bevy_enoki's real capabilities, and — critically — tells you **when a verb needs an asset that
may not exist yet** so you go *look for one* (and request authoring only if none is found)
instead of faking it badly. Keyed on the **visual verb**, not any specific creature — every
recipe transfers to any kit.

## The one correction that reframes everything: the hero look is a GLOW, not a shader

The biggest mistake to avoid: assuming "spectacular = cel edges = flipbook + WGSL". The hero
fire/explosion in this family is **a glow** — a white-hot core that *burns out* under bloom,
wrapped in a warm halo, with light spilling onto the scene. There are **no hard cel edges, no
hand-drawn flame shape, no custom shader.** `wgsl-hero.md` says it outright: *if the verb is "a
glow / a burst / a swirl", it is L0/L1, not L4.*

So the lever between a "soft but dull" procedural fire and a "spectacular" one is **not new
technology** — it is calibration of three knobs the engine already gives you:

1. **HDR core, pushed hard.** A hero core is *burned white* — `color_curve` channels in the
   **~4–6** range at t=0, not a timid `1.0–1.2`. That overdrive is what makes bloom explode.
2. **Density + a solid core.** The body must be near-opaque at the center (dense overlapping
   soft particles), not wispy and see-through. Tune `spawn_rate` small (= dense) and a tight,
   high-`spawn_amount` core layer.
3. **Bloom strength.** The spill-onto-scene look is the camera `Bloom` doing its job; if it
   reads flat, the bloom intensity in the windowed render is the suspect.

If after maxing all three a verb *still* doesn't read, only then consider an asset (below). Do
not jump to flipbook/shader because the first procedural pass looked dull — it's almost always
under-driven HDR + too sparse.

## Two technique families (decides whether enoki can even do it)

| Family | What it is | enoki-native? | Verbs |
|---|---|---|---|
| **A — particles + glow + bloom** | mass of soft (or shaped) sprites, HDR value-ramp, glow from bloom | ✅ yes — L0/L1 (+L2 shaped, +L3 flipbook) | glow core, fire orb, explosion, charge aura, swirl, spark, ember, dissolve, shockwave |
| **B — mesh/ribbon + scrolling-texture shader** | a strip of geometry with panned texture + erosion mask | ❌ **no** — enoki has no ribbon/trail mesh, no pan-UV | **slash arc, beam, sweeping cut, ribbon trail** |

Most hero verbs are Family A. A slash/beam is the lone Family B case — see its row below for
the "request a different subsystem" flag.

## Recipes (start here, then calibrate in the windowed build)

### Fire-orb projectile — Family A, L1, no new asset
The clearest fire reference: a streak that becomes a flying incandescent orb, then impacts.
Decompose into a charge → projectile → impact lifecycle:
- **Charge** (`PersistentEmitter` at a mouth/caster anchor, `relative_positioning: Some(true)`):
  dense soft cluster, warm HDR, anticipation.
- **Projectile head** (a projectile lifecycle with `flight_ticks` + an on-arrival hook,
  `relative_positioning: Some(false)` so it leaves a **world-space trail** as the emitter moves):
  co-spawn 2 layers on the head — a **white-hot core** (`Circle(3)`, `color_curve` channels
  ~4–6 → bloom) + a **turbulent flame body** (wider, lower HDR, turbulent drift). The trail is
  emergent from the moving dense emitter, not a separate ribbon.
- **Impact** (the arrival hook chains the impact effect): a `OneShot` flash + outward shards.
- **Light spill** onto the scene = the camera bloom, free; no extra layer.
Lever for "spectacular": core HDR + density + bloom (above), not shape.

### Charge aura (gold / cyan) — Family A, L1, no new asset
A dome/column of energy gathering around the unit, culminating in a flash.
- Rising soft motes: `direction: Some(((0,1), 0.15))` (tight, coherent column not scatter) +
  positive `linear_acceleration`, `PersistentEmitter` at a caster-center anchor.
- Layer core + outer aura via co-spawn (fan-out) for depth; warm gold or cool cyan
  `color_curve` HDR ramp.
- End on a flash (chain a `OneShot` on expiry) for the "charge complete" beat.

### Impact / hit-spark burst — Family A, L0–L1; **shaped texture if spikes must read**
- Base: `OneShot`, `spawn_rate: 0`, large `spawn_amount`, high outward `linear_speed` + strong
  **negative** `linear_acceleration` + `linear_damp` (radiate then stop), `scale_curve` collapses
  to 0, hot→cool `color_curve`.
- Star-burst identity: radial rotation (shards index outward and spin — the HSR motif) — still
  no asset.
- **Asset trigger:** if the spark must read as **sharp-tipped streaks/spikes** (not round
  motes) at 14–34px, a radial soft blob can't give that silhouette → look for a **shaped
  streak/spike atom** in the project; request one if absent. See the asset table.

### Explosion — Family A, L1 (L3 only if the flash shape must evolve)
- Dense white-hot core flash (`OneShot`, `Circle`, HDR ~5) + outward debris shards + warm halo.
- Stays L1 if the flash is a brightening-then-fading glow. **Escalate to L3 (flipbook) only**
  when the central flash must show an *evolving keyframed shape* (rolling fireball silhouette) a
  curve can't express — hero moment, request a ≤4×4 sheet if none exists.

### Enemy-dissolve burst — Family A, L1, no new asset
Unit despawn punctuated by dispersing sparks.
- Rising/scattering soft particles, `color_curve` alpha fading to 0 + `scale_curve` collapse,
  short life, `OneShot` or brief `PersistentEmitter` timed to the despawn.
- Note: a true **dissolve mask on the unit's own sprite** (erosion clip) is a different surface
  — that's L4 WGSL on the sprite material, not a particle effect. The *particle* dissolve burst
  is L1.

### Slash arc / beam / ribbon trail — Family B, **NOT enoki-native** → request/flag
An anime slash is a **mesh ribbon with a scrolling texture + erosion mask**, or a shader — it is
**not** a particle system. enoki has no ribbon/trail mesh and no pan-UV. Options, in order:
1. **Flag it as out-of-enoki** and request a dedicated mesh/ribbon + WGSL subsystem (the honest
   path for a real slash).
2. **Fake within enoki**: one large particle carrying a hand-drawn **slash flipbook** (L3 sheet,
   `SpriteParticle2dMaterial::new(tex, h, v)`) oriented toward the swing — passable for a quick
   cut, not a true ribbon. Requires the asset *and* per-effect material wiring (below).
Do not author a "trail" by hoping enoki produces one; it won't.

## When to request an asset (the explicit trigger table)

Default is **no asset** — a radial soft-particle texture covers every Family-A *body* (glow,
orb, explosion, aura, dissolve, ember). **Before requesting anything, search the project** for an
existing reusable atom (look in the asset directory for `*.png` soft/radial/streak textures and
read any existing `.particle.ron` to see what's already in use). Request a *new* asset only when
the search finds nothing fitting **and** a concrete verb trigger applies:

| You need… | Because… | Asset to look for / request | Level | Code prerequisite |
|---|---|---|---|---|
| Round soft glow body | the universal blob | a **radial soft-particle PNG** (white center → transparent edge) | L0/L1 | none if the project already shares one material |
| A **non-radial silhouette** (sharp spark spike, poison spine, blade streak, leaf, claw, petal) that must be recognizable at 14–34px | a radial blob can't make a directional/pointed shape | **1 cel atom**, canonically oriented (luminance→alpha), reused via rotation/scale | L2 | **per-effect material** (see gap) |
| A shape that **evolves over time** (keyframed explosion flash, flickering flame body) | curves + rotation can't keyframe a silhouette | **≤4×4 sprite-sheet** (`new(tex,h,v)`) | L3 (hero) | per-effect material |
| **Dissolve / distortion / rim / gradient-band** material signature | flat color + bloom can't produce a material look | **WGSL `Particle2dMaterial`** | L4 (hero) | custom material plugin |
| A **slash / beam / ribbon trail** | enoki has no ribbon mesh / pan-UV | a **mesh+shader subsystem** outside enoki | Family B | new subsystem |

How to request: state the verb, why a primitive can't carry it at scale, the exact atom
(dimensions, orientation, what reads in grayscale), and that a deterministic generator script can
produce it if hand-art isn't available.

## Code prerequisite gap — per-effect material (verify in *this* codebase before promising an asset)

Many bevy_enoki setups spawn **every** effect through one shared material (the default
`ParticleSpawner::default()` = `ColorParticle2dMaterial`, or a single shared
`SpriteParticle2dMaterial` handle). If so, **all effects use the same texture**, and any L2/L3
asset (shaped atom, flipbook) is inert until the spawn path can assign a *different* texture per
`.particle.ron`. **Find the spawn site** (search for `ParticleSpawner` in the codebase) and
confirm whether it threads a per-effect material handle. If it doesn't, per-effect material
assignment is a code change that must land *before* a shaped/flipbook asset does anything —
surface it as a prerequisite, don't discover it mid-author. Visual signoff requires the windowed
build (request manual UAT).

## Authoritative verification per verb
- **Density / motion / spawn_rate** of a single layer → the bevy_enoki web editor or any windowed
  preview's particle count (logic only).
- **Parse / counts / HDR-core / direction invariants** → headless tests.
- **The composite look (layering + soft material + bloom = "does it read?")** → the windowed
  build only (manual UAT). The web editor renders with its own material and no bloom — never
  authoritative for the aesthetic.

## Sources (technique decomposition)
- Digimon Survive combat VFX (the reference look this art direction targets).
- VFX Apprentice — fire properties, flipbooks (≤4×4 stylized rule), Booms & Blasts.
- Gabriel Aguiar — stylized fire (Unity VFX Graph): flipbook + ramp.
- Minions Art — 2D stylized VFX (Unity): closest to the 2D case.
- RealTimeVFX / Godot threads — air slash = mesh ribbon + scrolling texture (Family B).
- Sheila Stipnieks — stylized fire Niagara: layer-by-layer.
