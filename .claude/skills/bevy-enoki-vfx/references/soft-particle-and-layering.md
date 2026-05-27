# Soft particles, layering, and the art techniques — making it read as fire/water/etc.

This is the "why doesn't it look like fire?" reference. The decision rule (L0→L4) tells you
*how cheap* to author; this tells you *what makes the effect read* once you're authoring. It
folds in the cross-engine art research (VFX Apprentice, 80.lv anime breakdowns, Minions Art)
translated onto enoki's real knobs.

## The #1 root cause: enoki's default material draws hard flat squares

The single biggest reason a procedural effect "doesn't read as fire" is **not** the curve
values — it's the material. `ParticleSpawner::default()` is `ParticleSpawner<ColorParticle2dMaterial>`
with the default handle (`bevy_enoki-0.6.0/src/lib.rs:178`). That material's entire fragment
shader is:

```wgsl
// bevy_enoki-0.6.0/src/shaders/particle_color_frag.wgsl
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color * color;   // flat color across the whole quad — no radial falloff, no soft edge
}
```

So **every particle is a solid-color square.** No soft round blob, no alpha that falls off
toward the edge. Scattered squares never accumulate into a glowing mass — they read as
confetti, not flame. No amount of `color_curve`/`scale_curve`/HDR tuning fixes a square.

**The fix is a code change, not an asset change.** The "soft particle" look needs
`SpriteParticle2dMaterial` (`bevy_enoki-0.6.0/src/sprite.rs:11`) carrying a **radial-gradient
soft texture** (white center → transparent edge). That sampled alpha is what turns each quad
into a soft blob; overlapping soft blobs + HDR bloom = a glowing body. The material is chosen
by the *spawn code*, not by the `.particle.ron` — the `.ron` (`Particle2dEffect`) carries no
texture or material reference. To switch an effect to soft particles you must spawn
`ParticleSpawner::<SpriteParticle2dMaterial>(handle)` instead of the default, where `handle`
wraps the soft texture. This is the highest-leverage change available for the whole VFX system.

> Editor caveat: the bevy_enoki **web editor** (and SwiftShader software-WebGL in headless
> capture) renders with its own material and no HDR/bloom, so it shows flat 1px dots regardless
> of the asset. It is authoritative for the **particle count / motion / logic**, never for the
> aesthetic. Visual signoff is windowed-only — request manual UAT.

### Wiring the soft material (generic shape — adapt to the host project)

`ParticleSpawner` is generic: `ParticleSpawner<T: Particle2dMaterial>(pub Handle<T>)`
(`bevy_enoki-0.6.0/src/lib.rs:170`). `ParticleSpawner::default()` is only implemented for
`ColorParticle2dMaterial` (`lib.rs:178`) → flat squares. Both material plugins are **already
registered** by `EnokiPlugin` (`lib.rs:93-94`), so switching is purely a spawn-site change — no
new plugin, no `RonAssetPlugin`. The soft texture handle must live in a resource (the material is
an asset; you insert it once and clone the handle per spawn):

```rust
// 1. A resource holding the shared soft-particle material handle.
#[derive(Resource)]
struct SoftParticleMaterial(Handle<SpriteParticle2dMaterial>);

// 2. Startup system: load the radial PNG, build the material, store the handle.
fn init_soft_particle_material(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    mut commands: Commands,
) {
    let tex = asset_server.load("<your soft-particle texture path>");
    let handle = materials.add(SpriteParticle2dMaterial::from_texture(tex));
    commands.insert_resource(SoftParticleMaterial(handle));
}

// 3. At the spawn site — spawn with the soft material, not the default:
commands.spawn((
    ParticleSpawner(soft_material.0.clone()),   // was ParticleSpawner::default()
    ParticleEffectHandle(effect_handle.clone()),
    Transform::from_xyz(x, y, z),
));
```

Find the project's spawn site by searching for `ParticleSpawner`, and thread the material handle
in the same way the project already threads its effect registry. `SpriteParticle2dMaterial` is
imported from `bevy_enoki::`. The texture is a deterministic, headless-safe radial PNG (white
RGB, radial alpha falloff) committed to the project's asset directory — search for an existing
one before generating a new one.

## Value before color (the grayscale test)

Cel/anime readability comes from **value contrast**, not hue. Pro rule: imagine the effect in
**grayscale** — if the light-to-dark contrast doesn't read in black & white, it won't read in
color. Fire is white-hot core → bright yellow → orange → dark red → fade: a steep *value* ramp.
In enoki, drive `color_curve` with that ramp and push the hot end's channels **>1.0** (HDR) so
an `Hdr + Bloom + Tonemapping` camera blooms the core. Pick the hue only after the value ramp
reads. A flat mid-value gradient is the second most common "looks wrong" cause after the
square-particle problem.

## Silhouette: intentional shape, not scatter

Fire has a shape — wide base, slim leaning tongue, sharp tip. Water has an arc. That shape
comes from **low directional randomness + a deliberate `scale_curve`**, not from wide scatter.
`direction: Some(((0.0,1.0), 0.35))` with high randomness sprays particles every which way and
reads as sparks. Tighten the randomness (≈0.10–0.20) so the column leans coherently, and use
`scale_curve` to grow-then-collapse so the body tapers to a tip. Reserve a *recognizable*
silhouette (an actual flame-tongue cutout) for L2 — but most of the shape is free at L0/L1 via
direction discipline + scale curve.

## Layering: the missing 70% — one emitter is never enough

No serious fire is a single emitter. Real stylized fire is **2–4 overlapping systems** on the
same anchor, each a separate `.particle.ron` chained/co-spawned:

| Layer | Role | enoki recipe |
|---|---|---|
| **Core** | bright narrow white-hot body | tight `Circle`, small scale, `color_curve` channels well >1.0, low scatter, short life |
| **Flames** | wider, darker, leaning tongues | larger scale, more lifetime, lower HDR, slightly more scatter, upward `direction`+accel |
| **Embers** | sparse rising sparks | small `spawn_amount`, small scale, high speed randomness, long-ish life, turbulent drift |
| **Smoke** (opt.) | dark mass rising above | large scale, low/no HDR (dark), slow rise, long life, high `linear_damp` |

The glow is an *emergent* property of many soft, semi-transparent particles overlapping under
bloom — density + soft alpha. A single layer can't produce it. Water layers analogously: jet
core + droplet spray + falling mist + (opt.) splash ring on impact.

### How co-spawn works (no engine edit if the project supports a cue → many-ids map)

The layering mechanism needs **no engine edit** if the project's spawn layer maps one authored
cue name → a **list** of effect ids (a `HashMap<String, Vec<String>>`-style registry): the
on-enter handler iterates that list and spawns each id at the same anchor on the same frame, so
every id co-spawns. Search the spawn code for where a cue resolves to effect ids; if it resolves
to a single id, extend it to a vector (a small, kit-data change, not an engine change).

To build a layered fire you (1) register each layer's `.particle.ron` as its own effect entry
(anchor + a `PersistentEmitter` lifecycle), and (2) push all the layer ids into that one cue's
list. They co-spawn on the same anchor; lifecycle is per-entry. Use the **list fan-out** for
*simultaneous* layers; use **projectile-on-arrival** or per-effect **`OneShot`** chains for
*sequential* stages.

## Density & the additive-blend proxy

Stylized glow relies on **additive blending** — overlapping bright particles sum toward white.
enoki's render pipeline hardcodes `BlendState::ALPHA_BLENDING` (`bevy_enoki-0.6.0/src/material.rs:534`)
— there is **no per-material additive blend** and no asset/material knob to change it (true
additive = L4 custom pipeline). HDR overbright + bloom is the *proxy*. Practically: keep
particles soft and dense enough that the overlap-plus-bloom reads as a luminous mass. Too
few/too sparse and the proxy has nothing to accumulate. (`spawn_rate` controls density — see the
gotcha in `enoki-cookbook.md`: it is the *interval between emissions*, so a *small* value =
dense.)

## Motion = the physical story

- **Fire** = hot air rising: upward `direction` + positive `linear_acceleration` (speeds up as
  it climbs) + a little flicker via modest speed randomness. Light `linear_damp` settles the tip.
- **Water** = ballistic: fast upward launch + downward `gravity_*` → a parabola. Almost no damp,
  so gravity (not drag) shapes the arc.
- **Impact** = fast out then decelerate: high outward speed + strong *negative* accel + damp, so
  shards radiate then stop; `scale_curve` collapses to 0.
- Match the motion to the substance — the eye reads "fire" vs "spark" largely from how it moves.

## Timing (for casts, not loops)

Multi-stage casts read as anime via **anticipation → peak → dissipation**: charge orb →
release/projectile → impact burst → residue embers. At 12fps a single bright hold frame on
contact *is* the impact frame. Continuous test loops (a standalone fire/charge/ember emitter)
don't have stages — judge them on density/value/silhouette/soft-particle, not on timing.

## Cross-engine technique → enoki knob (conversion table)

The art references are Unreal/Unity/Niagara; here's how each transferable technique lands on
enoki's real capabilities:

| Technique (other engine) | enoki equivalent |
|---|---|
| Soft particle (radial-gradient sprite) | `SpriteParticle2dMaterial::from_texture(soft.png)` — **the key fix**; needs the spawn-code change |
| Hand-drawn flame flipbook | `SpriteParticle2dMaterial::new(tex, hframes, vframes)` → animates frames over lifetime (`particle_sprite_frag.wgsl`); ≤4×4 for stylized (L3) |
| Value/temperature gradient | `color_curve` HDR hot→cool ramp + bloom |
| Intentional silhouette | low `direction` randomness + `scale_curve` taper (L0/L1); cutout sprite only at L2 |
| Layered systems (core/flames/embers/smoke) | **multiple `.particle.ron` co-spawned on one anchor** (the table above) |
| Additive blend glow | HDR overbright + bloom proxy (true additive = L4 custom material) |
| Turbulence / organic drift | a turbulence placement wrapper if the project provides one (sum-of-sines, deterministic) |
| Rotating star-burst shards (HSR) | a radial-rotation wrapper + `OneShot` fan-out (no sprite) |
| Dissolve / distortion / rim | custom `Particle2dMaterial` WGSL (L4, hero only — `wgsl-hero.md`) |

What enoki simply lacks: cone emission shape (only `Point`/`Circle` — fake the cone with
`direction`+`scale`), trail/ribbon/beam mesh, native sub-emitters, screen-space compositing.
Don't author as if these exist.

## Sources
- VFX Apprentice — fire properties, flipbooks (the structured "VFX art" curriculum).
- 80.lv — "anime-inspired stylized fire beam" (Unreal): beam + outline + shockwave + ember
  decomposition, and the grayscale-contrast tip.
- Sheila Stipnieks — Stylized Fire breakdown (UE4 Niagara): layer-by-layer.
- Minions Art — stylized 2D VFX (Unity): gradient + hand-made flipbooks, closest to our 2D case.
- torch in sky — Stylized VFX in Unity: step/smoothstep/dissolve shader techniques.
- realtimevfx.com — the field's reference community.
