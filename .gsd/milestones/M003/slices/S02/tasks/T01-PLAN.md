---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Authored Baby Flame + Baby Burner ReleaseKernel cues and proved the impact-frame-on-rendered-frame invariant headless with four new atlas-parity tests.

Why: The windowed bridge can only suspend/release on beats carrying presentation + a ReleaseKernel anim cue. Baby Flame's impact_damage beat has presentation:None and baby_flame_impact has no ReleaseKernel cue, so its damage auto-resolves with no rendered-frame gating; Baby Burner's windup (baby_burner_charge) and recovery (baby_burner_recovery) nodes also lack release cues (only baby_burner_launch has one). Per D031 we author ReleaseKernel cues as the uniform release trigger and prove the contract before any render.rs churn (research build-order step 1, highest unblocker). Do: (1) In assets/data/digimon/agumon/skills.ron, change the SkillId("baby_flame") impact_damage beat from presentation:None to presentation: Some((cue_id: "agumon/baby_flame/impact", anim: Some("baby_flame_impact"), vfx: None, sfx: None)) — leave payload/hook/selector unchanged. (2) In assets/digimon/agumon/anim_graph.ron, add cues: [(at: 1, command: ReleaseKernel(()))] to the baby_flame_impact node (mirrors sharp_claws_strike at:1 -> clip frame 70, inside skill [59,75]); add a ReleaseKernel cue at the end-of-node local frame of baby_burner_charge (frames 23-30 -> at: 7 -> frame 30) and baby_burner_recovery (frames 38-45 -> at: 7 -> frame 45). (3) Verify no compiled-timeline layer injects presentation onto baby_flame impacts: grep src/data/skill_timeline.rs and src/combat/runtime/ for presentation defaulting; record in the SUMMARY that the explicit-None-vs-Some authoring is intentional. (4) Extend tests/animation/atlas_binding.rs (mirroring the TC-6 sharp_claws_release_cue test and the parity drives): add baby_flame_impact_release_cue_resolves_to_in_range_impact_atlas_tile (scan baby_flame* nodes for a ReleaseKernel cue, compute clip_frame_at_cue, assert ∈ skill [59,75] and atlas_index==identity); add baby_burner_launch_release_cue_resolves_to_in_range_impact_atlas_tile (scan baby_burner* nodes, assert ∈ heavy_attack [23,45] and identity); add a baby_burner parity drive starting at NodeId("baby_burner_charge") firing fire_kernel_cue at the charge->launch and launch->recovery KernelCue boundaries, asserting every frame ∈ heavy_attack [23,45] and identity through to exit; add a baby_flame parity drive starting at NodeId("baby_flame_cast") asserting every frame ∈ the UNION of authored node ranges [60,77] (NOT the coarse clip skill label — recover frames 76/77 overshoot skill end 75 per research finding #5) and identity. Add SKILL_RANGE 59..=75 and HEAVY_ATTACK_RANGE 23..=45 consts sourced from clip.ron. Done-when: cargo test --test animation is green with all S01 tests plus the four new tests, and full cargo test is green (authoring presentation on baby_flame must not regress headless combat — headless Clock auto-resolves presentation beats, MEM025/research finding #1).

## Inputs

- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/agumon/clip.ron`
- `tests/animation/atlas_binding.rs`
- `src/windowed/render.rs`
- `src/animation/player.rs`

## Expected Output

- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/animation/atlas_binding.rs`

## Verification

cargo test --test animation 2>&1 | tail -20 && cargo test 2>&1 | tail -20

## Observability Impact

No new runtime logging; this task adds data + headless tests only. Document in SUMMARY that baby_flame impact now carries presentation (intentional, not a defaulting bug).
