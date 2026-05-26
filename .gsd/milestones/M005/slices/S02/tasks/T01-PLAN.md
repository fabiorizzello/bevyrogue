---
estimated_steps: 9
estimated_files: 1
skills_used: []
---

# T01: Death event driver + mode-reconciliation guard (first-proof seam)

Why: This is the load-bearing, highest-risk seam (S02-RESEARCH First Proof). A unit can be KO'd mid-cast; the death node must interrupt the skill and must NOT be clobbered back into the skill node by sync_agumon_mode on the still-active barrier, nor restored to idle when the death node exits.

Do: All edits in `src/windowed/render.rs`.
1. Add a terminal marker component `#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)] struct DeathExiting;` (a separate component, NOT a new `AgumonPlaybackMode` variant — keeps the `mode` match arms in `sync_agumon_mode`/`classify_same_skill_sync` closed; per research).
2. Add `fn drive_death_reactions(mut commands: Commands, events: MessageReader<bevyrogue::combat::events::CombatEvent>, stance_reg: Res<StanceGraphRegistry>, graphs: Res<Assets<AnimGraph>>, mut sprites: Query<(Entity, &mut AgumonSprite)>)` mirroring `drive_hurt_reactions` (render.rs:907) but: filter `stance_reaction_for(&event.kind) == Some(StanceReaction::Death)`; dedup targets into `HashSet<UnitId>`; resolve the stance snapshot via `stance_reg.resolve_snapshot(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()), &graphs)`; for each struck-and-matching sprite, call `sprite.drive_stance_reaction(StanceReaction::Death.stance_node(), stance_graph.clone())` WITHOUT the `matches!(sprite.mode, Idle)` guard (death interrupts skills), and `commands.entity(entity).insert(DeathExiting)`. Emit a `trace!(target: "windowed.agumon_playback", ...)` naming the unit_id, reaction, node, and prior mode.
3. Reconciliation guard: in `advance_agumon_presentation` (render.rs:558) add `Option<&DeathExiting>` to the p0 query tuple `Query<(&mut AgumonSprite, &mut Sprite, &Transform)>`. When the marker is present for a sprite, skip the `sync_agumon_mode(...)` call (render.rs:627) so a still-active barrier cannot re-`start_skill` the dying sprite. In the `if advance.exited` branch (render.rs:870), when the marker is present, do NOT call `return_to_idle` — leave the sprite resting on its final death frame (the fade is wired in T02). Keep the existing non-death path unchanged.
4. Register the system in `RenderPlugin::build` (render.rs:291) as `drive_death_reactions.after(drive_hurt_reactions).after(spawn_unit_sprites).after(resolve_action_system).before(advance_agumon_presentation).before(continue_suspended_timeline_system)` — ordering it AFTER the hurt driver enforces death-precedence (research watch-out: death must win when a target is both struck and killed in one window).
5. Add a small pure helper `fn is_death_reaction(kind: &bevyrogue::combat::events::CombatEventKind) -> bool { stance_reaction_for(kind) == Some(StanceReaction::Death) }` and unit-test it in the existing `#[cfg(test)] mod tests` (render.rs:1603) following the `entered_node_only_reports_actual_node_changes` / `decrement_vfx_ttl_saturates_at_zero` free-fn-test pattern: assert it is true for a `UnitDied` kind and false for an `OnHitTaken` kind (Q7 negative test: a non-death event never enters the death pipeline).

Done-when: `cargo build --features windowed` and `cargo test --features windowed` are green; the new `is_death_reaction` test passes; the dying sprite is seeded into the death node and marked, sync is skipped for it, and the exited branch no longer idle-restores it. Skills: rust-skills, tdd, bevy-ecs-expert.

Threat surface (Q3): none new — reads the internal `CombatEvent` bus, mutates only presentation components/entities (R010); never touches `CombatState`/kernel/barrier. Failure modes (Q5): a dropped/duplicated `UnitDied` degrades to 'stays on death frame' (idempotent marker insert, no stuck-frame panic); a death event for a unit with no live `AgumonSprite` is a no-op (filtered by the query). Determinism (R004): the driver is windowed-only and produces no event-observable kernel state.

## Inputs

- `src/windowed/render.rs`
- `src/animation/reaction.rs`
- `assets/digimon/agumon/stance.ron`
- `.gsd/milestones/M005/slices/S02/S02-RESEARCH.md`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed && cargo test --features windowed 2>&1 | tail -5

## Observability Impact

Adds trace! on death-seed (unit_id, reaction, death node, prior mode) and implicitly proves the interrupt path: a sprite seeded into death while mode=Skill confirms the un-gated driver fired and the sync guard held.
