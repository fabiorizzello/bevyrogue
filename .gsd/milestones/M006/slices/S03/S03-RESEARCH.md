# S03 Research: Generalize windowed sprite + wire cue dispatch

## Summary

S03 is the structural core of M006: rename/generalize the Agumon-named windowed
presentation component into a data-carrying `DigimonSprite` (graph ids become
fields, not hardcoded const lookups), and route the transient flash/shake/
camera-shake feedback through the S02 `CueRegistry` + parametric math instead of
the `hit_feedback.rs` consts. **S03 generalizes the component and the cue path;
it does NOT yet remove the `AGUMON_*` consts from the engine — that extraction is
S04.** The risk is blast radius, not novelty: every windowed query and
source-contract test referencing `AgumonSprite`/`AgumonPlaybackMode` must move in
lockstep, and `src/windowed/` is binary-crate code unreachable from `tests/`
(MEM030), so the only automated guards are the windowed source-contract suite +
`cargo build --features windowed`. Camera-shake is genuinely new and needs a rest
anchor + arm/decay state mirroring the existing flash/shake pattern.

## Active Requirements This Slice Owns/Supports

- **Extension-first presentation** (local constraint, no R-id): S03 makes the
  component generic so S04/S05 add Digimon without editing it. The component must
  carry stance/skill graph ids as data.
- **R002 (headless-first) / R005 (dep gating):** all new pure math already lives
  in the lib (`src/ui/cues.rs`, S02). S03 only applies it in the binary. The
  `dependency_gating` test must stay green — no enoki/`bevy_color` leak. Note the
  S02 seam returns `SrgbTriple`, mapped to `Color::srgb(r,g,b)` at the binary call
  site (MEM113/MEM114).
- **R004 (determinism):** cue math is a pure function of tick counters; no
  wall-clock, no RNG. Already satisfied by `shake_offset_parametric` /
  `flash_tint_parametric`.

## Implementation Landscape

### Primary file: `src/windowed/render.rs` (~2050 lines)

**Component generalization (the rename + data-carry):**
- `AgumonSprite` (L38-46): fields `unit_id, player, graph, mode,
  last_release_frame, last_missing_skill_graph_cue`. Becomes `DigimonSprite`.
  To satisfy extension-first it must additionally carry the **graph ids as data**
  — e.g. `stance_graph_id: AnimGraphId`, `skill_graph_id: AnimGraphId` — so the
  four `resolve_snapshot` call sites stop reading `AGUMON_STANCE_GRAPH_ID`/
  `AGUMON_SKILL_GRAPH_ID` consts and read `sprite.stance_graph_id` instead.
- `AgumonPlaybackMode` (L130-142): `Idle | Skill{skill_id, awaiting_cue_id,
  start_node}`. Rename to `DigimonPlaybackMode` (variants are already generic
  strings — no structural change, just the name). MEM058 documents the
  skill-parameterized seam; `classify_same_skill_sync` stays pure.
- `impl AgumonSprite` (L151-…): `idle_for`, `start_skill`, `return_to_idle`,
  `seed_stance_reaction`. `idle_for`/spawn must now thread the graph ids in.
- Spawn site `spawn_unit_sprites` (L702-748): builds `AgumonSprite::idle_for`.
  In S03 it still hardcodes `AGUMON_STANCE_GRAPH_ID` (const removal is S04) but
  passes it into the component as a field. Note `AgumonAtlas`/`SPRITE_DISPLAY_SCALE`
  are also Agumon-named but out of S03 scope (S04 extraction).

**`resolve_snapshot` sites that must switch from const → `sprite.<field>`:**
- L711 (`spawn_unit_sprites`), L778 (`advance_agumon_presentation`),
  L1176 (`drive_hurt_reactions`), L1252 (`drive_death_reactions`),
  L1712/1747 (`sync_agumon_mode` skill + stance fallback). The hurt/death drivers
  query sprites — they can read the id off the queried `DigimonSprite`.

**Cue dispatch wiring (the flash/shake/camera-shake path):**
- Current flash/shake is **armed off `CombatEvent::OnHitTaken`** by
  `observe_hit_feedback` (lib `hit_feedback.rs` L105) into `HitFlashState`/
  `HitShakeState` resources, decayed once per frame in
  `advance_agumon_presentation` (L807-821), and applied at L887-898 via
  `flash_tint(remaining, FLASH_TICKS)` and `shake_offset(remaining, SHAKE_TICKS)`.
  MEM094 documents the discipline: shake is an **absolute offset from a captured
  `SpriteRest{xy}`**, never accumulated; `flash_tint` is the sole colour writer
  and is skipped while `DeathExiting`/`FadeOut` owns the colour.
- S03 must make these reads source their params (peak/ticks, amp/freq/ticks) from
  the `CueRegistry` (S02 `CueDef::Flash`/`SpriteShake`/`CameraShake`) and call the
  `*_parametric` fns, mapping `SrgbTriple` → `Color::srgb`. The legacy consts
  (`FLASH_TICKS`, `SHAKE_TICKS`, peak `(1.0,0.45,0.45)`, amp/freq `4.0/1.7/2.3`)
  become the registered cue def values — `cues.rs` already proves the parametric
  forms are bit-for-bit identical to the legacy fns at those params, so this is a
  behaviour-preserving swap.

**Camera-shake (new):**
- `setup_camera` (L431-439) spawns the single `Camera2d` (+ `Hdr`, `Bloom`,
  `Tonemapping`, `DebandDither` — MEM082). S03 adds a `CameraRest` component
  capturing the camera's rest translation at spawn, and a system that, on impact,
  writes `camera_transform.translation = rest + shake_offset_parametric(...)`
  (absolute offset from rest — the same anti-drift pattern as `SpriteRest`,
  flagged in the milestone risks). Needs a `CameraShakeState` arm/decay resource
  mirroring `HitShakeState`, armed on the same `OnHitTaken` signal, decayed on the
  `PendingAnimationTicks` clock (single decay source of truth).

### S02 seam being consumed: `src/ui/cues.rs`
- `CueRegistry` (Resource, `register`/`get`), `CueDef::{Flash, SpriteShake,
  CameraShake, ParticleBurst{effect_id}}`, `flash_tint_parametric`,
  `shake_offset_parametric`, `SrgbTriple`. All pure, headless-tested (10 tests).
  `register` fail-fast panics on conflicting def (D047). Camera-shake reuses
  `shake_offset_parametric` verbatim.

### S01 seam (already done): single enoki path
- `spawn_effect_by_id` is the sole enoki spawn path; `AgumonEnokiVfx` map keyed by
  all six effect ids; lifecycle layer (`ChargeEmberEnokiMarker`, `ProjectileFlight`,
  `advance_enoki_projectiles`). `ParticleBurst.effect_id` (S02) maps onto these
  enoki handle ids. S03 need not touch the enoki spawn path; whether `ParticleBurst`
  cue dispatch folds into `spawn_effect_by_id` or stays separate is a design call
  (recommend: leave the enoki path as-is in S03, fold under the registry in S04).

### Registration: `src/windowed/mod.rs`
- Resource init block at L90-97 / plugin build. S03 must `.init_resource::<CueRegistry>()`
  and register the Agumon cue defs (hit_flash, hit_shake, camera_impact) at
  startup. In S03 this registration is still Agumon-specific and lives in the
  engine; S04 moves it into `src/windowed/digimon/agumon/register(app)`.
- Existing `AGUMON_*` consts live here (L38-56): `AGUMON_STANCE_GRAPH_ID`,
  `AGUMON_SKILL_GRAPH_ID`, skill ids, node ids. **Left in place for S03**; removed
  in S04.

## Natural Seams (work units for the planner)

1. **Rename + data-carry component** (`AgumonSprite`→`DigimonSprite`,
   `AgumonPlaybackMode`→`DigimonPlaybackMode`, add graph-id fields, switch the 6
   `resolve_snapshot` sites to read fields). Highest blast radius — do first,
   verify build green, then proceed. Pure mechanical + threading.
2. **Register CueRegistry + Agumon cue defs** in `mod.rs` (init resource +
   register flash/shake/camera defs with the legacy const values).
3. **Re-point flash/shake to the registry + parametric math** in
   `advance_agumon_presentation` (look up cue def, call `*_parametric`, map
   `SrgbTriple`→`Color::srgb`). Behaviour-preserving.
4. **Camera-shake cue** (`CameraRest` capture in `setup_camera`, `CameraShakeState`
   arm/decay, apply system writing `Camera2d` transform absolute-from-rest).
5. **Update windowed source-contract tests** that pin the old type names.

## First Proof / Highest Risk

The component rename (seam 1) is both the highest-risk and the biggest unblocker:
it touches every query and the source-contract tests. Land it first and confirm
`cargo build --features windowed` + `cargo test --features windowed --test
windowed_only` stay green before wiring cues. Camera-shake (seam 4) is the only
genuinely new behaviour and carries the documented drift risk — verify it uses an
absolute offset from a captured rest, never accumulation.

## Verification

- `cargo build --features windowed` → exit 0, zero warnings.
- `cargo test --features windowed --test windowed_only` → all green (54 currently;
  count rises with new/updated contract tests). Source-contract tests pinning
  `AgumonSprite`/`AgumonPlaybackMode` must be updated to the new names, and a new
  contract should pin that the component carries graph ids as data + that a
  camera-shake cue/`Camera2d`-writing system exists.
- `cargo test --test dependency_gating` → 2 passed (no enoki/`bevy_color` leak
  into headless).
- `cargo test` (full headless) + `cargo test --no-default-features --features dev
  --test ui` → green (S02 cue tests unaffected).
- Structural: `rg "AgumonSprite|AgumonPlaybackMode" src/windowed/render.rs` → no
  matches after rename; `rg "flash_tint\\(|shake_offset\\(" src/windowed` → the
  legacy lib fns no longer called from the binary (replaced by `*_parametric` via
  registry). Note: `AGUMON_*` consts STILL present after S03 (removed in S04).
- **K001 manual:** `cargo winx` — hits still flash/shake the struck sprite,
  camera-shake now fires on impact, stance/skill/hurt/death playback unchanged.
  Auto-mode cannot run the windowed binary.

## Constraints & Gotchas

- **MEM030:** `tests/` link only the lib crate, not the binary; `src/windowed/`
  presentation is NOT integration-testable. All S03 proof is windowed
  source-contract tests (token assertions on `render.rs`) + build green + K001.
- **MEM094:** shake = absolute offset from captured rest, never accumulated; at
  remaining 0 hard-set translation back to rest; `flash_tint` is the sole colour
  writer and must be skipped under `DeathExiting`/`FadeOut`. Apply the **same
  discipline to camera-shake** (capture `CameraRest`, absolute offset) — this is
  the documented drift risk.
- **MEM113/MEM114:** `flash_tint_parametric` returns `SrgbTriple`, not
  `bevy::Color` (bevy_color is render-stack-only, absent headless). Map verbatim
  to `Color::srgb(r,g,b)` at the binary call site.
- **Single decay source of truth:** flash/shake/camera-shake windows all decay
  once per frame by `PendingAnimationTicks` in `advance_agumon_presentation`
  (MEM094). Camera-shake state must join that single decay site, not add a second.
- **D047:** `CueRegistry::register` panics on conflicting def — registration must
  be collision-free at startup.
- **Scope boundary:** S03 generalizes the component + cue path but leaves
  `AGUMON_*` consts, `AgumonAtlas`, and Agumon-specific cue registration **in the
  engine**. The module extraction (`src/windowed/digimon/agumon/`) and const
  removal is S04 — do not pull it forward.

## Key Design Tension for the Planner to Resolve

**How do cue ids get triggered?** Two models:
- **(a) Minimal / recommended for S03:** keep the existing `OnHitTaken`-armed
  flash/shake/camera windows, but source their *params* from fixed registered cue
  ids (e.g. `"hit_flash"`, `"hit_shake"`, `"camera_impact"`) looked up in
  `CueRegistry`. This satisfies "driven by cue dispatch reading CueRegistry
  instead of hit_feedback consts" with the lowest blast radius and preserves the
  proven arming/decay machinery.
- **(b) Full GAS-style generic dispatch:** animgraph `on_enter`/`CombatEvent`
  carry cue id strings dispatched generically through the registry. Higher risk,
  larger surface; the milestone's GAS framing points here eventually but S03's
  "After this" only requires the flash/shake/camera read to be registry-driven.

Recommend (a) for S03 — it is behaviour-preserving and the parametric-equivalence
tests already cover correctness. ParticleBurst dispatch can stay on the S01 enoki
path for now and be unified under the registry in S04.

## Skills Discovered

None installed. `bevy-ecs-expert` (already available) is relevant for the
ParamSet query / component-rename mechanics. No new skills needed — this is local
Bevy ECS work, not unfamiliar tech.

## Sources

Local code only: `src/windowed/render.rs`, `src/windowed/mod.rs`,
`src/ui/cues.rs`, `src/ui/hit_feedback.rs`; S01/S02 summaries; MEM030, MEM058,
MEM082, MEM094, MEM108, MEM113/MEM114; D042-D047.
