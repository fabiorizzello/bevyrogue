---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T05: Runtime AnimGraph player + RenderPlugin/UiPlugin split (windowed Agumon idle)

Why: highest-risk proof — turns the data into visible behavior (R004) while keeping headless tests green (D017). Do: Create src/animation/player.rs as a FEATURE-AGNOSTIC FSM core (no #[cfg(feature)], no world globals — MEM027): player state {current_node, elapsed_anim_frames}; a pure `advance(elapsed_anim_frames)` that derives the active sprite frame index from the node FrameRange honoring PlaybackModifier (Loop/Hold/SpeedMul) and `reverse`, and evaluates only TimeInNode/Always transitions (S01 scope); idle self-loops. On an unresolvable graph id, use a degenerate-instant fallback and emit a structured error log with graph_id/node context (D010/MEM030). Per D041 the core takes an explicit elapsed-anim-frames count — NO wall-clock, NO fps in the graph. Add headless unit tests: idle node self-loops; TimeInNode transitions after the node frame span elapses; derived frame index always within the node FrameRange; missing/unresolvable id -> fallback + error signal observable. Then split windowed.rs: introduce RenderPlugin (Camera2d + a #[cfg(feature="windowed")] sprite system that spawns Sprite with TextureAtlas over assets/digimon/agumon_atlas.png via TextureAtlasLayout::from_grid(UVec2::new(512,512), 10, 10, None, None), resolves "agumon_stance" from StanceGraphRegistry, ticks the FSM core by converting Time delta to anim frames at a fixed anim-FPS constant — timing-only, I3 — and writes Sprite.texture_atlas.index from the derived frame) and UiPlugin (the existing egui roster/turn/combat panels). Wire both through windowed::register; keep the FSM core compiled in headless. Confirm the Bevy 0.18.1 Sprite/TextureAtlas/TextureAtlasLayout API against the pinned crate source before coding (Sprite{ image, texture_atlas: Option<TextureAtlas>, .. }; TextureAtlas{ layout: Handle<TextureAtlasLayout>, index }; from_grid(UVec2,u32,u32,Option<UVec2>,Option<UVec2>)). Keep every new/edited source file under the 500-LOC cap (split player.rs / render plugin if needed). Done when: cargo test passes headless (FSM core tests run; no winit/wgpu compiled — D017); cargo build --features windowed compiles; documented soak run shows Agumon cycling idle with no panic. Decisions: D041. Failure modes (Q5): unresolvable stance id -> logged fallback, no panic; missing atlas image -> Bevy asset error surfaced, run aborts visibly (strict-on-boot).

## Inputs

- `src/animation/anim_graph.rs`
- `src/animation/registry.rs`
- `src/animation/plugin.rs`
- `src/windowed.rs`
- `src/main.rs`
- `assets/digimon/agumon/stance.ron`
- `assets/digimon/agumon/clip.ron`
- `Cargo.toml`

## Expected Output

- `src/animation/player.rs`
- `src/animation/mod.rs`
- `src/animation/plugin.rs`
- `src/windowed.rs`
- `src/main.rs`
- `tests/anim_player_fsm.rs`

## Verification

cargo test --test anim_player_fsm
