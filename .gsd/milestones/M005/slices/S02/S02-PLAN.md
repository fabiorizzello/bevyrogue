# S02: Death reaction and field exit

**Goal:** In `cargo winx`, a unit reaching 0 HP plays its authored `death` frames (14-22) — interrupting any in-flight skill — and then fades off the field. All new code lives in the windowed crate (`src/windowed/render.rs`); the lib death mapping (`StanceReaction::Death`, `stance_reaction_for`, `resolve_stance_reaction`) is consumed read-only with zero new lib symbols, preserving R002/R005 dep-gating.
**Demo:** In cargo winx, a unit reaching 0 HP plays the death frames and fades off the field.

## Must-Haves

- A `drive_death_reactions` windowed system reads `CombatEvent`, classifies `UnitDied -> StanceReaction::Death` via the lib mapping, and seeds the struck sprite's `death` node un-gated by playback mode (death interrupts an in-flight skill), marking the sprite with a terminal `DeathExiting` component. Registered `.after(drive_hurt_reactions)` so death wins death-precedence.
- A death-marked sprite is NOT reseeded back into its skill node by `sync_agumon_mode` and is NOT returned to idle by the `advance.exited` branch (the load-bearing reconciliation guard / first-proof seam).
- When the `death` node exits (`advance.exited`), a death-marked sprite starts an alpha fade (`FadeOut` component) instead of returning to idle; `advance_death_fade` lerps `Sprite.color` alpha to 0 on the `PendingAnimationTicks` clock and despawns the entity at 0.
- `cargo test` (headless), `cargo test --features windowed`, `cargo build --features windowed`, and `cargo build` (headless) all green; dep-leak grep on changed files clean; no lib crate file modified.
- K001: visible death-frames-then-fade in `cargo winx` requires manual human sign-off — auto-mode stops at the build/test boundary.

## Proof Level

- This slice proves: Headless + windowed automated proof to the build/test boundary: new pure helpers (fade-alpha math, death-reaction filter predicate) unit-tested in the `render.rs` `#[cfg(test)] mod tests`; full `cargo test --features windowed` exercises the windowed test mod; `cargo build --features windowed` confirms the new systems compile and register; `cargo build` + dep-leak grep confirm no windowed/lib leak. Visible death+fade is K001 manual-only (auto-mode never runs the windowed binary).

## Integration Closure

The death pipeline closes the M005 event-to-reaction wiring: `UnitDied` (emitted by `combat/resolution/apply.rs` with `target` = dying unit) is bridged into the authored `death` stance node and an off-field fade, mirroring the S01 hurt bridge. The kernel/barrier and combat state are never mutated (R010); the fade is strictly downstream of presentation and cannot feed back into the deterministic kernel (R004). Post-KO overshoot observability (M002) is preserved: the sprite lingers through its death frames and a short fade rather than vanishing the instant `UnitDied` is read.

## Verification

- `trace!(target: "windowed.agumon_playback")` fires when the death driver seeds a sprite (target unit_id, resolved reaction, death node), when a mid-skill sprite is interrupted by death (proving the reconciliation guard activated), and when fade-out completes and the entity despawns — so an agent can confirm the death pipeline fired from logs without running the windowed binary, matching the S01 trace surface.

## Tasks

- [x] **T01: Death event driver + mode-reconciliation guard (first-proof seam)** `est:M`
  Why: This is the load-bearing, highest-risk seam (S02-RESEARCH First Proof). A unit can be KO'd mid-cast; the death node must interrupt the skill and must NOT be clobbered back into the skill node by sync_agumon_mode on the still-active barrier, nor restored to idle when the death node exits.
  - Files: `src/windowed/render.rs`
  - Verify: cargo build --features windowed && cargo test --features windowed 2>&1 | tail -5

- [x] **T02: FadeOut component + advance_death_fade system (off-field exit)** `est:M`
  Why: Per success criteria and research recommendation 2, a KO'd unit must fade off the field AFTER its death node completes — not instant-despawn on `UnitDied` read (that would break the M002 post-KO overshoot observability). The fade reuses the established `advance_vfx_particles` alpha/despawn-on-`PendingAnimationTicks` pattern.
  - Files: `src/windowed/render.rs`
  - Verify: cargo build --features windowed && cargo test --features windowed 2>&1 | tail -5 && cargo build 2>&1 | tail -3

- [x] **T03: Regression sweep + dep-gating closeout** `est:S`
  Why: Lock the S02 contract and prove no headless/lib leak before closing the slice (R002/R005/R016). Mirrors the S01 T03 regression task and the verify-before-complete discipline: fresh evidence, no stale claims.
  - Verify: cargo test 2>&1 | tail -3 && cargo test --features windowed 2>&1 | tail -3 && cargo build --features windowed 2>&1 | tail -3 && cargo build 2>&1 | tail -3 && git diff --name-only

## Files Likely Touched

- src/windowed/render.rs
