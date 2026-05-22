# S03 Research: VFX flash renders as visible particles

**Lane:** research · **Depth:** targeted (known tech — Bevy 2D + established local seam pattern; new to *this* render path)

## Summary

The VFX seam is **fully defined and validated but completely inert on screen.** `Command::SpawnParticle { name: ParticleId, origin: VfxLocus, motion: VfxMotion }` exists as a closed-vocabulary command (`src/animation/anim_graph.rs:184`), round-trips through RON, is reachable by the validator (`src/animation/validation/command.rs:112`), and is asserted by `tests/animation/vfx_handle_seam.rs` to carry no numeric gameplay payload. **But nothing consumes it.** `advance_agumon_presentation` (the windowed playback tick in `src/windowed/render.rs`) reads only node `cues` for `ReleaseKernel`; it never reads node `on_enter` commands. So the lone authored `SpawnParticle` (on `baby_flame_cast`, `assets/digimon/agumon/anim_graph.ron:9`) is a no-op, and the Baby Burner "detonate flash" is today only an **egui chip** (`BabyBurnerFlashState`, `src/ui/combat_panel/mod.rs:128`) driven by a `CombatEvent` — never a world particle.

S03 closes this by mirroring the **exact S01 split** that succeeded for atlas binding: a **Bevy-free pure descriptor + resolver in the lib** (headless-testable, like `AtlasGeometry`) + a **windowed system that turns the descriptor into a real Bevy entity** with `Sprite`+`Transform`+lifetime. The headless structural test proves the seam yields a *renderable spawn descriptor* (visual-component intent, `VfxLocus`/`VfxMotion` honored, no numeric payload — `vfx_handle_seam` parity preserved); the user's `cargo winx` proves pixels appear.

## What exists / what's missing

| Piece | State | Location |
|---|---|---|
| `SpawnParticle` command + `ParticleId`/`VfxLocus`/`VfxMotion` closed enums | ✅ defined | `src/animation/anim_graph.rs:86,184,344,351` |
| Implemented enum variants | `VfxLocus`: `CasterCenter`, `TargetCenter`, `PrimaryTargetCenter`; `VfxMotion`: `Static`, `FollowTarget`, `ArcToTarget` | ⚠️ **much narrower than the §2.2d design draft** — use the *real* enums, not the draft's `SelfCenter/Travel/Radial/...` |
| `vfx_handle_seam.rs` parity tests | ✅ green (RON round-trip, closed-enum rejection, no-digit payload) | `tests/animation/vfx_handle_seam.rs` |
| `AnimNode.on_enter: Vec<Command>` | ✅ public, reachable from lib | `src/animation/anim_graph.rs:106-109` |
| on_enter consumption in windowed playback | ❌ **missing** — `advance_agumon_presentation` reads `cues` only, never `on_enter` | `src/windowed/render.rs:357-538` |
| Authored `SpawnParticle` commands | only `baby_flame_cast` has one; **sharp_claws & baby_burner have none** | `assets/digimon/agumon/anim_graph.ron` |
| Baby Burner detonate flash | ✅ as **UI chip only** (event-driven, `OnKernelTransition::Blueprint` detonate signal) | `src/ui/combat_panel/mod.rs:128-223` |
| World particle entity / VFX render system | ❌ **does not exist anywhere in `src/`** | — |

**Two distinct spawn sources must funnel into one renderable seam:**
1. **FSM `on_enter` `SpawnParticle`** — fires on node entry during a cast (Baby Flame cast already authored; skill surface).
2. **Reactive detonate signal** — Baby Burner's detonate, currently `observe_baby_burner_flash` → chip only (`mod.rs:211`). The ultimate's flash. The slice's phrase "SpawnParticle/**detonate** seam" names both.

## Active requirements / constraints owned here

- **R002/R005 (headless purity):** the testable contract (descriptor + `VfxLocus`→intent resolution) must live in the **lib** so `tests/` (lib-only link) can reach it; all `bevy/2d`/`Sprite`/`Transform`/`Commands` usage stays behind `#[cfg(feature = "windowed")]` in `src/windowed/` or `src/ui/`. The S01 `src/animation/atlas.rs` file is the proven template: pure descriptor, zero Bevy-2d imports, `pub use atlas::*`.
- **R004 (determinism):** VFX is cosmetic, windowed-only, **no RNG / no wall-clock in the resolution stream**. Particle lifetime/motion is driven by `Time`/`AnimationClock` in the presentation layer only. Design draft §2.2d §G confirms `SpawnParticle` is a no-op headless and never affects determinism.
- **CAP-7c065a44:** **no physics, no colliders** for VFX. Motion is pure presentation lerp (`Static`/`FollowTarget`/`ArcToTarget`).
- **K001:** auto-mode must not launch `cargo winx`; visual confirmation is the user's manual gate. Headless tests are the only CI proof.
- **§9 UI:** `BabyBurnerFlashState` chip **stays** as the UI affordance; the world-particle render is *added alongside* it (the boundary map is explicit on this). The detonate observer can spawn particles in addition to updating the chip.

## Natural seams (independent work units)

1. **Lib: pure VFX spawn descriptor + resolver** (`src/animation/vfx.rs`, new, Bevy-free; re-export via `pub use vfx::*` in `src/animation/mod.rs`). A struct (e.g. `ParticleSpawn`/`VfxSpawnRequest`) built from a `&Command::SpawnParticle`, carrying the `ParticleId`, `VfxLocus`, `VfxMotion`, and an explicit "renderable / has-visual-components" intent — the structural counterpart the headless test asserts on (vs. "only an opaque ParticleId"). Optionally a pure `resolve_locus(locus, caster_pos, target_pos) -> Vec2`-shaped helper taking plain coordinates (no Bevy types) so the windowed layer feeds it `Transform` translations. **This is the headless-testable contract.** Mirrors `AtlasGeometry::from_clip_meta` exactly.
2. **Lib/tests: structural test** (`tests/animation/` — new file, registered in `tests/animation.rs` via `#[path]`). Asserts: a `SpawnParticle` → spawn descriptor with visual-component intent; `VfxLocus`/`VfxMotion` preserved through the descriptor; serialized/observable form carries no numeric gameplay payload (re-run the `vfx_handle_seam` no-digit invariant against the descriptor). This is the slice's **first proof** and the only CI gate.
3. **Windowed: on_enter consumer + particle entity** (`src/windowed/render.rs`). Detect node entry (compare `current_node` before/after `advance_result` — render.rs already snapshots `current_node` at line 411), read the entered node's `on_enter` `SpawnParticle` commands, resolve `VfxLocus` from the caster/target sprite `Transform`s, spawn a short-lived entity (`Sprite::from_color` colored quad or a reused atlas tile — **no new asset needed**, keeps VRAM scope tight) with a `VfxParticle { ttl, motion }` component. A despawn/advance system ticks ttl + applies motion lerp, then despawns. Gate on the caster sprite (reuse `barrier_targets_sprite`).
4. **Windowed: detonate → particles** (`src/ui/combat_panel/mod.rs` or a windowed system). `observe_baby_burner_flash` already extracts targets per detonate signal; spawn one particle per target alongside the chip update. Keeps the chip; adds pixels.
5. **Asset authoring (likely required for full visual coverage):** add `SpawnParticle` `on_enter` to the impact/strike nodes of `sharp_claws` and the launch node of `baby_burner` in `anim_graph.ron` so all skill surfaces emit. **Note:** the validator (`validation/command.rs`) checks particle refs against a known set (`validation/types.rs` `particles: BTreeSet<ParticleId>`) — confirm any new particle name is registered or the validation test will fail. The slice's headless bar only requires the *seam*; deciding how many surfaces author a `SpawnParticle` is a planning call (minimum: Baby Flame cast already has one + Baby Burner detonate).

## First proof (highest risk / biggest unblocker)

**Seam #1 + #2: the Bevy-free spawn descriptor and its headless structural test.** It is the entire CI-provable deliverable and de-risks the lib/windowed split before any Bevy entity code is written. If the descriptor can be built from a `Command::SpawnParticle`, expose visual-component intent, preserve `VfxLocus`/`VfxMotion`, and pass the no-numeric-payload assertion — the contract is locked and the windowed work is "just" wiring (the S01/S02 pattern proved this is low-risk once the seam is fixed).

## Don't hand-roll

- **Don't invent a new VFX command or extend the closed enums.** `SpawnParticle` + the three-variant `VfxLocus`/`VfxMotion` are deliberately closed (MEM044). Render through the existing seam (matches the M003 CONTEXT "VFX via Cue/reactive bus, never physics" decision).
- **Don't resolve `VfxLocus` against the design-draft enum** (§2.2d lists `SelfCenter/Adj/WorldGrid/Travel/Radial/...` — those are **not implemented**). Use only `CasterCenter`/`TargetCenter`/`PrimaryTargetCenter` and `Static`/`FollowTarget`/`ArcToTarget`.
- **Don't add a particle asset/atlas** unless visual review demands it — a colored `Sprite` quad satisfies "visible particles" and avoids the ~100 MiB/atlas VRAM cost flagged in CONTEXT. Defer real particle textures to roster scale.
- **Don't drive motion with physics/RNG/wall-clock** (CAP-7c065a44, R004) — presentation-layer lerp on `Time` only.

## Patterns to reuse

- **S01 Bevy-free seam** (`src/animation/atlas.rs` + `pub use`): pure descriptor in lib, real Bevy type built windowed-side. Direct template for `vfx.rs`.
- **S01/S02 impact-frame test pattern**: load the RON graph, scan nodes for the relevant command, assert structural properties — reuse for the on_enter `SpawnParticle` scan.
- **`barrier_targets_sprite`** (`render.rs:696`): caster-gating so only the casting actor emits — reuse so the dummy doesn't double-spawn.
- **Node-entry detection**: `render.rs:411` already clones `current_node` pre-advance; compare to post-advance node to fire on_enter exactly once on entry (mirror the `already_released_frame` dedup discipline).

## Verification

- **CI (headless, the real gate):** `cargo test --test animation` — new structural test green + all 6 atlas_binding + 4 `vfx_handle_seam` tests still pass (closed-enum + no-numeric-payload parity preserved). `cargo test` full suite green (no windowed dep leak, R002/R005). `cargo build --features windowed` clean. If new particle names are authored: `cargo test --test animation` validation tests must still pass (check `validation/types.rs` registered set).
- **Visual (user manual, K001):** `cargo winx` — VFX flash renders as visible particles during Baby Flame (skill) and Baby Burner (ultimate/detonate) on **both** actors; particles appear at the correct locus and respect motion; chip still shows. `scripts/capture-windowed-smoke.sh` optional artifact.

## Skills discovered

- **bevy-ecs-expert** (installed) — entity spawn/despawn, component design, system ordering for the particle entity + ttl system.
- **rust-skills / rust-development** (installed) — closed-enum discipline, ownership for the descriptor seam.
- No external skill install needed (Bevy + RON already local; no new third-party tech).

## Sources

- `src/animation/anim_graph.rs` (Command/VfxLocus/VfxMotion), `src/windowed/render.rs` (playback tick, no on_enter consumption), `src/ui/combat_panel/mod.rs` (detonate flash chip), `tests/animation/vfx_handle_seam.rs` (parity), `assets/digimon/agumon/anim_graph.ron` (authored SpawnParticle), `docs/future_design_draft/02-02d_vfx_positioning.md` (design intent — note enum drift), S01/S02 SUMMARYs (seam pattern), MEM044/MEM053/MEM061.
