# S03: Hit feedback flash shake and canvas damage numbers — UAT

**Milestone:** M005
**Written:** 2026-05-26T09:09:13.980Z

# S03 UAT — Hit feedback flash, shake, and canvas damage numbers

## UAT Type
Manual windowed-binary verification (K001). The lib projection (arming, decay, kinematics, tint, offset, amount extraction) is headless-tested; only the visual appearance and placement require this UAT.

## Preconditions
- `cargo build --features windowed` is green (verified automatically).
- The reviewer runs `cargo winx` (or equivalent `cargo run --features windowed`) from `/home/fabio/dev/bevyrogue`.
- A two-combatant encounter is reachable via the normal startup flow (Agumon vs Agumon dummy).

## Test Steps

1. **Start a combat encounter.** Launch `cargo winx` and enter the combat loop so both combatants are on screen.

2. **Trigger a hit on either combatant.** Use any attack that produces an `OnHitTaken` event (e.g., Sharp Claws). Observe the struck sprite immediately.

3. **Verify flash tint.** The struck sprite briefly flashes (bright red-white tint) for the duration of `FLASH_TICKS` (~8 animation ticks ≈ sub-second), then returns to plain white. If the sprite is fading out due to death (`advance_death_fade`), the flash tint must NOT compete — the fade alpha takes over and the sprite still dies cleanly.

4. **Verify shake offset.** Simultaneously with the flash, the struck sprite visibly shakes (sub-pixel oscillating offset), then snaps back to its rest position with no residual drift after `SHAKE_TICKS`.

5. **Verify floating damage number.** A white integer damage number appears above the struck sprite on the pixel canvas (world-space `Text2d`, z=2.0, above VFX layer). The number floats upward and fades out over ~1 second (`DAMAGE_NUMBER_TICKS=12` anim ticks), then disappears.

6. **Verify multi-hit.** Trigger two rapid hits on the same combatant. Each hit should produce its own floating number; flash and shake re-arm to full on each hit (idempotent dedup keeps only one countdown per unit).

7. **Verify both sides.** Trigger a hit on the player-side Agumon and on the enemy-side dummy. Both should show flash + shake + number.

8. **Verify no residual effects.** After the flash/shake window expires, the sprite is back at exact rest position with white tint; no stale damage numbers are visible on screen.

## Expected Outcomes
- Flash: visible bright tint for ~8 animation ticks, no tint outside that window, no interference with death fade.
- Shake: visible oscillating offset for ~8 animation ticks, exact rest position restored with no drift.
- Damage number: legible white integer above the target, floats up and fades out, gone after ~1 second.
- No orphaned entities or visual artifacts after effects complete.

## Edge Cases
- Hit on a unit that is already dying (death-fade in progress): flash guard (`death_exiting.is_some() || fade_out.is_some()`) suppresses the tint; the death fade plays through cleanly.
- Hit on a unit with no live sprite (target unresolved by `find_sprite_xy`): `spawn_canvas_damage_numbers` skips silently with a `debug!` log; no orphan `Text2d` is spawned.
- Multiple rapid hits: each re-arms flash/shake to full; each spawns an independent `Text2d` number; they stack and float/fade independently.

## Not Proven By This UAT
- Headless correctness of arming, decay, kinematics, tint/offset values, and amount extraction — covered by 9 windowed_hit_feedback headless tests.
- Dep-gate (no Text2d/render types leaking into headless builds) — proven by `cargo test --no-run` exit 0.
- Build and test suite hygiene — proven by automated verification checks above.
