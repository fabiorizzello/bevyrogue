# M015: M013 Closure and Combat Architecture Coherence

**Gathered:** 2026-05-08
**Status:** Ready for planning

## Project Description

M015 completes what M013 left partial and verifies that the combat engine is coherent rather than patched together from multiple implementation passes. The milestone must classify failed/obsolete tests, repair or prune them by evidence, prove the CLI through shared combat surfaces, close relevant M013 validation/artifact gaps, and audit whether gameplay authority has a single clear path.

The user’s core concern is not merely red tests. The concern is that code written in several “passate” may now be rattoppato, scollegato, or using multiple mixed logic models. M015 must therefore prove or repair the architecture around the agreed direction:

```text
RON data + custom signals
  → per-Digimon blueprint-like Rust module
  → hooks into generic combat kernel
  → canonical state / transition / event / snapshot
  → CLI / tests / future UI
```

## Why This Milestone

M013 introduced important combat-kernel, line-loop, beat, and CLI prior art, but its closure state is contradictory. `.gsd/STATE.md` reports M013 as complete, while `.gsd/milestones/M013/M013-VALIDATION.md` says `needs-attention`. Current verification is blocked before compile because `Cargo.toml` declares missing `tests/battery_loop_resolution.rs`. Some tests may be obsolete, but that must be proven, not assumed.

M015 exists to create a truthful baseline before future combat or UI work builds on this foundation.

## User-Visible Outcome

### When this milestone is complete, the user can:

- run the relevant combat test and CLI verification flow knowing stale M013 blockers have been classified and resolved;
- inspect a concrete combat architecture map showing where RON, custom signals, per-Digimon blueprint logic, kernel authority, presentation beats, snapshots, and CLI consumers live;
- trust that clear mixed-pattern drift has either been normalized or explicitly split into a follow-up with evidence;
- use the CLI as proof of shared combat/query/event/beat/snapshot surfaces rather than a separate CLI combat implementation.

### Entry point / environment

- Entry point: `cargo test --no-run`, `cargo test --no-fail-fast`, `cargo run --bin combat_cli`
- Environment: local dev / headless-first Rust + Bevy 0.18 runtime
- Live dependencies involved: none external; internal Rust code, Bevy schedules/messages, RON assets, CLI, tests, and GSD artifacts

## Completion Class

- Contract complete means: stale test declarations are classified, current contracts are mechanically checked, and responsibilities are documented/protected around RON, per-Digimon blueprint logic, kernel authority, presentation, and CLI/query/snapshot consumers.
- Integration complete means: CLI proof, kernel state/transitions/events, action query, beat metadata, validation snapshots, and tests exercise the same shared combat surfaces.
- Operational complete means: current test baseline is green or every remaining failure is explicitly classified with evidence and owner; M013/M015 closure artifacts truthfully describe what was proven.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- a stale/missing test blocker such as `battery_loop_resolution` cannot hide the true suite state;
- clear architecture drift is either corrected toward `RON custom signals → per-Digimon blueprint module → kernel hooks → canonical state/events` or split into a named follow-up if rewrite-scale;
- presentation beat/trigger metadata is non-authoritative and cannot decide gameplay outcomes;
- CLI proof consumes the shared action query, event, beat, kernel-observable state, and snapshot surfaces;
- closure artifacts no longer claim completion while validation evidence still says needs-attention;
- what cannot be simulated away is the shared-surface proof: CLI, tests, and snapshots must read the same combat authority path future UI will consume.

## Architectural Decisions

### Normalize clear drift, not unlimited rewrite

**Decision:** M015 should normalize clear architecture drift found in combat code, tests, CLI, RON metadata, and artifacts, but it should not silently become an unlimited combat-engine rewrite.

**Rationale:** The user chose “Normalize clear drift.” The milestone must fix real mixed-pattern contradictions, not only compile blockers, while keeping rewrite-scale work explicit and bounded.

**Alternatives Considered:**
- Only blockers — rejected because green tests could still leave a patched, disconnected engine.
- Full normalization of all combat architecture — rejected as default because it could expand M015 beyond closure into broad rewrite.

---

### Blueprint-like logic is per Digimon

**Decision:** The desired blueprint-like extension surface is per Digimon, not primarily per line/mechanic. Mechanic modules such as `twin_core`, `holy_support`, `battery_loop`, `predator_loop`, and `precision_mind_game` may remain as shared primitives, but long-term identity belongs to per-Digimon blueprint-like Rust modules/logical ownership.

**Rationale:** The user clarified: “per digimon non per linea/meccanica.” The goal is unique Digimon behavior; line/mechanic modules alone risk producing reused loops with different skins.

**Alternatives Considered:**
- Per-line/mechanic modules as the primary abstraction — useful transitional prior art, but not enough for unique Digimon identity.
- RON script-like DSL — rejected because RON would become a hidden gameplay engine.
- Central per-Digimon branching in the core kernel — rejected because it breaks Open/Closed and creates patchwork combat logic.

---

### RON emits data and custom signals, not final gameplay authority

**Decision:** RON owns numbers, skill metadata, target metadata, animation/presentation trigger metadata, and custom signal/intent declarations. It does not decide final gameplay outcomes for unique Digimon behavior.

**Rationale:** RON should stay data-first and content-first. Unique behavior belongs in typed Rust blueprint modules that can inspect current state and accept/reject/pay off requests deterministically.

**Alternatives Considered:**
- RON owns a richer gameplay scripting DSL — rejected because it creates a second source of truth.
- Hardcode all kit behavior in Rust without RON signal/data inputs — rejected because content iteration and validation would suffer.

---

### Combat kernel remains generic authority

**Decision:** The combat kernel owns canonical gameplay timing, state, transitions, and queryable output. It must remain generic and must not centrally encode `Agumon does X` or equivalent per-Digimon branches.

**Rationale:** The kernel is the wheel. Per-Digimon blueprint modules extend it through hooks/systems; they do not rewrite combat or fork authority.

**Alternatives Considered:**
- Core match ladder per Digimon — rejected because it creates brittle central branching.
- Each Digimon implements a mini combat engine — rejected because consumers would lose one shared source of truth.

---

### Kernel transitions are canonical observable output, not blueprint source

**Decision:** `CombatKernelTransition` is a typed observable/mutation contract after blueprint-like logic resolves what happened. It is useful for audit, tests, snapshots, CLI, and future UI, but it is not where unique Digimon behavior begins.

**Rationale:** The correct flow is `RON data + custom signals → per-Digimon blueprint module → hook into generic combat kernel → canonical state / transition / event / snapshot`. This avoids mistaking a transition enum for the unique behavior layer.

**Alternatives Considered:**
- RON directly emits final kernel transitions — rejected because it bypasses unique Digimon logic.
- Blueprint logic exists only after kernel transitions — rejected because it puts behavior in the wrong place.

---

### Presentation beats are non-authoritative cues

**Decision:** Animation notifies, beat metadata, and presentation triggers can route timing to cue-like presentation output, but they cannot decide damage, state transitions, or gameplay outcomes.

**Rationale:** User-provided UE5/GAS research supports gameplay authority in abilities/effects and notifies/cues for presentation feedback. In bevyrogue, that maps to combat authority in the kernel/blueprint logic and non-authoritative presentation beats.

**Alternatives Considered:**
- Animation-authoritative gameplay timing — rejected because it makes assets a second combat engine.
- Hybrid presentation/gameplay authority — rejected because it creates unclear ownership and test fragility.

## Error Handling Strategy

- Stale or missing test targets are classified before fix/delete: stale manifest, obsolete test, missing implementation, accidentally deleted relevant coverage, or unrelated pre-existing failure.
- Red tests are separated into compile/config blockers and runtime/gameplay failures.
- Obsolete tests are removed only when replacement coverage is named.
- Relevant tests are updated against current architecture instead of adding compatibility shims for obsolete APIs.
- Invalid RON custom signals or metadata fail validation with stable reason/path.
- Unknown or unsupported custom signals are exposed as disabled/deferred where applicable, not silently ignored.
- Architecture drift with duplicated authority or disconnected logic is normalized if clear and local.
- Rewrite-scale drift is documented with exact evidence and split into a follow-up milestone instead of silently expanding M015.
- CLI-only combat logic is a blocker.
- Missing GSD summaries/validation packaging are closure gaps, not gameplay regressions; M015 repairs or supersedes them when needed for truthful downstream state.

## Risks and Unknowns

- The full test failure set is unknown because verification is currently blocked by the missing `battery_loop_resolution` target.
- Some red tests may encode obsolete M013 expectations; fixing them blindly could preserve an old model.
- The combat code may contain mixed authority paths from multiple implementation passes; audit findings may require normalization beyond one file.
- A complete per-Digimon blueprint migration may be larger than M015; M015 must seed/normalize direction without exploding scope.
- CLI may appear functional while bypassing shared surfaces; that would prove the wrong architecture.
- Presentation metadata may be wired closely enough to gameplay that tests/docs must explicitly enforce non-authority.

## Existing Codebase / Prior Art

- `.gsd/milestones/M013/M013-CONTEXT.md` — prior M013 closure framing and decisions.
- `.gsd/milestones/M013/M013-VALIDATION.md` — current needs-attention validation evidence and closure gaps.
- `.gsd/STATE.md` — currently reports M013 as all slices complete, contradicting needs-attention validation.
- `Cargo.toml` — declares missing `tests/battery_loop_resolution.rs`, blocking `cargo test --no-run`.
- `src/combat/kernel.rs` — typed combat kernel state, transitions, registry/hook concepts, and canonical beat IDs.
- `src/combat/twin_core.rs`, `holy_support.rs`, `battery_loop.rs`, `predator_loop.rs`, `precision_mind_game.rs` — current mechanic/line-oriented prior art and likely shared primitive candidates.
- `src/data/skills_ron.rs` — RON skill DSL and metadata; must remain data/custom-signal layer, not hidden gameplay engine.
- `assets/data/skills.ron`, `assets/data/units.ron` — current content declarations and metadata.
- `src/combat/action_query.rs` — shared action legality/affordance query surface.
- `src/combat/observability.rs` — validation snapshot surfaces used by tests/CLI/future UI.
- `src/bin/combat_cli.rs` — CLI proof target and risk area for CLI-only logic.
- `tests/*kernel*.rs`, `tests/*resolution*.rs`, `tests/*affordance*.rs`, `tests/validation_snapshot.rs`, `tests/combat_coherence.rs` — current verification surface and likely audit inputs.

## Relevant Requirements

- R089 — M015 must make M013/M015 closure truthful.
- R090 — M015 must classify failing/stale tests before fixing/removing.
- R091 — M015 must leave a green or fully explained regression baseline.
- R092 — M015 must audit combat single-source-of-truth responsibilities.
- R093 — M015 must normalize clear drift.
- R094 — M015 must establish per-Digimon blueprint architecture direction.
- R095 — M015 must keep RON as data/custom-signal layer.
- R096 — M015 must keep combat kernel authority generic.
- R097 — M015 must enforce non-authoritative animation/presentation triggers.
- R098 — M015 must prove CLI uses shared surfaces.
- R099 — M015 must repair or supersede validation/artifact gaps.
- R100 — M015 must keep verification deterministic and headless-first.

## Scope

### In Scope

- Classify and resolve stale/missing test declarations and red tests.
- Run and classify `cargo test --no-run` and `cargo test --no-fail-fast` once blockers allow.
- Audit gameplay authority and mixed-pattern drift across combat modules, RON, CLI, tests, snapshots, and artifacts.
- Normalize clear drift toward `RON custom signals → per-Digimon blueprint module → kernel hooks → canonical state/events/snapshots`.
- Seed or establish a concrete per-Digimon blueprint seam if needed to prevent future drift.
- Prove presentation beat/trigger metadata is non-authoritative.
- Prove CLI uses shared action query, event, beat, kernel-observable state, and snapshot surfaces.
- Repair or supersede M013 closure artifacts where needed for truthful state.

### Out of Scope / Non-Goals

- Full migration of every Digimon into explicit per-Digimon modules if that is rewrite-scale.
- Full visual UI or animation/VFX presentation implementation.
- Animation metadata as gameplay authority.
- Preserving obsolete tests for stale APIs.
- DNA Chips, roguelite run graph, or broader meta-loop work.
- Unbounded combat-engine rewrite hidden inside a closure milestone.

## Technical Constraints

- Headless-first remains mandatory.
- Tests must be deterministic; no wall-clock or unseeded RNG assumptions.
- Bevy messages/events require schedule advancement before snapshot assertions.
- No winit/wgpu/egui dependencies outside the `windowed` feature.
- Do not touch `Cargo.lock` manually.
- Do not read `.claude/skills/digimon/data/digimon.json` directly; use the local Digimon query CLI if Digimon factual data is needed.
- Shared combat systems must stay branch-light and generic.
- Per-Digimon behavior must extend through typed hooks/systems, not central core branching.

## Integration Points

- `cargo test --no-run` — first blocker classification and compile target sanity.
- `cargo test --no-fail-fast` — broad regression discovery.
- `cargo run --bin combat_cli` — local CLI proof entry point.
- `src/combat/kernel.rs` — generic combat authority and transition contract.
- `src/data/skills_ron.rs` / `assets/data/skills.ron` — RON content/custom signal boundary.
- `src/combat/action_query.rs` — shared legality/affordance surface.
- `src/combat/observability.rs` — snapshot proof surface.
- `src/bin/combat_cli.rs` — shared-surface consumer proof.
- `.gsd/milestones/M013/M013-VALIDATION.md` — prior closure gap evidence.

## Testing Requirements

- Start with `cargo test --no-run`; classify and resolve stale target/config blockers.
- Run `cargo test --no-fail-fast` after initial blockers are removed; classify every failure before fixing.
- Add or repair tests that mechanically protect the responsibility split: RON/custom signals, per-Digimon blueprint seam, kernel authority, presentation non-authority, shared CLI surface.
- Prefer deterministic headless tests over manual CLI inspection.
- CLI e2e/smoke proof should complement kernel/query/snapshot tests, not replace them.
- Tests that enqueue Bevy messages must call `app.update()` or equivalent wrapper before observing state.
- Obsolete tests may be removed only with replacement coverage named.

## Acceptance Criteria

- S01: stale/missing test declarations and M013 artifact gaps are inventoried and classified with evidence.
- S02: a concrete source-of-truth audit maps current gameplay authority and mixed-pattern risks.
- S03: clear drift around RON/custom signals, per-Digimon blueprint logic, kernel hooks, and canonical output is normalized or split with evidence.
- S04: tests/docs prove presentation beat/trigger metadata remains non-authoritative and RON remains data/custom-signal input.
- S05: CLI proof consumes shared action query, event, beat, kernel-observable state, and snapshot surfaces without CLI-only combat logic.
- S06: final regression baseline and GSD closure artifacts truthfully state what passed, failed, was fixed, was deferred, and what future migration remains.

## Open Questions

- How broad is the current test failure set after `battery_loop_resolution` is resolved? — Unknown until S01 runs verification.
- Which current mechanic/line modules can remain as shared primitives versus needing per-Digimon ownership now? — To be determined in S02/S03 audit.
- Does M015 need to create one explicit per-Digimon module as a seed seam, or is a documented boundary plus tests enough? — Decide based on S02 findings.
- Are M013 artifact gaps best repaired in-place or superseded by M015 closure summaries? — Decide in S06 based on GSD state needs.
