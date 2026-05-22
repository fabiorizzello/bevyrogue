---
id: T01
parent: S02
milestone: M003
key_files:
  - assets/data/digimon/agumon/skills.ron
  - assets/digimon/agumon/anim_graph.ron
  - tests/animation/atlas_binding.rs
key_decisions:
  - Per D031, ReleaseKernel cues are the uniform release trigger; added them at end-of-node local frames for baby_burner_charge (at:7) and baby_burner_recovery (at:7), and at:1 for baby_flame_impact (mirrors sharp_claws_strike).
  - Baby Flame parity drive asserts against the authored node UNION [60,77], not the coarse skill clip label [59,75], because baby_flame_recover overshoots to frames 76/77 (research finding #5; captured as MEM057).
  - baby_burner release-cue test iterates ALL baby_burner* nodes rather than the first match, since the slice now authors three barriers (charge/launch/recovery).
duration: 
verification_result: passed
completed_at: 2026-05-22T10:45:29.102Z
blocker_discovered: false
---

# T01: Authored Baby Flame + Baby Burner ReleaseKernel cues and proved the impact-frame-on-rendered-frame invariant headless with four new atlas-parity tests.

**Authored Baby Flame + Baby Burner ReleaseKernel cues and proved the impact-frame-on-rendered-frame invariant headless with four new atlas-parity tests.**

## What Happened

Closed the asymmetric authoring gaps that left Baby Flame and Baby Burner without rendered-frame release gating, then proved the impact-frame invariant before any render.rs churn (research build-order step 1).

Data authoring:
1. assets/data/digimon/agumon/skills.ron — changed the SkillId("baby_flame") impact_damage beat from presentation:None to presentation: Some((cue_id: "agumon/baby_flame/impact", anim: Some("baby_flame_impact"), vfx: None, sfx: None)). Payload/hook/selector unchanged.
2. assets/digimon/agumon/anim_graph.ron — added cues: [(at: 1, command: ReleaseKernel(()))] to baby_flame_impact (frames 69-72; at:1 -> clip frame 70, inside skill [59,75], mirroring sharp_claws_strike). Added end-of-node ReleaseKernel cues per D031: baby_burner_charge (frames 23-30, at:7 -> frame 30) and baby_burner_recovery (frames 38-45, at:7 -> frame 45). baby_burner_launch already carried its at:1 cue.

Verified no compiled-timeline layer injects presentation: grepped src/data/skill_timeline.rs (only a doc comment references presentation; the compile path never defaults it) and src/combat/runtime/ (runner reads beat.presentation, preview.rs intern_presentation passes it through via map). The explicit None-vs-Some authoring is therefore intentional and load-bearing, not a defaulting bug.

Tests (tests/animation/atlas_binding.rs): added SKILL_RANGE 59..=75 and HEAVY_ATTACK_RANGE 23..=45 consts sourced from clip.ron, plus a BABY_FLAME_NODE_UNION 60..=77 const documenting the recover-tail overshoot (research finding #5). Four new tests, all mirroring the TC-6 sharp_claws patterns:
- baby_flame_impact_release_cue_resolves_to_in_range_impact_atlas_tile: scans baby_flame* nodes for a ReleaseKernel cue, computes clip_frame_at_cue, asserts the impact frame (70) is in skill [59,75] and atlas_index is identity.
- baby_burner_launch_release_cue_resolves_to_in_range_impact_atlas_tile: scans ALL baby_burner* nodes (now three barriers), asserting each ReleaseKernel cue resolves in heavy_attack [23,45] and identity.
- baby_burner_player_frames_map_identity_within_heavy_attack_range: parity drive from NodeId("baby_burner_charge"), firing fire_kernel_cue at the charge->launch and launch->recovery KernelCue boundaries (after each node plays its full span), asserting every frame in [23,45] and identity through to exit.
- baby_flame_player_frames_map_identity_within_node_union: parity drive from NodeId("baby_flame_cast") asserting every frame in the authored node union [60,77] (not the coarse clip skill label, since recover overshoots to 76/77) and identity.

Confirmed authoring presentation on baby_flame does not regress headless combat — the headless Clock auto-resolves presentation beats (MEM025), so the full suite stays green.

## Verification

Ran the task verification commands. `cargo test --test animation` is green: 61 passed (all S01 tests plus the four new tests). Full `cargo test` is green across all 23 test binaries (23 "test result: ok" lines, zero failures); the only warning is a pre-existing unused `BeatEdge` import unrelated to these data/test changes. Authoring presentation on baby_flame did not regress headless combat.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation 2>&1 | tail -25` | 0 | pass | 15000ms |
| 2 | `cargo test 2>&1 | tail -30` | 0 | pass | 30000ms |
| 3 | `cargo test 2>&1 | grep -c 'test result: ok'  (=> 23, zero non-'0 failed' lines)` | 0 | pass | 30000ms |

## Deviations

none — followed the inlined task plan; the baby_burner release-cue test iterates all matching nodes (the plan said 'scan baby_burner* nodes') since three barriers now exist.

## Known Issues

none

## Files Created/Modified

- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/animation/atlas_binding.rs`
