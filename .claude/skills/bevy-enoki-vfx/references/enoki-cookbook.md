# enoki cookbook — fields, lifecycle, anchors, per-verb recipes

Backend reference for authoring `.particle.ron`. enoki's `Particle2dEffect` is the rendering
backend; a host project may wrap it in its own canonical authored schema. The RON loader has
**no field defaults** — every field below must be present in every `.particle.ron`.

## `.particle.ron` fields (authorable without code)

`spawn_rate`, `spawn_amount`, `emission_shape` (`Point` | `Circle(r)`), `lifetime`,
`linear_speed`, `linear_acceleration`, `direction`, `angular_speed`, `angular_acceleration`,
`scale`, `color`, `gravity_direction`, `gravity_speed`, `linear_damp`, `angular_damp`,
`scale_curve`, `color_curve`, `attractors`, `relative_positioning`.

- **GOTCHA — `spawn_rate` is an interval, not a rate.** It is the *seconds between emissions*,
  fed straight into a timer (`bevy_enoki-0.6.0/src/update.rs:137`
  `timer.set_duration(from_secs_f32(spawn_rate))`). So a **small** value = dense feed; a large
  value = nearly empty. For `R` particles/sec, author `spawn_rate: 1/R` (e.g. 55/sec → `0.018`).
  The name reads like "rate" and invites the inverse mistake — verified in the web editor: an
  emitter with `spawn_rate: 55.0` showed `Particles: 0`. `OneShot` effects use `spawn_rate: 0.0`
  and burst once via the `OneShot` component, so the gotcha doesn't bite them.
- Numeric fields are `Rval` = `(value, randomness)` — e.g. `lifetime: (0.32, 0.25)` is 0.32s ±0.25.
- `direction` and `gravity_direction` are `Rval<Vec2>`; a `Vec2` is a 2-float tuple `(x, y)`, so
  the whole field is `Some(((x, y), randomness))` — e.g. `direction: Some(((0.0, 1.0), 0.35))` is
  "up, ±0.35 scatter" (+Y is up in bevy 2D). `None` = enoki scatters radially from the emission
  shape. `linear_speed` is the magnitude along `direction`; `linear_acceleration` adds to that
  magnitude (positive = speed up, e.g. rising fire). `gravity_*` is independent of `direction` —
  launch up + gravity down gives a fountain arc.
- Curves are `MultiCurve`: `(points: [(value, t, easing), ...])`. `scale_curve` is f32;
  `color_curve` is `LinearRgba` written `(red: .., green: .., blue: .., alpha: ..)`. `t` is the
  normalized lifetime 0→1. Easing e.g. `SineOut`, `SineInOut`, or `None`.
- **GOTCHA — `scale_curve` is the ABSOLUTE size and OVERRIDES `scale`, in world units.** When a
  `scale_curve` is present, enoki sets `transform.scale = splat(scale_curve.lerp(progress))` every
  tick (`bevy_enoki-0.6.0/src/update.rs:290`) — it does NOT multiply the `scale` field, it replaces
  it, so `scale` becomes dead the moment a curve exists. The curve's *values* are the literal
  particle size: the base quad is 1×1 world unit (`particle_vertex.wgsl`), and the default
  `Camera2d` maps 1 unit → 1 px, so **a curve peaking at `1.0` renders a 1-pixel dot**. This is the
  trap that makes a whole VFX system render as sparse 1px specks despite "correct" `scale` fields —
  the curves get authored as if they were normalized 0→1 multipliers. Author curve values as real
  pixel sizes (e.g. a ~28px white-hot core peaks at `28.0`, a ~55px flame tongue at `55.0`). For an
  effect with no `scale_curve`, the `scale` Rval is used directly instead.
- `relative_positioning: Some(true)` makes particles track the moving anchor as a body
  (no world-space trail) — use for a charge orb that follows the mouth. `Some(false)` leaves a
  world-space trail as the emitter moves — use for a projectile head.

## Material reality — the `.ron` does NOT pick the particle look

Critical and easy to miss: a `.particle.ron` (`Particle2dEffect`) carries **no texture and no
material**. The material is chosen by the *spawn code*. `ParticleSpawner::default()` =
`ParticleSpawner<ColorParticle2dMaterial>`, whose shader paints each quad a **flat solid square**
(`particle_color_frag.wgsl`). That is why procedural effects read as scattered confetti rather
than a glowing mass — see `soft-particle-and-layering.md` for the root cause and the soft-sprite
fix (it is a code change to spawn `ParticleSpawner::<SpriteParticle2dMaterial>(soft_texture)`, not
an asset edit). Sprite-sheet flipbook animation over lifetime also rides `SpriteParticle2dMaterial`
(`new(tex, hframes, vframes)`, H×V frames; hot-reload works).

## The wrapper layer above enoki (where the look actually lives)

enoki is a low-level backend. A host project typically adds a wrapper layer that supplies the
verbs that make effects read — search the project for its actual API; the capability *categories*
to expect are:

- **Placement transforms** applied over the emitter (converge-inward, fan-out, arc-launch,
  turbulent drift, static). Turbulence as a deterministic sum-of-sines gives organic flicker.
- **Rotation modes** — static, radial (offset + angular velocity), toward-target, fixed-angle.
  Radial/toward-target map *exactly* onto the HSR "particle indexing + rotation at offset"
  technique → rotating star-burst shards with **no asset**.
- **Semantic anchors** — placement by meaning (mouth / caster-center / target-center) rather than
  pixel coords.
- **Lifecycles** — persistent emitter, projectile (with flight duration + on-arrival hook),
  one-shot burst.
- **Multi-stage chaining** — on-expiry / impact-spawn hooks for charge → release → impact →
  residue, with effect ids referenced from data, not hardcoded in engine control flow.
- **Render path** — an `Hdr` + `Bloom` + tonemapping camera makes an overbright white-hot core
  "free" via bloom when curve channels exceed 1.0.

## What requires CODE / shader (not authorable in `.ron`)
- Fragment-shader logic → custom `Particle2dMaterial` (WGSL): dissolve, distortion, fresnel/rim,
  mask/erosion, procedural gradient-map. See `wgsl-hero.md`.
- Non-native motion verbs (e.g. a caster→target projectile is orchestrated by the host project's
  advance system, not the `.ron`).
- Trail/ribbon/beam mesh, complex sub-emitters, screen-space compositing → **not native**. Do not
  author them as if enoki provides them.

## Per-verb recipes (start here, then tune)

- **Glow / white-hot core (L0):** `spawn_rate: 0`, tiny `spawn_amount`, `Point`/small `Circle`,
  short `lifetime`, `color_curve` with channels >1.0 → bloom carries the core. No motion.
- **Spark / motes (L0–L1):** small `Circle` emission, moderate `linear_speed` with high
  randomness, fast `scale_curve` shrink, short life.
- **Continuous aura / charge orb (L1, `PersistentEmitter`):** `spawn_rate > 0` for a dense
  steady cluster, strong `linear_damp` to keep it tight, `relative_positioning: Some(true)` so
  it tracks the anchor.
- **Impact burst / fan-out (L0–L1, `OneShot`):** `spawn_rate: 0`, larger `spawn_amount`,
  high outward `linear_speed` + strong negative `linear_acceleration` + `linear_damp` so shards
  radiate then decelerate; `scale_curve` collapses to 0. Hot→cool `color_curve`. Fold the central
  flash + outward shards into one burst.
- **Rotating star-burst (L1):** `OneShot` burst + a radial rotation wrapper so shards index
  outward and spin — the HSR look, no sprite.
- **Projectile (L1):** a projectile lifecycle with flight duration + an on-arrival hook; the hook
  chains the impact effect id from data, not a const.

## Examples: search the repo, don't assume
Before authoring, **search the repo for existing `.particle.ron` files** and read them for shape
— directional procedural atoms (rising fire via up `direction` + positive `linear_acceleration`;
ballistic water via up launch + downward `gravity`) and kit effects (warm, mouth-anchored,
`direction: None` radial scatter). Read them as *examples*, then author the new verb fresh.

Author/tune a `.particle.ron` in the bevy_enoki web editor (https://lommix.github.io/bevy_enoki)
or the upstream `enoki2d_editor` crate; there is no headless aesthetic preview — the look only
renders in a windowed build. If the editor saves with a newer `ron` than the game reads,
round-trip a changed file through the game once to confirm it still parses.
