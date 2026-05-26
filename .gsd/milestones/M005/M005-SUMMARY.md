---
id: M005
title: "Combat visual feedback completion (reactions + enoki VFX)"
status: complete
completed_at: 2026-05-26T10:24:00.892Z
key_decisions:
  - StanceReaction is a closed fully-enumerated enum so new event variants force deliberate classification
  - Death precedence enforced via system ordering rather than per-event priority logic
  - enoki added as an intercept inside spawn_effect_by_id reusing existing placement math, with the quad system kept as fallback
  - Hit flash/shake implemented as an ad-hoc third system (hit_feedback.rs) coupled to AgumonSprite — flagged for generalization in M006
key_files:
  - src/animation/reaction.rs
  - src/windowed/render.rs
  - src/ui/hit_feedback.rs
lessons_learned:
  - (none)
---

# M005: Combat visual feedback completion (reactions + enoki VFX)

**Wired emitted combat events to stance hurt/death reactions, added hit flash + shake + canvas damage numbers, and replaced placeholder quad VFX with bevy_enoki for Agumon's three skills.**

## What Happened

M005 closed the gap between complete combat logic and incomplete presentation across five slices. S01 wired CombatEvent OnHitTaken to a pure lib stance-reaction mapping (reaction.rs) plus a windowed bridge (drive_hurt_reactions) flinching the struck sprite. S02 added death reactions and field-exit fade via drive_death_reactions with system-ordering death precedence and a DeathExiting marker. S03 delivered hit flash (color tint), sprite shake (deterministic sinusoidal offset), and floating damage numbers rendered on the pixel canvas. S04 wired bevy_enoki windowed-gated with a static dep-gating test proving no leak into the headless build, routing one Agumon effect through enoki at the spawn_effect_by_id seam. S05 extended the enoki intercept to all three Agumon skills (sharp_claws.slash, baby_flame.impact, baby_burner.detonate) while keeping the custom quad system as the fallback. The K001 visual sign-off (the only auto-mode-unreachable gate) was accepted by the user. Follow-up architectural concerns about the windowed presentation layer being Agumon-hardcoded rather than extension-first are captured in the separately-planned M006.

## Success Criteria Results

All six success criteria met: pure headless-tested reaction mapping; flinch both sides + death/field-exit (K001 accepted); flash/shake/canvas damage numbers (K001 accepted); enoki windowed-gated with dep-leak test; all three Agumon skills through enoki (K001 accepted); full cargo test + windowed build green (headless 51, windowed 49, dep-gating 2).

## Definition of Done Results

Not provided.

## Requirement Outcomes

Not provided.

## Deviations

None.

## Follow-ups

M006 (Extension-first presentation refactor + enoki-only VFX) already planned: retire the quad system for enoki-only rendering, introduce a CueRegistry with generic flash/blink/sprite-shake/camera-shake primitives, generalize AgumonSprite→DigimonSprite, extract Agumon presentation out of render.rs, and register Renamon with zero core edits as the extension-first acceptance gate (D042–D045).
