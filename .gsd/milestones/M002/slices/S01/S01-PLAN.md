# S01: Runtime player + sprite render + Stance FSM foundation

**Goal:** Turn M001's animation data into the first on-screen behavior: a windowed Agumon sprite cycling idle driven by a data-authored Stance FSM through a feature-agnostic runtime AnimGraph player, with the schema seam closed-extended (id, cues, ReleaseKernelCue, Predicate::KernelCue) so S02–S07 plug in without rewrites. M001 headless tests stay green; the D001 anti-DRY gate (zero gameplay numbers in anim graphs) is executable; clip↔atlas geometry parity stays green.
**Demo:** cargo run --features windowed shows Agumon cycling idle via the stance graph (not hardcoded); M001 headless tests green; clip-atlas geometry parity test present and passing.

## Must-Haves

- cargo test (headless default) is fully green including updated anim_graph_asset/anim_validation/anim_graph_parse, clip_geometry_parity, new headless player FSM tests, the GameplayCommandForbidden anti-DRY test, registry resolution tests, and stance.ron parse/validate test. cargo build --features windowed compiles with no winit/wgpu compiled into the headless default (D017). A documented windowed soak run (BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=3 cargo run --features windowed) exits cleanly with Agumon visibly cycling idle driven by the stance graph (R004 qualitative).

## Proof Level

- This slice proves: integration — proves the data→behavior seam end-to-end headless (FSM core, validation gate, registries) plus a compiling windowed render path; full visible-combat assembly is later slices.

## Integration Closure

Upstream consumed: M001 AnimGraph/Clip schema + AnimationAssetPlugin + clip_geometry_parity. New wiring: RenderPlugin/UiPlugin split in windowed::register; AnimationStancePaths load path; SkillGraphRegistry/StanceGraphRegistry populated from loaded handles. Remaining before milestone usable: S02 two-clock impact barrier consumes the inert cues/ReleaseKernelCue/Predicate::KernelCue seam and the player; S03 §9 strip consumes the UiPlugin split.

## Verification

- Player logs a structured error and uses a degenerate-instant fallback on missing/unresolvable graph id (D010/MEM030). Stance/skill graph validation failures surface via the existing AnimationValidationState resource (strict-on-boot). Registry resolution is a pure lookup whose miss path is observable via the fallback log.

## Tasks

- [ ] **T01: Closed-enum schema extensions + atomic id/asset/test migration** `est:3h`
  Why: every later task and slice keys off AnimGraph.id (registry, D004/MEM024) and the inert cue seam (S02). Schema-first is lowest risk and unblocks all. Do: In src/animation/anim_graph.rs add a required `id: AnimGraphId` field to AnimGraph (closed transparent newtype mirroring ClipId/NodeId; keep #[serde(deny_unknown_fields)]). Add `cues: Vec<FrameCue>` with #[serde(default)] to AnimNode. Add `FrameCue { at: u32, command: FrameCueCommand }` where FrameCueCommand is a CLOSED enum carrying either a presentation Command or `ReleaseKernelCue` (no untagged, no Custom(String); follow the existing closed-enum convention exactly — MEM023/D003). Add `ReleaseKernelCue` (no id, no number — D003). Add a `KernelCue` variant to the closed Predicate enum (inert in S01, consumed S02). Re-export new public types where ClipId/NodeId are re-exported. Add `id` to assets/digimon/agumon/anim_graph.ron and assets/digimon/renamon/anim_graph.ron (do NOT yet remove EmitDamage — that is T02). Atomically update every test that constructs an AnimGraph literal or asserts graph structure to include the new id field: tests/anim_graph_asset.rs, tests/anim_graph_parse.rs, tests/anim_validation.rs. Add a round-trip parse test proving (a) a cues-absent graph still loads via #[serde(default)], (b) a graph with cues:[FrameCue(at:N, command: <ReleaseKernelCue and a presentation Command>)] and a transition with when: KernelCue parses through the closed enum with no untagged fallback, (c) an unknown cue/predicate variant is rejected. Keep each touched/new source file under the 500-LOC cap (source_file_loc_limit). Done when: cargo test passes headless with id required everywhere and the new round-trip seam test green. Decisions: D040 (id required, not serde default), D041/D042 context. Negative tests (Q7): unknown FrameCue command variant and unknown Predicate variant must fail to parse; cues-absent RON must still succeed.
  - Files: `src/animation/anim_graph.rs`, `src/animation/mod.rs`, `assets/digimon/agumon/anim_graph.ron`, `assets/digimon/renamon/anim_graph.ron`, `tests/anim_graph_asset.rs`, `tests/anim_graph_parse.rs`, `tests/anim_validation.rs`
  - Verify: cargo test --test anim_graph_parse --test anim_graph_asset --test anim_validation

- [ ] **T02: GameplayCommandForbidden validation check + anti-DRY test + EmitDamage remediation** `est:2h30m`
  Why: the D001/MEM021 mandate — anim graphs must author zero gameplay numbers — must become an executable invariant; the M001 mul:18 (agumon) and mul:16 (renamon) duplicates must be removed behind it. Do: In src/animation/validation/types.rs add AnimationValidationCheck::GameplayCommandForbidden and AnimationValidationReason::GameplayCommandInAnimGraph (follow the CommandParam precedent for diagnostic plumbing). In src/animation/validation/graph.rs (validate_graph_nodes) and/or validation/command.rs, add a check that any Command::EmitDamage | EmitStatus | EmitHeal appearing in node.on_enter OR in node.cues (walk cues too — FrameCue command from T01) produces an Error diagnostic. Keep existing param/status validation working for non-graph use if shared. Remove the EmitDamage block from assets/digimon/agumon/anim_graph.ron (the mul:18 dup) and from assets/digimon/renamon/anim_graph.ron (mul:16) — keep SpawnParticle (presentation, allowed). Fix the now-broken structural assertions in tests/anim_graph_asset.rs and tests/anim_validation.rs that asserted the EmitDamage mul:18 / FrameOutsideNamedClipRange-adjacent shape. Add an executable anti-DRY test (new tests/anim_gameplay_command_forbidden.rs): assert the live loaded agumon graph contains zero gameplay commands; assert a synthetic graph with EmitDamage in on_enter fails with GameplayCommandForbidden; assert a synthetic graph with EmitDamage inside a FrameCue cue also fails. Done when: cargo test green, both production graphs gameplay-command-free, the new test enforces D001. Decisions: D042 (renamon remediated identically). Q7 negatives: EmitStatus and EmitHeal in a graph also rejected, not just EmitDamage.
  - Files: `src/animation/validation/types.rs`, `src/animation/validation/command.rs`, `src/animation/validation/graph.rs`, `assets/digimon/agumon/anim_graph.ron`, `assets/digimon/renamon/anim_graph.ron`, `tests/anim_graph_asset.rs`, `tests/anim_validation.rs`, `tests/anim_gameplay_command_forbidden.rs`
  - Verify: cargo test --test anim_gameplay_command_forbidden --test anim_graph_asset --test anim_validation

- [ ] **T03: SkillGraphRegistry + StanceGraphRegistry (pure id->Handle resolution, R008)** `est:2h`
  Why: R008/D004/MEM024 require id->graph resolution with zero if-else; the player (T05) resolves the stance graph through StanceGraphRegistry and S02+ resolve skills via SkillGraphRegistry by the shared skill-id (CompiledTimeline.id = skill_id, skill_timeline.rs:73). Do: Create src/animation/registry.rs with two Resource types each wrapping a map from AnimGraphId -> Handle<AnimGraph>, plus a pure `resolve(&self, id) -> Option<&Handle<AnimGraph>>` (no if-else dispatch; map lookup only). Classify by load-path provenance per D042: graphs loaded via AnimationGraphPaths populate SkillGraphRegistry; graphs loaded via the new AnimationStancePaths (added in T04) populate StanceGraphRegistry. Add a system (or extend the existing track_*_loads path in plugin.rs) that inserts entries keyed by the loaded AnimGraph.id once each handle resolves. Register both resources in AnimationAssetPlugin. Keep registry.rs under the 500-LOC cap. Done when: a headless unit/integration test registers graphs and asserts hit (known id) and miss (unknown id -> None) for both registries; cargo test green. Decisions: D042. Q7: unknown id resolves to None (not panic); duplicate id is last-write-wins or rejected — pick and assert one.
  - Files: `src/animation/registry.rs`, `src/animation/mod.rs`, `src/animation/plugin.rs`, `tests/anim_registry.rs`
  - Verify: cargo test --test anim_registry

- [ ] **T04: Agumon Stance FSM asset + whole-sheet clip range + stance load path** `est:2h`
  Why: R005/D004 require a data-authored per-Digimon Stance FSM (not hardcoded); the player ticks it. Resolves the highest research risk (stance vs single-named-clip-range) via D039. Do: Add a whole-sheet named range to assets/digimon/agumon/clip.ron: "all": (start: 0, end: 92) (total_frames=93, max index 92; leaves the existing 8 ranges untouched so clip_geometry_parity stays green). Author assets/digimon/agumon/stance.ron: id "agumon_stance", clip "all", entry "idle", nodes idle(frames 53-58, modifier Loop), hurt(46-52), death(14-22), victory(76-92); transitions minimal and S01-supported only — idle self-cycles (Loop modifier; no transition needed to loop, or an Always self-edge), other nodes present but inert (no KernelEvent/UserInput predicates required in S01, those are S02+). No gameplay commands (T02 check will reject any). Add DEFAULT_ANIM_STANCE_PATHS (Agumon only — D042) and an AnimationStancePaths resource in src/animation/plugin.rs; load + validate stance graphs through the existing AnimationAssetPlugin pipeline (RonAssetPlugin<AnimGraph>, validate_anim_graph) and feed them to StanceGraphRegistry (T03 provenance). Done when: a test asserts stance.ron parses, validates with zero Error diagnostics (clip "all" makes idle/hurt/death/victory all in-range — D039), clip_geometry_parity still green, and StanceGraphRegistry resolves "agumon_stance". Decisions: D039, D042. Q7: a stance node outside [0,92] still fails FrameOutsideClipTotal (guard intact).
  - Files: `assets/digimon/agumon/clip.ron`, `assets/digimon/agumon/stance.ron`, `src/animation/plugin.rs`, `tests/anim_stance_asset.rs`
  - Verify: cargo test --test anim_stance_asset --test clip_geometry_parity

- [ ] **T05: Runtime AnimGraph player + RenderPlugin/UiPlugin split (windowed Agumon idle)** `est:5h`
  Why: highest-risk proof — turns the data into visible behavior (R004) while keeping headless tests green (D017). Do: Create src/animation/player.rs as a FEATURE-AGNOSTIC FSM core (no #[cfg(feature)], no world globals — MEM027): player state {current_node, elapsed_anim_frames}; a pure `advance(elapsed_anim_frames)` that derives the active sprite frame index from the node FrameRange honoring PlaybackModifier (Loop/Hold/SpeedMul) and `reverse`, and evaluates only TimeInNode/Always transitions (S01 scope); idle self-loops. On an unresolvable graph id, use a degenerate-instant fallback and emit a structured error log with graph_id/node context (D010/MEM030). Per D041 the core takes an explicit elapsed-anim-frames count — NO wall-clock, NO fps in the graph. Add headless unit tests: idle node self-loops; TimeInNode transitions after the node frame span elapses; derived frame index always within the node FrameRange; missing/unresolvable id -> fallback + error signal observable. Then split windowed.rs: introduce RenderPlugin (Camera2d + a #[cfg(feature="windowed")] sprite system that spawns Sprite with TextureAtlas over assets/digimon/agumon_atlas.png via TextureAtlasLayout::from_grid(UVec2::new(512,512), 10, 10, None, None), resolves "agumon_stance" from StanceGraphRegistry, ticks the FSM core by converting Time delta to anim frames at a fixed anim-FPS constant — timing-only, I3 — and writes Sprite.texture_atlas.index from the derived frame) and UiPlugin (the existing egui roster/turn/combat panels). Wire both through windowed::register; keep the FSM core compiled in headless. Confirm the Bevy 0.18.1 Sprite/TextureAtlas/TextureAtlasLayout API against the pinned crate source before coding (Sprite{ image, texture_atlas: Option<TextureAtlas>, .. }; TextureAtlas{ layout: Handle<TextureAtlasLayout>, index }; from_grid(UVec2,u32,u32,Option<UVec2>,Option<UVec2>)). Keep every new/edited source file under the 500-LOC cap (split player.rs / render plugin if needed). Done when: cargo test passes headless (FSM core tests run; no winit/wgpu compiled — D017); cargo build --features windowed compiles; documented soak run shows Agumon cycling idle with no panic. Decisions: D041. Failure modes (Q5): unresolvable stance id -> logged fallback, no panic; missing atlas image -> Bevy asset error surfaced, run aborts visibly (strict-on-boot).
  - Files: `src/animation/player.rs`, `src/animation/mod.rs`, `src/animation/plugin.rs`, `src/windowed.rs`, `src/main.rs`, `tests/anim_player_fsm.rs`
  - Verify: cargo test --test anim_player_fsm

## Files Likely Touched

- src/animation/anim_graph.rs
- src/animation/mod.rs
- assets/digimon/agumon/anim_graph.ron
- assets/digimon/renamon/anim_graph.ron
- tests/anim_graph_asset.rs
- tests/anim_graph_parse.rs
- tests/anim_validation.rs
- src/animation/validation/types.rs
- src/animation/validation/command.rs
- src/animation/validation/graph.rs
- tests/anim_gameplay_command_forbidden.rs
- src/animation/registry.rs
- src/animation/plugin.rs
- tests/anim_registry.rs
- assets/digimon/agumon/clip.ron
- assets/digimon/agumon/stance.ron
- tests/anim_stance_asset.rs
- src/animation/player.rs
- src/windowed.rs
- src/main.rs
- tests/anim_player_fsm.rs
