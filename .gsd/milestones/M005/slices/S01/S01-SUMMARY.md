---
id: S01
parent: M005
milestone: M005
provides:
  - StanceReaction::Hurt and StanceReaction::Death lib types for S02 Death reaction
  - stance_reaction_for and resolve_stance_reaction pure functions for any future reaction consumer
  - drive_hurt_reactions windowed system establishing the event-bridge registration pattern for S03 flash/shake consumer
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - stance_reaction_for uses a fully-enumerated explicit match (no catch-all) over CombatEventKind — future event variants force a compile error in reaction.rs, requiring deliberate classification rather than silent None
  - drive_hurt_reactions only fires when AgumonSprite mode is Idle; mid-cast hurt interrupt is explicitly out of scope for S01 and documented as a known assumption
  - Death classification is produced by the lib mapping but filtered out in the windowed bridge (drive_hurt_reactions) and fully deferred to S02 — clean separation so S02 can consume the Death role without touching reaction.rs
  - Targets are deduped into a HashSet<UnitId> so a unit struck twice in one event window flinches once rather than double-firing the player seed
  - drive_hurt_reactions keeps mode=Idle (not mode=Skill) when seeding the hurt node — a stance reaction is a transient detour within the stance graph, not a skill cast, so the authored hurt→idle TimeInNode transition handles the return with no new asset
patterns_established:
  - Pure lib reaction mapping pattern: CombatEventKind → Option<StanceReaction> via fully-enumerated match, with a batch resolver encoding domain-level precedence rules (death beats hurt) — same purity seam as AnimGraphInput (R009), composable for future reaction kinds
  - Windowed event-bridge pattern: MessageReader<CombatEvent> → classify via lib fn → deduplicate targets → drive presentation component — mirrors spawn_detonate_particles bridge; no combat state mutations (R010)
  - Trace logging on windowed.agumon_playback target for both reaction-driven and mid-cast-skip paths, enabling future agent diagnostics without running the windowed binary
observability_surfaces:
  - trace!(target: "windowed.agumon_playback") fires on every hurt reaction driven (struck unit_id, resolved reaction, node) — confirms bridge fired from logs without running windowed binary
  - trace!(target: "windowed.agumon_playback") fires on mid-cast skip path — confirms the idle-guard activated
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-26T08:21:59.053Z
blocker_discovered: false
---

# S01: Hurt-on-hit reaction

**Wired the CombatEvent→hurt-stance pipeline: a pure headless-tested lib mapping (reaction.rs) + a windowed Bevy system (drive_hurt_reactions) that flinches the struck sprite into frames 46–52 and returns it to idle.**

## What Happened

S01 delivered the hurt-on-hit reaction pipeline in two clean layers, with a regression sweep confirming no regressions anywhere.

**T01 — Pure lib mapping (`src/animation/reaction.rs`):**  
Defined a closed `StanceReaction` enum (`Hurt`, `Death`) and three pure functions: `stance_reaction_for(&CombatEventKind) -> Option<StanceReaction>` maps `OnHitTaken → Hurt`, `UnitDied → Death`, and every other variant → None via a fully-enumerated explicit match with no catch-all (future event variants force a compile error here rather than silently mapping to None). `resolve_stance_reaction` encodes death-precedence: returns Death on first Death in the batch, else Hurt if any Hurt present, else None. `StanceReaction::stance_node()` returns `NodeId("hurt")` / `NodeId("death")` matching authored node names in `stance.ron`. Registered as `pub mod reaction` in `src/animation/mod.rs`. Four headless integration tests in `tests/animation/stance_reaction_mapping.rs` cover hit→Hurt, death→Death, mixed-batch death-precedence, and non-reaction+empty-batch→None — all linking only against the lib crate, zero windowed deps.

**T02 — Windowed bridge (`src/windowed/render.rs`):**  
Added `drive_hurt_reactions`, a Bevy `Update` system that reads `MessageReader<bevyrogue::combat::events::CombatEvent>`, classifies each event via `stance_reaction_for`, and collects struck targets into a `HashSet<UnitId>` (dedup: a unit struck twice in one window flinches once). Death and non-reaction events are filtered out here — Death classification is deferred to S02. For each struck target whose `AgumonSprite` exists and is in `Idle` mode, the system seeds the stance player at the "hurt" node via `AgumonSprite::drive_stance_reaction`, keeping `mode = Idle` so the authored `hurt → idle` TimeInNode transition in `stance.ron` returns the sprite to idle once frames 46–52 complete. Mid-cast sprites are intentionally left uninterrupted (documented S01 assumption). Registered `.after(spawn_unit_sprites)`, `.after(resolve_action_system)`, `.before(advance_agumon_presentation)` to avoid races. Emits `trace!(target: "windowed.agumon_playback", ...)` on both the reaction-driven and mid-cast-skip paths so a future agent can confirm the bridge fired from logs without running the windowed binary.

**T03 — Regression sweep:**  
`cargo test` (headless), `cargo test --features windowed`, and `cargo build --features windowed` all exit 0. Direct grep confirmed `reaction.rs` has no windowed/wgpu/winit/egui symbols — only a doc-comment mention. No regressions introduced; the 4 new mapping tests were left intact.

**Closeout verification (fresh):** `cargo test --test animation` → 119 passed; `cargo test --features windowed` → 33 passed; dep-leak grep → CLEAN. Visible flinch (frames 46–52 then idle) is K001 — requires manual `cargo winx` sign-off by the user.

## Verification

Fresh closeout sweep via gsd_exec (all exit 0):
1. `cargo test --test animation` → 119 passed, 0 failed (headless suite green; all 4 stance_reaction_mapping tests: hit_maps_to_hurt_node, death_maps_to_death_node, death_takes_precedence_over_hurt_in_batch, non_reaction_kinds_and_empty_batch_map_to_none — all passing)
2. `cargo test --features windowed` → 33 passed, 0 failed (full windowed regression sweep green)
3. Dep-leak check: grep of `src/animation/reaction.rs` for windowed/wgpu/winit/egui/bevy_render/cfg(feature → only a doc-comment reference found, no actual windowed symbols — CLEAN; confirms R002/R005 invariant holds
4. Wiring structural check: `drive_hurt_reactions` present at render.rs:907; `stance_reaction_for` imported (line 16) and called (line 918); `StanceReaction::Hurt` used at lines 918, 930, 941, 952
5. All 4 test function signatures confirmed present in tests/animation/stance_reaction_mapping.rs
K001 (visible flinch in cargo winx) is not auto-asserted; requires manual user sign-off.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

K001: visible flinch (frames 46–52 then idle) requires manual cargo winx sign-off; auto-mode cannot run the windowed binary. Pre-existing non-fatal compiler warning (unused import BeatEdge) under --features windowed; not introduced by this slice.

## Follow-ups

S02 (Death reaction and field exit) can consume StanceReaction::Death directly from reaction.rs — the lib mapping already classifies UnitDied→Death and the batch precedence resolver is in place. S03 (flash/shake/canvas damage numbers) can register an additional windowed event-bridge system following the drive_hurt_reactions pattern.

## Files Created/Modified

- `src/animation/reaction.rs` — 
- `src/animation/mod.rs` — 
- `tests/animation/stance_reaction_mapping.rs` — 
- `tests/animation.rs` — 
- `src/windowed/render.rs` — 
