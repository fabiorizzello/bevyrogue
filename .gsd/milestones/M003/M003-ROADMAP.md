# M003: Make Agumon Render On-Screen

**Vision:** Turn M002's proven-but-invisible combat stack into something you can watch fight. Bind the existing 512px Agumon grid atlas to the on-screen Sprite, drive Sprite.texture_atlas.index from the AnimGraphPlayer's current frame, and make all five animation surfaces — idle stance, basic (Sharp Claws), skill (Baby Flame), ultimate (Baby Burner), and the VFX flash — render as real pixels on both the Agumon ally and a mirrored Agumon dummy, with damage landing on the rendered impact frame via the existing two-clock cue barrier.

## Success Criteria

- cargo winx: Agumon ally + mirrored Agumon dummy both render; idle stance loops; basic/skill/ultimate play their correct atlas frame ranges as visible animation; VFX flash renders as particles. (User-validated visually; K001.)
- Damage lands on the animation impact frame, not on keypress — driven by the two-clock cue barrier bound to the rendered frame. (Headless invariant + visual.)
- Headless suite green: on-screen Sprite carries a bound Handle<Image> + TextureAtlas (not ..default()); AnimGraphPlayer frame maps onto TextureAtlas.index; clip↔atlas range parity holds; impact-frame damage invariant holds on the rendered frame for all three skill timelines.
- No windowed-gated deps leak into headless paths; full headless suite and both builds still pass.

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: Headless: tests prove the on-screen Sprite carries a bound Handle<Image>+TextureAtlas (not ..default()), the player frame maps 1:1 onto TextureAtlas.index across idle/attack ranges, clip↔atlas parity holds, and the impact-frame damage invariant holds on the rendered frame for Sharp Claws. Visual (user-run cargo winx, K001): two Agumon idle-loop and the basic Sharp Claws animation plays with damage landing on the impact frame.

- [x] **S02: S02** `risk:high` `depends:[]`
  > After this: Headless: impact-frame damage invariant holds on the rendered frame for Baby Flame (skill range) and Baby Burner (heavy_attack range) — release fires on the cue frame, not auto-released. Visual (user-run cargo winx, K001): skill (Baby Flame) and ultimate (Baby Burner) play their correct atlas frame ranges as smooth animation on both actors with damage/effects landing on the impact frame.

- [ ] **S03: VFX flash renders as visible particles** `risk:medium` `depends:[S01,S02]`
  > After this: Headless: a structural test asserts the SpawnParticle/detonate seam yields a renderable particle spawn (entity with visual components, VfxLocus/VfxMotion honored) rather than only an opaque ParticleId, with no numeric gameplay payload in the serialized form (vfx_handle_seam parity preserved). Visual (user-run cargo winx, K001): the VFX flash appears as visible particles during skill and ultimate on both actors.

## Boundary Map

## Boundary Map

- **asset_server** — loads assets/digimon/agumon_atlas.png (Handle<Image>) alongside the already-loaded Clip/AnimGraph RON. New: image load + load-state surface for the atlas PNG.
- **Lib (bevyrogue::animation)** — new testable contract: build TextureAtlasLayout-equivalent geometry (columns/rows/frame_size) from Clip.meta and an identity frame→atlas-index map, reachable from headless tests under tests/ (which link only the lib, not the binary).
- **src/windowed/render.rs** — binds Handle<Image> + TextureAtlas onto the on-screen Sprite (replacing Sprite{..default()}), drives texture_atlas.index from AnimGraphPlayer each tick, and extends the presentation bridge from Sharp Claws-only to Baby Flame + Baby Burner.
- **Two-clock cue barrier (TimelineClock / Clock::Windowed, SuspendedTimelineState)** — releases on the rendered impact frame for all three timelines.
- **VFX seam (SpawnParticle / ParticleId / VfxLocus / VfxMotion)** — routed through the Cue/reactive bus into real particle entities; no physics/colliders (CAP-7c065a44).
- **§9 UI panels** — continue to read CombatEvent (event-driven, never mutate CombatState); BabyBurnerFlashState egui chip stays as the UI affordance while the world-particle render is added.
