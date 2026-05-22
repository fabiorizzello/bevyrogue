---
id: T05
parent: S09
milestone: M002
key_files:
  - .gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md
  - .gsd/milestones/M002/slices/S09/S09-CLOSEOUT.md
key_decisions:
  - Grounded every boundary-map row in a real on-disk test function name (extracted via grep) rather than only the file path, so the contract is machine-checkable and reader-verifiable
  - Treated the soak console log and live frame-time verdict as a deferred manual step per KNOWLEDGE rule K001 (auto-mode must not launch the windowed binary) instead of attempting a live capture; documented the manual commands and cited the headless proofs of the generating code
  - Wrote both docs as dedicated S09 artifacts (not the empty DB-rendered roadmap Boundary Map stub) per the task plan, since the roadmap section is DB-rendered
duration: 
verification_result: passed
completed_at: 2026-05-22T08:27:46.147Z
blocker_discovered: false
---

# T05: Wrote the M002 producer→consumer boundary map (5 test-cited rows) and the S09 closeout bundle tying together S08 R009/R013 proofs, soak log, and frame-time comparison verdict

**Wrote the M002 producer→consumer boundary map (5 test-cited rows) and the S09 closeout bundle tying together S08 R009/R013 proofs, soak log, and frame-time comparison verdict**

## What Happened

T05 was a documentation/evidence-assembly task closing out M002 operationally. I first confirmed all five cited test files and all input artifacts (S08-SUMMARY, frame-time-comparison.md, soak-console.log) exist on disk, and that neither output doc existed yet. I read S08-SUMMARY to ground the R009/R013 citations, the frame-time-comparison and soak-console.log to ground the manual-soak/K001 status, and grepped the five cited test files to extract the actual enforcing test function names so each boundary-map row cites a real, on-disk assertion rather than a prose promise.\n\nM002-BOUNDARY-MAP.md is a table mapping producer subsystem → contract/data type → consumer subsystem for all five required seams: (1) kernel skills.ron gameplay numbers → compiled SkillTimeline → anim-graph opaque commands (tests/timeline/boundary_contract.rs); (2) anim player cue emission → cue barrier → kernel resume (tests/windowed_only/phase_strip_readonly.rs); (3) CombatEvent stream → §9 UI/HUD read-only projection (tests/preview_ai/presentation_metadata_boundary.rs + phase_strip_readonly.rs); (4) SkillGraphRegistry skill-id → windowed player with InstantFallback, flagging the hardcoded-constant wiring as the M003+ constraint to lift (tests/animation/skill_graph_mapping_extensibility.rs); (5) opaque ParticleId VFX seam → windowed validate-only consumer (tests/animation/vfx_handle_seam.rs). It includes a reader-test paragraph stating the unifying invariant (gameplay numbers stay kernel-side; downstream gets opaque closed-enum commands / read-only projections) so someone with no M002 context can use it.\n\nS09-CLOSEOUT.md bundles, by reference (not re-validation): the S08 R009/R013 passing test names, a link to the boundary map, the captured soak console log (NOT captured in auto-mode per K001, with manual commands documented), the D027 frame-time comparison verdict (PENDING manual live soak; aggregator/verdict math proven headlessly), and the regression-guard headless command results table (cargo test --lib frame_time 10 passed, windowed_only frame_time 2 passed, windowed build exit 0, windowed_validation 4 passed).

## Verification

Ran the exact T05 slice verification command (bash one-liner): asserts both output docs exist, all five cited test paths resolve on disk AND are cited in the boundary map, and the boundary map contains all seven required row keywords (kernel, timeline, anim-graph, cue, CombatEvent, SkillGraphRegistry, ParticleId). Command printed BOUNDARY_MAP_AND_CLOSEOUT_OK with exit 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `bash -c 'test -f M002-BOUNDARY-MAP.md && test -f S09-CLOSEOUT.md; for each cited test: test -f && grep -qF in boundary map; for each row keyword: grep -qiF; echo BOUNDARY_MAP_AND_CLOSEOUT_OK'` | 0 | pass | 120ms |

## Deviations

None.

## Known Issues

SkillGraphRegistry skill-id→graph wiring still uses hardcoded constants (boundary enforced, data-driven registration deferred to M003+). Live full-vs-baseline frame-time soak verdict remains PENDING a manual run on a display-equipped machine per K001.

## Files Created/Modified

- `.gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md`
- `.gsd/milestones/M002/slices/S09/S09-CLOSEOUT.md`
