---
id: T01
parent: S02
milestone: M005
key_files:
  - src/windowed/render.rs
key_decisions:
  - Terminal death state is a separate DeathExiting marker component, not a new AgumonPlaybackMode variant, keeping the mode match arms closed.
  - Death-precedence is enforced via system ordering (drive_death_reactions.after(drive_hurt_reactions)), not a per-event check.
  - drive_death_reactions drives the death node un-gated by mode (death interrupts in-flight skills), unlike the idle-gated hurt driver.
duration: 
verification_result: passed
completed_at: 2026-05-26T08:38:56.355Z
blocker_discovered: false
---

# T01: Added the windowed death-event driver + DeathExiting reconciliation guard that interrupts in-flight skills and holds the dying sprite on its final death frame.

**Added the windowed death-event driver + DeathExiting reconciliation guard that interrupts in-flight skills and holds the dying sprite on its final death frame.**

## What Happened

Implemented the load-bearing first-proof seam for S02 entirely in `src/windowed/render.rs` (binary crate, windowed-gated), consuming the lib death mapping read-only with zero new lib symbols (preserves R002/R005).

1. Added a terminal marker component `DeathExiting` — a *separate* component, not a new `AgumonPlaybackMode` variant, so the `mode` match arms in `sync_agumon_mode`/`classify_same_skill_sync` stay closed (per S02-RESEARCH).

2. Added `drive_death_reactions`, mirroring `drive_hurt_reactions` but: filters events through `is_death_reaction` (`stance_reaction_for == Some(Death)`), dedups targets into a `HashSet<UnitId>`, resolves the stance snapshot via `stance_reg.resolve_snapshot`, and for each struck-and-matching sprite calls `drive_stance_reaction(Death.stance_node(), ...)` WITHOUT the `matches!(mode, Idle)` guard (death interrupts skills) and inserts `DeathExiting` via `commands`. Emits a `trace!(target: "windowed.agumon_playback", ...)` naming unit_id, reaction, node, and prior mode.

3. Reconciliation guard in `advance_agumon_presentation`: added `Option<&DeathExiting>` to the p0 query tuple. When present, `sync_agumon_mode` is skipped (a still-active barrier cannot re-`start_skill` the dying caster), and in the `advance.exited` branch `return_to_idle` is skipped (the sprite rests on its final death frame; a trace records the hold). The non-death path is unchanged.

4. Registered `drive_death_reactions` in `RenderPlugin::build` ordered `.after(drive_hurt_reactions).after(spawn_unit_sprites).after(resolve_action_system).before(advance_agumon_presentation).before(continue_suspended_timeline_system)` — the `.after(drive_hurt_reactions)` ordering enforces death-precedence.

5. Added pure helper `is_death_reaction(&CombatEventKind) -> bool` and a unit test `is_death_reaction_only_matches_unit_died` (Q7 negative test): true for `UnitDied`, false for `OnHitTaken`.

Note: `is_death_reaction` is a free fn (lib-side `resolve_stance_reaction` already enforces batch death-precedence; this task's precedence comes from system ordering). The fade-out + despawn is intentionally deferred to T02 — this task holds the sprite on the final death frame.

## Verification

Ran the slice-level verification commands. `cargo build --features windowed` finished clean (exit 0). `cargo test --features windowed` passed all suites (exit 0; e.g. windowed_only 33 passed, 0 failed). The new `is_death_reaction_only_matches_unit_died` test passes in the binary unittests (`cargo test --features windowed --bins is_death_reaction`: 1 passed). `cargo clippy --features windowed --lib` produced only the 128 pre-existing warnings, no new errors. Visible death+fade (K001) is left for manual `cargo winx` sign-off per the slice plan; auto-mode never runs the windowed binary.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 2680ms |
| 2 | `cargo test --features windowed` | 0 | pass | 20000ms |
| 3 | `cargo test --features windowed --bins is_death_reaction` | 0 | pass | 3000ms |
| 4 | `cargo clippy --features windowed --lib` | 0 | pass (128 pre-existing warnings, 0 new) | 230ms |

## Deviations

none

## Known Issues

Fade-out and despawn are not yet wired — the dying sprite holds its final death frame until T02 adds the fade driver. This is by design per the slice plan.

## Files Created/Modified

- `src/windowed/render.rs`
