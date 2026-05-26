# M006: Extension-first presentation refactor + enoki-only VFX

**Gathered:** 2026-05-26
**Status:** Ready for planning

## Project Description

Rework the windowed presentation layer so adding a new Digimon needs zero engine edits, mirroring the extension-first model the combat-logic blueprint layer already follows. Retire the custom quad VFX system and make bevy_enoki the sole particle renderer. Replace the ad-hoc, Agumon-coupled flash/shake code with a data-driven `CueRegistry` of cosmetic primitives (flash/blink, target shake, camera shake, particle burst) modeled on UE5 GAS GameplayCues. Prove it by registering a second Digimon (Renamon) — combat logic and presentation — without touching a single core/engine file.

## Why This Milestone

M005 delivered working combat visual feedback, but it followed the *existing* presentation architecture: `src/windowed/render.rs` is a 2400+ line monolith with Agumon-specific identifiers hardcoded throughout. Adding a second Digimon today requires editing the engine/binary in ≥12 places (`const AGUMON_*`, the closed `on_enter_effect_ids` match, `load_agumon_enoki_vfx`, `AgumonSprite`/`AgumonPlaybackMode`, despawn filters, skill-id checks, bootstrap). This directly violates the user's extension-first principle: each Digimon should live in its own module with no engine specificity — only generic primitives in the core. The combat-logic layer (`src/combat/blueprints/<name>/` + `SkillGraphRegistry`) already works this way; the presentation layer must catch up. M005's enoki integration is also only a partial intercept (3 contact bursts) over a still-live quad system; the user wants enoki promoted to the full renderer since it strictly dominates the quad system in simulation capability (velocity, gravity, emission shapes — none of which the quad has).

## User-Visible Outcome

### When this milestone is complete, the user can:

- Run `cargo winx`, trigger all three Agumon skills, and see every VFX (charge, ember, projectile, all three impacts) render through bevy_enoki — no quad placeholder remains
- See hit flash/blink, target sprite-shake, AND camera-shake fire on impact, all driven by registered cue primitives
- See Renamon appear as a combatant with working idle/skill/hurt/death presentation and cue-driven feedback, added without any engine edit

### Entry point / environment

- Entry point: `cargo winx` (alias for `cargo run --features windowed`)
- Environment: local dev, windowed (egui + bevy render)
- Live dependencies involved: bevy_enoki (windowed-only, dep-gated); none external

## Completion Class

- Contract complete means: cue primitive math (flash/blink, sprite-shake, camera-shake param + decay) and `CueRegistry` lookup are pure and headless-tested; the dep-gating test still proves no enoki/windowed symbol leaks into the headless build
- Integration complete means: in `cargo winx` all Agumon VFX render through enoki, cue dispatch drives flash/shake/camera-shake from the registry, generic `DigimonSprite` drives stance/skill/hurt/death playback, and the engine files contain zero Agumon identifiers
- Operational complete means: Renamon registers end-to-end (combat + presentation) with zero engine edits, proven by grep/diff

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Adding Renamon touches only the two new module trees (`src/combat/blueprints/renamon/` already exists + `src/windowed/digimon/renamon/`) plus their registration call — `git diff` shows zero edits to engine/core files
- `cargo winx` shows Renamon with working idle/skill/hurt/death + cue-driven flash/shake (K001 manual sign-off)
- The custom quad VFX system is fully deleted and every Agumon effect renders through enoki — cannot be simulated; requires K001 visual confirmation that the VFX look right

## Architectural Decisions

### D042 — Two modules per Digimon (combat + windowed), not a root-level digimon/

**Decision:** Per-Digimon logic stays in `src/combat/blueprints/<name>/` (lib) and per-Digimon presentation lives in `src/windowed/digimon/<name>/` (binary crate). No root-level `src/digimon/<name>/` unifying both.

**Rationale:** User chose to keep the existing crate split rather than move the windowed presentation into the lib. Avoids the high-risk crate move; respects the established boundary.

**Alternatives Considered:**
- Single root-level `src/digimon/<name>/` with logic + presentation together — rejected: would require moving windowed presentation into the lib (cfg windowed), high risk, and the user explicitly declined.

### D043 — Retire the quad VFX system; enoki is the sole renderer

**Decision:** Delete `VfxAsset`/`VfxParticle`/`resolve_effect`/`advance_vfx_particles` and route all particle VFX through bevy_enoki from `.particle.ron`.

**Rationale:** Enoki strictly dominates the quad system (velocity, gravity, acceleration, damping, emission shapes — the quad has none). User confirmed the trade-off: lose headless intent-proxy testability of VFX curves (D037) in exchange for one real renderer. VFX correctness becomes K001 manual.

**Alternatives Considered:**
- Keep the custom system as a headless schema/intent-proxy with enoki rendering on top — rejected by user ("ritira, va bene enoki").

### D044 — Flash/blink, target-shake, and camera-shake are all CueRegistry primitives

**Decision:** A `CueRegistry` (GAS GameplayCue analogue) maps string ids to parametrized cosmetic cue primitives: `Flash{peak,ticks,curve}`, `SpriteShake{amp,freq,ticks}` (target shake), `CameraShake{amp,freq,ticks}`, `ParticleBurst{enoki_handle}`. All data-driven, fire-and-forget, decoupled from gameplay logic. Camera-shake writes the `Camera2d` transform; target-shake writes the struck sprite transform; both are just cues with different targets.

**Rationale:** User confirmed camera-shake AND target-shake AND blink should all be cues — not special cases. Matches the UE5 GAS model where camera shake is just another cue handler. Replaces the zero-flexibility hardcoded consts in `hit_feedback.rs`.

**Alternatives Considered:**
- Keep flash/shake as bespoke systems and add camera-shake as a new special case — rejected: perpetuates the ad-hoc pattern the milestone exists to remove.

### D045 — Renamon is the extension-first acceptance gate

**Decision:** Register a second Digimon (Renamon) end-to-end as the milestone's acceptance proof, requiring zero engine edits.

**Rationale:** The only real proof that hardcoding is gone is adding a new Digimon without engine edits. Renamon already has `anim_graph.ron` + `clip.ron` in `assets/digimon/renamon/`, minimizing asset authoring.

**Alternatives Considered:**
- Structural-only proof (grep for absence of consts) without a real second Digimon — rejected: weaker; doesn't exercise the full registration path.

---

> See `.gsd/DECISIONS.md` for the full append-only register of all project decisions.

## Error Handling Strategy

Asset load failures (`.particle.ron`) surface at startup via Bevy's asset error path; the enoki loader requires all 19 fields explicitly (no serde defaults) so malformed assets fail loud at load, not silently. Cue dispatch for an unregistered id should be a logged no-op (trace on `windowed.*` target), never a panic — a missing cue degrades gracefully to no visual effect. Registration collisions (two modules claiming the same id) should be surfaced at startup rather than last-writer-wins silently.

## Risks and Unknowns

- Generalizing `AgumonSprite`→`DigimonSprite` (S03) touches every windowed query referencing the Agumon-named component — high blast radius, risk of subtle playback regressions — why it matters: this is the structural core of the refactor and the windowed suite is the only guard
- Deleting the quad system (S01) removes the only headless-testable VFX proxy (D037) — why it matters: VFX correctness regressions can only be caught by K001 manual after this
- `src/windowed/` is in the binary crate, unreachable from integration tests (project gotcha) — why it matters: the "zero engine edits" proof for Renamon must be structural (grep/diff) + K001 manual, not an integration test
- Camera-shake interacting with the existing camera/canvas setup — why it matters: an absolute-offset-from-rest pattern (like sprite-shake) must be used to avoid drift

## Existing Codebase / Prior Art

- `src/windowed/render.rs` — the 2400+ line monolith holding all `AGUMON_*` consts, `on_enter_effect_ids` match, `load_agumon_enoki_vfx`, `spawn_effect_by_id`; primary refactor target
- `src/ui/hit_feedback.rs` — `flash_tint()` / `shake_offset()` with hardcoded consts; to be replaced by cue primitives
- `src/combat/blueprints/<name>/` + `SkillGraphRegistry` — the extension-first model to mirror in presentation
- `src/animation/reaction.rs` — `StanceReaction` pure mapping from M005; the accepted generic-primitive pattern (kept as-is)
- `assets/digimon/renamon/` — existing `anim_graph.ron` + `clip.ron` for the S05 acceptance gate
- `src/combat/blueprints/renamon/` — existing Renamon combat logic to wire to presentation

## Relevant Requirements

- Extension-first presentation (local constraint, no global R-id yet) — this milestone makes the windowed layer satisfy it
- R002/R005 (headless-first, dep gating) — must hold: no enoki/windowed leak into headless build, proven by the dep-gating test

## Scope

### In Scope

- Delete quad VFX system; enoki as sole particle renderer for all Agumon effects
- `CueRegistry` + pure cue primitive math (flash/blink, sprite-shake, camera-shake, particle-burst) in the lib
- Generalize `AgumonSprite`/`AgumonPlaybackMode` → data-carrying `DigimonSprite`
- Extract Agumon presentation into `src/windowed/digimon/agumon/`; remove all Agumon ids from engine
- Camera-shake as a registered cue
- Register Renamon (combat + presentation) with zero engine edits

### Out of Scope / Non-Goals

- Moving windowed presentation into the lib crate (explicitly declined — D042)
- Authoring new Renamon skill/VFX assets beyond what registration needs (reuse existing anim_graph/clip)
- New combat logic or gameplay mechanics
- Editor/GUI tooling for cues or VFX (D033 future goal, not here)

## Technical Constraints

- `src/windowed/` lives in the binary crate, unreachable from `tests/` — presentation modules are not integration-testable; "zero engine edits" proof is grep/diff + K001 manual
- bevy_enoki is windowed-only, dep-gated; the headless build must stay clean (dep-gating test)
- `.particle.ron` must list all 19 fields explicitly (no serde defaults) or RON parse fails at load
- Determinism (R004): cue math must use no wall-clock and no unseeded RNG; shake is a deterministic function of remaining ticks
- Headless-first (R002): every non-presentation system runs without `windowed`

## Integration Points

- bevy_enoki — sole particle renderer; spawned via `spawn_effect_by_id` enoki path with `OneShot::Despawn`
- Bevy `Camera2d` — camera-shake cue writes its transform (absolute offset from rest)
- `CombatEvent` / animgraph `on_enter` — cue triggers; gameplay emits, cue handlers consume (GAS pattern)

## Testing Requirements

- Headless unit/integration: cue primitive math (flash/blink, sprite-shake, camera-shake param + decay) and `CueRegistry` lookup — pure, deterministic, snapshot-stable where applicable
- Dep-gating static test: no enoki/windowed symbols in the headless build
- Full `cargo test` (headless) and `cargo test --features windowed` green; `cargo build --features windowed` green
- Structural acceptance (S05): grep/diff proving Renamon registration touched zero engine files
- K001 manual `cargo winx` sign-off: enoki VFX quality, flash/blink, target-shake, camera-shake, and Renamon presentation — auto-mode cannot run the windowed binary

## Acceptance Criteria

Per-slice criteria are encoded in the roadmap slice "After this:" lines. Milestone-level gate: Renamon registered end-to-end with zero engine edits (grep/diff) + K001 visual sign-off, with all automated suites green.

## Open Questions

- Exact cue-id naming scheme (per-skill vs per-event tags) — current thinking: string tags addressed like effect ids, resolved in S02 when the CueRegistry shape is built
- Whether camera-shake amplitude should scale with damage — current thinking: out of scope for M006; fixed per-cue params first, data-driven scaling later if desired
