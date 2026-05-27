# Angle 3 — Reaction animation and cue pipeline

## Question

Why do hurt/dead animations and hit-feedback cues (flash, camera shake, sprite shake) appear not to work correctly?

## Findings

### 1. The feedback pipeline is wired in the expected order

`RenderPlugin` orders the relevant systems like this:
- resolve combat
- observe hit feedback / observe camera shake / drive hurt / drive death
- advance presentation
- apply camera shake

That ordering is correct for the intended model:
- `OnHitTaken` arms flash/shake/camera-shake first
- presentation consumes the armed state on the next presentation tick
- camera transform is written after the single decay site

So there is **no obvious ordering bug** in the current top-level render schedule.

### 2. Cue registration exists

Agumon registers the three global cue ids in the `CueRegistry`:
- `hit_flash`
- `hit_shake`
- `camera_impact`

The render path reads those cues parametrically rather than through hardcoded constants.

This means the issue is **not** “the cue ids were never registered” for the current Agumon-backed windowed app.

### 3. Hurt reactions are intentionally suppressed while a unit is mid-skill

`drive_hurt_reactions` contains an explicit scope guard:
- only react if the struck sprite is in `DigimonPlaybackMode::Idle`
- if the unit is already mid-cast, hurt is skipped on purpose

So one important part of “hurt anim non funziona bene” is not a mystery regression — it is the currently authored behavior.

Practical implication:
- if a player expects a struck unit to visibly flinch during a cast, the current implementation will not do that
- the behavior will look intermittent or “broken” whenever hits land during active skills

### 4. Death reactions have broader coverage than hurt

`drive_death_reactions` does interrupt in-flight skills and pushes the sprite into the `death` node, followed by `FadeOut` once the death node exits.

So the code intends:
- hurt = idle-only
- death = unconditional interrupt

That means “dead anim non funziona bene” is less likely to be the same bug as the hurt case. It may instead be:
- missing sprite/presentation for the affected unit, or
- dissatisfaction with how quickly the death node exits into fade-out

### 5. Renamon’s missing sprite can make all Renamon reaction feedback look absent

From Angle 2, Renamon currently fails to spawn because its stance graph is not readable when sprite spawn runs.

That has a direct downstream effect:
- no sprite entity
- no sprite-based hurt/death presentation
- no visible sprite flash
- no visible sprite shake

So at least part of the reaction/cue complaint can be explained as a **secondary symptom of the Renamon sprite-spawn failure**, not a separate independent cue bug.

### 6. Current automated tests prove the pure projection layer, not the live visual read

Fresh run:
- `cargo test --features windowed --test windowed_only`

Relevant passing tests:
- `windowed_hit_feedback::*`
- `digimon_sprite_cue_dispatch::*`

What this proves:
- `OnHitTaken` arms the flash/shake resources
- decay math is correct
- camera-shake/writeback seam exists in source
- cue registry tokens are present

What it does **not** prove:
- that a live struck sprite visibly flashes/shakes during a real cast sequence
- that the camera shake is perceptible in the current live windowed build
- that the observed user complaint is fully covered by headless/projection tests

## Best current interpretation

There are likely **two layers** to the reaction complaint:

1. **Real intended limitation**
   - hurt does not interrupt in-flight skills by design
   - this can absolutely read as “hurt animation is not working well”

2. **Secondary fallout from other presentation bugs**
   - Renamon missing sprite means Renamon reactions/cues cannot render visibly
   - if live action/skill flow is impaired elsewhere, the cue systems may never get a fair visible read in practice

## Evidence

- `src/windowed/render.rs::RenderPlugin::build`
- `src/windowed/render.rs::drive_hurt_reactions`
- `src/windowed/render.rs::drive_death_reactions`
- `src/windowed/render.rs::advance_death_fade`
- `src/windowed/digimon/agumon/mod.rs::register_agumon_cues`
- `src/ui/hit_feedback.rs`
- `cargo test --features windowed --test windowed_only` → 67 passed
- filtered runtime logs from the windowed validation soak (Renamon sprite spawn deferred)

## Conclusion

The reaction/cue path does not currently point to one clean standalone root cause.

Most likely:
- **hurt behavior** is partly a design limitation (idle-only reaction)
- **Renamon-visible reaction failure** is downstream of the missing Renamon sprite
- **flash/camera shake/sprite shake** are structurally wired, but still lack a cast-driven runtime proof that they visibly read the way the user expects

## Recommended next verification

1. Decide whether hurt should keep its current idle-only rule or be allowed to interrupt some skill states.
2. Fix the Renamon sprite-spawn issue first; otherwise Renamon feedback cannot be meaningfully judged.
3. Add a live windowed smoke proof for one resolved hit that asserts:
   - target sprite exists
   - hurt/death node transition occurs when expected
   - flash/shake resources arm
   - camera shake resource arms

## Confidence

**Medium.** The code explains why the behavior can feel wrong, but the current automated coverage still stops short of proving the final live visual read.