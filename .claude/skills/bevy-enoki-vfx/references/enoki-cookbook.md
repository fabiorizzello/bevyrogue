# enoki cookbook — fields, lifecycle, anchors, per-verb recipes

Backend reference for authoring `.particle.ron`. enoki's `Particle2dEffect` is the rendering
backend; `VfxAsset` is the canonical authored schema (D052). The RON loader has **no field
defaults** — every field below must be present in every `.particle.ron` (MEM098).

## `.particle.ron` fields (authorable without code)

`spawn_rate`, `spawn_amount`, `emission_shape` (`Point` | `Circle(r)`), `lifetime`,
`linear_speed`, `linear_acceleration`, `direction`, `angular_speed`, `angular_acceleration`,
`scale`, `color`, `gravity_direction`, `gravity_speed`, `linear_damp`, `angular_damp`,
`scale_curve`, `color_curve`, `attractors`, `relative_positioning`.

- Numeric fields are `Rval` = `(value, randomness)` — e.g. `lifetime: (0.32, 0.25)` is 0.32s ±0.25.
- `direction` and `gravity_direction` are `Rval<Vec2>`; a `Vec2` is a 2-float tuple `(x, y)`, so
  the whole field is `Some(((x, y), randomness))` — e.g. `direction: Some(((0.0, 1.0), 0.35))` is
  "up, ±0.35 scatter" (+Y is up in bevy 2D). Verified by RON round-trip 2026-05-27. `None` = enoki
  scatters radially from the emission shape. `linear_speed` is the magnitude along `direction`;
  `linear_acceleration` adds to that magnitude (positive = speed up, e.g. rising fire). `gravity_*`
  is independent of `direction` — launch up + gravity down gives a fountain arc (see `water_test`).
- Curves are `MultiCurve`: `(points: [(value, t, easing), ...])`. `scale_curve` is f32;
  `color_curve` is `LinearRgba` written `(red: .., green: .., blue: .., alpha: ..)`. `t` is the
  normalized lifetime 0→1. Easing e.g. `SineOut`, `SineInOut`, or `None`.
- `relative_positioning: Some(true)` makes particles track the moving anchor as a body
  (no world-space trail) — use for a charge orb that follows the mouth.
- Sprite-sheet animation over lifetime → `SpriteParticle2dMaterial` (H×V frames). Hot-reload works.

## Repo layer above enoki (the verbs that make the look)

These are this project's wrappers, mapped onto enoki spawns — not raw `.particle.ron` fields:

- **`PlacementParams`**: `ConvergeInward`, `FanOut`, `ArcLaunch`, `Turbulence`, `Static`.
- **`RotationParams`**: `Static`, `Radial{offset, omega}`, `TowardTarget{offset, omega}`,
  `Fixed{angle, omega}`. `Radial`/`TowardTarget` map *exactly* onto the HSR "particle indexing +
  rotation at offset" technique → rotating star-burst shards with no asset.
- **`PlacementAnchor`**: `Mouth`, `CasterCenter`, `TargetCenter` (semantic, not pixel coords).
- **`EnokiLifecycle`**: `PersistentEmitter`, `Projectile{flight_ticks, on_arrival}`, `OneShot`.
- **`on_expire` / `ImpactSpawnPlan`**: multi-stage chaining (charge → release → impact → residue).
- Render path: `Hdr` + `Bloom::NATURAL` + `Tonemapping::TonyMcMapface` → the overbright
  white-hot core is "free" via bloom when curve channels exceed 1.0.

## What requires CODE / shader (not authorable in `.ron`)
- Fragment-shader logic → custom `Particle2dMaterial` (WGSL): dissolve, distortion, fresnel/rim,
  mask/erosion, procedural gradient-map. See `wgsl-hero.md`.
- Non-native motion verbs (e.g. the caster→target `Projectile` is orchestrated by the repo's
  `advance_enoki_projectiles`, not the `.ron`).
- Trail/ribbon/beam mesh, complex sub-emitters, screen-space compositing → **not native**
  (confirmed spike 3). Do not author them as if enoki provides them.

## Per-verb recipes (start here, then tune)

- **Glow / white-hot core (L0):** `spawn_rate: 0`, tiny `spawn_amount`, `Point`/small `Circle`,
  short `lifetime`, `color_curve` with channels >1.0 → bloom carries the core. No motion.
- **Spark / motes (L0–L1):** small `Circle` emission, moderate `linear_speed` with high
  randomness, fast `scale_curve` shrink, short life.
- **Continuous aura / charge orb (L1, `PersistentEmitter`):** `spawn_rate > 0` for a dense
  steady cluster, strong `linear_damp` to keep it tight, `relative_positioning: Some(true)` so
  it tracks the anchor. (Real example: `agumon/baby_flame_charge.particle.ron`.)
- **Impact burst / fan-out (L0–L1, `OneShot`):** `spawn_rate: 0`, larger `spawn_amount`,
  high outward `linear_speed` + strong negative `linear_acceleration` + `linear_damp` so shards
  radiate then decelerate; `scale_curve` collapses to 0. Hot→cool `color_curve`. (Real example:
  `agumon/baby_flame_impact.particle.ron` — note it folds the central flash + outward shards
  into one burst.)
- **Rotating star-burst (L1):** `OneShot` burst + `RotationParams::Radial{offset, omega}` so
  shards index outward and spin — the HSR look, no sprite.
- **Projectile (L1):** `EnokiLifecycle::Projectile{flight_ticks, on_arrival}`; `on_arrival`
  chains the impact effect id from data, not a const (D049).

## Real on-disk examples (use as examples, NOT templates)
- Kit effects (warm, radial / mouth-anchored): `assets/digimon/agumon/{baby_flame_charge,
  baby_flame_ember, baby_flame_projectile, baby_flame_impact, baby_burner_detonate,
  sharp_claws_slash}.particle.ron`. All use `direction: None` (radial scatter).
- Generic procedural atoms (directional motion): `assets/vfx/{fire_test, water_test}.particle.ron`.
  These are the canonical examples of the `Rval<Vec2>` `direction`/`gravity_*` verbs the kit
  effects don't exercise — `fire_test` rises (up `direction` + positive `linear_acceleration`),
  `water_test` arcs (up launch + downward `gravity`). Author/tune `.particle.ron` in the
  bevy_enoki web editor (https://lommix.github.io/bevy_enoki) or the upstream `enoki2d_editor`
  crate — there is no in-repo headless VFX preview (K001: the look only renders in the windowed
  combat stack). The editor saves with `ron 0.12`; the game reads with `ron 0.8`, so round-trip a
  changed file through the game once.

Read these for shape, then author the new verb fresh.
