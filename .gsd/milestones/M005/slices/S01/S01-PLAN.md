# S01: Hurt-on-hit reaction

**Goal:** Wire the already-emitted OnHitTaken combat event to the already-authored Agumon stance `hurt` node so that, in `cargo winx`, hitting either combatant makes that sprite play the hurt frames (46–52) and then return to idle. The classification is delivered as a pure, headless-tested lib function (event kind → stance reaction) that also encodes Death classification and death-precedence, so S02 can consume the Death role without touching the lib mapping.
**Demo:** In cargo winx, hitting either combatant makes that sprite play the hurt frames then return to idle.

## Must-Haves

- A pure lib function maps CombatEventKind to an optional typed stance reaction (OnHitTaken → Hurt, UnitDied → Death, everything else → None) and a batch resolver picks Death over Hurt; headless tests cover hit, death, death-precedence, and no-op and are green under `cargo test --test animation`. In the windowed binary, render.rs reads EventReader<CombatEvent>, resolves the reaction for the struck unit (event.target), and drives that sprite's stance graph into the `hurt` node; the existing `hurt → idle` TimeInNode transition returns it to idle. `cargo build --features windowed`, `cargo test`, and `cargo test --features windowed` all stay green. Visible flinch-on-hit is K001 (manual `cargo winx` sign-off), not auto-asserted.

## Proof Level

- This slice proves: Contract (headless, CI-provable) for the mapping function; integration wiring for the windowed consumer is K001-visual (proven only by green builds, not auto-runnable).

## Integration Closure

Upstream consumed: CombatEventKind::OnHitTaken/UnitDied from src/combat/observability/events.rs (read-only, R010 — no CombatState writes); the bound Agumon stance snapshot from StanceGraphRegistry with its `hurt` (46–52) and `death` (14–22) nodes; the AnimGraphRole/AnimGraphInput purity pattern in src/animation/anim_graph.rs. New wiring: a pure reaction-mapping module in the lib (src/animation/reaction.rs) and an EventReader<CombatEvent> consumer in src/windowed/render.rs that drives the struck sprite into the hurt stance node. Remaining for the milestone after S01: S02 windowed death node + field-exit/fade (consumes the Death role this slice already classifies), S03 flash/shake/canvas damage numbers, S04/S05 enoki VFX.

## Verification

- Reaction mapping is total over the event taxonomy: non-reaction events return None explicitly (no panic, no stance corruption). The windowed consumer should emit a trace! on the `windowed.agumon_playback` target when it drives a reaction (struck unit_id, resolved reaction, node) so a future agent can confirm the bridge fired from logs without running the windowed binary. A dropped/duplicated event degrades to "stays idle" via the existing hurt→idle transition rather than a stuck frame.

## Tasks

- [x] **T01: Add pure stance-reaction mapping function with headless hit/death/precedence/no-op tests** `est:1h`
  Why: The milestone requires the event-to-stance-reaction mapping to be a pure, deterministic lib function (mirroring the R009 AnimGraphInput purity seam) so it has a headless contract instead of living only behind K001 in the windowed binary; integration tests link only against the lib crate, not the windowed binary.
  - Files: `src/animation/reaction.rs`, `src/animation/mod.rs`, `tests/animation/stance_reaction_mapping.rs`, `tests/animation.rs`
  - Verify: cargo test --test animation stance_reaction_mapping

- [x] **T02: Wire windowed EventReader<CombatEvent> consumer to drive the struck sprite into the hurt stance node** `est:2h`
  Why: This is the visible half of the slice — in `cargo winx`, the struck sprite must flinch (play hurt frames 46–52) then return to idle. Auto-mode cannot run the windowed binary (K001), so this task is proven only by green builds; the visible flinch is the user's manual sign-off.
  - Files: `src/windowed/render.rs`
  - Verify: cargo build --features windowed

- [x] **T03: Regression sweep: headless and windowed test suites green, no windowed dep leak** `est:30m`
  Why: The milestone contract requires the full headless suite, the windowed test suite, and the windowed build to stay green after the reaction wiring, and that the new lib mapping introduces no windowed/render dependency into the headless build (R002/R005).
  - Verify: cargo test --features windowed

## Files Likely Touched

- src/animation/reaction.rs
- src/animation/mod.rs
- tests/animation/stance_reaction_mapping.rs
- tests/animation.rs
- src/windowed/render.rs
