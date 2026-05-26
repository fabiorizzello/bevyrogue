# M005: Combat visual feedback completion (reactions + enoki VFX)

**Vision:** Close the gap between complete combat logic and incomplete presentation so a full encounter is watchable end-to-end: wire the already-emitted OnHitTaken/UnitDied events to the already-authored stance hurt and death nodes (plus hit flash, shake, and canvas damage numbers), and replace the placeholder flat-quad VFX with bevy_enoki for Agumon's skills. Enemy stays an Agumon dummy; the shared sprite gives both-sides reactions for free.

## Success Criteria

- Event-to-stance-reaction mapping is a pure lib function with deterministic headless tests (hit, death, death-precedence, no-op)
- In cargo winx, every hit flinches the struck unit (both sides) and a 0-HP unit plays death and leaves the field
- Hit flash + shake visible on the struck sprite; damage numbers render on the pixel canvas, not only the egui panel
- bevy_enoki wired windowed-gated; at least one Agumon effect renders through it; static test proves no dep leak into headless build
- All three Agumon skills' VFX render through enoki with user K001 sign-off that they look better than the placeholder
- Full cargo test (headless + windowed) and cargo build --features windowed stay green

## Slices

- [ ] **S01: S01** `risk:low` `depends:[]`
  > After this: In cargo winx, hitting either combatant makes that sprite play the hurt frames then return to idle.

- [ ] **S02: Death reaction and field exit** `risk:low` `depends:[S01]`
  > After this: In cargo winx, a unit reaching 0 HP plays the death frames and fades off the field.

- [ ] **S03: Hit feedback flash shake and canvas damage numbers** `risk:low` `depends:[S01]`
  > After this: In cargo winx, each hit flashes and shakes the struck sprite and shows a floating damage number on the canvas over the target.

- [ ] **S04: bevy_enoki integration spike (one effect)** `risk:high` `depends:[]`
  > After this: In cargo winx, one Agumon skill's impact VFX renders through bevy_enoki from a .particle.ron asset; cargo test stays green and the dep-gating test passes.

- [ ] **S05: Full Agumon VFX migration to enoki** `[sketch]` `risk:medium` `depends:[S04]`
  > After this: In cargo winx, Sharp Claws, Baby Flame, and Baby Burner all render through enoki and the user signs off on the look.

## Boundary Map

Not provided.
