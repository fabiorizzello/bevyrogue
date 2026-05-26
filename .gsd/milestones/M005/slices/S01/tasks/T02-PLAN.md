---
estimated_steps: 3
estimated_files: 1
skills_used: []
---

# T02: Wire windowed EventReader<CombatEvent> consumer to drive the struck sprite into the hurt stance node

Why: This is the visible half of the slice — in `cargo winx`, the struck sprite must flinch (play hurt frames 46–52) then return to idle. Auto-mode cannot run the windowed binary (K001), so this task is proven only by green builds; the visible flinch is the user's manual sign-off.

Do: In `src/windowed/render.rs`, add a system (or extend the presentation flow) that reads `MessageReader<bevyrogue::combat::events::CombatEvent>` (the windowed code already uses `MessageReader<...CombatEvent>` — match that pattern, do not add a duplicate event source) and, for each event, calls the new lib `stance_reaction_for(&event.kind)` (import from `bevyrogue::animation`). When the resolved reaction is `Hurt`, find the `AgumonSprite` whose `unit_id == event.target` and drive its stance graph into the `hurt` node — resolve the Agumon stance snapshot via `StanceGraphRegistry` (same as `advance_agumon_presentation`) and seed the player at `StanceReaction::Hurt.stance_node()`, reusing the existing player-seeding pattern (analogous to `start_skill` / `return_to_idle`). Only drive the reaction when the struck sprite is in `AgumonPlaybackMode::Idle` (do not interrupt an in-flight skill cast on that unit — document this as the S01 assumption; mid-cast hurt is out of scope). Do NOT handle the `Death` reaction here — that is S02. Do NOT mutate any combat/kernel state from this system (R010); it only reads events and writes presentation components. Add a `trace!(target: "windowed.agumon_playback", ...)` line when a reaction is driven (struck unit_id, reaction, node) per the observability note. The existing `hurt → idle` TimeInNode transition in stance.ron returns the sprite to idle once the hurt frames complete — no new transition asset is needed.

Done when: `cargo build --features windowed` compiles clean and the new system is registered in the windowed schedule alongside `advance_agumon_presentation`; the reaction path reads events and never writes combat state.

## Inputs

- `src/windowed/render.rs`
- `src/animation/reaction.rs`
- `src/animation/mod.rs`
- `src/combat/observability/events.rs`
- `assets/digimon/agumon/stance.ron`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed
