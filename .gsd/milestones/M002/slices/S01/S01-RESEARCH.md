# M002 / S01 — Runtime player + sprite render + Stance FSM foundation — Research

**Date:** 2026-05-19

## Summary

S01 turns M001's *data* into the first *behavior*: an on-screen Agumon sprite cycling idle, driven by a data-authored Stance FSM through a runtime AnimGraph player, with the schema seam extended so S02–S07 plug in without rewrites. M001 delivered the `AnimGraph`/`Clip` asset structs, a load+validate plugin (`src/animation/plugin.rs`), and a passing clip↔atlas geometry parity test. **Nothing renders today** — `windowed.rs::setup` spawns only `Camera2d`; there is no sprite, texture atlas, runtime player, or graph registry anywhere in the codebase (confirmed: `rg` for player/registry/stance returns nothing in `src/`).

The work splits into five low-coupling concerns: (1) **closed-enum schema extensions** (`AnimGraph.id`, `AnimNode.cues: Vec<FrameCue>`, `FrameCue{at,command}`, `ReleaseKernelCue`, `Predicate::KernelCue`) — additive, `#[serde(default)]`, no untagged, per the established convention in `anim_graph.rs`; (2) a **`GameplayCommandForbidden` validation check + executable test** (D001) that forbids `EmitDamage`/`EmitStatus`/`EmitHeal` in anim graphs, plus remediating the M001 `mul:18` duplicate at `assets/digimon/agumon/anim_graph.ron:18-26`; (3) **`SkillGraphRegistry` + `StanceGraphRegistry`** resolving id→graph with zero if-else (R008/D004); (4) a **Stance FSM RON asset** for Agumon (R005/D004); (5) the **runtime player + RenderPlugin/UiPlugin split** that ticks the FSM and binds a Bevy `Sprite`+`TextureAtlas` to the current node's frame range (R004).

The single biggest design tension surfaced during research (see Open Risks): the current schema binds **one `AnimGraph` to exactly one named clip range** (`AnimGraph.clip: ClipId`, validated by `FrameOutsideNamedClipRange` in `validation/graph.rs:114`). A Stance FSM spanning idle/hurt/death/victory needs **four** ranges. This must be resolved before the stance asset can be authored honestly per R005.

## Recommendation

Build in dependency order, schema-first, with the runtime player as the highest-risk proof. Keep the player's FSM core in a **feature-agnostic module** (headless unit-testable: node ticking, transition evaluation for `TimeInNode`/`Always`, frame-index derivation from `FrameRange`); gate **only** the wgpu sprite-sync system behind `#[cfg(feature="windowed")]`. This preserves D017 (headless default builds no winit/wgpu) and keeps the player testable without a GPU — mirroring how `Clock` (`src/combat/runtime/clock.rs`) keeps headless/windowed parity as a timing-only difference (I3).

Resolve the stance-clip-range tension explicitly (planner decision, see Open Risks) before authoring the stance asset. For the schema, prefer making `AnimGraph.id` a **required** field (it is the registry key per D004) and updating the two RON files + affected tests atomically, rather than `#[serde(default)]` which would allow an unkeyed graph to silently fail registry resolution. `cues` and `KernelCue` are additive with `#[serde(default)]` and need only round-trip coverage in S01 — they are *consumed* in S02, so S01 just lands the closed seam.

## Implementation Landscape

### Key Files

- `src/animation/anim_graph.rs` — schema. Add `id` field to `AnimGraph` (struct has `clip/entry/nodes/transitions`, `#[serde(deny_unknown_fields)]`). Add `cues: Vec<FrameCue>` to `AnimNode` (`#[serde(default)]`). Add `FrameCue{at: u32, command: Command|ReleaseKernelCue}`, `ReleaseKernelCue` (no id/number per D003), and `Predicate::KernelCue` variant to the closed `Predicate` enum. Follow the existing closed-enum + transparent-newtype convention exactly.
- `src/animation/validation/command.rs` — add a `GameplayCommandForbidden` rejection path: any `Command::EmitDamage|EmitStatus|EmitHeal` appearing in a graph (on_enter **or** new cues) is an Error diagnostic. `validate_command` currently *accepts and validates* these — invert for graph context.
- `src/animation/validation/types.rs` — add `AnimationValidationCheck::GameplayCommandForbidden` and a matching `AnimationValidationReason` variant (e.g. `GameplayCommandInAnimGraph`). Diagnostic plumbing is uniform; follow `CommandParam` precedent.
- `src/animation/validation/graph.rs` — `validate_graph_nodes` iterates `node.on_enter`; extend it to also walk `node.cues` and apply the forbidden-command check. Note `FrameOutsideNamedClipRange` at line 114 — central to the stance-range tension.
- `src/animation/plugin.rs` — `AnimationAssetPlugin` (load + track + validate). "Pre-split" this: separate asset-load/validate from a new render/player concern so `RenderPlugin` can attach the sprite system. `DEFAULT_ANIM_GRAPH_PATHS`/`DEFAULT_ANIM_CLIP_PATHS` here load **both agumon and renamon** — see Constraints.
- `src/windowed.rs` — currently all render+UI inline (`setup` spawns `Camera2d`; egui panels in `EguiPrimaryContextPass`). Introduce the `RenderPlugin` (sprite/atlas/player) vs `UiPlugin` (egui) split, both `#[cfg(feature="windowed")]`. Reuse `AnimationValidationState` already surfaced in `roster_panel`.
- `src/main.rs` — `#[cfg(feature)]` swap between `headless::register`/`windowed::register` already exists; wire the new plugin split through `windowed::register`.
- `assets/digimon/agumon/anim_graph.ron` — author `id`; **remove** the `EmitDamage(hits:1, mul:18, ...)` at lines 18-26 (the D001/MEM008 `mul:18` duplicate of `skills.ron` `DealDamage(amount:18)`). Keep `SpawnParticle` (presentation, allowed).
- `assets/digimon/agumon/clip.ron` — authoritative geometry: 512×512, 10×10, 93 frames; ranges idle=53-58, hurt=46-52, death=14-22, victory=76-92, skill=59-75. Atlas PNG `assets/digimon/agumon_atlas.png`.
- **New** `assets/digimon/agumon/stance.ron` (or chosen name) — Stance FSM (idle/hurt/death/victory), separate file per D004/R005, loaded via a stance path list and resolved by `StanceGraphRegistry`.
- **New** `src/animation/player.rs` (+ `registry.rs`) — feature-agnostic FSM core + the registries; windowed sprite-sync system gated separately.
- `tests/anim_graph_asset.rs`, `tests/anim_validation.rs`, `tests/anim_graph_parse.rs`, `tests/clip_geometry_parity.rs` — existing tests assert exact graph structure (e.g. `graph.entry == "baby_flame_cast"`, the `EmitDamage mul:18` assertion at the `baby_flame_impact` match arm). Adding `id` and removing `EmitDamage` **will break these**; update atomically.

### Build Order

1. **Schema extensions** (lowest risk, unblocks all). Add `id`, `cues`/`FrameCue`/`ReleaseKernelCue`, `Predicate::KernelCue`. Prove via serde round-trip + a parse test that `cues`-absent RON still loads (`#[serde(default)]`) and `KernelCue`/`ReleaseKernelCue` decode in the closed enum. No untagged.
2. **`GameplayCommandForbidden` check + test + asset remediation.** Add the check, the executable anti-DRY test (assert the agumon graph contains zero gameplay commands; assert a synthetic graph with `EmitDamage` fails validation), then remove the `mul:18` command from `agumon/anim_graph.ron` and fix the broken structural assertions in `anim_graph_asset.rs`. This is the user's DRY mandate made enforceable (D001) — isolate it.
3. **Registries** (`SkillGraphRegistry`, `StanceGraphRegistry`): id→`Handle<AnimGraph>` maps, resolution with zero if-else (R008). `CompiledTimeline.id = skill_id` is confirmed at `src/data/skill_timeline.rs:73` — the natural sync key.
4. **Stance FSM asset** for Agumon — *after* resolving the stance-clip-range tension (Open Risks). Load it via a new default-paths resource mirroring `AnimationGraphPaths`.
5. **Runtime player + RenderPlugin/UiPlugin split** (highest risk). FSM core headless-testable; windowed system spawns the sprite via `TextureAtlasLayout::from_grid` over `agumon_atlas.png`, advances frame index from the active node's `FrameRange` honoring `PlaybackModifier`, evaluates `TimeInNode`/`Always` transitions, loops idle.

### Verification Approach

- `cargo test --test clip_geometry_parity` — **already green** (verified this session: `1 passed`). MEM008 is **stale**: clip.ron was corrected in M001 commit `1cd2dbe` (heavy_attack 23-45, total 93, 512×512 — matches `agumon_atlas.json`). Keep it green.
- `cargo test` (headless default, no `windowed`) — all M001 anim/clip/combat tests stay green; success criterion "M001 headless tests green". Updated `anim_graph_asset.rs`/`anim_validation.rs` must pass with the new `id` + forbidden-command behavior.
- New executable test: graph authoring `EmitDamage`/`EmitStatus`/`EmitHeal` ⇒ validation Error (`GameplayCommandForbidden`); the live agumon graph ⇒ zero gameplay commands. This is the D001 anti-DRY gate.
- New headless FSM unit tests: idle node self-loops; `TimeInNode` advances after the node's frame span; frame index stays within the node `FrameRange`; missing skill-id ⇒ degenerate-instant fallback + error log (D010).
- `cargo run --features windowed` (or with `BEVYROGUE_VALIDATION_WINDOWED=1 …SOAK_SECS=…` for clean auto-exit, see `windowed.rs::config_from_env`) — Agumon visibly cycles idle driven by the stance graph, no panic. Qualitative; this is R004's visible-output criterion.

## Constraints

- **Bevy `=0.18.1` pinned**, `default-features=false`. The `windowed` feature adds `bevy/2d` (+ `bevy_egui =0.39.1`); headless builds **must not** compile sprite/wgpu systems (D017). The codebase uses Bevy 0.18 messaging (`MessageReader`/`MessageWriter`/`add_message`, not `EventReader`/`add_event`) — see `plugin.rs:159`. Sprite rendering in 0.18 uses the `Sprite` component with `TextureAtlas` + a `TextureAtlasLayout` asset (grid layout), not legacy `SpriteSheetBundle`. Confirm exact 0.18 API against installed bevy source before coding the sprite system.
- `#[serde(deny_unknown_fields)]` on `AnimGraph`/`AnimNode`: adding a **required** `id` invalidates both existing `anim_graph.ron` files and any test constructing `AnimGraph` literals; `cues` with `#[serde(default)]` is backward-compatible. Plan the asset+test edits as one atomic change.
- Closed-enum convention is mandatory (no `Custom(String)`, no `#[serde(untagged)]`) — D003 and the `anim_graph.rs` module doc state this explicitly.
- Boot validation is **strict-on-boot, resilient-on-reload** (MEM034/D010). An invalid graph at boot fails fast; this interacts with the renamon issue below.
- `src/source_file_loc_limit.rs` test enforces a per-file LOC cap — keep new modules small (the codebase already shards `validation/` and `turn_system/pipeline/`).

## Common Pitfalls

- **`DEFAULT_ANIM_GRAPH_PATHS` loads renamon too** (`plugin.rs:15`). `assets/digimon/renamon/anim_graph.ron` authors `EmitDamage(mul:16)` — the new `GameplayCommandForbidden` check will flag it, and strict-on-boot will fail the windowed run. M002 is Agumon-only. **Decision needed:** either (a) remediate renamon's graph too, or (b) scope the default graph/clip paths to Agumon-only for M002. The roadmap only mandates remediating the agumon `mul:18`; flag for planner.
- **Renamon side data is inconsistent and out of scope:** `renamon_atlas.json` says `total_frames:68` but `renamon/clip.ron` says `71`; there is no renamon parity test. Do not "fix" renamon in S01 — it is not in M002 scope and risks scope creep. Just ensure it doesn't block the Agumon boot path.
- **Stance vs single-clip-range mismatch** (see Open Risks) — do not author a 4-range stance file against the current single-`clip` schema without resolving it; `FrameOutsideNamedClipRange` will reject idle+hurt+death+victory nodes.
- **Two-clock barrier is S02, not S01.** `ReleaseKernelCue`/`Predicate::KernelCue` land as an inert closed seam here; `Clock` (`clock.rs`) and the `timeline_exec.rs` auto-resume (`run_to_completion`, ~line 124) are *consumed/rewired in S02* (D002). Do not wire suspension in S01.
- Player FSM logic in a `#[cfg(feature="windowed")]` module would make it untestable headless and break the "M001 headless tests green" criterion — keep FSM core feature-agnostic.

## Open Risks

- **Stance FSM ↔ clip-range schema tension (highest).** `AnimGraph.clip: ClipId` is a single named range; `validate_graph_nodes` (`validation/graph.rs:114`) rejects any node frame outside that one range. R005/D004 require a stance graph with idle(53-58)/hurt(46-52)/death(14-22)/victory(76-92) — four ranges. Resolution options for the planner: (i) let stance graphs key `clip` to a whole-sheet pseudo-range and rely on per-node `total_frames`-bounds only; (ii) make `clip` per-node or a set; (iii) relax `FrameOutsideNamedClipRange` for stance graphs. Option (i) is least invasive but changes validation semantics. **This blocks step 4** and likely warrants a recorded decision.
- **Frame timing model unspecified.** Nodes carry `FrameRange` + optional `PlaybackModifier{Hold/SpeedMul/Loop}` but no fps/per-frame duration. The player needs a frame-advance cadence (fixed ticks/frame vs wall-clock). S01 only needs idle looping, but the choice constrains S02's two-clock impact frame — pick deliberately and note it.
- **`bevy/2d` sprite API drift.** Bevy 0.18 sprite/atlas API differs from most online tutorials and from earlier 0.1x. Verify `Sprite`, `TextureAtlas`, `TextureAtlasLayout::from_grid`, and `Assets<TextureAtlasLayout>` signatures against the pinned crate source, not memory.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Bevy ECS (systems, queries, resources, plugin/schedule) | bevy-ecs-expert | installed (in `<available_skills>`) — apply for the player system, RenderPlugin/UiPlugin split, and `#[cfg(feature)]` schedule gating |

No additional skills needed: serde/RON, asset loading, and validation diagnostics all follow strong existing local patterns (`AnimationAssetPlugin`, `validate_command`, closed-enum schema).

## Sources

- Local code/asset inspection only; Bevy 0.18 sprite API to be confirmed against the pinned `bevy =0.18.1` crate source during planning/execution (no external docs fetched).
