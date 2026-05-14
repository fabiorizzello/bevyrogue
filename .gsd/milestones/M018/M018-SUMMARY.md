---
id: M018
title: "AdvanceTurn/DelayTurn split + TargetShape resolver expansion"
status: complete
completed_at: 2026-05-13T22:06:13.959Z
key_decisions:
  - BounceSelector/RepeatPolicy kept Copy so TargetShape remains Copy — avoids pervasive refactor of pass-by-value call sites
  - DamageCurve stored on ResolvedAction at cast time, not re-read per hop — kernel zero-bias from skill data at execution time
  - TargetableSnapshot rebuilt each hop so KOs shrink the candidate pool in real time (snapshot-once-at-cast rejected)
  - chain_bolt kept as inline test fixture (not added to skills.ron) to preserve the 74-skill catalog size assertion
  - Bounce pool exhaustion breaks loop silently — OnActionFailed deferred to a later slice
  - SlotIndex(u8) inserted post-spawn by apply_composition, not passed into spawn_unit_from_def — keeps spawn API stable for 6+ test callers
  - AdvanceTurn(u32) + DelayTurn(u32) replace TurnAdvance(i32) — separate sign into distinct variants, cap/floor at emission
key_files:
  - src/combat/av.rs
  - src/combat/action_query.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - src/combat/state.rs
  - src/combat/unit.rs
  - src/data/skills_ron.rs
  - src/combat/follow_up.rs
  - tests/target_shape_bounce_chain.rs
  - tests/target_shape_blast_spillover.rs
  - tests/target_shape_aoe_all_order.rs
  - tests/slot_index_tiebreak.rs
  - tests/turn_advance_split.rs
  - assets/data/skills.ron
  - src/bin/combat_cli.rs
lessons_learned:
  - ResolveActorsQuery aliases in turn_system/mod.rs and follow_up.rs must be kept in sync — any new component added must be added to both files simultaneously (field position, not name, is used in tuple queries)
  - In vertically-sliced tasks where T01 produces DSL code and T03 writes integration tests, close T01 as provisionally complete with a downstream reference note rather than marking 'untested'
  - Per-hop CombatEvent emission was deferred — must be planned explicitly in a follow-up slice, not assumed to appear incidentally
  - DamageCurve::PerHop runtime length guard deferred — add before Digimon blueprints emit their own ResolvedActions
  - AoE(All) was an alias for an existing AllEnemies variant — auditing existing enum variants before adding new ones saved a wide refactor diff
---

# M018: AdvanceTurn/DelayTurn split + TargetShape resolver expansion

**Shipped two foundational combat primitives — AdvanceTurn/DelayTurn split with ±50% cap and TargetShape resolver expansion (Blast, AoE(All), Bounce(N)) — enabling deterministic multi-target skill execution across the full roster.**

## What Happened

M018 delivered its two stated foundation primitives across three slices with zero regressions on the M017 baseline.

**S01 — AdvanceTurn/DelayTurn split:** The existing `TurnAdvance(i32)` was replaced by two typed enum variants `AdvanceTurn(u32)` and `DelayTurn(u32)`. Cap enforcement of ±50% is applied at emission; consumers of the event bus never see an unclamped value. TempoResistance curve was integrated at the same emission site. A CLI scenario `advance-delay-cap` demonstrates step-by-step JSONL output showing cap enforcement (80→50) and floor clamp (delta=0 at AV=0). All M017 Slowed-related tests (status_slowed_delay, tempo_resistance, turn_advance_split) were migrated to the new variants and remain green.

**S02 — Blast/AoE resolver + SlotIndex:** `SlotIndex(u8)` was introduced as an ECS component assigned post-spawn by `apply_composition`, keeping the spawn API stable for 6+ existing test callers. A pure `resolve_targets(TargetableSnapshot)` function (no ECS) handles Blast (primary target + slot_index ±1 adjacents) and AllEnemies (AoE(All) alias). JSONL output for the `aoe-blast` CLI scenario is byte-for-byte identical across 10 seeded runs. Six integration tests cover Blast spillover, AoE ordering, and SlotIndex tiebreak.

**S03 — BounceSelector + RepeatPolicy + hop loop:** `BounceSelector` and `RepeatPolicy` enums (both Copy, preserving TargetShape's Copy bound) were added to the DSL. `TargetShape::Bounce` was migrated from a tuple variant to a struct variant. A pure `select_bounce_hop(TargetableSnapshot)` dispatcher handles NextSlot and LowestHp selectors. The Bounce hop loop in `pipeline.rs` rebuilds TargetableSnapshot each hop so KO'd units are excluded from the candidate pool in real time. DamageCurve scaling is read from ResolvedAction (cast at skill-cast time), not from the skill book. Four integration tests cover the full Bounce chain including mid-chain KO.

**Cross-slice integration:** S01 primitives (AdvanceTurn/DelayTurn applicators, pipeline.rs infrastructure) were consumed by S02 and S03 without modification. S02's SlotIndex component and resolve_targets helper were consumed directly by S03's Bounce hop loop and select_bounce_hop dispatcher. The full test suite ran 201 tests with 0 failures.

**Known deferred items (not blocking):** Per-hop CombatEvent emission for UI/log observability; OnActionFailed on Bounce pool exhaustion (currently silent truncation); DamageCurve::PerHop runtime length guard in the kernel hop loop.

## Success Criteria Results

- **Advance/Delay CLI scenario with cap & floor** ✅ CLI `advance-delay-cap` scenario runs deterministically; JSONL shows cap enforcement (80→50) and floor clamp (delta=0 at AV=0); exit 0.
- **Turn order recalculation with AdvanceTurn/DelayTurn** ✅ `TurnAdvance(i32)` fully replaced by `AdvanceTurn(u32)` + `DelayTurn(u32)` with cap ±50% at emission; 201 tests green; rg legacy sweep clean.
- **M017 Slowed regression maintained** ✅ `status_slowed_delay.rs` and `tempo_resistance.rs` both green; AV outcome preserved; event variant migrated to `DelayTurn{30}`.
- **Blast targeting with spillover to adjacents** ✅ CLI `aoe-blast` scenario resolves Blast (primary + slot_index ±1); JSONL deterministic across 10 runs (byte-for-byte identical); edge slots and KO'd adjacents handled.
- **AoE(All) with slot_index tie-break ordering** ✅ AllEnemies targets resolved in slot_index ascending order; fixture test `target_shape_aoe_all_order` passes; per-target damage applied in order.
- **Bounce(N) multi-hop with mid-chain KO** ✅ Bounce hop loop rebuilds TargetableSnapshot each hop; KO'd units excluded from candidate pool in subsequent hops; integration test `bounce_next_slot_no_repeat_falloff_ko_mid_chain` passes.
- **Extended selectors (NextSlot, LowestHp)** ✅ BounceSelector enum added; pure `select_bounce_hop()` dispatcher; both selectors exercised in integration tests (4/4 pass).
- **Zero regressions on existing test suite** ✅ `cargo test` full suite shows 201 total tests, 0 failures; all M017 tests remain green.

## Definition of Done Results

- **All slices complete** ✅ S01, S02, S03 all status=complete in DB; all task checkboxes ticked.
- **Summaries exist** ✅ S03-SUMMARY.md present and detailed; S01 and S02 summaries confirmed in validation artifact.
- **Integration works** ✅ Cross-slice boundaries all PASS per validation: S01→S02 event bus + pipeline infrastructure; S02→S03 SlotIndex + resolve_targets consumed by Bounce hop loop.
- **Validation artifact** ✅ M018-VALIDATION.md present with verdict: pass, all success criteria checked, all verification classes PASS.
- **Determinism gate** ✅ CLI `aoe-blast` output byte-for-byte identical across 10 seeded runs; `advance-delay-cap` JSONL stable.

## Requirement Outcomes

No formal REQUIREMENTS.md requirement IDs were advanced or invalidated in M018 (requirements were assessed against the two core primitives stated in M018-CONTEXT.md). All materially testable acceptance criteria are COVERED:

- Primitive 1 (AdvanceTurn/DelayTurn split): COVERED — S01 delivers full implementation with 201 tests green and CLI scenario evidence.
- Primitive 2 (TargetShape expansion — Blast, AoE, Bounce): COVERED — S02/S03 deliver all three shapes via pure resolve_targets() and select_bounce_hop() with 10 integration tests and deterministic CLI evidence.
- No requirements were moved to Deferred, Blocked, or Out of Scope.

## Deviations

T01 (BounceSelector/RepeatPolicy/select_bounce_hop) was closed with verification status 'untested' because T03's integration tests had not been written yet at T01 close time. The code produced by T01 was exercised by T03's integration tests (target_shape_bounce_chain, 4/4 pass) — no source gaps were found. The deviation was in the task-completion workflow, not in the implementation.

## Follow-ups

- Per-hop CombatEvent emission for Bounce (UI/log observability of intermediate hop state) — plan as explicit slice in next targeting milestone
- OnActionFailed on Bounce pool exhaustion (currently silent truncation) — requires new event variant + UI handling
- DamageCurve::PerHop runtime length guard in the kernel hop loop — add before Digimon blueprints emit dynamic ResolvedActions
- Consider M019 (DR pipeline + Heal/Cleanse Effects) or M020 (reactive event variants) as next milestone
