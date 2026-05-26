# Scope — windowed/render too big, refactor research to split it in multiple files

## Question

What is the safest, lowest-churn way to split the oversized windowed presentation code into smaller files without changing behavior or weakening the existing source-contract boundaries?

Primary focus:
- `src/windowed/render.rs` — 2344 lines
- `src/windowed/mod.rs` — 512 lines

Secondary context:
- existing per-Digimon registration seam in `src/windowed/digimon/`
- existing source-contract tests in `tests/windowed_only/`
- `windowed` feature-gated runtime and manual-only visual verification flow

## What a good answer must include

A useful recommendation should answer:

1. **Where to split**
   - Proposed module/file boundaries for `src/windowed/render.rs`
   - Proposed module/file boundaries for `src/windowed/mod.rs`
   - Which responsibilities should stay centralized vs move out

2. **What constraints must not be broken**
   - Keep engine files species-agnostic (`Agumon`/`Renamon` data stays under `src/windowed/digimon/`)
   - Preserve `#[cfg(feature = "windowed")]` boundaries and headless build behavior
   - Preserve the current test strategy, especially source-contract tests for binary-crate-only code
   - Avoid plans that require launching the real windowed app in auto-mode (`K001`)

3. **How to execute the refactor safely**
   - Incremental migration order
   - Likely tests/contracts to run after each extraction step
   - Main risks: circular dependencies, over-fragmentation, hidden shared types/resources, increased churn in tests/imports

4. **Decision format**
   - Recommended structure
   - Viable alternative
   - Reasons to choose one over the other

## Success criteria

The spike succeeds if it produces:

- a clear decomposition strategy for the windowed presentation layer
- a comparison of at least 2 viable split approaches
- a concrete recommendation with rationale
- a step-by-step migration sequence that can be executed later as production work
- explicit verification guidance for preserving current behavior/contracts

## Constraints and current evidence

- `src/windowed/render.rs` is the dominant hotspot at **2344 lines**.
- `src/windowed/mod.rs` is also oversized at **512 lines**.
- Existing tests already pin important seams:
  - `tests/windowed_only/agumon_module_extraction.rs`
  - `tests/windowed_only/renamon_extension_contract.rs`
- `docs/agent-testing.md` confirms the real `windowed` app is for human visual verification; automated proof should stay headless/source-contract based where possible.

## Research angles

### Angle 1 — Split `src/windowed/render.rs` by presentation responsibility
Investigate how to carve `render.rs` into cohesive subsystems such as:
- camera setup/shake
- sprite spawn + atlas setup
- presentation state advancement
- hurt/death reactions
- damage-number overlays
- VFX/effect spawning and projectile advancement
- barrier/release tracing helpers

Goal: find the most natural ownership boundaries inside the largest file.

### Angle 2 — Split `src/windowed/mod.rs` and clarify the app-wiring seam
Investigate whether `mod.rs` should become a thin composition root that delegates to smaller files for:
- validation config/parsing
- bootstrap/demo composition
- UI panels
- plugin registration / system wiring

Goal: decide how thin `mod.rs` should become, and what should remain there.

### Angle 3 — Refactor strategy and verification plan
Investigate the safest execution order and proof strategy:
- which extractions can happen independently
- which shared types/resources should move first or stay put
- which existing tests/contracts already protect the seams
- whether additional source-contract tests should be added before/while refactoring

Goal: produce a migration plan that minimizes breakage and review noise.

## Out of scope

- Shipping the refactor itself
- Changing gameplay/kernel behavior
- Reworking asset formats or Digimon content definitions
- Replacing the current windowed architecture wholesale
- Introducing new runtime features unrelated to file/module boundaries
