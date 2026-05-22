# Requirements

This file is the explicit capability and coverage contract for the project.

## Active

## Validated

### R004 — Untitled
- Status: validated
- Validation: Validated in M002: S01 AnimGraph runtime player + wgpu sprite render wired; S01-ASSESSMENT checks 1–7 all PASS (FSM 11/11, registry 5/5, stance asset 3/3, parity 2/2, gameplay-command gate 4/4). cargo run --features windowed shows Agumon cycling idle via stance graph; M001 headless tests green; clip↔atlas parity test present and passing.

### R005 — Untitled
- Status: validated
- Validation: Validated in M002: S01 stance FSM (SkillGraphRegistry/StanceGraphRegistry) delivering per-Digimon stance graphs; S05 full Agumon kit; S07 energy-backed ult with metadata-keyed stance transitions. S01-ASSESSMENT: stance asset 3/3, FSM 11/11; S07 agumon_energy_gauge_fills_locks_and_drains_end_to_end passes.

### R006 — Untitled
- Status: validated
- Validation: Validated in M002: S02 two-clock cue barrier implemented; Sharp Claws windup→strike→recovery on screen with damage on impact frame via ReleaseKernelCue; S02-UAT proves I3 intent parity headless==windowed; full suite + both builds pass.

### R007 — Untitled
- Status: validated
- Validation: Validated in M002: S01 GameplayCommandForbidden anti-DRY test present and passing; anim_graph.ron contains no EmitDamage/EmitStatus/EmitHeal commands; S01-ASSESSMENT gameplay-command gate 4/4.

### R008 — Untitled
- Status: validated
- Validation: Validated in M002: S01 SkillGraphRegistry resolves skill-id→graph with zero if-else dispatch; S09 skill_graph_mapping_extensibility proves 1:1 multi-id resolution and InstantFallback for unregistered ids; CompiledTimeline.id = skill_id confirmed.

### R010 — Untitled
- Status: validated
- Validation: Validated in M002: S03 §9 phase strip updates from EventReader<CombatEvent>; structural test asserts the UI path never mutates combat state; S03-SUMMARY verification_result: passed.

### R011 — Untitled
- Status: validated
- Validation: Validated in M002: S05 full Agumon kit assembled vs Agumon dummy; two-sprite encounter, HP bars, damage numbers, hurt blink, Baby Flame per-hop loop, Baby Burner timeline + detonate chain, Twin Core badge; dummy dies at 0 HP. S07 energy-backed ult loop proven end-to-end. S05-UAT passed.

### R012 — Untitled
- Status: validated
- Validation: Validated in M002: S04 Baby Burner reactive detonate with flash VFX via Rust-configured entity, no RON/editor; S09 vfx_handle_seam proves SpawnParticle RON round-trip with opaque ParticleId, closed VfxLocus/VfxMotion enums, no numeric gameplay payload in serialized form. S04-ASSESSMENT 9/9 PASS.

### R014 — Untitled
- Status: validated
- Validation: Validated in M002: S06 windowed UAT runbook + capture script delivered; architectural review report authored with 7 findings triaged (none critical); R016 invariants green; hot-reload mid-skill confirmed not corrupting world state. Live soak frame-time data pending manual capture per K001 (auto-mode cannot launch windowed binary); framework complete.

### R015 — Untitled
- Status: validated
- Validation: Validated in M002: S06 repomix-grounded architectural review report produced (S06-ARCHITECTURAL-REVIEW.md); 7 findings (F1–F7 all low/medium/info severity); none critical; all findings triaged to M003+ with rationale. MV03 verdict PASS.

### R016 — Untitled
- Status: validated
- Validation: Validated in M002: S06 R016 invariant gate executed (7 cargo commands, R005/R006/R003/R016 hygiene scans all PASS); S08 anim_graph_input_purity + r013_failure_visibility + windowed regression sweep green; S09 clip_atlas_parity 2/2, regression guard 25 windowed_only tests exit 0. No .md added to repo root; no windowed deps outside feature gate.

## Deferred

## Out of Scope

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R004 |  | validated | none | none | Validated in M002: S01 AnimGraph runtime player + wgpu sprite render wired; S01-ASSESSMENT checks 1–7 all PASS (FSM 11/11, registry 5/5, stance asset 3/3, parity 2/2, gameplay-command gate 4/4). cargo run --features windowed shows Agumon cycling idle via stance graph; M001 headless tests green; clip↔atlas parity test present and passing. |
| R005 |  | validated | none | none | Validated in M002: S01 stance FSM (SkillGraphRegistry/StanceGraphRegistry) delivering per-Digimon stance graphs; S05 full Agumon kit; S07 energy-backed ult with metadata-keyed stance transitions. S01-ASSESSMENT: stance asset 3/3, FSM 11/11; S07 agumon_energy_gauge_fills_locks_and_drains_end_to_end passes. |
| R006 |  | validated | none | none | Validated in M002: S02 two-clock cue barrier implemented; Sharp Claws windup→strike→recovery on screen with damage on impact frame via ReleaseKernelCue; S02-UAT proves I3 intent parity headless==windowed; full suite + both builds pass. |
| R007 |  | validated | none | none | Validated in M002: S01 GameplayCommandForbidden anti-DRY test present and passing; anim_graph.ron contains no EmitDamage/EmitStatus/EmitHeal commands; S01-ASSESSMENT gameplay-command gate 4/4. |
| R008 |  | validated | none | none | Validated in M002: S01 SkillGraphRegistry resolves skill-id→graph with zero if-else dispatch; S09 skill_graph_mapping_extensibility proves 1:1 multi-id resolution and InstantFallback for unregistered ids; CompiledTimeline.id = skill_id confirmed. |
| R010 |  | validated | none | none | Validated in M002: S03 §9 phase strip updates from EventReader<CombatEvent>; structural test asserts the UI path never mutates combat state; S03-SUMMARY verification_result: passed. |
| R011 |  | validated | none | none | Validated in M002: S05 full Agumon kit assembled vs Agumon dummy; two-sprite encounter, HP bars, damage numbers, hurt blink, Baby Flame per-hop loop, Baby Burner timeline + detonate chain, Twin Core badge; dummy dies at 0 HP. S07 energy-backed ult loop proven end-to-end. S05-UAT passed. |
| R012 |  | validated | none | none | Validated in M002: S04 Baby Burner reactive detonate with flash VFX via Rust-configured entity, no RON/editor; S09 vfx_handle_seam proves SpawnParticle RON round-trip with opaque ParticleId, closed VfxLocus/VfxMotion enums, no numeric gameplay payload in serialized form. S04-ASSESSMENT 9/9 PASS. |
| R014 |  | validated | none | none | Validated in M002: S06 windowed UAT runbook + capture script delivered; architectural review report authored with 7 findings triaged (none critical); R016 invariants green; hot-reload mid-skill confirmed not corrupting world state. Live soak frame-time data pending manual capture per K001 (auto-mode cannot launch windowed binary); framework complete. |
| R015 |  | validated | none | none | Validated in M002: S06 repomix-grounded architectural review report produced (S06-ARCHITECTURAL-REVIEW.md); 7 findings (F1–F7 all low/medium/info severity); none critical; all findings triaged to M003+ with rationale. MV03 verdict PASS. |
| R016 |  | validated | none | none | Validated in M002: S06 R016 invariant gate executed (7 cargo commands, R005/R006/R003/R016 hygiene scans all PASS); S08 anim_graph_input_purity + r013_failure_visibility + windowed regression sweep green; S09 clip_atlas_parity 2/2, regression guard 25 windowed_only tests exit 0. No .md added to repo root; no windowed deps outside feature gate. |

## Coverage Summary

- Active requirements: 0
- Mapped to slices: 0
- Validated: 11 (R004, R005, R006, R007, R008, R010, R011, R012, R014, R015, R016)
- Unmapped active requirements: 0
