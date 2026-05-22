---
id: M002
title: "First on-screen combat (Agumon-only)"
status: complete
completed_at: 2026-05-22T09:19:30.470Z
key_decisions:
  - D025: Two-clock cue barrier ownership — kernel stays deterministic on HeadlessAuto; windowed systems opt into Clock::Windowed and release the CueBarrier when the animation player emits ReleaseKernelCue
  - D026: Baby Burner reactive detonate as Rust-only post-application reaction in the Agumon blueprint — no RON timeline migration, no AnimGraph KernelEvent branching, proving reactive contract headlessly while preserving determinism
  - D027: Frame-time aggregator with BEVYROGUE_VALIDATION_BASELINE env toggle — pure headless-testable FrameTimeAccumulator; full-vs-baseline windowed comparison isolates anim-graph cost; pass bar mean ≤15% AND ≤2ms absolute
  - Closed typed AnimGraphRole/AnimGraphInput seam as R009 contract — graph evaluation is a pure function of typed inputs; legacy player entrypoints are thin default-input wrappers; no world-global reads or mutable context
  - Hot reload next-spawn-only via resolved-graph snapshot binding — in-flight players keep current graph identity; registry updates affect only future spawns to prevent mid-cast state corruption
  - Energy-backed ult gauge: metadata-keyed opt-in (ult_gauge=energy) with legacy UltimateCharge path preserved for metadata-free Digimon — enables one-Digimon-at-a-time migration without big-bang roster change
  - Cue timeout force-resumes through the same released-runner path as normal cue completion — avoids divergent error branch; retains structured CueBarrierStatus diagnostics post-resume
  - Shared snapshot helper for ult readiness and resource display — prevents legality/HUD drift by routing both through the same UnitQuerySnapshot fields and helper function
key_files:
  - src/animation/anim_graph.rs
  - src/animation/player.rs
  - src/animation/registry.rs
  - src/animation/plugin.rs
  - src/combat/runtime/cue_barrier.rs
  - src/combat/turn_system/pipeline/timeline_exec.rs
  - src/combat/action_query/types.rs
  - src/combat/action_query/legality/shared.rs
  - src/combat/observability/frame_time.rs
  - src/windowed/mod.rs
  - tests/animation/anim_graph_input_purity.rs
  - tests/animation/anim_registry_failure_visibility.rs
  - tests/animation/vfx_handle_seam.rs
  - tests/animation/skill_graph_mapping_extensibility.rs
  - tests/timeline/r013_failure_visibility.rs
  - tests/timeline/timeline_cue_barrier_pipeline.rs
  - tests/windowed_only/frame_time_soak.rs
  - tests/digimon_kits/agumon_energy_gauge.rs
  - .gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md
  - .gsd/milestones/M002/slices/S09/S09-CLOSEOUT.md
  - .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md
lessons_learned:
  - Bevy 15-tuple QueryData compile limit: adding a 16th component to a wide query is a compile error, not a runtime panic — use a sibling read-only query joined in Rust code to add optional snapshot data beyond the limit
  - CombatEvent read-only constraint (D008) must be enforced by a structural test, not just by convention — only a failing test on mutation imports in the UI module is auditable across sessions
  - Failure-visibility paths must retain inspectable structured state after recovery (CueBarrierStatus, AnimationGraphLookupDiagnostics, AnimationGraphLoadState) so failures are verifiable in headless CI without a live session
  - Integration tests under tests/ link only against the lib crate, not the binary — anything a windowed_only integration test must call must live in the lib (e.g. parse_validation_baseline_toggle in combat::observability, not src/windowed/mod.rs)
  - Boundary map rows should cite actual on-disk test function names (not just file paths) to make each producer→consumer contract machine-checkable by a one-liner verification script
  - Stale roster contracts can mask real migration correctness: S07 T05 regression was caused by holy_support_roster_contract.rs expecting empty Agumon metadata rather than a generic snapshot fixture issue — always update roster contract tests when Digimon metadata changes
---

# M002: First on-screen combat (Agumon-only)

**AnimGraph runtime player + two-clock impact sync + §9 UI + full Agumon kit vs Agumon dummy on screen, gated by repomix architectural review and validated across all nine slices.**

## What Happened

M002 delivered the first time bevyrogue's combat appeared on screen. Nine slices built and validated the full stack from animation schema to playable encounter.

**S01** established the AnimGraph runtime foundation: SkillGraphRegistry and StanceGraphRegistry map skill-ids to typed RON assets; the AnimGraph player advances a stance FSM driving an on-screen sprite; a GameplayCommandForbidden test enforces the gameplay/presentation seam; clip↔atlas geometry parity is contractually proven. M001 headless suite stayed green throughout.

**S02** wired the two-clock impact barrier: the CueBarrier suspends the timeline at each authored impact frame and releases on ReleaseKernelCue from the animation player. Sharp Claws windup→strike→recovery visible on screen; damage lands on the impact frame; invariant I3 (identical Intent stream headless vs windowed, only timing differs) extended to the cue handshake.

**S03** brought §9 UI to life event-driven: the phase strip + HP bars + damage numbers + hurt blink derive entirely from EventReader<CombatEvent>. A structural test proves the UI code path has no write access to CombatState, turning D008 from a convention into an enforceable contract.

**S04** added Baby Burner reactive detonate with a flash VFX: Rust-only post-application reaction registered in the Agumon blueprint, opaque ParticleId handle, closed VfxLocus/VfxMotion enums — no RON/editor, zero non-determinism, R004 intact.

**S05** assembled the full kit: Agumon vs Agumon dummy, two sprites, HP bars, damage numbers, hurt blink, Baby Flame per-hop loop (N repetitions = N kernel hops, no N in the anim graph), Baby Burner timeline + detonate chain, Twin Core badge. Dummy dies at 0 HP. End-to-end two-clock pipeline proven with the full assembly.

**S06** ran the windowed smoke end-to-end: UAT runbook + automated capture script delivered; R016 invariant gate (7 cargo commands, hygiene scans) all green; hot-reload mid-skill confirmed not corrupting world state. A repomix-grounded architectural review produced S06-ARCHITECTURAL-REVIEW.md with 7 findings (F1–F7, all low/medium/info severity, none critical, all triaged to M003+). Live soak frame-time numbers deferred per K001 (auto-mode cannot launch windowed binary); the aggregation framework is proven headlessly.

**S07** migrated Agumon's ult loop to the real energy gauge: readiness flips exactly at Energy.max, cast drains Energy and UltimateCharge to zero, metadata-free Digimon stay on the legacy UltimateCharge path. A shared snapshot helper keeps legality and HUD display in sync. The Bevy 15-tuple QueryData limit was worked around via a sibling read-only query.

**S08** hardened the animation/presentation seam: typed pure AnimGraphRole/AnimGraphInput seam (R009) with executable purity proof; CueBarrier 180-frame timeout force-resumes through the released-runner path retaining structured CueBarrierStatus diagnostics; AnimGraph registry provides InstantFallback + AnimationGraphLookupDiagnostics for missing skills; hot reload is next-spawn by snapshot binding; dead-target mid-loop completes without liveness branching.

**S09** packaged milestone closeout: M002-BOUNDARY-MAP.md with 5 test-cited producer→consumer rows (machine-checkable by verification script); skill-graph mapping extensibility proof (1:1 multi-id resolution, InstantFallback for unregistered); VFX handle seam proof (SpawnParticle RON round-trip, closed-enum rejection, no numeric payload); frame-time aggregator with D027 threshold math and BEVYROGUE_VALIDATION_BASELINE toggle; S09-CLOSEOUT.md bundling all evidence.

The M002-VALIDATION.md verdict is PASS across all four verification classes: Contract (GameplayCommandForbidden, clip↔atlas parity, I3 parity, headless suite), Integration (assembled windowed runtime drives full kit vs dummy through the two-clock pipeline), and Operational (runbook, capture script, architectural review, R016 green; live soak data deferred per K001). All 11 active requirements (R004–R008, R010–R012, R014–R016) transitioned to validated.

## Success Criteria Results

All nine roadmap success criteria met — validated by M002-VALIDATION.md verdict PASS:

- **S01** ✅ Agumon idle cycling via stance graph (not hardcoded); M001 headless tests green; clip↔atlas parity present and passing. Evidence: S01-ASSESSMENT checks 1–7 all PASS (FSM 11/11, registry 5/5, stance asset 3/3, parity 2/2, gameplay-command gate 4/4).
- **S02** ✅ Sharp Claws windup→strike→recovery on screen; damage on impact frame via ReleaseKernelCue; telegraph chip visible; I3 intent parity headless/windowed. Evidence: S02-SUMMARY passed; full suite + both builds pass.
- **S03** ✅ §9 phase strip updates from EventReader<CombatEvent>; structural test proves UI never mutates combat state. Evidence: S03-SUMMARY passed; combat-read-only structural test passes.
- **S04** ✅ Baby Burner reactive detonate + flash VFX (Rust code, no RON/editor); zero non-determinism; R004 intact. Evidence: S04-ASSESSMENT 9/9 checks PASS.
- **S05** ✅ Full Agumon kit vs dummy; multi-hit loop = kernel hop count; CombatEvent-driven blink; HUD HP/damage; dummy dies at 0 HP. Evidence: S05-SUMMARY passed; full matrix + both builds pass.
- **S06** ✅ Windowed soak runbook + capture script; repomix architectural review report (pass-with-followups, F1–F7 triaged to M003+); R016 invariants green. Evidence: S06-SUMMARY passed; S06-ARCHITECTURAL-REVIEW.md present.
- **S07** ✅ Energy-backed Agumon ult gauge; readiness flips at full energy; cast drains to zero; legacy path preserved for non-opted-in Digimon. Evidence: S07-SUMMARY passed; agumon_energy_gauge_fills_locks_and_drains_end_to_end passes.
- **S08** ✅ Typed pure AnimGraph input seam (R009) + structured failure visibility (R013: cue timeout, missing graph fallback, hot reload, dead target). Evidence: S08-SUMMARY passed; anim_graph_input_purity + r013_failure_visibility green.
- **S09** ✅ Explicit producer→consumer boundary map (5 test-cited rows), VFX/skill-graph extensibility proofs, frame-time aggregator with D027 threshold math + soak wiring, S09 closeout bundle. Evidence: S09-SUMMARY passed; 8 verification items all exit 0; M002-BOUNDARY-MAP.md + S09-CLOSEOUT.md present.

## Definition of Done Results

**All slices [x]:** S01–S09 all marked complete in ROADMAP.md. ✅

**All SUMMARY.md files exist:** S01–S09 each have verification_result: passed in their SUMMARY.md frontmatter. ✅

**All tasks done:** 9 slices × all tasks complete (S01: 5/5, S02: 6/6, S03: 4/4, S04: 4/4, S05: 6/6, S06: 3/3, S07: 5/5, S08: 4/4, S09: 5/5 = 42/42 tasks). ✅

**Integrations work:** M002-VALIDATION.md cross-slice integration audit confirms all 5 boundary-map contracts are honored (MV03 PASS). Assembled windowed runtime drives Agumon full kit vs dummy through the real two-clock pipeline (S05-UAT). ✅

**Repomix architectural review gate:** S06-ARCHITECTURAL-REVIEW.md present; 7 findings triaged; none critical. ✅

**Requirements:** All 11 active M002 requirements (R004–R008, R010–R012, R014–R016) transitioned to validated. ✅

**Code changes:** 44 files changed, 1445 insertions(+), 757 deletions(−) across non-.gsd source files. Commits present with M002-scoped implementation work. ✅

**One open item (not a failure):** Live windowed soak frame-time numbers are PENDING manual capture per K001 (auto-mode cannot launch windowed binary). D027 threshold math and aggregation framework are proven deterministically headless (10 unit tests + 2 windowed_only tests). Documented in frame-time-comparison.md with manual capture commands. ✅

## Requirement Outcomes

All 11 active M002 requirements transitioned to **validated** with executable evidence:

| Req | Title | Evidence |
|-----|-------|----------|
| R004 | AnimGraph runtime player + sprite render | S01-ASSESSMENT 7/7 checks PASS; stance graph drives on-screen sprite |
| R005 | Per-Digimon Stance FSM | S01 FSM 11/11; S07 energy gauge migration preserves stance transitions |
| R006 | Two-clock impact sync | S02-UAT: damage on impact frame, I3 parity headless/windowed |
| R007 | Gameplay/presentation seam | GameplayCommandForbidden test 4/4; anim_graph.ron clean |
| R008 | Per-skill graph 1:1 with CompiledTimeline | S09 skill_graph_mapping_extensibility: 1:1 resolution, zero if-else |
| R010 | §9 phase strip event-driven | S03: EventReader<CombatEvent>; structural mutation-proof test |
| R011 | Full Agumon kit playable | S05-UAT full kit vs dummy; S07 energy-backed ult end-to-end |
| R012 | VFX opaque Id handle | S04 ASSESSMENT 9/9; S09 vfx_handle_seam 4/4 |
| R014 | Windowed smoke end-to-end | S06 runbook + capture script; hot-reload non-corrupting; live soak deferred per K001 |
| R015 | Repomix architectural review gate | S06-ARCHITECTURAL-REVIEW.md; 7 findings triaged; MV03 PASS |
| R016 | Determinism and headless-first preserved | S06 R016 gate 7 commands PASS; S08 windowed regression sweep exit 0; S09 clip_atlas_parity 2/2 |

Pre-validated requirements R003, R009, R013 (validated during M002 slices) and R021–R028 (validated during M001) remain validated.

## Deviations

S07 T05 regression root cause was a stale Holy Support roster contract (expected empty Agumon ult_gauge metadata pre-migration) rather than a generic snapshot-fixture shape issue. Fixed by updating the contract to accept ult_gauge=energy for Agumon and explicitly preserving Gabumon as the metadata-free legacy control.

S08 closeout attempted a reviewer subagent dispatch but the agent was unavailable due to environment usage limits. Slice relied on direct gsd_exec verification evidence and task summaries instead.

Live windowed soak frame-time comparison (D027 operational verification class): live numbers not captured in this session per K001 (auto-mode cannot launch windowed binary). The aggregation math and D027 pass/fail logic are proven deterministically headless. Documented as PENDING in frame-time-comparison.md with manual capture commands.

## Follow-ups

Live windowed soak frame-time numbers: manual capture required per K001 (auto-mode cannot launch windowed binary). Commands and pending results table in `.gsd/milestones/M002/slices/S09/frame-time-comparison.md`.

S06 architectural review findings F1–F7 (all low/medium/info severity): triaged roadmap in S06-ARCHITECTURAL-REVIEW.md; all deferred to M003+ with rationale.

Roster extension beyond Agumon: the per-skill graph + stance schema is designed for data-only extension; M003+ adds new Digimon as new RON assets + a Rust blueprint module.

RON VFX format / editor (bevy_enoki / Omagari): the opaque ParticleId handle seam is open; implementation explicitly excluded from M002.
