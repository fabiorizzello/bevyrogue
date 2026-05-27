# Anime cel-shading principles (translated onto enoki)

The "Digimon Survive / Honkai Star Rail" look is **composition + timing + shape language +
value contrast** — not a single rendering trick. These are the generic `vfx-realtime`
principles landed on this backend. Load `vfx-realtime` for the full treatment; this is the
enoki-specific self-check.

## Value before color
Cel shading = flat color bands + sharp contrast + outline. Readability comes from **value
contrast and abrupt transitions**, not soft gradients. In enoki: drive `color_curve` with a
hot→cool ramp where the hot end has channels >1.0 (HDR), and keep the value jump crisp rather
than a long smooth fade. Pick the color *after* the value reads.

## Timing + impact frames make the "anime" feel
Disney's 12 principles applied to real-time VFX: anticipation, slow-in/slow-out, timing,
impact frames. The anime feel is born from *timing and a clear impact frame*, not the material.
At 12fps every frame counts — a single bright hold frame on contact reads as an impact frame.

## Multi-stage, not a single burst
Anime effects are *sequences*: **charge → release → impact → residue**. Use enoki's
`on_expire`/`ImpactSpawnPlan` chaining and `EnokiLifecycle` to stage them. A skill cast is
anticipation (charge orb) → release (projectile) → impact (burst) → follow-through (residue
embers that persist briefly).

## Shape language: star-burst & shatter
HSR-style hits use a **star / kaleidoscope shape** and a "glass-break/shatter" motif; avoid
decals, build the burst from particles. In enoki this is `RotationParams::Radial`/`TowardTarget`
+ a `OneShot` fan-out — shards index outward and rotate. The shape language carries the identity
more than any texture.

## HDR white-hot core
The overbright core is "free" via `Hdr` + `Bloom::NATURAL`: push `color_curve` channels past
1.0 at the hot end and bloom blooms it. This is the headless-testable proxy for HDR-bloom
intent (D037: all RGB channels >1.0).

## Direction & follow-through
Initial damage reads fast and strong; the follow-through persists a beat longer. Directional
impacts with high-contrast, readability-optimized color. Keep the camera/stage in mind even
though signoff is manual (K001 — never run windowed from auto-mode).

## Self-check before declaring an effect done
- [ ] Value contrast reads at 14–34px, 12fps (squint test).
- [ ] There is a clear impact frame on contact.
- [ ] If multi-stage: anticipation and follow-through are both present.
- [ ] HDR core present where the verb wants a hot center (channels >1.0).
- [ ] Shape language fits (star-burst/shatter for hits) rather than a generic round puff.
