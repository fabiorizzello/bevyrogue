---
id: T02
parent: S01
milestone: M005
key_files:
  - src/windowed/render.rs
key_decisions:
  - Stance reactions are driven by a dedicated event-bridge system that keeps mode=Idle and seeds the player node, distinct from skill casts which set mode=Skill — the authored hurt->idle TimeInNode transition handles the return to idle with no new asset.
  - Reactions only fire on an idle sprite (matches!(mode, Idle)); an in-flight skill cast on the struck unit is left uninterrupted (documented S01 assumption, mid-cast hurt out of scope).
  - Death classification from the lib mapping is filtered out here and deferred to S02; only Hurt is consumed.
  - Targets are deduped into a HashSet<UnitId> so a unit struck twice in one window flinches once.
  - Registered .before(advance_agumon_presentation) for explicit ordering since both systems write AgumonSprite.
duration: 
verification_result: passed
completed_at: 2026-05-26T08:16:52.950Z
blocker_discovered: false
---

# T02: Wired the windowed CombatEvent consumer to drive the struck sprite into the hurt stance node via the pure lib reaction mapping

**Wired the windowed CombatEvent consumer to drive the struck sprite into the hurt stance node via the pure lib reaction mapping**

## What Happened

Added `drive_hurt_reactions`, a new Bevy `Update` system in `src/windowed/render.rs`, that reads `MessageReader<bevyrogue::combat::events::CombatEvent>` (matching the existing windowed event-reader pattern used by `spawn_detonate_particles`) and classifies each event through the T01 pure lib function `stance_reaction_for(&event.kind)`. Events resolving to `StanceReaction::Hurt` collect their `event.target` into a deduped `HashSet<UnitId>` (a unit struck twice in one window flinches once); `Death` and all non-reaction kinds map to `None` and are filtered out, so Death is never driven here (deferred to S02 per the task contract).

For each struck target whose `AgumonSprite` exists, the system resolves the Agumon stance snapshot via `StanceGraphRegistry` (same call as `advance_agumon_presentation`) and seeds the sprite's player at `StanceReaction::Hurt.stance_node()` ("hurt") through a new `AgumonSprite::drive_stance_reaction` method that mirrors the `start_skill`/`return_to_idle` seeding pattern. Crucially `drive_stance_reaction` keeps `mode = AgumonPlaybackMode::Idle`: a stance reaction is a transient detour within the stance graph, not a skill cast. The reaction is only driven when the struck sprite is already `Idle` — an in-flight skill cast on that unit is left uninterrupted (documented S01 assumption; mid-cast hurt is out of scope). The authored `hurt -> idle` TimeInNode transition in `stance.ron` returns the sprite to idle once frames 46–52 complete, so a dropped/duplicated event degrades to "stays idle" rather than a stuck frame — no new transition asset was needed.

The system is registered in the windowed schedule with `.after(spawn_unit_sprites)` (sprite must exist), `.after(resolve_action_system)` (events emitted), and `.before(advance_agumon_presentation)` (explicit ordering so the two systems that both write `AgumonSprite` never race). A `trace!(target: "windowed.agumon_playback", ...)` line fires when a reaction is driven (struck unit_id, resolved reaction, node) and a separate trace fires on the mid-cast skip path, per the slice observability note, so a future agent can confirm the bridge fired from logs without running the windowed binary (K001). The system reads events and writes only presentation components — it never mutates combat/kernel state (R010).

## Verification

`cargo build --features windowed` compiles clean (exit 0). The new system is registered in the windowed schedule alongside `advance_agumon_presentation`. All 16 existing `windowed::render::tests` unit tests pass via `cargo test --features windowed --bin bevyrogue render::tests`. `cargo clippy --features windowed --bin bevyrogue` introduces no new warnings (the 9 reported are pre-existing collapsible_if lints in `advance_agumon_presentation` and `combat_panel/render.rs`; the two new functions at lines 143 and 907 are clean). Auto-mode cannot run the windowed binary (K001), so the visible flinch (frames 46–52 then idle) requires the user's manual sign-off in `cargo winx`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 43560ms |
| 2 | `cargo test --features windowed --bin bevyrogue render::tests` | 0 | pass | 9700ms |
| 3 | `cargo clippy --features windowed --bin bevyrogue` | 0 | pass (no new warnings; 9 pre-existing) | 21280ms |

## Deviations

none

## Known Issues

The visible flinch behavior in cargo winx is unverified by auto-mode (K001 forbids running the windowed binary); it requires manual user sign-off.

## Files Created/Modified

- `src/windowed/render.rs`
