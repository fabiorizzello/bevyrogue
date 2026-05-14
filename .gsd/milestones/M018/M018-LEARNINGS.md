---
phase: complete-milestone
phase_name: Milestone Closeout
project: bevyrogue
generated: "2026-05-14T00:00:00Z"
counts:
  decisions: 7
  lessons: 4
  patterns: 4
  surprises: 2
missing_artifacts: []
---

# M018 Learnings

## Decisions

- **BounceSelector/RepeatPolicy kept Copy so TargetShape remains Copy.** Avoided a pervasive refactor of pass-by-value call sites throughout the pipeline. Using owned enums (Vec members) would have required changing every fn signature that passes TargetShape by value.
  `Source: S03-SUMMARY.md/Key decisions`

- **DamageCurve stored on ResolvedAction at cast time, not re-read per hop.** The kernel stays zero-bias from skill data at execution time: the hop loop reads from the action struct, not from the skill book. This prevents timing bugs where a skill could theoretically be mutated mid-chain.
  `Source: S03-SUMMARY.md/Key decisions`

- **TargetableSnapshot rebuilt each hop inside the Bounce loop.** KOs shrink the candidate pool in real time — the selector always sees the world as it currently is, not as it was at cast time. Alternative (snapshot-once at cast) would silently include dead targets.
  `Source: S03-SUMMARY.md/Key decisions`

- **chain_bolt kept as inline test fixture, not added to skills.ron.** Preserves the 74-skill catalog size assertion in the test suite. Introducing a new named skill would require updating the assertion and increase the maintenance surface for a test-only artifact.
  `Source: S03-SUMMARY.md/Key decisions`

- **Pool exhaustion breaks Bounce loop silently (no OnActionFailed).** Deferred error signaling to a later slice. Explicit failure surfacing is non-trivial (requires new event variant + UI handling) and was descoped to avoid blocking the core Bounce primitive.
  `Source: S03-SUMMARY.md/Key decisions`

- **SlotIndex(u8) inserted post-spawn by apply_composition, not passed into spawn_unit_from_def.** Avoids breaking 6+ existing test callers that construct units without a slot. Keeps the spawn API stable; slot assignment is a composition-time concern, not a unit-definition concern.
  `Source: M018-VALIDATION.md/Slice Delivery Audit (S02)`

- **AdvanceTurn(u32) + DelayTurn(u32) replace TurnAdvance(i32) with cap ±50% at emission.** Separating the sign into distinct enum variants eliminates the pre-cap accumulator bug and makes DSL intent explicit. Cap/floor enforcement at the emission site means consumers never see an unclamped value.
  `Source: M018-VALIDATION.md/Success Criteria Checklist`

## Lessons

- **ResolveActorsQuery aliases must be kept in sync across turn_system/mod.rs and follow_up.rs.** SlotIndex was the 14th element added to both aliases. If they drift, the wrong field will be read silently (field position, not name, is used in tuple queries). Any new component added to ResolveActorsQuery must be added to both files simultaneously.
  `Source: S03-SUMMARY.md/Patterns established (MEM005 extended)`

- **Task-level verification recorded as 'untested' can be misleadingly narrow before downstream tests exist.** T01's summary said 'untested' because T03's integration tests hadn't been written yet. The code was correct but the task was closed prematurely. In vertically-sliced work where T01 produces DSL enums consumed by T03 tests, accept T01 as provisionally complete and note that full verification will land in T03.
  `Source: S03-SUMMARY.md/Deviations`

- **Per-hop CombatEvent emission was deferred — plan it explicitly in the follow-up slice.** Without per-hop events, the UI/log cannot observe the intermediate state of a Bounce chain. This was knowingly descoped but must be scheduled; the integration is non-trivial (requires the kernel to emit N events for a single skill cast).
  `Source: S03-SUMMARY.md/Known limitations`

- **DamageCurve::PerHop runtime length guard deferred — add an assertion in the hop kernel.** The load-time check (RON validation) guards skill authoring, but a runtime guard in the hop loop would catch dynamically constructed ResolvedActions. This is low-priority now but becomes load-bearing once Digimon blueprints emit their own ResolvedActions.
  `Source: S03-SUMMARY.md/Known limitations`

## Patterns

- **select_bounce_hop() and resolve_targets() are pure Rust fns (no ECS) taking a TargetableSnapshot slice.** This design makes multi-target resolution fully testable without a running Bevy App. All selector logic is exercised with plain unit tests; ECS is only involved in building the snapshot before the first pure call.
  `Source: S03-SUMMARY.md/Patterns established`

- **Bounce hop loop rebuilds TargetableSnapshot each hop so selectors always see current health/KO state.** The alternative (snapshot-once at cast) was rejected because it would include dead targets in subsequent hops. Rebuild cost is negligible compared to the correctness guarantee.
  `Source: S03-SUMMARY.md/Patterns established`

- **DamageCurve scaling is applied at the kernel level (pipeline.rs) by reading from ResolvedAction, not from the skill book.** The skill book is read once at cast time; the kernel is data-driven by the action struct for the rest of execution. This is consistent with the broader pattern: zero skill-book reads inside hot execution paths.
  `Source: S03-SUMMARY.md/Patterns established`

- **Resource consumption (SP/ult charge/basic-streak) is hoisted before the per-target loop.** For AoE and Bounce, resources are consumed exactly once per cast, not per target. This prevents double-spending and keeps resource economy simple regardless of fan-out width.
  `Source: M018-CONTEXT.md/Architectural Decisions (MEM002 confirmed)`

## Surprises

- **AoE(All) in roadmap/design docs is a DSL alias for the existing TargetShape::AllEnemies variant — no new variant was added.** This saved a wide diff across 11 deferred skills that already reference AllEnemies. The insight emerged during S02 when the shape enum was audited: AllEnemies already existed and only needed a documentation alias, not a new code path.
  `Source: S03-SUMMARY.md/Key decisions`

- **T01 → T02 → T03 pipeline required accepting upstream tasks provisionally before downstream verification existed.** T01 (BounceSelector/RepeatPolicy/select_bounce_hop) was closed as 'untested' because T03 (integration tests) hadn't been written yet. The correct workflow is: close T01 with a provisional note referencing the downstream test task, then retroactively verify when T03 lands. The task summary system doesn't yet model this dependency.
  `Source: S03-SUMMARY.md/Deviations`
