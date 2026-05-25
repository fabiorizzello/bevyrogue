---
id: M003
title: "Make Agumon Render On-Screen"
status: complete
completed_at: 2026-05-25T07:54:47.886Z
key_decisions:
  - M003 reframed as the pixel-binding milestone (bind existing atlas, render all five Agumon surfaces) rather than roster extension, which slides to M004+
  - Slice order binding-first: S01 atlas binding riding Sharp Claws → S02 Baby Flame/Baby Burner cue bridges → S03 VFX world particles
  - Bounce is VFX/gameplay-only and stays out of the animation FSM; Baby Flame path cast→impact→recover is linear by construction
  - AnimationClock decoupled from render frame rate (12 fps default, catch-up capped at 4 ticks); barrier release fires only on animation ticks
  - Baby Burner detonate reuses the authored VFX seam by synthesizing a SpawnParticle, preserving the egui chip path alongside the new world particle
key_files:
  - src/windowed/render.rs
  - src/animation/vfx.rs
  - src/animation/mod.rs
  - src/combat/blueprints/agumon/baby_burner.rs
  - tests/animation/vfx_spawn_descriptor.rs
  - assets/digimon/agumon/anim_graph.ron
  - assets/digimon/agumon/clip.ron
lessons_learned:
  - Asymmetric skill authoring (Baby Flame recover overshoots the coarse clip label by two frames) means headless atlas-parity drives must assert against the union of authored node ranges, not the clip label.
  - Integration tests link only the bevyrogue library crate, not the binary — anything a windowed_only test must call has to live in the lib, not src/windowed/mod.rs.
  - K001 (no windowed-binary execution in auto-mode) makes the final visual surface a mandatory manual gate; plan for the user sign-off as the closure step rather than treating headless green as completion.
---

# M003: Make Agumon Render On-Screen

**Bound the existing Agumon atlas and rendered all five combat surfaces (idle, basic, skill, ultimate, VFX particles) on both actors with frame-accurate impact timing.**

## What Happened

M003 turned the textureless on-screen actors into fully rendered, frame-accurate Agumon sprites across three vertical slices. S01 established atlas binding with zero prior art: it bound `Handle<Image>` + `TextureAtlas` into `src/windowed/render.rs`, built the Bevy-free `AtlasGeometry`/`atlas_index(frame)` seam, proved identity frame→index mapping and clip↔atlas parity headless, and rode the already-bridged Sharp Claws path to prove release-on-impact-frame end to end. S02 made timing frame-accurate for the asymmetrically-authored Baby Flame and Baby Burner skills, adding the two-clock cue barrier with rendered-impact-frame release, the decoupled `AnimationClock` (12 fps default, catch-up capped at 4 ticks), caster-side gating, and scale fixes — with recorded user K001 confirmation of pacing and timing. S03 delivered the last missing surface: authored node-entry `SpawnParticle` commands now render as short-lived sprite-quad world particles through a shared per-tick budget, Baby Burner detonate synthesizes the same renderable seam while preserving the existing egui flash chip, and the `VfxSpawnDescriptor`/`resolve_locus` seam stays Bevy-free with no numeric gameplay payload.

Closure was gated on a round-0 `needs-attention` validation with three gaps, all since resolved: the milestone-level manual `cargo winx` sign-off is now recorded in `S03-UAT.md` (user confirmed all five surfaces on both actors, damage on impact/launch frame, clean particle despawn); the missing `S03-ASSESSMENT.md` was reconstructed with all 10 automated checks re-verified from disk plus the 4 visual K001 checks confirmed; and the R012/R016 requirement bookkeeping mismatch was reconciled in `REQUIREMENTS.md` as a supporting re-verification slice. Round-1 validation is `pass`.

## Success Criteria Results

All four success criteria PASS: (1) all five surfaces render on both actors — headless evidence + manual `cargo winx` sign-off 2026-05-25; (2) damage lands on the animation impact/launch frame, not on keypress — headless invariants + visual confirmation; (3) headless suite green (atlas binding, frame→index, parity, impact invariant) — `cargo test --test animation` 65/65, full `cargo test` exit 0; (4) no windowed-gated deps leak into headless paths — `cargo test`, `cargo build --features windowed`, `cargo test --features windowed` all exit 0, R016 upheld.

## Definition of Done Results

Not provided.

## Requirement Outcomes

No requirement status changed in M003 (focused rendering milestone). R012 and R016 — already validated in M002 — were re-verified (renderable descriptor extension and headless/windowed boundary hygiene); REQUIREMENTS.md records M003/S03 as a supporting re-verification slice. All prior M002-validated requirements remain validated and uninvalidated.

## Deviations

None.

## Follow-ups

["Baby Flame particle aesthetic polish (out of scope for S03): swirling charge-in-mouth feeding the flame, fast launch, flame-dissolve burst on impact — reuses the three existing Baby Flame assets with at most one optional ember sprite. Track as M004+ slice or a /gsd quick task.", "Roster extension (Gabumon/Dorumon/Tentomon/Renamon/Patamon rendering) deferred to M004+.", "Target-side hurt/flinch animation is unbuilt: clip.ron defines a hurt range but anim_graph.ron has no hurt node and no reaction-driven bridge."]
