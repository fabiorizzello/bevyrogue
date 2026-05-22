# M002 Producer → Consumer Boundary Map

Milestone M002 / Slice S09 / Task T05.

This is the explicit producer→consumer boundary map required by M002's hard
acceptance. It states, for each cross-subsystem seam in the milestone, **who
produces what contract** and **who consumes it**, the directionality
constraint, and the **on-disk test** that enforces the boundary. Every cited
test path resolves in this repository (see the Verification section at the
bottom for the path-existence check).

## How to read this

A "boundary" here is a one-directional data contract between two subsystems
where the consumer must **not** reach back through the contract into the
producer's internals. The recurring M002 invariant is *gameplay numbers stay in
the kernel; presentation receives opaque, closed-enum commands and read-only
projections.* Each row names the enforcing test so the contract is
machine-checkable rather than a prose promise.

## Boundary table

| # | Producer subsystem | Contract / data type | Consumer subsystem | Direction & constraint | Enforcing test (on disk) |
|---|--------------------|----------------------|--------------------|------------------------|--------------------------|
| 1 | **Kernel** (combat turn system, `skills.ron` gameplay numbers — damage, toughness, meter) | Compiled `SkillTimeline` resolved from typed gameplay data | **Timeline → anim-graph** (opaque presentation commands) | One-way: gameplay numbers are resolved kernel-side; the timeline carries only opaque presentation commands downstream, never raw gameplay literals back up. | `tests/timeline/boundary_contract.rs` (`turn_advance_then_ultimate_consumes_meter_and_damages_target` — proves meter/damage are consumed kernel-side from the timeline contract) |
| 2 | **Animation player** (cue emission at beat boundaries) | Cue barrier signal / `CombatBeatId` projection | **Kernel** (cue-barrier resume → turn pipeline) | Two-clock handshake: the player blocks the kernel at a cue barrier and the kernel resumes only on cue release; the kernel never reads animation internal frame state. | `tests/windowed_only/phase_strip_readonly.rs` (`phase_strip_projects_latest_beat_without_mutating_combat_state`, `combat_event_reader_seam_is_read_only`) |
| 3 | **`CombatEvent`** (kernel event stream) | Read-only `CombatEvent` records | **§9 UI / HUD** (phase strip, presentation metadata) | Read-only: UI/HUD subscribes to the event stream and projects it; it must not mutate combat state or branch gameplay on presentation metadata. | `tests/preview_ai/presentation_metadata_boundary.rs` (presentation metadata cannot alter kernel outcome) and `tests/windowed_only/phase_strip_readonly.rs` (`phase_strip_ignores_non_beat_events_and_empty_updates`) |
| 4 | **`SkillGraphRegistry`** (skill-id → anim-graph mapping) | Opaque skill-id keyed graph lookup with deterministic `InstantFallback` | **Windowed player** (graph resolution at spawn) | One-way lookup: the player resolves a graph by id and degrades to an instant fallback with diagnostics on a miss. **M003+ constraint to lift:** the skill-id→graph wiring still relies on hardcoded constants rather than fully data-driven registration. | `tests/animation/skill_graph_mapping_extensibility.rs` (`skill_registry_supports_multiple_distinct_graph_ids`, `unregistered_skill_id_returns_instant_fallback_with_diagnostic`, `stance_graph_snapshot_entry_is_non_empty_for_return_to_idle_boundary`) |
| 5 | **VFX seam** (opaque `ParticleId` in `SpawnParticle { name, origin, motion }`) | Opaque `ParticleId` + closed `VfxLocus` / `VfxMotion` enums | **Windowed VFX consumer** (validate-only) | One-way, validate-only: the consumer round-trips the opaque id and closed enums; unknown variants must fail to deserialize and **no numeric gameplay payload** may cross the seam. | `tests/animation/vfx_handle_seam.rs` (`spawn_particle_ron_round_trips_losslessly`, `unknown_vfx_locus_variant_fails_to_deserialize`, `unknown_vfx_motion_variant_fails_to_deserialize`, `spawn_particle_has_no_numeric_gameplay_payload`) |

## Reader test (no-M002-context check)

A reader with no prior M002 context can use this table to answer: *"If I touch
subsystem X, what contract am I allowed to change, and which test will fail if I
break the boundary?"* The unifying rule across all five rows: **gameplay
numbers live in the kernel; everything downstream of the kernel receives opaque,
closed-enum commands or read-only projections, and never feeds gameplay state
back upstream.** Rows 1–2 cover the kernel↔timeline↔player loop, row 3 covers the
read-only UI/HUD projection, and rows 4–5 cover the two opaque-id seams
(skill-graph mapping and VFX) consumed by the windowed app.

## Known constraint carried to M003+

Row 4's `SkillGraphRegistry` mapping currently uses **hardcoded constants** for
the skill-id→graph association. The boundary itself (opaque id + instant
fallback) is enforced, but lifting the wiring to fully data-driven registration
is deferred to M003+. This is the one open extensibility item the boundary map
flags for future milestones.

## Verification

The cited test paths are checked for on-disk existence and citation by the T05
verification command in `S09-CLOSEOUT.md`. All five test files exist and each is
cited in the table above.
