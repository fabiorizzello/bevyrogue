# Scope — VFX / Renamon UI / reaction feedback spike

## Problem statement

Investigate three apparently related presentation regressions in the windowed Bevy build:

1. **Agumon VFX** — Agumon appears to be wired to Enoki particle assets, but the effects are not visibly rendered in the game.
2. **Renamon presentation/UI** — Renamon does not show the expected sprite in the UI/combat presentation, and the UI does not allow skill selection as expected.
3. **Reaction / hit-feedback layer** — hurt and dead animations do not behave correctly, and the hit-feedback cues (flash, camera shake, sprite shake) do not appear to fire correctly.

This spike is for **root-cause analysis and recommendation**, not for shipping production fixes.

## Preliminary code signals already found

- `src/windowed/digimon/agumon/mod.rs` explicitly registers Agumon Enoki effects, on-enter effect ids, cue definitions, and sprite presentation.
- `src/windowed/digimon/renamon/mod.rs` registers Renamon stance path, skill start node, sprite presentation, and demo entry, but its presentation surface is much thinner than Agumon’s.
- `assets/data/digimon/renamon/unit.ron` and `assets/data/digimon/renamon/skills.ron` do define Renamon unit + skills, so the missing UI behavior is likely in load/registration/runtime wiring rather than missing authored data.
- `src/ui/combat_panel/*`, `src/ui/cues.rs`, `src/animation/reaction.rs`, and `src/windowed/render.rs` appear to own the feedback / animation presentation path.

## Questions to answer

1. **Agumon / Enoki:** Where does the Agumon VFX pipeline break: asset load, registry registration, effect spawn, effect lifecycle advancement, render extraction, or visibility/placement?
2. **Renamon UI / sprite:** Why is Renamon’s sprite or presentation not shown, and why is skill selection unavailable even though Renamon data exists?
3. **Reaction feedback:** Why are hurt/dead animations and flash/camera-shake/sprite-shake not firing or not reading correctly in the runtime?
4. **Coupling:** Are these three failures caused by one shared presentation wiring regression, or by separate bugs that only look related at runtime?

## Success criteria

A useful answer must include:

- **A concrete failure map** for each issue area:
  - expected producer/consumer path
  - exact break point or strongest suspected break point
  - evidence (code references, tests, logs, or runtime behavior)
- **Scope separation:** clear statement of whether this is one shared bug cluster or multiple independent regressions.
- **Fix recommendation:** a practical repair order with lowest-risk-first steps.
- **Verification plan:** the smallest reliable set of checks to prove each fix later.

## Constraints

- No production code needs to ship in this spike.
- Prefer evidence from current repo code, tests, and local runtime behavior.
- Keep the recommendation grounded in current architecture seams:
  - per-Digimon windowed registration
  - generic windowed render engine registries
  - animation/reaction systems
  - combat-panel/UI selection state

## Research angles

### Angle 1 — Agumon Enoki render path
Trace the full Agumon VFX path:
- registration of Enoki handles/effect ids
- event/node trigger -> spawned effect id
- effect lifecycle / placement / visibility
- render-time prerequisites (camera/HDR/material/entity state)

Goal: determine why registered Agumon Enoki effects do not become visible particles.

### Angle 2 — Renamon presentation and skill-selection path
Trace the Renamon path from authored data to runtime UI:
- unit + skill asset load
- blueprint/runtime registration
- sprite-presentation registry lookup
- combat-panel pending-action / skill affordance logic

Goal: determine why Renamon data exists but the sprite and skill-selection behavior do not show up correctly.

### Angle 3 — Reaction animation and cue pipeline
Trace the feedback path for:
- hurt/dead reaction animation state
- flash cue
- camera shake
- sprite shake

Goal: determine whether these cues fail because of event production, registry lookup, animation-state transitions, or windowed render application order.

## Expected outputs from later phases

- `research/ANGLE-1.md` — Agumon Enoki findings
- `research/ANGLE-2.md` — Renamon presentation/UI findings
- `research/ANGLE-3.md` — hurt/dead + cue pipeline findings
- `RECOMMENDATION.md` — comparison, root-cause grouping, repair order, and verification plan

## Proposed decision format

A prioritized recommendation:
1. what to fix first
2. what is likely shared infrastructure vs per-Digimon data
3. what to verify headlessly vs what must be checked in the windowed build
