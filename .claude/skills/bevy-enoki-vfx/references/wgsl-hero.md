# WGSL hero layer — when (rarely) to write a custom `Particle2dMaterial`

L4 of the decision rule. **Signature / hero effects only.** This is the most expensive level by
maintenance cost; reach it only when a stated escalation trigger justifies it.

## When it's justified
A custom `Particle2dMaterial` (WGSL fragment shader) is the right call **only** for a material
look that flat color + curves + bloom cannot produce:
- **Dissolve / erosion** (noise-masked alpha clipping over lifetime).
- **Distortion / heat-haze / refraction** (screen-space UV offset).
- **Rim / fresnel** edge lighting.
- **Procedural gradient-map** recolor / banding beyond `color_curve`.

If the effect is "a bright burst", "a swirl", "a star-shatter", or "a glow" — that is L0/L1,
not L4. Do not write a shader for those.

## Hard limits to remember
- Bevy's built-in 2D `ColorMaterial` has **no true additive blending**. Use HDR bloom +
  overbright colors as the additive *proxy*. A custom material is the only path to genuine
  additive blend if that ever becomes a hard requirement.
- A `Particle2dMaterial` is GPU fragment logic; it cannot do trail/ribbon/beam *mesh*,
  sub-emitter spawning, or screen-space compositing — those are not enoki primitives at all.

## Process if you do reach L4
1. Confirm the trigger in writing: which material property is impossible at L0–L3.
2. Implement the `Particle2dMaterial` trait with a WGSL fragment shader; keep the vertex path
   stock.
3. Keep the shader a *signature layer* on top of the procedural base, not a replacement — the
   silhouette/timing should still read with the shader disabled.
4. Headless testability: the shader's visual intent isn't headless-verifiable; gate visual
   signoff to windowed manual UAT. Keep an overbright-channel or parameter assertion as the
   headless proxy where possible.

## Alternative before committing
If too many effects are drifting to L3/L4, that's a smell — re-examine whether the look really
needs material work or whether timing/shape/value (L0–L1 + anime-cel principles) was
under-exploited. The spike's "alternative path": enoki covers ~70–80%, a documented hero layer
covers the rest — keep that ratio honest.
