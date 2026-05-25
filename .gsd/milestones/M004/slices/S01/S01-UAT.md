# S01: Owned vfx.ron schema + appearance curve eval (tracer bullet) — UAT

**Milestone:** M004
**Written:** 2026-05-25T10:17:12.332Z

# UAT — S01: Owned vfx.ron schema + appearance curve eval (tracer bullet)

## UAT Type
Visual / human — windowed runtime required. Auto-mode cannot open a window (K001); this UAT must be performed by a human.

## Preconditions
- Bevy development environment installed (`rustup`, `cargo`)
- `cargo winx` alias available (runs `cargo run --features windowed`)
- `assets/digimon/agumon/vfx.ron` present at project root

## Steps

1. Run `cargo winx` to launch the windowed game.
2. Allow the game to reach the combat screen with Agumon loaded.
3. Trigger Agumon's Baby Flame attack to fire a projectile.
4. Observe the moment the Baby Flame projectile impacts.

## Expected Outcomes

- **Fan-out burst:** 8 orange-hued shard particles emit from the impact point, spreading outward in a fan.
- **Ease-out spread:** Shards expand outward with ease-out motion — faster initially, slowing toward the end — consistent with the authored `scale` curve keyframes (0.0→0.0, 0.5→0.75, 1.0→1.0 × spread_px 64).
- **Alpha fade:** Shard color holds the orange hue (approximately srgba 1.0, 0.55, 0.2) while alpha fades from ~0.9 to 0.0 over 5 ticks.
- **Central flash:** A brief bright central flash (approximately srgba 1.0, 0.82, 0.45) appears simultaneously for ~2 ticks before fading.
- **No crash or panic** during normal gameplay.

## Edge Case — Missing Asset Fallback

5. Rename `assets/digimon/agumon/vfx.ron` to `vfx.ron.bak` and relaunch `cargo winx`.
6. Trigger Baby Flame impact again.

Expected: Impact fan-out still renders (hardcoded fallback path). A structured `[WARN windowed.agumon_playback]` message naming the missing asset appears in the terminal output. No panic or silent VFX drop.

7. Restore `assets/digimon/agumon/vfx.ron` and confirm normal behavior returns.

## Not Proven By This UAT

- Placement-verb Registry resolution (S02 — `VfxParticleKind` / `vfx_particle_kind` still exist)
- Baby Flame charge and launch rendered through Registry-resolved placement verbs (S02)
- Baby Burner detonate rendered from vfx.ron data path (S03)
- Effect variant selection via `VfxContext` (S03)
- Full removal of hardcoded VFX paths from render.rs (S02)
