# Decision rule — asset vs primitive (L0 → L4)

The core of the skill. **Effect-agnostic**: it starts from the *visual verb*, not the Digimon,
so it applies to any kit (Petit Thunder, Blue Cyclone, Koyosetsu, Metal Cannon, Baby Flame, or
an effect not yet authored). Default bias: **the lowest level that holds the look at the real
scale (14–34px, 12fps).** Escalate only on a concrete trigger.

## Levels

### L0 — Pure primitive (default)
enoki particle with solid `color`/`scale` + `color_curve`/`scale_curve` + HDR bands for the cel
look. **No asset.**
→ glow core, spark, motes, point debris, flash, simple radial shockwave.
→ Condition: the silhouette reads as an **abstract shape** (point / circle / streak).

### L1 — Primitive + placement/rotation math
Add `PlacementParams` + `RotationParams` + `Turbulence` to give *intent* (converge, fan-out,
radial spin, toward-target). **Still no asset.**
→ aura swirl, converging ember, **rotating star-shards (HSR star-burst look)**, impact fan-out.
→ Covers **most of the anime-cel look** at this project's scale.

### L2 — Minimal cel sprite (1 reusable atom)
One small single-element cel texture, **canonically oriented** (e.g. `flame_tongue` points-up),
luminance→alpha, reused across many effects via rotation/scale. Reusable atoms live in `assets/vfx/`.
→ Use **only** when a **recognizable silhouette** is needed that the primitive can't give:
flame-tongue, leaf, blade, petal, claw, stylized lightning bolt, graphic symbol.
→ Rule: *1 atom, N effects.* The same sprite, rotated/scaled, serves many kits.

### L3 — Animated sprite-sheet / atlas
`SpriteParticle2dMaterial` flipbook, when the *shape changes over time* in a way curves +
rotation can't express (a flame that flickers, a keyframed explosion).
→ Medium-high cost (frame-by-frame authoring). **Hero moments only**, not common VFX.

### L4 — Custom `Particle2dMaterial` (WGSL)
**Signature/hero only**: dissolve, distortion, rim/fresnel, procedural gradient-map, mask
erosion. Maximum power, maximum maintenance cost. See `wgsl-hero.md`.

## Escalation triggers (when to go up a level)
- "Can't tell what it is" at 14–34px → maybe L2 (silhouette).
- "The shape needs to evolve over time" → L3.
- "Needs a material/energetic look flat color can't give" → L4.
- Otherwise: **stay at the lowest level that works** (`vfx-realtime`: the cheapest effect is
  the one that isn't there).

## Decision table (verb → level)

| Target verb | Level | Asset? |
|---|---|---|
| Glow core / white-hot center | L0 | No (bloom does the work) |
| Spark / debris / motes | L0–L1 | No |
| Flash / simple shockwave | L0 | No |
| Aura / swirl / converging ember | L1 | No |
| Rotating star-burst shards (HSR) | L1 | No (`RotationParams::Radial`) |
| Projectile caster→target | L1 (lifecycle `Projectile`) | Optional comet sprite |
| Recognizable silhouette (flame-tongue / blade / petal / claw / bolt) | L2 | Yes, 1 cel atom |
| Keyframed explosion / flicker | L3 | Yes, sprite-sheet |
| Dissolve / distortion / rim hero | L4 | WGSL shader |

## Confidence
**High** on backend capabilities (verified in code + README). **Medium** on the exact L1→L2
threshold ("silhouette legible") — it's a qualitative call, calibrate it by looking in-game at
12fps. The skill deliberately biases toward the lower level to counter the temptation to jump
to L3/L4.
