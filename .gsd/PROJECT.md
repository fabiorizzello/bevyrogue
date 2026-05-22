# Project

## What This Is

**bevyrogue** è un roguelite RPG monster-taming turn-based in Rust + Bevy 0.18, headless-first by default, with optional `windowed` egui/wgpu UI.

Core value: una run giocabile end-to-end dove combat, party build, e futuri layer UI/CLI leggono una sola combat authority.

## Current State

**M002 is complete.** First on-screen combat is delivered: AnimGraph runtime player + wgpu sprite render + §9 UI core + two-clock impact sync, Agumon-only. `cargo run --features windowed` runs Agumon vs an Agumon dummy at full kit — Sharp Claws, Baby Flame, Baby Burner, Twin Core — with damage landing on the visible impact frame, HP bars, damage numbers, hurt blink, and the dummy dying at 0 HP. M002 was gated at closeout by a repomix architectural review (S06-ARCHITECTURAL-REVIEW.md, 7 findings F1–F7, none critical, all deferred to M003+).

All M002 success criteria verified:
- S01: Agumon idle cycling via stance graph (not hardcoded); M001 headless suite green; clip↔atlas parity present.
- S02: Sharp Claws windup→strike→recovery on screen; damage on impact frame via ReleaseKernelCue; I3 intent parity headless/windowed.
- S03: §9 phase strip from EventReader<CombatEvent>; structural test proves UI never mutates combat state.
- S04: Baby Burner reactive detonate + flash VFX (Rust code, no RON/editor); zero non-determinism.
- S05: Full Agumon kit vs dummy; multi-hit loop = kernel hop count; CombatEvent-driven blink; HUD HP/damage; dummy dies at 0 HP.
- S06: Windowed soak runbook + capture script; repomix architectural review report (pass-with-followups); R016 invariants green.
- S07: Energy-backed Agumon ult gauge; readiness flips at full energy; cast drains to zero; legacy path preserved for non-opted-in Digimon.
- S08: Typed pure AnimGraph input seam (R009) + hardened failure visibility (R013: cue timeout, missing graph fallback, hot reload, dead target).
- S09: Explicit producer→consumer boundary map (5 test-cited rows), VFX/skill-graph extensibility proofs, frame-time aggregator with D027 threshold math + soak wiring.

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

See `.gsd/REQUIREMENTS.md`. Active requirements: **0** (all M002 requirements are now validated). Current validated baseline: M002 first on-screen combat, M001 animation asset pipeline. All requirements R003–R016 and R021–R028 validated.

## Milestone Sequence

- [x] M001: Animation asset pipeline — Typed AnimGraph RON asset loading, validation, hot-reload, and clip↔atlas parity contract established headless-first.
- [x] M002: First on-screen combat (Agumon-only) — AnimGraph runtime player + two-clock impact sync + §9 UI core + full Agumon kit vs dummy on screen, gated by repomix architectural review.
- [ ] M003: Roster extension — Add next Digimon(s) using the M002 animation/skill seam as a pure data-only extension.

## Recommended Next Milestone

**M003: Roster extension** — add the next Digimon(s) using the M002 animation/skill seam as a pure data-only extension. The per-skill graph + stance schema is designed for this: a new Digimon is a new set of RON assets + a Rust blueprint module, with zero changes to the kernel.

Alternative next options:
- **RON VFX pipeline** (bevy_enoki / Omagari evaluation) — the opaque `ParticleId` handle seam is already open.
- **M021 trait Skill + SkillCtx abstraction** — the generalization layer for ATK-based Heal scaling, selective cleanse, mixed Heal+Cleanse, custom immunity hooks, and the full blueprint seam.
- **CLI / encounter loop** — extend the windowed runtime with a playable encounter chain.

Deferred items from M002 (carry-forward to M003+):
- Live windowed soak frame-time numbers (K001: must be captured manually; `frame-time-comparison.md` has pending results table)
- S06 architectural review findings F1–F7 (all low/medium/info; triaged roadmap in S06-ARCHITECTURAL-REVIEW.md)
- Roster extension beyond Agumon (out-of-scope for M002; per-skill graph seam designed for data-only extension)
- RON VFX format / editor (bevy_enoki / Omagari) — seam is open, implementation excluded from M002
- Full playable CLI UX and encounter chain

## Operational Notes

- `cargo test` for headless baseline; `cargo test --features windowed` for full suite including windowed_only.
- `cargo run --features windowed` for visual verification (requires display; K001 blocks in auto-mode).
- `M002-BOUNDARY-MAP.md`: 5 producer→consumer contracts with machine-checkable test citations.
- `S06-ARCHITECTURAL-REVIEW.md`: repomix architectural review findings; check for M003+ follow-up items.
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
