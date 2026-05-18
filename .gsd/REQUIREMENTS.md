# Requirements

This file is the explicit capability and coverage contract for the project.

## Active

### R001 — The animation system must live behind one cohesive generic module boundary covering schema, loading, validation, orchestration, and future runtime/player behavior without hardcoded Digimon-specific logic.
- Class: quality-attribute
- Status: active
- Description: The animation system must live behind one cohesive generic module boundary covering schema, loading, validation, orchestration, and future runtime/player behavior without hardcoded Digimon-specific logic.
- Why it matters: A clean module seam prevents early asset-pipeline decisions from coupling the future animation runtime to individual Digimon or one-off content assumptions.
- Source: user
- Primary owning slice: M001/S01
- Validation: mapped
- Notes: User explicitly requested one animation FSM module and a strict separation between motore and specificità Digimon.

### R002 — The project must load `anim_graph.ron` as a typed Bevy asset through the animation module using closed schema types for graph nodes, edges, predicates, commands, parameter references, and target shapes.
- Class: core-capability
- Status: active
- Description: The project must load `anim_graph.ron` as a typed Bevy asset through the animation module using closed schema types for graph nodes, edges, predicates, commands, parameter references, and target shapes.
- Why it matters: Later visual runtime work depends on a validated animation graph contract rather than ad hoc stringly-typed asset data.
- Source: user
- Primary owning slice: M001/S01
- Validation: mapped
- Notes: Seeded from M022 S01, adapted so the owner module is animation rather than a Digimon-specific or blueprint-specific seam.

### R003 — The project must load `clip.ron` as a typed Bevy asset and prove geometry parity against the existing atlas source data for Agumon.
- Class: core-capability
- Status: active
- Description: The project must load `clip.ron` as a typed Bevy asset and prove geometry parity against the existing atlas source data for Agumon.
- Why it matters: Frame geometry must be trustworthy before animation graphs and future runtime playback depend on it.
- Source: user
- Primary owning slice: M001/S02
- Validation: mapped
- Notes: Seeded from M022 S02. Agumon remains the first full proof path, but the schema must not be Agumon-specific.

### R004 — Invalid animation assets at boot must fail fast with typed diagnostics instead of being deferred to runtime behavior.
- Class: continuity
- Status: active
- Description: Invalid animation assets at boot must fail fast with typed diagnostics instead of being deferred to runtime behavior.
- Why it matters: Bad animation data should be caught when assets load so later runtime and rendering work is not debugging schema mistakes indirectly.
- Source: user
- Primary owning slice: M001/S03
- Validation: mapped
- Notes: Boot-time failures should name the offending file/check where practical. Advisory checks may remain warnings.

### R005 — Cross-asset animation validation must prove references against real project data through explicit adapter seams rather than hard-coupling the animation core to Digimon or gameplay internals.
- Class: integration
- Status: active
- Description: Cross-asset animation validation must prove references against real project data through explicit adapter seams rather than hard-coupling the animation core to Digimon or gameplay internals.
- Why it matters: The milestone needs real validation strength without turning the generic animation engine into a dependency sink for all game-specific data.
- Source: user
- Primary owning slice: M001/S03
- Validation: mapped
- Notes: Examples include parameter references, clip names, and other catalogs derived from existing data. The validator should stay quasi-pure while adapters supply project-specific catalogs.

### R006 — M001 completion must include a real manual `cargo run --features windowed` hot-reload proof showing edited animation assets reload without crash or corrupted world state.
- Class: launchability
- Status: active
- Description: M001 completion must include a real manual `cargo run --features windowed` hot-reload proof showing edited animation assets reload without crash or corrupted world state.
- Why it matters: Hot reload is an operational authoring capability and cannot be fully proven by headless contract tests alone.
- Source: user
- Primary owning slice: M001/S04
- Validation: mapped
- Notes: The user confirmed the manual demo is required, not optional. Invalid hot reload should keep the last valid asset and log clearly.

### R007 — The architecture must support the non-Agumon roster from the start through the same generic validation/loading path, rather than treating non-Agumon as careless one-off stubs.
- Class: quality-attribute
- Status: active
- Description: The architecture must support the non-Agumon roster from the start through the same generic validation/loading path, rather than treating non-Agumon as careless one-off stubs.
- Why it matters: A pipeline that only works for Agumon risks baking content-specific assumptions into the engine at the exact point the module boundary is being established.
- Source: user
- Primary owning slice: M001/S04
- Validation: mapped
- Notes: The user asked to be stronger for non-Agumon to get good architecture from the beginning.

### R008 — Animation asset loading and validation must remain headless-first, with `windowed` used only for the live hot-reload demo and any UI-dependent behavior.
- Class: constraint
- Status: active
- Description: Animation asset loading and validation must remain headless-first, with `windowed` used only for the live hot-reload demo and any UI-dependent behavior.
- Why it matters: The project relies on deterministic, agent-friendly command verification and must not make asset validation depend on a graphical runtime.
- Source: inferred
- Primary owning slice: M001/S03
- Supporting slices: M001/S01, M001/S02, M001/S04
- Validation: mapped
- Notes: Carries forward project rules: no winit, wgpu, or egui dependency outside the `windowed` feature gate.

## Validated

### R013 — Recovered historical baseline: preserve the validated M015 Combat Authority Closure contract covering deterministic headless verification, canonical combat authority boundaries, shared combat surfaces, and truthful supersession of incomplete M013 closure evidence.
- Class: core-capability
- Status: validated
- Description: Recovered historical baseline: preserve the validated M015 Combat Authority Closure contract covering deterministic headless verification, canonical combat authority boundaries, shared combat surfaces, and truthful supersession of incomplete M013 closure evidence.
- Why it matters: This baseline defines the last validated pre-M001 combat contract and should remain queryable in the DB rather than only in a recovered markdown snapshot.
- Source: git-history 00c0812
- Validation: Validated baseline from recovered historical requirements snapshot; evidence in `.gsd/milestones/M015/M015-VALIDATION.md`, `docs/combat_current.md`, and listed M015 contract docs.
- Notes: Recovered from historical Git requirements. Corresponds to the validated baseline section naming R086, R088, R089-R100.

## Deferred

### R009 — Runtime animation player and FSM execution are intentionally deferred until after the typed asset and validation contract is proven.
- Class: core-capability
- Status: deferred
- Description: Runtime animation player and FSM execution are intentionally deferred until after the typed asset and validation contract is proven.
- Why it matters: Building runtime playback before the asset contract is proven would blur milestone risk and make schema issues harder to isolate.
- Source: user
- Primary owning slice: none
- Validation: unmapped
- Notes: This preserves the M022 boundary: no `tick_fsm`, runtime player, or playback system in M001 unless required for validation seams.

### R010 — Command to gameplay runtime translation from animation graph commands into gameplay/kernel effects is deferred until the runtime/player milestone.
- Class: integration
- Status: deferred
- Description: Command to gameplay runtime translation from animation graph commands into gameplay/kernel effects is deferred until the runtime/player milestone.
- Why it matters: This avoids coupling the schema and validator milestone to runtime behavior that belongs to the next stage.
- Source: inferred
- Primary owning slice: none
- Validation: unmapped
- Notes: M001 may define command schema and validate references, but should not implement live gameplay translation as part of the asset-pipeline foundation.

### R014 — Recovered deferred work: complete per-Digimon blueprint migration for the whole roster.
- Class: differentiator
- Status: deferred
- Description: Recovered deferred work: complete per-Digimon blueprint migration for the whole roster.
- Why it matters: The historical contract explicitly tracked full roster blueprint migration as future work beyond the validated combat baseline.
- Source: git-history 00c0812
- Validation: Deferred historical work item; no current proof required.
- Notes: Recovered from the historical Git requirements snapshot.

### R015 — Recovered deferred work: complete revised 12-Digimon roster behavior and balance validation.
- Class: quality-attribute
- Status: deferred
- Description: Recovered deferred work: complete revised 12-Digimon roster behavior and balance validation.
- Why it matters: Historical planning kept full roster behavior and balance validation out of the validated baseline while preserving it as explicit future work.
- Source: git-history 00c0812
- Validation: Deferred historical work item; no current proof required.
- Notes: Recovered from the historical Git requirements snapshot.

### R016 — Recovered deferred work: deliver a full playable CLI UX and windowed presentation pipeline consuming canonical combat surfaces.
- Class: launchability
- Status: deferred
- Description: Recovered deferred work: deliver a full playable CLI UX and windowed presentation pipeline consuming canonical combat surfaces.
- Why it matters: The historical contract distinguished validated combat authority from later playable UX and presentation delivery.
- Source: git-history 00c0812
- Validation: Deferred historical work item; no current proof required.
- Notes: Recovered from the historical Git requirements snapshot.

### R017 — Recovered deferred work: integrate Roguelite Fatigue and the run-loop into the combat stack.
- Class: primary-user-loop
- Status: deferred
- Description: Recovered deferred work: integrate Roguelite Fatigue and the run-loop into the combat stack.
- Why it matters: The historical requirements snapshot preserved run-loop integration as a future capability beyond the validated baseline.
- Source: git-history 00c0812
- Validation: Deferred historical work item; no current proof required.
- Notes: Recovered from the historical Git requirements snapshot.

### R018 — Recovered deferred work: complete boss conversion and hard-control policy integration.
- Class: integration
- Status: deferred
- Description: Recovered deferred work: complete boss conversion and hard-control policy integration.
- Why it matters: The historical contract kept boss conversion and hard-control policy as explicit pending integration work.
- Source: git-history 00c0812
- Validation: Deferred historical work item; no current proof required.
- Notes: Recovered from the historical Git requirements snapshot.

### R019 — Recovered deferred work: complete the Heavy taxonomy.
- Class: core-capability
- Status: deferred
- Description: Recovered deferred work: complete the Heavy taxonomy.
- Why it matters: The historical requirements snapshot tracked Heavy taxonomy completion as unresolved future capability work.
- Source: git-history 00c0812
- Validation: Deferred historical work item; no current proof required.
- Notes: Recovered from the historical Git requirements snapshot.

## Out of Scope

### R011 — Digimon-specific behavior must not be implemented inside the core animation engine module.
- Class: anti-feature
- Status: out-of-scope
- Description: Digimon-specific behavior must not be implemented inside the core animation engine module.
- Why it matters: This prevents the animation motore from becoming Agumon-specific or roster-specific at inception.
- Source: user
- Primary owning slice: none
- Validation: n/a
- Notes: Specificity belongs in RON assets, catalogs, data adapters, or later owner-specific content layers, not in generic engine code.

### R012 — Production-complete authored animation content for the full roster is out of scope for M001.
- Class: constraint
- Status: out-of-scope
- Description: Production-complete authored animation content for the full roster is out of scope for M001.
- Why it matters: The milestone is about the asset pipeline and module seam, not finishing all roster animation content.
- Source: inferred
- Primary owning slice: none
- Validation: n/a
- Notes: M001 should make non-Agumon assets validate through the same architecture, but full bespoke animation authoring for every Digimon is later work.

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R001 | quality-attribute | active | M001/S01 | none | mapped |
| R002 | core-capability | active | M001/S01 | none | mapped |
| R003 | core-capability | active | M001/S02 | none | mapped |
| R004 | continuity | active | M001/S03 | none | mapped |
| R005 | integration | active | M001/S03 | none | mapped |
| R006 | launchability | active | M001/S04 | none | mapped |
| R007 | quality-attribute | active | M001/S04 | none | mapped |
| R008 | constraint | active | M001/S03 | M001/S01, M001/S02, M001/S04 | mapped |
| R009 | core-capability | deferred | none | none | unmapped |
| R010 | integration | deferred | none | none | unmapped |
| R011 | anti-feature | out-of-scope | none | none | n/a |
| R012 | constraint | out-of-scope | none | none | n/a |
| R013 | core-capability | validated | none | none | Validated baseline from recovered historical requirements snapshot; evidence in `.gsd/milestones/M015/M015-VALIDATION.md`, `docs/combat_current.md`, and listed M015 contract docs. |
| R014 | differentiator | deferred | none | none | Deferred historical work item; no current proof required. |
| R015 | quality-attribute | deferred | none | none | Deferred historical work item; no current proof required. |
| R016 | launchability | deferred | none | none | Deferred historical work item; no current proof required. |
| R017 | primary-user-loop | deferred | none | none | Deferred historical work item; no current proof required. |
| R018 | integration | deferred | none | none | Deferred historical work item; no current proof required. |
| R019 | core-capability | deferred | none | none | Deferred historical work item; no current proof required. |

## Coverage Summary

- Active requirements: 8
- Mapped to slices: 8
- Validated: 1 (R013)
- Unmapped active requirements: 0
