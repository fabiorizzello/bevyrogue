---
id: S08
parent: M006
milestone: M006
provides:
  - Renamon cast emits diamond_storm_leaf enoki effect through registry seam
  - Agumon cast-effect seam locked by structural test
  - Warn-once on unregistered cast cue
  - Idle-only-vs-hurt design resolved and documented (D055)
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - D055: Renamon non-idle reactions (hurt/death) use shared engine defaults via StanceReaction::stance_node() — Renamon's authored stance.ron already conforms; no species-specific override code needed.
  - Cast cue spawn-miss diagnostics reuse the S06 dedup pattern (Local<HashSet<String>>) rather than a shared helper — keeps the two miss sites independent and avoids threading a shared resource through the system signature.
patterns_established:
  - Per-species cue registration: OnEnterEffectRegistry + EnokiVfxRegistry both populated from species mod.rs register(app), zero engine control-flow edits required for a new Digimon's cast VFX.
  - Warn-once spawn-miss diagnostics: Local<HashSet<String>> dedup set per call site ensures unregistered cues are visible once by name rather than silently skipping or flooding logs.
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-27T11:23:24.622Z
blocker_discovered: false
---

# S08: Register Renamon diamond_storm_leaf cue, Agumon cast proof, spawn-miss diagnostics

**Renamon's cast emits its diamond_storm_leaf enoki effect; Agumon cast-driven particle proof added; warn-once on spawn miss; idle-only-vs-hurt design call resolved and documented.**

## What Happened

S08 finishes Renamon's presentation extension with four coordinated tasks, completing the zero-edit species-registration thesis established in S05.

**T01 — Renamon diamond_storm_leaf cue registered.** `src/windowed/digimon/renamon/mod.rs` gained `register_renamon_on_enter_effects` and `register_renamon_enoki_vfx`, mapping the authored `SpawnParticle(name: "diamond_storm_leaf")` node in `assets/digimon/renamon/anim_graph.ron` to the `diamond_storm.leaf` effect id and loading `digimon/renamon/diamond_storm_leaf.particle.ron` into `EnokiVfxRegistry`. No edits to `render.rs` control flow — the cue travels through the existing registry seam.

**T02 — Agumon cast-driven particle proof.** A new windowed-scope test in `tests/windowed_only/case.rs` proves that Agumon's cast cue resolves through the same `OnEnterEffectRegistry → EnokiVfxRegistry` path to its registered enoki effect, locking the cast→effect contract for the reference Digimon and confirming the seam works end-to-end for both species.

**T03 — Warn-once on cast cue spawn miss.** `src/windowed/render.rs` gained a `Local<HashSet<String>>` dedup set (`cast_cue_spawn_miss_warned`) parallel to the S06 spawn-miss helper. Any `SpawnParticle` cue that produces zero spawned particles (unregistered in `OnEnterEffectRegistry` or whose effect id is absent from `EnokiVfxRegistry`) now logs a `warn!` exactly once per cue id, making unregistered cast cues visible by name rather than silently skipping. Registered cues produce no warning — confirmed by the green windowed test suite.

**T04 — Idle-only-vs-hurt design call resolved (D055).** The open design question from spike 2 was resolved: Renamon's non-idle reactions (`hurt`, `death`) use the shared engine defaults (`StanceReaction::stance_node()` targeting canonical node ids) rather than species-specific data. Renamon's authored `stance.ron` already conforms — it contains `hurt` and `death` nodes with the engine's degrade-to-idle and death-exit transitions — so no engine or species code was needed. A new behavioral test `renamon_reactions_use_shared_engine_defaults` in `tests/windowed_only/renamon_extension_contract.rs` parses Renamon's stance graph and asserts every shared reaction node exists with the expected transitions. Decision recorded as D055.

**Verification outcome:** `cargo test --features windowed --test windowed_only` — 75 passed, 0 failed. Full headless `cargo test` — all suites green (550+ tests across 20 binaries). No engine control-flow edits; grep confirms `diamond_storm_leaf` mapping and warn-once both present in their respective files.

## Verification

1. `cargo test --features windowed --test windowed_only` → 75 passed, 0 failed (renamon cue mapping test + Agumon cast proof + reaction contract all included).
2. `cargo test` (headless) → all 20 suites green, 0 failures.
3. `grep diamond_storm_leaf src/windowed/digimon/renamon/mod.rs` → mapping present in `register_renamon_on_enter_effects` and `register_renamon_enoki_vfx`.
4. `grep cast_cue_spawn_miss_warned src/windowed/render.rs` → warn-once dedup set wired at lines 1040-1346.
5. All four task summaries present under `.gsd/milestones/M006/slices/S08/tasks/`.
6. Decision D055 recorded (idle-only-vs-hurt: shared engine defaults chosen).

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

- `src/windowed/digimon/renamon/mod.rs` — Added register_renamon_on_enter_effects (diamond_storm_leaf cue mapping) and register_renamon_enoki_vfx (loads .particle.ron into EnokiVfxRegistry)
- `src/windowed/render.rs` — Added cast_cue_spawn_miss_warned Local<HashSet<String>> dedup set and warn! on zero-spawn cast cues
- `tests/windowed_only/case.rs` — Added Agumon cast-driven particle proof test
- `tests/windowed_only/renamon_extension_contract.rs` — Added renamon_reactions_use_shared_engine_defaults behavioral test
- `.gsd/DECISIONS.md` — D055 recorded: Renamon uses shared engine reaction defaults (idle-only-vs-hurt design call)
