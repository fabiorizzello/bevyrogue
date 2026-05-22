# M003: Make Agumon Render On-Screen

**Gathered:** 2026-05-22
**Status:** Ready for planning

## Project Description

M002 wired the combat presentation stack — the AnimGraph FSM ticks, the two-clock cue barrier suspends the kernel until the sprite reaches its impact frame, and the §9 UI panels (phase strip, HP, damage numbers) draw from `CombatEvent`s. But the on-screen actors are **textureless**: `src/windowed/render.rs` spawns `Sprite { ..default() }` with no `Handle<Image>` and no `TextureAtlas`, there is **zero atlas code anywhere in `src/`**, and the VFX "flash" is an opaque `ParticleId` seam, not rendered pixels. You see the GUI and nothing of the Digimon.

M003 closes that gap: bind the existing 512px grid atlas to the on-screen `Sprite`, drive `Sprite.texture_atlas.index` from the `AnimGraphPlayer`'s current frame, and make all five animation surfaces — idle stance, basic, skill, ultimate, and the VFX flash — render as real pixels on **both** the Agumon ally and a mirrored Agumon dummy. This is the portfolio's "primo reality check visivo."

This is **not** the GSD-DB title "Roster extension" — that label is stale by one milestone. M002 over-delivered the plumbing the portfolio reserved for its M003, so the real M003 is the pixel-binding the portfolio promised, and roster extension (Gabumon/Dorumon/Tentomon/…) slides to M004+.

## Why This Milestone

The combat is currently invisible. Every kernel mechanic, FSM transition, cue-barrier handshake, and UI panel exists and is tested headless — but a human running `cargo winx` sees a UI frame around empty space where the Digimon should be. Until the actors render, no on-screen feature can be validated, demoed, or iterated visually. M003 is the milestone that turns the proven-but-invisible stack into something you can actually watch fight. It must come before any roster work, because every future Digimon inherits this exact binding path.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Run `cargo winx` and **see** an Agumon sprite idling (looping idle stance) on the ally side and a mirrored Agumon on the enemy-dummy side.
- Watch Agumon play its basic (Sharp Claws → `attack` range), skill (Baby Flame → `skill` range), and ultimate (Baby Burner → `heavy_attack` range) as actual animated frames, with the VFX flash rendering as visible particles — on both actors.
- See damage land **on the impact frame** of the animation (when the strike visually connects), not on keypress — the two-clock cue barrier driving the visible frame.

### Entry point / environment

- Entry point: `cargo winx` (== `cargo run --features windowed`)
- Environment: local dev, windowed (egui + winit + wgpu), requires the bevy `.so` from `target/` (dev-only)
- Live dependencies involved: none external; reads on-disk atlas PNG/JSON + RON clip/graph/stance assets via `asset_server`

## Completion Class

- **Contract complete means:** headless tests prove the structural wiring — the on-screen `Sprite` carries a bound `Handle<Image>` + `TextureAtlas`; `AnimGraphPlayer` current frame maps correctly onto `TextureAtlas.index`; clip↔atlas frame-range parity holds (idle/attack/skill/heavy_attack ranges align between `clip.ron` and the atlas JSON); and the impact-frame damage invariant holds (damage fires when the player reaches the impact frame via the cue barrier, per CAP-159d33b5).
- **Integration complete means:** the binding works against the *real* assets through the windowed render path — the atlas loads, the layout builds from the 512px grid, and the player drives the index across all five surfaces on both actors, with the cue barrier releasing on the real visible frame.
- **Operational complete means:** under a real `cargo winx` session the user visually confirms all five surfaces render and animate correctly on both actors with damage landing on impact — this is the user's manual sign-off (K001: auto-mode must never launch the windowed binary).

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- A `cargo winx` run shows Agumon (ally) and a mirrored Agumon (dummy) both rendering: idle stance loops, and basic/skill/ultimate each play their correct atlas frame ranges as visible animation, with the VFX flash rendering as particles. **(User-validated visually.)**
- Damage lands on the animation's impact frame — the strike visually connects before HP changes — driven by the existing two-clock cue barrier, not on keypress. **(Headless invariant + visual confirmation.)**
- **Cannot be simulated:** that the pixels are actually correct on screen — sprite is visible, frames advance smoothly, the right animation plays for the right action, VFX appears where expected. Headless tests prove the wiring is structurally sound; only the user's eyes prove it looks right.

## Architectural Decisions

### Atlas pipeline reused as-is — no rework

**Decision:** Load the existing 512px uniform-grid atlases through Bevy's `TextureAtlasLayout::from_grid` with no per-character cropping, no resize, and no format change. Map the `AnimGraphPlayer`'s current frame directly onto `TextureAtlas.index`.

**Rationale:** The on-disk atlas format already maps 1:1 onto what Bevy wants — `frame_size (512×512)` + `columns`/`rows` → `from_grid`, and the named ranges (`idle`, `attack`, `skill`, `heavy_attack`, …) align 1:1 with the clip ranges the player already advances. The missing piece is purely binding code, not assets. (Extends the prior "512px uniform frames, no per-character cropping" decision.)

**Alternatives Considered:**
- Shrink frames to 256px to quarter VRAM — rejected: caps visual quality before we've seen it on screen; wrong tradeoff to make blind.
- Per-character cropped atlases — rejected: complicates load and breaks uniform scale across animations.

### Dummy = mirrored Agumon

**Decision:** The enemy dummy renders as a mirrored Agumon (`flip_x`), reusing Agumon's complete RON triplet and atlas — the same scenario M002 used.

**Rationale:** Only Agumon has the full `clip` + `anim_graph` + `stance` triplet. The enemy data Digimon (goblimon/ogremon/devimon) have no atlas and no clip/graph; animating a real enemy would mean authoring those assets, which is roster work (M004+). Mirroring Agumon proves all five surfaces on both on-screen actors without pulling roster authoring into M003.

**Alternatives Considered:**
- Render a different enemy Digimon — rejected: pulls asset authoring (atlas + clip + graph + stance) into M003, which is M004+ scope.

### VFX via Cue/reactive bus, never physics

**Decision:** VFX render through the existing Cue + reactive bus (`CueExt` in the `CompiledTimeline`, the `SpawnParticle`/`ParticleId`/`VfxLocus`/`VfxMotion` seam). No physics, no colliders for VFX. The §9 UI shows action-queue Intents in sync with the animation.

**Rationale:** Per CAP-7c065a44 — colliders/physics break the deterministic turn-based kernel and add pointless overhead. The VFX seam is already the established path; M003 renders through it rather than inventing a new one.

**Alternatives Considered:**
- Physics-driven particle motion — rejected: breaks determinism (R004), unnecessary in a turn-based deterministic kernel.

## Error Handling Strategy

Asset-binding failures must be loud and diagnosable, not silent-textureless (the exact failure mode M002 left behind). If the atlas PNG or JSON fails to load, or the grid dimensions / frame count disagree with the clip ranges, the windowed boot should surface a clear error (logged with the offending path and the mismatch) rather than spawning an empty `Sprite`. Clip↔atlas parity is asserted in headless tests so a mismatch fails CI before it ever reaches the screen. The cue-barrier impact-frame contract already has an invariant guard; M003 extends it to the rendered frame index. No new external failure surfaces are introduced.

## Risks and Unknowns

- **Impact-frame sync correctness across three skill timelines** — the basic/skill/ultimate windup→strike→recover bindings each need their impact frame to align with the cue-barrier release; getting one wrong shows damage landing off-beat. Why it matters: it's the core M003 mechanic (CAP-159d33b5).
- **Atlas VRAM cost** — each 512px atlas decodes to ~100 MiB in VRAM (5120×5120×4 RGBA8); two actors ≈ 200 MiB resident. A non-issue for the 2-actor demo on any modern GPU, but flagged as a known cost. Why it matters: at full-roster scale (6+ animated actors → 1+ GB) this needs GPU texture compression (KTX2/Basis Universal), not frame shrinking — revisit at M004+ or on a measured frame-time regression, whichever bites first.
- **On-screen verification is gated behind manual eyeball, not CI** (K001) — headless tests prove wiring but cannot prove the pixels look right. Why it matters: the acceptance bar depends on the user actually running `cargo winx` and confirming.

## Existing Codebase / Prior Art

- `src/windowed/render.rs` — spawns the textureless `Sprite { ..default() }` today; the `AgumonSprite` carries the `AnimGraphPlayer`. This is where the atlas binding + frame→index mapping lands. The only hardcoded Digimon-id reference in the lib/binary split lives here.
- `src/animation/` (`anim_graph.rs`, `clip.rs`, `player.rs`, `registry.rs`) — the FSM + clip + player that already advances frames and fires cue releases; M003 reads `current_frame` from the player.
- `assets/digimon/agumon/{clip,anim_graph,stance}.ron` — Agumon's complete RON triplet (the only complete one).
- `assets/digimon/agumon/*_atlas.{png,json}` — the 512px uniform-grid atlas (10×10 grid, 93 frames) with named ranges; format maps 1:1 onto `TextureAtlasLayout::from_grid`.
- Two-clock cue barrier (`TimelineClock`, `Clock::Windowed`) — pauses the kernel until the sprite hits its impact frame; M003 binds this to the visible rendered frame.
- VFX seam — `SpawnParticle`/`ParticleId`/`VfxLocus`/`VfxMotion` (`tests/animation/vfx_handle_seam.rs`).
- `scripts/capture-windowed-smoke.sh` — windowed smoke capture harness from M002's S06.

## Relevant Requirements

- (To be linked at planning time — M003 advances the on-screen presentation/visual-validation capability that M002's plumbing set up but left unproven.)

## Scope

### In Scope

- Atlas loading + `TextureAtlasLayout::from_grid` construction from the existing 512px Agumon atlas.
- Binding `Handle<Image>` + `TextureAtlas` onto the on-screen `Sprite` in `src/windowed/render.rs`.
- Driving `Sprite.texture_atlas.index` from the `AnimGraphPlayer` current frame, for all five surfaces: idle stance loop, basic (Sharp Claws → `attack`), skill (Baby Flame → `skill`), ultimate (Baby Burner → `heavy_attack`).
- VFX flash rendering through the existing Cue/reactive bus.
- Rendering on **both** the Agumon ally and the mirrored Agumon dummy.
- Damage landing on the impact frame via the two-clock cue barrier, bound to the rendered frame.
- Headless tests: sprite has bound image+atlas, player frame → atlas index, clip↔atlas range parity, impact-frame damage invariant.

### Out of Scope / Non-Goals

- Roster extension — Gabumon, Dorumon, Tentomon, Renamon, Patamon (M004+).
- Authoring enemy Digimon assets (goblimon/ogremon/devimon atlas/clip/graph/stance).
- GPU texture compression (KTX2/Basis Universal) — deferred to roster-scale VRAM pressure.
- Atlas rework, resize, or per-character cropping.
- Any new VFX motion model or physics.

## Technical Constraints

- **R002 / R005:** every system must run headless without `windowed`; egui/winit/wgpu deps gated only behind `#[cfg(feature = "windowed")]`. Atlas binding lives in the windowed layer; the testable contract (frame→index, parity) lives in the lib so headless tests can reach it.
- **R004:** no wall-clock, no unseeded RNG in the resolution stream; presentation timing stays in `Clock::Windowed`.
- **K001:** auto-mode must never launch the windowed binary; on-screen verification is the user's manual responsibility.
- **Gotcha:** integration tests under `tests/` link only against the `bevyrogue` lib, not the binary — anything a windowed_only test must call has to live in the lib.
- VFX through the Cue/reactive bus only; no colliders/physics (CAP-7c065a44).

## Integration Points

- `asset_server` — loads the atlas PNG + JSON and the RON clip/graph/stance (already used for RON hot-reload).
- `AnimGraphPlayer` / `SkillGraphRegistry` — source of the current frame index that drives `TextureAtlas.index`.
- Two-clock cue barrier (`TimelineClock` / `Clock::Windowed`) — gates damage on the rendered impact frame.
- §9 UI panels — show action-queue Intents in sync with the animation (event-driven, never mutating `CombatState`).
- VFX seam (`SpawnParticle` / `ParticleId` / `VfxLocus` / `VfxMotion`) — the flash render path.

## Testing Requirements

Headless integration tests (under the appropriate `tests/<scope>/`, per R003) must prove the structural wiring without launching the window:

- The on-screen `Sprite` is built with a bound `Handle<Image>` and `TextureAtlas` (not `..default()`).
- `AnimGraphPlayer` current frame maps onto `TextureAtlas.index` correctly across the idle/attack/skill/heavy_attack ranges.
- Clip↔atlas frame-range parity: the named ranges in `clip.ron` align with the atlas JSON grid layout.
- Impact-frame damage invariant: damage fires when the player reaches the impact frame via the cue barrier, on the rendered frame.

On-screen visual correctness (all five surfaces animate on both actors, VFX visible, damage on impact) is verified by the **user's manual `cargo winx` run** — K001 forbids auto-mode from launching the windowed binary. `scripts/capture-windowed-smoke.sh` may produce a capture artifact to accompany the manual sign-off.

## Acceptance Criteria

> Per-slice criteria gathered at planning time. Milestone-level bar:

- `cargo winx`: Agumon ally + mirrored Agumon dummy both render; idle stance loops; basic/skill/ultimate play correct atlas frame ranges as visible animation; VFX flash renders as particles. (User-validated.)
- Damage lands on the animation impact frame, not on keypress. (Headless invariant + visual.)
- Headless suite green: bound image+atlas, frame→index mapping, clip↔atlas parity, impact-frame invariant.
- No `windowed`-gated deps leak into headless paths; full headless suite still passes.

## Open Questions

- Exact requirement IDs to link — resolve at planning by reading `REQUIREMENTS.md`.
- Whether a capture artifact from `capture-windowed-smoke.sh` should be a required planning deliverable alongside the manual sign-off, or optional — current thinking: optional aid, the manual eyeball is the real gate.
