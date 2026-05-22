# S01: S01 — UAT

**Milestone:** M003
**Written:** 2026-05-22T10:30:01.989Z

# UAT: S01 — Bind the atlas: idle stance + basic render on both actors

## Preconditions

- Project builds clean for both headless and windowed targets (`cargo test` and `cargo build --features windowed` both exit 0)
- `assets/digimon/agumon_atlas.png` present (512×512 grid, 10 columns × 10 rows, 93 frames)
- `assets/digimon/agumon/clip.ron` defines `frame_size: (512,512)`, `columns: 10`, `rows: 10`, `total_frames: 93`

## Test Cases — Headless (automated, run with `cargo test --test animation`)

**TC-1: AtlasGeometry geometry contract**
- Test: `atlas_binding::agumon_atlas_geometry_matches_clip_meta`
- Expected: `AtlasGeometry::from_clip_meta` reproduces 512×512 / cols 10 / rows 10 / total 93
- Result: PASS ✓

**TC-2: Identity atlas_index within range**
- Test: `atlas_binding::atlas_index_is_identity_within_range`
- Expected: atlas_index(0)==Some(0), atlas_index(92)==Some(92)
- Result: PASS ✓

**TC-3: Out-of-range atlas_index returns None**
- Test: `atlas_binding::atlas_index_rejects_out_of_range_frames`
- Expected: atlas_index(93)==None, atlas_index(u32::MAX)==None
- Result: PASS ✓

**TC-4: Idle player frames map identity within idle range**
- Test: `atlas_binding::idle_player_frames_map_identity_within_idle_range`
- Expected: All frames from stance graph entry over 24 ticks stay in [53,58] and satisfy atlas_index(frame)==Some(frame)
- Result: PASS ✓

**TC-5: Sharp Claws player frames map identity within attack range**
- Test: `atlas_binding::sharp_claws_player_frames_map_identity_within_attack_range`
- Expected: windup→strike→recover frames stay in [0,8] and satisfy atlas_index(frame)==Some(frame)
- Result: PASS ✓

**TC-6: Impact-frame-on-rendered-frame invariant**
- Test: `atlas_binding::sharp_claws_release_cue_resolves_to_in_range_impact_atlas_tile`
- Expected: ReleaseKernel cue's clip frame (resolved from loaded anim_graph.ron, not hardcoded) lies in [0,8] and atlas_index(frame)==Some(frame)
- Result: PASS ✓

## Test Cases — Windowed build (automated)

**TC-7: Windowed build compiles clean**
- Command: `cargo build --features windowed`
- Expected: exit 0, no errors, on-screen Sprite bound with Handle<Image>+TextureAtlas (not `..default()`)
- Result: PASS ✓ (Finished dev profile, 0 errors)

## Test Cases — Visual (user-run per K001; auto-mode cannot launch windowed binary)

**TC-8: Idle stance loops on both actors**
- Command: `cargo winx`
- Steps: Launch the windowed binary; observe both Agumon ally and mirrored dummy
- Expected: Both actors cycle through visible idle frames continuously (not blank sprites)
- Outcome: Deferred to manual user verification

**TC-9: Sharp Claws animation plays with damage on the impact frame**
- Command: `cargo winx`, trigger basic attack
- Steps: Press the basic attack key; observe the attacker's animation and damage timing
- Expected: Sharp Claws windup→strike→recover animation plays as visible atlas tiles; damage number appears on the rendered impact frame, not on keypress
- Outcome: Deferred to manual user verification

## Edge Cases Covered

- Atlas image not yet loaded (transient): `build_agumon_atlas` silently early-returns; no warn! log spam during normal startup
- Atlas image `LoadState::Failed`: one-time `warn!` emitted; sprites do not spawn until atlas is ready
- Clip load state ready but asset missing: one-time `warn!` emitted (genuine failure signal)
- Frame out-of-range from atlas_index: index left unchanged (no panic, graceful no-op)

## UAT Type

Contract + integration headless (automated); windowed build automated; visual proof user-run.

## Not Proven By This UAT

- Baby Flame / Baby Burner atlas-driven animations (S02)
- VFX flash renders as visible particles (S03)
- Damage numbers, HP bars, UI panels (covered in R011/M002)
- Hot-reload mid-skill correctness (covered in R014/M002)
- Performance / frame time under load (K001 live soak, framework from M002/S06)
