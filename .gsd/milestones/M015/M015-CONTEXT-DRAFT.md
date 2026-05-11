# M015: M013 Closure and Combat Architecture Coherence — Context Draft

**Gathered:** 2026-05-08
**Status:** Draft during discussion

## Vision Input
The user wants M015 to complete what M013 left partial: failed tests, possibly obsolete tests, CLI proof, validation/discussion/context gaps, and any necessary combat-engine review. The user specifically emphasized verifying whether the combat engine is fully coherent or whether multiple implementation passes left it patched, disconnected, or using mixed logic patterns.

## Reflection Summary
M015 is understood as one robust milestone, likely 5–7 slices, unless the audit proves the combat engine needs a deeper redesign that should become a follow-up milestone. This is not a greenfield feature; it is closure, validation, and normalization after M013.

## Scope Decisions

### In Scope
- Close what M013 left partial, including code, tests, CLI proof, and relevant GSD validation/artifact gaps.
- Classify failing tests into real regression, obsolete test, stale manifest/declaration, or missing implementation.
- Fix or remove tests based on current architecture, not by preserving old APIs blindly.
- Audit the combat engine for single source of truth and mixed-pattern drift.
- Verify that code written in multiple passes is not patched together, disconnected, or carrying multiple incompatible combat logic models.
- Normalize clear architectural contradictions, not just failures that block cargo test.
- Preserve the intended hierarchy: RON owns data/numbers/metadata; core combat kernel owns canonical state/transitions/timing/authority; per-Digimon blueprint-like logic extends through typed hooks/systems; animation triggers/cues remain non-authoritative presentation.
- Ensure CLI proves the shared combat/query/event/beat surfaces, not a CLI-only path.

### Out of Scope / Deferred
- Full rewrite of combat engine if only localized drift is found.
- Presentation/UI implementation beyond proving metadata/beat surfaces.
- Gameplay authority in animation metadata.
- Keeping obsolete tests only to satisfy stale contracts.
- DNA Chips / roguelite meta-loop work.

## Architectural Decisions

### Blueprint-like module per Digimon
The future surface should tend toward modules/logical ownership per Digimon (`agumon`, `greymon`, `gabumon`, etc.), not only modules per mechanic (`twin_core`, `battery_loop`, `holy_support`). Mechanic modules can remain as shared primitives, but the primary identity should be unique Digimon logic.

Rationale: the user wants each Digimon to feel unique. If the primary abstraction remains line/mechanic, the system risks becoming families of reused loops with different skins.

Evidence: M013 currently has loop/mechanic modules (`src/combat/twin_core.rs`, `holy_support.rs`, `battery_loop.rs`, `predator_loop.rs`, `precision_mind_game.rs`). Prior decisions already forbid central per-Digimon branching and require typed hooks. The user clarified: `per digimon non per linea/meccanica`.

Alternatives considered: per-line/mechanic modules as primary abstraction; RON script-like DSL; central core match ladder per Digimon.

### RON emits custom signals/intents, not final gameplay authority
RON declares numbers, skill metadata, animation/presentation trigger metadata, and custom signals/intents. It should not decide final gameplay outcomes when unique logic is required.

### Combat kernel remains generic authority layer
The kernel core must not centrally know `Agumon does X`. It exposes generic timing, state, transition/event/query surfaces. Per-Digimon blueprint logic hooks into the kernel without rewriting it.

### Kernel transitions are audit/output contract, not blueprint source
`CombatKernelTransition` is a canonical observable/mutation contract after blueprint-like logic resolves what happened. Correct flow:

```text
RON data + custom signals
  → per-Digimon blueprint module
  → hook into generic combat kernel
  → canonical state / transition / event / snapshot
  → CLI / tests / future UI
```

### M015 normalization target
M015 should normalize clear drift toward this architecture, but not become an unlimited rewrite. If a complete per-Digimon migration is too broad, M015 should create a coherent baseline and a precise follow-up plan.

## Investigation Notes
- `.gsd/STATE.md` says M013 has all slices complete and next action is milestone summary.
- `.gsd/milestones/M013/M013-VALIDATION.md` has verdict `needs-attention`, citing missing/incomplete slice summaries, weak cross-slice consumption proof, and incomplete validation packaging.
- `cargo test --no-run` currently fails before compiling tests because `Cargo.toml` declares `tests/battery_loop_resolution.rs`, which is missing.
- Bevy 0.18 docs confirm Messages require schedule advancement/app updates; some stale tests may fail because they enqueue messages without advancing the app.
- User-provided UE5/GAS research supports the intended split: gameplay authority in abilities/effects, animation notifies/montage events as timing hooks, and gameplay cues for presentation feedback.

## Open Questions
- Error handling/failure-mode depth still needs confirmation.
- Quality bar still needs confirmation.
- Requirements and roadmap still need explicit confirmation before writes.