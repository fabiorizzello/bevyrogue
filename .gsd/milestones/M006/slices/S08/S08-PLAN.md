# S08: Register Renamon diamond_storm_leaf cue, Agumon cast proof, spawn-miss diagnostics

**Goal:** Complete Renamon's presentation extension and add an Agumon cast proof: register the diamond_storm_leaf enoki/OnEnterEffect cue (authored at assets/digimon/renamon/anim_graph.ron:10 but unmapped in src/windowed/digimon/renamon/mod.rs), add an Agumon cast-driven particle proof, wire warn-once spawn-miss diagnostics, and resolve the idle-only-vs-hurt design call surfaced in the spike.
**Demo:** Renamon cast emits its enoki effect; Agumon cast-driven proof; warn-once on spawn miss

## Must-Haves

- Renamon's cast emits its diamond_storm_leaf enoki effect through the existing registry seam (no engine control-flow edits); Agumon cast-driven proof confirms the cue path end-to-end; spawn-miss is warned once, not silent. Manual K001 windowed confirmation that both species cast with particles. With S06+S07, Renamon now renders, has legal skills, and casts with VFX — S05's zero-edit thesis genuinely holds.

## Proof Level

- This slice proves: headless test for registry mapping + manual windowed sign-off (K001)

## Verification

- Reuse the S06 warn-once spawn-miss helper for cast cue spawn misses so an unregistered cue is logged once with the cue id rather than silently producing no particle.

## Tasks

- [ ] **T01: Register Renamon diamond_storm_leaf cue** `est:M`
  Add the OnEnterEffect/enoki mapping for the diamond_storm_leaf node authored in renamon/anim_graph.ron into renamon's windowed module register(app), populating only Renamon's own registry entries per the established per-species seam. No edits to render.rs control flow.
  - Files: `src/windowed/digimon/renamon/mod.rs`, `assets/digimon/renamon/anim_graph.ron`
  - Verify: cargo test --features windowed --test windowed_only (Renamon cue mapping present); manual cargo winx shows Renamon cast particle

- [ ] **T02: Add Agumon cast-driven particle proof** `est:M`
  Add a windowed-scope test proving Agumon's cast cue resolves to its registered enoki effect through the same seam, locking the cast->effect contract for the reference Digimon.
  - Files: `tests/windowed_only/case.rs`, `src/windowed/digimon/agumon/mod.rs`
  - Verify: cargo test --features windowed --test windowed_only (Agumon cast proof green)

- [ ] **T03: Warn-once on cast cue spawn miss** `est:S`
  Reuse the S06 deduplicated warn helper so a cast cue with no registered effect logs once with the cue id instead of silently spawning nothing.
  - Files: `src/windowed/render.rs`, `src/windowed/digimon/mod.rs`
  - Verify: cargo test --features windowed --test windowed_only (no warn on happy path); manual winx clean for registered cues

- [ ] **T04: Resolve idle-only-vs-hurt design call and document it** `est:M`
  Decide and implement whether Renamon's non-idle reactions (hurt/death) use shared engine reaction defaults or species-specific data, per the open design question from spike 2. Record the choice with gsd_save_decision and ensure the chosen behavior is covered by the windowed reaction path.
  - Files: `src/windowed/digimon/renamon/mod.rs`, `src/animation/reaction.rs`
  - Verify: cargo test --features windowed --test windowed_only (reaction behavior covered); cargo test (headless green)

## Files Likely Touched

- src/windowed/digimon/renamon/mod.rs
- assets/digimon/renamon/anim_graph.ron
- tests/windowed_only/case.rs
- src/windowed/digimon/agumon/mod.rs
- src/windowed/render.rs
- src/windowed/digimon/mod.rs
- src/animation/reaction.rs
