---
estimated_steps: 18
estimated_files: 3
skills_used: []
---

# T03: Pipeline wiring (Single/SelfOnly + AllAllies fan-out) + tests/cleanse_effect.rs integration suite

Wire apply_cleanse_only into the pipeline and add the integration test file. Two pipeline sites and one new test file.

Pipeline wiring (src/combat/turn_system/pipeline.rs):

1) Single/SelfOnly cleanse hook — beside the existing `status_to_apply` mutation site (~pipeline.rs:1722). When `resolved.cleanse_count.is_some()` and the outcome from apply_effects succeeded (no failure, action not aborted) and the defender is alive at this point, fetch the defender's StatusBag (already in scope as defender_bag in the status_to_apply block — reuse the same mut-borrow path; mirror the fresh-bag fallback at pipeline.rs:1742 for the missing-component case, in which case emit OnCleansed { kinds: [] }). Call apply_cleanse_only(&resolved, &mut bag, defender_alive) and forward its events to the combat event bus.

2) AllAllies cleanse fan-out — extend the existing AllAllies branch at pipeline.rs:340-358 (added in S02 for heal). Today that branch only mut-borrows def_unit for heal; extend the actors query row to also mut-borrow def_bag (StatusBag). At dispatch time, decide per-skill which helper to call:
   - If resolved.heal_pct > 0 → apply_heal_only (existing path).
   - Else if resolved.cleanse_count.is_some() → apply_cleanse_only.
   - The T01 validator forbids mixed Heal+Cleanse, so this either-or contract is sound.

Handle the MEM001 gotcha: follow_up.rs maintains a local ResolveActorsQuery. We are NOT adding a new component (StatusBag already exists in resolution and pipeline queries), so no tuple-arity change is expected in follow_up.rs's local query. Verify cargo check stays clean; if a tuple-arity mismatch surfaces, update the follow_up.rs local query to match.

New test file (tests/cleanse_effect.rs) — apply_effects direct-call pattern (per MEM003); 8 deterministic cases, functional names (no s##_ prefix), no RNG, no wall-clock. Reuse the `ally()` helper pattern from tests/heal_effect.rs. Cases:

- cleanse_count_some_two_removes_two_longest_debuffs — bag with 4 debuffs at durations 3,1,2,4; count=Some(2) → removes the dur4 and dur3 entries in that order; remaining bag has the dur1 and dur2 entries; OnCleansed { kinds: [dur4_kind, dur3_kind] }.
- cleanse_count_some_two_tie_break_lower_insertion_index_first — two debuffs same duration: the entry inserted first is removed first in the kinds vec.
- cleanse_count_none_removes_all_debuffs_keeps_blessed — bag with Blessed + 3 debuffs; count=None → all 3 debuffs removed, Blessed survives.
- cleanse_count_some_zero_emits_empty_event_no_state_change — bag with 2 debuffs; count=Some(0) → no removals, OnCleansed { kinds: [] } still emitted.
- cleanse_blessed_only_no_op — bag with only Blessed; count=Some(5) → no removals; event with empty kinds.
- cleanse_count_exceeds_debuff_count_removes_all_no_panic — bag with 2 debuffs; count=Some(10) → removes both, no panic.
- cleanse_on_ko_target_no_op_no_event — KO defender → no state change, no event emitted (mirrors heal KO policy).
- cleanse_on_empty_bag_emits_empty_event — empty StatusBag, count=Some(3) → no removals, OnCleansed { kinds: [] }.

Verification: tests must run via direct apply_cleanse_only invocations (or apply_effects if the routing is set up to dispatch cleanse through it) — NOT through a Bevy world spin-up. JSONL trace identity: existing test fixtures contain no Cleanse skills, so byte-identical traces are preserved.

## Inputs

- `.gsd/milestones/M019/slices/S03/S03-RESEARCH.md`
- `.gsd/milestones/M019/slices/S02/S02-SUMMARY.md`
- `.gsd/milestones/M019/slices/S02/tasks/T03-SUMMARY.md`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/follow_up.rs`
- `src/combat/resolution.rs`
- `src/combat/status_effect.rs`
- `src/combat/state.rs`
- `tests/heal_effect.rs`
- `tests/dr_pipeline.rs`

## Expected Output

- `src/combat/turn_system/pipeline.rs`
- `src/combat/follow_up.rs`
- `tests/cleanse_effect.rs`

## Verification

cargo test --test cleanse_effect — all 8 cases pass deterministically. cargo test — full integration suite green (heal_effect.rs, dr_pipeline.rs, follow_up_triggers.rs, status_blessed_offensive.rs, validation_snapshot.rs unaffected). cargo check clean.

## Observability Impact

Cleanse application emits CombatEventKind::OnCleansed once per non-KO target through the pipeline event bus and JSONL logger. No-op cleanses (count=0, empty bag, Blessed-only) emit empty-kinds events for telemetry parity.
