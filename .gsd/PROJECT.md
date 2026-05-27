# Project

## What This Is

**bevyrogue** è un roguelite RPG monster-taming turn-based in Rust + Bevy 0.18, headless-first by default, with optional `windowed` egui/wgpu UI.

Core value: una run giocabile end-to-end dove combat, party build, e futuri layer UI/CLI leggono una sola combat authority.

## Current State

**M006 is active.** M002–M005 are complete: first on-screen combat, two-clock impact sync, §9 UI read-model surfaces, data-driven per-Digimon VFX, HDR/bloom rendering closeout, hurt/death reactions, and enoki-backed Agumon contact bursts are all in place. The current open milestone is **M006: Extension-first presentation refactor + enoki-only VFX**. Its first four slices are complete (single enoki VFX path, generic CueRegistry cosmetic feedback, DigimonSprite/camera-shake wiring, and Agumon extraction out of engine files). The remaining open slice is **S05: Second digimon (Renamon) with zero engine edits**. Automated proof is green (`cargo test`, `cargo test --features windowed`, `cargo build --features windowed`); live `cargo winx` visual sign-off remains K001 human-only.

Delivered capabilities (M001–M005, all validated):
- **Animation pipeline (M001):** typed AnimGraph RON loading, validation, hot-reload, clip↔atlas parity.
- **On-screen combat (M002):** AnimGraph runtime player + two-clock impact sync + §9 UI core; full Agumon kit vs dummy with energy-backed ult gauge, typed pure AnimGraph input seam (R009), and hardened failure visibility (R013).
- **Atlas rendering (M003):** atlas-bound sprites with frame→index mapping and rendered impact-frame damage bridge.
- **Data-driven VFX (M004):** owned per-Digimon `VfxAsset` schema (placement/appearance/variation verbs), HDR/bloom overbright rendering proxy.
- **Visual feedback (M005):** hurt/death reactions, flash/shake, floating damage numbers, and enoki-backed Agumon contact bursts.

Current headless baseline: all integration and lib test targets green (cargo test + cargo test --features windowed).

## Architecture / Key Patterns

### Headless-first Bevy
Default features avoid UI/windowing. All gameplay logic — combat authority, skill resolution, legality, AI, targeting — runs without wgpu or winit. The `windowed` feature gates: `RenderPlugin`, `UiPlugin`, `egui`, sprite systems, and the windowed-only egui phase strip.

### Combat authority
Action query + turn pipeline + resolution + kernel/hooks decide legality, timing, damage, and state. Content layer (RON) owns numbers, tags, target shape, scaling, SP/ult costs, and presentation metadata — no skill logic. Skill behavior lives in Rust.

### Two-clock model (D012 / D025)
`Clock::HeadlessAuto` drives the resolution timeline deterministically. `Clock::Windowed` is the presentation clock: the `CueBarrier` suspends the timeline at each authored impact frame and releases when the animation player emits `ReleaseKernelCue`. Damage lands on the visible impact frame; the kernel stays frame-ignorant.

### AnimGraph seam (R004–R009)
- `SkillGraphRegistry` maps skill-id to `AnimGraph` RON asset (1:1 with CompiledTimeline).
- `StanceGraphRegistry` maps Digimon id to default stance graph (idle/hurt/death/victory).
- `AnimGraphRole` (closed enum) + `AnimGraphInput` (read-only set) form the typed pure input seam; graph evaluation is a pure function of these inputs with no world-global reads.
- `GameplayCommandForbidden` validation test ensures no EmitDamage/EmitStatus/EmitHeal in anim_graph.ron.
- Hot reload: animation players bind cloned resolved-graph snapshots at spawn; registry updates affect only future spawns (next-spawn-only policy).

### Failure visibility (R013)
- Cue barrier: 180-frame timeout, force-resume through released-runner path; `CueBarrierStatus` retains cast/skill/timeline/beat/cue/hop/animation context.
- AnimGraph registry: missing skill-id strict boot error for known M002 assets; runtime degrades to `InstantFallback` with `AnimationGraphLookupDiagnostics`.
- Hot reload: next-spawn by snapshot binding (in-flight players keep current graph identity).
- Dead target mid-loop: presentation completes without liveness branching; post-KO overshoot observable via `CombatEvent` and `ActionLog`.

### VFX seam (R012)
Opaque `ParticleId(String)` handle + closed `VfxLocus`/`VfxMotion` enums. `SpawnParticle` round-trips through RON with no gameplay payload. Unknown variants fail to deserialize. The seam is open for a future RON VFX pipeline (bevy_enoki/Omagari) without touching the anim graph or kernel.

### §9 UI (R010)
Phase strip + HP bars + damage numbers + hurt blink driven exclusively by `EventReader<CombatEvent>`. A structural test proves the UI code path has no write access to `CombatState` (D008 enforced structurally, not by convention).

### Energy gauge (R011 / S07)
Digimon with `ult_gauge=energy` metadata read/write `Energy` for ult readiness and drain; metadata-free Digimon stay on the legacy `UltimateCharge` path. A shared snapshot helper (`UnitQuerySnapshot`) prevents legality/display drift. Each runtime finalize seam honors `UltEffect::Reset` for energy-backed actors individually.

### Frame-time observability (D027)
`FrameTimeAccumulator` in `src/combat/observability/frame_time.rs` is a pure, Bevy-free module fed `Time::delta_secs` from the `Clock::Windowed` presentation tick. `frame_time_regression` implements the D027 threshold: mean ≤15% regression AND ≤2ms absolute (AND semantics protect fast baselines). `BEVYROGUE_VALIDATION_BASELINE` env toggle skips `RenderPlugin` for an apples-to-apples kernel-only baseline. Live soak data pending manual capture (K001: auto-mode cannot launch windowed binary).

### Boundary map
`M002-BOUNDARY-MAP.md` documents 5 producer→consumer contracts, each row citing on-disk test function names (machine-checkable by verification script).

### Pre-M002 combat patterns (still current)
- **Blueprint seam:** per-Digimon Rust modules produce generic kernel intents; canonical paths `blueprints::<name>::<Type>`.
- **Typed kernel:** Tactical Cycle, Strain, Flow, Fatigue, beats, tags, mechanic transitions in typed Rust.
- **Event bus:** `CombatEvent` canonical consumer stream (includes `UltimateUsed`, `UnitDied`, `Damage` with pre_dr/final).
- **StatusBag:** per-unit component with single-instance-per-kind enforcement; Heated/Chilled/Paralyzed/Slowed/Blessed semantics (§H.1).
- **TargetShape resolver:** pure `resolve_targets` + `select_bounce_hop` ECS-free fns.
- **DR mitigation:** `DrBag` with `(1.0 - sum_dr).max(0.0)` multiplicative factor; observability via pre_dr/final in `CombatEvent::Damage`.
- **Turn manipulation:** `AdvanceTurn(u32)` / `DelayTurn(u32)` with ±50% cap.

## Capability Contract

See `.gsd/REQUIREMENTS.md`. Active requirements: **0** (the current baseline has no open capability requirements tracked there). Current validated baseline: **M001–M005 complete; M006 in progress**.

## Milestone Sequence

- [x] M001: Animation asset pipeline — Typed AnimGraph RON asset loading, validation, hot-reload, and clip↔atlas parity contract established headless-first.
- [x] M002: First on-screen combat (Agumon-only) — AnimGraph runtime player + two-clock impact sync + §9 UI core + full Agumon kit vs dummy on screen, gated by repomix architectural review.
- [x] M003: Make Agumon render on-screen — atlas-bound sprites, impact-frame bridge, and manual windowed sign-off for the five presentation surfaces.
- [x] M004: Per-Digimon data-driven VFX (owned, extension-first).
- [x] M005: Combat visual feedback completion (reactions + enoki VFX).
- [ ] M006: Extension-first presentation refactor + enoki-only VFX — active; S05 (Renamon zero-engine-edit closeout) remains open.

## Current Execution Focus

**M006 / S05: Second digimon (Renamon) with zero engine edits** — finish slice closeout around the active extension-first presentation seam. The automated contracts are already in place; the remaining boundary is honest milestone closure plus the human-only K001 visual surface.

Deferred / still-open items carried forward from earlier milestones:
- Live windowed soak frame-time numbers (K001: still requires manual capture; `frame-time-comparison.md` remains the historical method/proof artifact)
- S06 architectural review findings F1–F7 (all low/medium/info; triaged roadmap in S06-ARCHITECTURAL-REVIEW.md)
- Editor tooling for combined AnimGraph + VFX authoring (the data-driven VFX path exists; authoring/editor UX is still open)
- Full playable CLI UX and encounter chain

## Operational Notes

- `cargo test` for headless baseline; `cargo test --features windowed` for full suite including windowed_only.
- `cargo winx` for human visual verification (requires display; K001 blocks in auto-mode).
- `M002-BOUNDARY-MAP.md`: 5 producer→consumer contracts with machine-checkable test citations.
- `S06-ARCHITECTURAL-REVIEW.md`: repomix architectural review findings F1–F7 (low/medium/info; triage status tracked in the Deferred items list above).
- `src/animation/anim_graph.rs`: AnimGraphRole, AnimGraphInput, AnimGraph schema.
- `src/animation/player.rs`: typed input threading, legacy default-input wrappers.
- `src/animation/registry.rs`: SkillGraphRegistry, StanceGraphRegistry, resolved-graph snapshots.
- `src/combat/runtime/cue_barrier.rs`: CueBarrierStatus, 180-frame timeout, force-resume.
- `src/combat/observability/frame_time.rs`: FrameTimeAccumulator, frame_time_regression (D027 thresholds).
- `src/windowed/mod.rs`: WindowedValidationState, BEVYROGUE_VALIDATION_BASELINE toggle.
- `tests/animation/anim_graph_input_purity.rs`: R009 typed input proof.
- `tests/timeline/r013_failure_visibility.rs`: R013 failure-visibility proof.
- `tests/animation/vfx_handle_seam.rs`: R012 VFX seam proof.
- Status taxonomy: `src/combat/status_effect.rs`.
- Turn manipulation: `src/combat/av.rs` (AdvanceTurn/DelayTurn with ±50% cap).
- DR bag: `src/combat/buffs.rs`; damage integration: `src/combat/damage.rs`.
- Event bus variants: `src/combat/events.rs`.
- Blueprint canonical paths: `src/combat/blueprints/<name>/mod.rs`.
- When widening ResolveActorsQuery beyond 15 components: use a sibling read-only query (Bevy 15-tuple QueryData compile limit).
