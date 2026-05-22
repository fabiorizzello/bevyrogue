# S09 Research — Remediate validation evidence and operational closeout

**Slice:** S09 (M002) · `depends:[S08]` · risk:medium
**Lane:** research · **Depth:** targeted (known codebase, evidence-packaging slice with one genuinely-missing capability)

## Summary

S09 is the **operational closeout** of M002. It is mostly evidence packaging and documentation over already-built seams, with **one real implementation gap**: there is no frame-time measurement or kernel-only baseline comparison anywhere in the tree. Everything else S09 must "prove" already exists in code and S08 tests — S09's job is to *map it explicitly* and *capture it as durable artifacts*.

The "After this" contract decomposes into five deliverables:
1. **Explicit producer→consumer boundary map** (M002 roadmap stub is empty — two literal `## Boundary Map` headers, no content).
2. **Evidence for stance return** (`return_to_idle` exists; needs an asserted closeout proof + boundary entry).
3. **Evidence for skill-graph mapping** (1:1 skill-id↔AnimGraph; currently hardcoded to one id — document the seam + extensibility).
4. **Evidence for the VFX handle seam** (opaque `ParticleId` + closed `VfxLocus`/`VfxMotion`; parse/validate-only until a windowed consumer exists).
5. **Captured console output + measured frame-time baseline comparison** for the windowed soak — the only piece requiring new code.

The highest-risk / biggest-unblocker is **#5 frame-time baseline**, because it doesn't exist and the milestone's hard acceptance bar is "no anim-graph-attributable frame-time regression vs a kernel-only baseline." Everything else is documentation + thin assertion tests.

## Active Requirements This Slice Supports

- **R009 / R013** — validated in S08; S09 *packages* their proof into milestone-level closeout evidence (no re-validation needed; cross-reference S08-SUMMARY/UAT).
- **R002 / R005** — any new soak/frame-time code must stay behind `#[cfg(feature="windowed")]`; do not leak winit/wgpu/egui into non-windowed builds.
- **R004** — frame-time measurement must not introduce wall-clock/RNG into the deterministic stream; measure in the *windowed presentation* layer only (Clock::Windowed side, per MEM012/MEM025).
- **R003** — clip↔atlas parity must stay green (`tests/animation/clip_atlas_parity.rs`).
- **R006** — evidence artifacts land under `.gsd/milestones/M002/slices/S09/`; keep repo hygiene (no stray binaries; prefer text CSV/JSON + captured console log).

## Implementation Landscape

### Soak / validation harness (EXISTS — extend, don't rebuild)
- `src/windowed/mod.rs:42-162` — `WindowedValidationConfig { soak_secs }`, `WindowedValidationState`, `config_from_env()`.
  - Env vars: `BEVYROGUE_VALIDATION_WINDOWED` (enable), `BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS` (duration).
- `src/windowed/mod.rs:242-292` — `windowed_validation_tick(world)`: logs `validation_windowed:start`, a **one-time** `validation_snapshot`, then `validation_windowed:finish` and `AppExit::Success` after `soak_secs`.
- **Gap:** the snapshot is captured **once**, not per-frame. No frame-time delta, no min/max/p95, no baseline. The finish path is the natural seam to also emit aggregated frame-time stats.
- Hot-reload today is a **manual egui button** (`src/windowed/mod.rs:308-321`, `asset_server.reload(...)` over unit/skill paths) — not auto-exercised by the soak. S08 already proved hot-reload-next-spawn behavior in tests (`anim_registry_failure_visibility`), so S09 can cite that rather than wire reload into the soak loop unless the planner wants a live soak proof.

### Frame-time baseline (DOES NOT EXIST — new code)
- No `fps`/`frame.*time`/`baseline` measurement found. Only `cue_barrier.rs` frame *counts* (animation budget, not wall-time).
- Needs: per-frame `Time::delta_secs()` accumulation during the soak, a **kernel-only baseline** run (soak with anim-graph/render path disabled or a headless tick loop) vs **full windowed** run, and a comparison emitted as an artifact. Decide the regression threshold (context says "no anim-graph-attributable regression" — pick a concrete %/ms bar in planning).
- **observability skill** applies directly: emit *structured* frame-time stats (count, mean, p95, max) as a parseable log line (mirror the existing `validation_snapshot:` prefix convention) so the artifact is machine-checkable, not eyeballed. Persist to disk as the durable artifact.

### Stance return (EXISTS — assert + map)
- `src/windowed/render.rs:77-88` — `return_to_idle(graph, preserve_missing_skill_graph_cue)` resets player to `graph.graph().entry`, mode→`Idle`, clears `last_release_frame`.
- Call site `src/windowed/render.rs:267` in `animate_agumon_sprite()` on skill-playback exit.
- Stance entry source: `src/animation/registry.rs:195` `DEFAULT_ANIM_STANCE_PATHS = ["digimon/agumon/stance.ron"]`.

### Skill-graph mapping (EXISTS but hardcoded — document seam)
- `src/windowed/render.rs:305-310` — `resolve_snapshot_or_instant_fallback(&AnimGraphId(AGUMON_SKILL_GRAPH_ID), ...)`; all skills currently resolve to one hardcoded `"agumon_skill"` id (`src/windowed/mod.rs:36-40`).
- Registry: `src/animation/registry.rs:86` `SkillGraphRegistry(HashMap<AnimGraphId, Handle<AnimGraph>>)`, `:129` `StanceGraphRegistry`.
- Decision D004 (per-skill 1:1 graph). S09 should document that the *registry* already supports many ids and the only constraint to lift for M003+ is the hardcoded constant — i.e. extensibility is a data/lookup change, not a rewrite. (design-an-interface: this is the seam to describe — the deep module is `SkillGraphRegistry`; the shallow leak is the hardcoded id at the call site.)

### VFX handle seam (EXISTS — document)
- `src/animation/anim_graph.rs:184-187` — `SpawnParticle { name: ParticleId, origin: VfxLocus, motion: VfxMotion }` (opaque newtype + closed enums at `:344-351`).
- Baby Burner reactive flash: `src/combat/blueprints/agumon/baby_burner.rs:11-63` (`DETONATE_SIGNAL_NAME = "baby_burner_detonate"`); consumer `src/ui/combat_panel/mod.rs:164-221`.
- Boundary fact to record: gameplay numbers live only in `skills.ron`; the anim graph emits opaque presentation ids; rendering is no-op until a windowed consumer exists (`src/combat/preview.rs`). This is the producer/consumer split the boundary map must state.

### Existing tests to cite as evidence (no re-run obligation, but list for the planner)
- S08 proof: `tests/animation/anim_graph_input_purity.rs` (R009), `tests/timeline/r013_failure_visibility.rs` + `tests/animation/anim_registry_failure_visibility.rs` (R013).
- Boundary contracts already encoded: `tests/timeline/boundary_contract.rs` (kernel↔timeline, no stringly/no gameplay numbers in anim graphs), `tests/windowed_only/phase_strip_readonly.rs` + `tests/preview_ai/presentation_metadata_boundary.rs` (UI query-only).
- Windowed sweep baseline: `cargo test --features windowed --test animation --test timeline --test windowed_only` (118 passing at S08 close).

## Natural Seams (for the planner)

1. **Producer→consumer boundary map** — author the M002 boundary map (fill the empty `M002-ROADMAP.md` stub *and/or* a dedicated S09 artifact). Columns: producer subsystem → contract/data type → consumer subsystem, covering: kernel(skills.ron gameplay numbers) → timeline → anim-graph(opaque cmds); anim player → cue barrier → kernel resume; CombatEvent → §9 UI/HUD (read-only); registry(skill-id) → player; VFX opaque id → windowed consumer. Cite the test that *enforces* each boundary. (write-docs: reader-test it for someone without M002 context.)
2. **Stance return + skill-graph mapping evidence** — thin windowed_only test(s) asserting `return_to_idle` lands on stance entry after a skill, and that `SkillGraphRegistry` resolves a non-default id (extensibility proof for the 1:1 mapping). Document the hardcoded-constant constraint explicitly.
3. **VFX handle seam evidence** — serialization/lookup test for `SpawnParticle`/`ParticleId` + a boundary-map note that VFX is opaque-id, validate-only until a windowed consumer.
4. **Frame-time baseline + console capture** (highest risk) — new windowed code to accumulate per-frame delta stats during the soak, run kernel-only baseline vs full windowed, emit structured stats line, and persist captured console output + frame-time comparison as an artifact under `.gsd/milestones/M002/slices/S09/`. Define the concrete regression threshold during task planning.
5. **Closeout evidence bundle** — assemble S08 R009/R013 proof references, the boundary map, captured soak console output, and the frame-time comparison into the S09 closeout artifact set; confirm repomix architectural review report status (gate from milestone context).

## First Proof / Build Order

Do **#4 frame-time baseline first** — it's the only unbuilt capability and the milestone's hard FPS bar depends on it; if anim-graph-attributable regression shows up, it changes the closeout verdict. #1 boundary map can proceed in parallel (pure documentation). #2/#3 are thin tests over existing code. #5 is the final assembly and depends on the others.

## Verification

- Boundary map: reader-test (write-docs) — a fresh reader can trace each gameplay number / presentation command to its single producer and its consumers; every claimed boundary names an enforcing test.
- Stance/mapping/VFX: `cargo test --features windowed --test windowed_only` (+ `--test animation`) green, including new assertions.
- Frame-time: artifact present under `.gsd/milestones/M002/slices/S09/` with kernel-only vs windowed numbers and an explicit pass/fail against the chosen threshold; structured log line parseable; zero panics in the soak.
- Regression guard: `cargo test --features windowed --test animation --test timeline --test windowed_only` stays green (118+ baseline); `tests/animation/clip_atlas_parity.rs` green (R003).

## Don't Hand-Roll

- Don't add a new soak harness — extend `windowed_validation_tick` / `WindowedValidationState`.
- Don't invent a new registry — `SkillGraphRegistry` already keys by `AnimGraphId`; only the hardcoded call-site constant limits it.
- Don't re-validate R009/R013 — cite S08-SUMMARY/UAT and existing passing tests.

## Skills Discovered

No new external skills installed. Relevant principles applied: **observability** (structured, parseable frame-time stats + persisted failure/evidence artifacts over eyeballing), **design-an-interface** (frame the skill-graph mapping as a deep registry with a shallow hardcoded leak to lift), **write-docs** (boundary map must be reader-testable by someone without M002 context).
