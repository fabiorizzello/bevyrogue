# S01: Hurt-on-hit reaction — UAT

**Milestone:** M005
**Written:** 2026-05-26T08:21:59.054Z

# S01 UAT: Hurt-on-hit reaction

**UAT Type:** K001 — manual `cargo winx` sign-off (auto-mode cannot execute the windowed binary)

## Preconditions

1. `cargo winx` (alias for `cargo run --features windowed`) launches without error and the two-sprite encounter is visible
2. At least one side can initiate a skill that deals damage (e.g. Sharp Claws)

## Steps and Expected Outcomes

| Step | Action | Expected Outcome |
|------|--------|-----------------|
| 1 | Launch `cargo winx` | Window opens; both Agumon sprites visible in idle stance |
| 2 | Trigger a skill from the player Agumon (Sharp Claws) | Skill animation plays; damage event emits on the impact frame |
| 3 | Observe the **opponent** sprite on the impact frame | Sprite transitions immediately into hurt frames (46–52) |
| 4 | Observe the opponent sprite ~0.5s after impact | Sprite returns to idle via the authored `hurt → idle` TimeInNode transition; no stuck frame |
| 5 | Let the opponent counter-attack (if applicable) | Player sprite also flinches into hurt frames 46–52 then returns to idle |
| 6 | Trigger two rapid strikes in the same frame (if possible) | Struck unit flinches **once**, not twice (dedup guard) |

## Edge Cases

- **Mid-cast protection:** If the struck unit is currently playing a skill animation when struck, the hurt reaction is suppressed; the skill plays to completion then idle resumes
- **Death not triggered here:** A unit at 0 HP after a hit does NOT play the hurt animation through this path (Death classification is filtered; S02 will handle it)
- **No stuck frames:** Dropped or duplicated events degrade to "stays idle" — the sprite never freezes on frame 52

## Not Proven By This UAT

- Death reaction and field exit (deferred to S02)
- Hit flash, sprite shake, and canvas floating damage numbers (deferred to S03)
- bevy_enoki VFX for Agumon skills (deferred to S04/S05)
- Frame-precise timing of the hurt animation (K001 is a visual pass/fail, not a frame-counter assertion)
- Behavior under headless/CI execution (proven by the 4 stance_reaction_mapping tests in T01 and the green cargo test suite)
