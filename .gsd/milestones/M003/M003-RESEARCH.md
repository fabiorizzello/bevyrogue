# M003 Research — Make Agumon Render On-Screen

**Researched:** 2026-05-22
**Lane:** research
**For:** the roadmap planner (slice ordering + boundaries)

## TL;DR for the planner

M003 is **two distinct pieces of work**, not one — and the context's framing ("the FSM already advances frames and fires cue releases; M003 just reads `current_frame`") understates the second:

1. **Atlas binding (greenfield).** The on-screen `Sprite` is spawned with `Sprite { flip_x, ..default() }` (`src/windowed/render.rs:140`) — no `Handle<Image>`, no `TextureAtlas`. There is **zero atlas image-loading code anywhere**: the animation plugin loads only RON (graph/clip/stance), never the `_atlas.png`. And the `advance.frame` the player already computes is **never written to any Sprite**. This piece is purely additive binding code.

2. **Extending the windowed playback bridge from one surface to all five (real scope).** Today only **idle + sharp_claws (basic)** have a true windowed sprite-animation path. `advance_agumon_presentation` **auto-releases the cue barrier for any non-`sharp_claws` skill without playing the animation** (`render.rs:160-173`). Baby Flame (skill) and Baby Burner (ultimate) currently surface only as **egui UI effects** (`BabyBurnerFlashState`, damage numbers, flash) — the *sprite actor* does not animate them. To get "all five surfaces render as sprite animation on both actors," the `AgumonPlaybackMode` enum + `sync_agumon_mode` must be generalized beyond the single `SharpClaws` variant, and the auto-release short-circuit removed.

**The good news:** the assets and the kernel-side machinery are fully in place. The skill graph (`anim_graph.ron`) already defines all the nodes — `baby_flame_cast/impact/recover`, `sharp_claws_windup/strike/recover`, `baby_burner_charge/launch/recovery` — each with `ReleaseKernel` cues. Clip↔atlas parity is asserted and green. The two-clock cue barrier, `AnimGraphPlayer`, and registries all work. The work is presentation-layer wiring concentrated in `src/windowed/render.rs`.

## What should be proven first

**Prove the binding on idle, end-to-end, before touching the skill surfaces.** Idle is the lowest-risk surface (a pure 6-frame loop, frames 53–58, no cue barrier, no impact timing). Binding the atlas + driving `texture_atlas.index = advance.frame` on the looping idle stance proves the entire image-load → layout-build → frame→index → on-screen-pixel path with none of the impact-frame complexity. This is also the literal "primo reality check visivo" — it's the first time *anything* Digimon-shaped appears on screen. Everything else (basic/skill/ultimate) reuses this exact binding and adds only playback-bridge state.

Suggested risk order for the planner: **idle binding → basic (already has a working bridge, just needs the index written) → skill + ultimate (needs new bridge state) → mirrored dummy → VFX flash + impact-frame proof.** The dummy is nearly free once one actor renders (`spawn_unit_sprites` already loops all units and sets `flip_x` for the enemy team).

## Existing patterns to reuse

- **`advance.frame` IS the atlas index.** `AnimGraphPlayer::advance_result` returns a sprite-sheet frame index in `0..=92` directly (`player.rs:46-111`, `frame_index()`). Idle returns 53–58, attack 0–8, skill 59–75, heavy_attack 23–45. Binding is literally `sprite.texture_atlas.as_mut().unwrap().index = advance.frame as usize;` — no remapping, no per-range math. This is the central simplification; the clip ranges and atlas indices are the same coordinate space (proven by `tests/animation/clip_atlas_parity.rs`).
- **`spawn_unit_sprites` already iterates every unit and flips the enemy.** The mirrored-dummy requirement is satisfied by the existing loop (`render.rs:132-146`); it sets `flip_x = (team == Enemy)` already. The dummy renders for free once the atlas is bound — no second code path.
- **`AgumonSprite` already carries the `AnimGraphPlayer` + resolved graph + playback mode.** The component is the right home for atlas state too, but the cleaner Bevy-idiomatic move is to put `Sprite` (with bound `TextureAtlas`) on the same entity and update its `index` in `advance_agumon_presentation` where `advance.frame` is already in scope (`render.rs:186`).
- **Registry-driven asset paths.** `DEFAULT_ANIM_GRAPH_PATHS` / `DEFAULT_ANIM_CLIP_PATHS` / `DEFAULT_ANIM_STANCE_PATHS` (`plugin.rs:19`, `registry.rs:195`) are `Vec<String>` resources. The atlas PNG path should follow the same pattern — a registry-typed list (or a per-character preset, see F1 below) so M004+ roster work is one entry per Digimon, not a code change.
- **Failure-visibility precedent (R013).** The cue-barrier timeout + `AnimationGraphLookupDiagnostics` + `InstantFallback` pattern is the established "loud, diagnosable failure" shape. Atlas-load failure (missing PNG, grid/frame-count disagreement with the clip) should surface through the same kind of logged-with-path error rather than a silent textureless sprite — that silent-textureless mode is exactly the M002 failure this milestone closes.

## Boundary contracts that matter

- **R005 / R002 dep gating.** `bevy/2d` (which pulls `bevy_sprite` + the PNG image decoder) is enabled **only** under the `windowed` feature (`Cargo.toml:41`). Default features are headless: `std`, `bevy_asset`, `bevy_log`, `bevy_state`, `file_watcher` — **no `bevy_sprite`, no image decoders.** Therefore *all* `TextureAtlas`/`Sprite`/`TextureAtlasLayout` code must live behind `#[cfg(feature = "windowed")]` in `src/windowed/`. It cannot leak into the lib's default build.
- **The testability tension (the key constraint for the planner).** Integration tests under `tests/` link only against the `bevyrogue` **lib**, not the windowed binary, and the lib's default build has no sprite types. So the headless-provable contract — "frame → atlas index mapping is correct," "clip↔atlas range parity" — must be expressed in **lib-reachable, sprite-type-free terms** (plain `u32` index arithmetic, RON parsing), exactly as `tests/animation/clip_atlas_parity.rs` already does (it parses the JSON/RON as plain structs, never touching `TextureAtlasLayout`). The actual `Sprite`-carries-a-bound-`TextureAtlas` assertion can only be a `windowed`-gated test or a structural check; the *mapping logic* it depends on should be a pure lib function the windowed code calls, so the headless test exercises the same code the screen does.
- **Two-clock impact contract (D025 / R006, CAP-159d33b5).** Damage must land on the rendered impact frame via `ReleaseKernelCue`, never on keypress. The existing release path (`should_release_kernel` → `barrier.request_release` → `player.fire_kernel_cue`, `render.rs:213-258`) already does this correctly for sharp_claws. Extending to skill/ultimate means the same `local_frame == cue.at && ReleaseKernel` check must run for the baby_flame/baby_burner nodes — the cues are already authored (`at: 1` on the impact nodes). The invariant to guard: the index written to the sprite on the release tick is the impact-frame index.
- **VFX through the Cue/reactive bus only (CAP-7c065a44).** The flash renders via `SpawnParticle`/`ParticleId`/`VfxLocus`/`VfxMotion` (`tests/animation/vfx_handle_seam.rs`) — no physics, no colliders. M003 renders through this existing seam; it does not invent a new VFX path.
- **§9 UI never mutates `CombatState` (D008, structurally enforced).** Anything M003 adds on the UI side stays event-driven.

## Constraints the codebase imposes

- **`bevy = "=0.18.1"`, `default-features = false`.** Pinned exact version. Relevant 0.18 API for the planner (verify exact signature at slice-execution time): build a layout with `TextureAtlasLayout::from_grid(UVec2::splat(512), 10, 10, None, None)`, insert into `Assets<TextureAtlasLayout>`, then `Sprite::from_atlas_image(image_handle, TextureAtlas { layout, index: 0 })`. The 10×10 grid / 512px / 93-frame metadata is in both `clip.ron` and `agumon_atlas.json` and they agree.
- **Asset path correction for the planner.** The CONTEXT says atlases live at `assets/digimon/agumon/*_atlas.{png,json}`. They actually live **flat**: `assets/digimon/agumon_atlas.png` and `assets/digimon/agumon_atlas.json` (the RON triplet is in the `agumon/` subdir; the atlas pair is one level up). All six roster atlases already exist flat (`agumon`, `gabumon`, `dorumon`, `tentomon`, `renamon`, `patamon`) — but only Agumon has the clip/graph/stance triplet, so only Agumon is animatable in M003 (roster = M004+).
- **Atlas JSON schema differs from clip RON.** `agumon_atlas.json` uses `meta.frame_size{w,h}` + `columns`/`rows`/`total_frames` and `animations.<name>.{start_index,end_index,count}`; `clip.ron` uses `meta` + `ranges.<name>.{start,end}`. The parity test (`clip_atlas_parity.rs`) already reconciles them — reuse its `Atlas`/`AtlasRange` deserialization shape rather than re-deriving.
- **Skill-graph entry oddity.** `anim_graph.ron` `entry: "baby_flame_cast"`, but the windowed bridge rewinds to the *windup/start* node per skill (`sharp_claws_start_node`, `render.rs:380`). The generalized bridge needs an analogous start-node mapping for baby_flame and baby_burner (their charge/cast nodes), not a blind use of the graph entry.

## Known failure modes that should shape slice ordering

- **Auto-release short-circuit masks missing playback.** Until the bridge is generalized, casting Baby Flame/Burner in `cargo winx` *advances the kernel correctly* (damage lands, UI updates) but the **sprite does not animate the skill** — it stays idle/snaps. A planner who slices "bind atlas" and "extend bridge" into separate slices should expect the intermediate state to render idle correctly while skill casts look inert on the actor. Order the bridge extension immediately after binding so the gap window is short.
- **Impact-frame sync across three timelines (CONTEXT risk, confirmed real).** Each of basic/skill/ultimate has its own windup→strike→recover node set with its own `ReleaseKernel` cue offset. Getting one `cue.at` ↔ `local_frame` alignment wrong shows damage landing off-beat. This is per-surface and only fully verifiable by eyeball (K001) — but the headless invariant (release fires at the authored cue frame) is provable in the lib. Slice the three surfaces so each carries its own impact-frame assertion.
- **K001 gates the real acceptance.** Auto-mode must never launch the windowed binary. Every slice's "render correctly" criterion bottoms out in a manual `cargo winx` sign-off; headless tests prove only the wiring. Plan slices so each ends with a *headless-provable* contract plus a *deferred manual-eyeball* item — don't let a slice's completion depend on auto-mode seeing pixels.
- **VRAM (advisory, not M003-blocking).** Each 512px atlas ≈ 100 MiB VRAM decoded (5120×5120×4); two Agumon actors share one atlas image (~100 MiB resident, not 200 — same `Handle<Image>`). Non-issue for the 2-actor demo. Flag for M004+ roster scale (KTX2/Basis), not now.

## Intersecting M002/S06 architectural follow-ups

The S06 review (`S06-ARCHITECTURAL-REVIEW.md`, verdict pass-with-followups) routed three low-severity follow-ups explicitly to **M003/S01** that directly touch the M003 surface — the planner should decide whether to fold them in or defer:

- **F1 (low) — lift Agumon-specific constants out of `src/windowed/mod.rs`.** `AGUMON_STANCE_GRAPH_ID`, `AGUMON_SKILL_GRAPH_ID`, `SHARP_CLAWS_*` (`mod.rs:39-43`). M003 will *add* an atlas-path constant in the same spot; doing F1 (a small `WindowedCharacterPreset` table) at the same time prevents the per-character copy-paste growth M004+ would otherwise inherit. **Recommend folding into the atlas-binding slice** since the atlas path wants the same home.
- **F3 (low) — split `timeline_exec.rs` (557 LOC).** Pure housekeeping, no behaviour change; only matters if M003 touches that file (it likely won't — M003 is presentation-layer). **Recommend deferring.**
- **F7 (info) — group `UiPlugin` systems into named `SystemSet`s.** Only relevant if M003 adds presentation systems to the chain (it may, for VFX). **Optional, fold in if the VFX slice adds systems.**
- **F2 (medium) — `unsafe` raw-pointer registry dance in `timeline_exec.rs`.** The one above-low finding, but it's kernel-pipeline, **out of M003's presentation scope**. Leave it where the review put it (M002 close-of-cycle or a dedicated housekeeping slice).

## Requirements posture

`.gsd/REQUIREMENTS.md` shows **0 Active requirements** — all 11 (R004–R016) are `validated` against M002. M003 has **no requirement currently scoped to it**. The CONTEXT's "Relevant Requirements" section is an explicit to-be-linked placeholder.

**Candidate requirements (advisory — surface for the planner/user, do not auto-bind):**

- **CANDIDATE — On-screen sprite render binding (core-capability).** "The on-screen `Sprite` carries a bound `Handle<Image>` + `TextureAtlas` built from the 512px grid atlas; the `AnimGraphPlayer` current frame drives `TextureAtlas.index`." This is the milestone's central deliverable and is currently unrequirement-backed. *Table stakes for M003.*
- **CANDIDATE — Atlas-load failure visibility (failure-visibility).** "Atlas image/grid load failure or clip↔atlas frame-count disagreement surfaces a logged error with the offending path, not a silent textureless sprite." Extends R013's failure-visibility posture to the new asset surface; closes the exact M002 silent-textureless failure mode. *Recommended — it's the CONTEXT's stated error-handling strategy.*
- **CANDIDATE — Rendered impact-frame damage invariant (quality-attribute / continuity of R006).** R006 already covers "damage on impact frame via ReleaseKernelCue" but was validated only for sharp_claws. M003 extends the same contract to skill + ultimate on the *rendered* frame. Likely an **update to R006's validation scope** rather than a new requirement.

**Likely NOT wanted in M003 (out of scope, confirm only if raised):** roster atlas binding for the other five Digimon (assets exist but no clip/graph — M004+); GPU texture compression; any new VFX motion model. The CONTEXT is explicit on all three.

## Skills Discovered

- **`bevy-ecs-expert`** — already installed (system skills). Relevant to the ECS/Sprite/system-ordering work; no install needed.
- No additional skill install warranted. The work is Bevy-0.18-specific sprite/atlas binding within an established codebase; `bevy-ecs-expert` + the in-repo `rust-*` skills cover it. (Bevy itself is already a project dependency, so library docs are reachable via the codebase rather than a new skill.)

## Suggested slice boundaries (input to the planner, not a mandate)

1. **Atlas load + idle binding** — register the atlas PNG path (reuse the `DEFAULT_*_PATHS` pattern; optionally fold in F1), build `TextureAtlasLayout::from_grid`, bind `Sprite::from_atlas_image` in `spawn_unit_sprites`, write `index = advance.frame` for the idle loop in `advance_agumon_presentation`. Headless: pure frame→index mapping fn + clip↔atlas parity (extend existing). Manual: idle loops on screen, both actors (dummy is free). *Proves the whole binding path.*
2. **Basic (Sharp Claws) rendered on impact** — the bridge already exists; this slice mainly confirms the bound index advances through windup→strike→recover and damage lands on the strike cue frame. Headless: release fires at authored `cue.at`. Manual: claws connect on impact.
3. **Skill + Ultimate playback bridge** — generalize `AgumonPlaybackMode`/`sync_agumon_mode` beyond the single `SharpClaws` variant; remove the non-sharp_claws auto-release short-circuit; add start-node mapping for baby_flame/baby_burner. Headless: per-surface impact-frame release invariant. Manual: Baby Flame + Baby Burner animate on the actor with damage on impact. *Highest-risk slice — three impact timings.*
4. **VFX flash through the Cue/reactive bus** — render `SpawnParticle` as visible particles on both actors. Headless: seam round-trip (exists). Manual: flash appears where expected.

Slices 1→2 are tightly coupled (2 is mostly verification of 1's path on a non-loop surface) and could merge; 3 is the real new code; 4 is additive. The mirrored dummy is not its own slice — it falls out of `spawn_unit_sprites` in slice 1.

## Open questions for planning

- Whether to fold F1 (lift Agumon constants) into slice 1 — **recommend yes**, the atlas path wants the same home and it's the cheapest time to do it.
- Whether a `capture-windowed-smoke.sh` artifact is a required deliverable or an optional aid alongside the manual sign-off — CONTEXT leans optional; the manual eyeball is the real gate (K001).
- Exact requirement IDs to bind vs. leaving M003 requirement-light and validating against the milestone-level acceptance bar — surface the three candidate requirements above to the user at planning.
