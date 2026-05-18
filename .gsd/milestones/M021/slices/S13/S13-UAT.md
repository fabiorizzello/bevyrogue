# S13: Close deferred foundation captures and boot invariants — UAT

**Milestone:** M021
**Written:** 2026-05-17T13:34:56.685Z

## UAT Type
- UAT mode: mixed
- Why this mode is sufficient: The slice is about proof capture and boot-time invariants, so the combination of focused runtime regression evidence and artifact-backed task summaries is enough to validate the closeout.

## Preconditions
- Current tree contains the compiled timeline regression test surface.
- CombatPlugin boot validation is enabled on App::finish().
- Canonical skill timeline assets parse successfully.

## Smoke Test
- Run the focused boot-validation regression for invalid timeline ids and observe the expected aggregated panic from CombatPlugin::finish.

## Test Cases

### 1. Boot validation rejects dangling timeline refs
1. Run `cargo test --test compiled_timeline_boot_validation -- --nocapture invalid_timeline_ids_fail_during_app_finish`.
2. Observe the panic output emitted by App::finish().
3. **Expected:** The test fails only because the boot seam panics as designed, and the panic message lists both the missing hook and missing predicate.

### 2. Foundation proof surface remains intact
1. Review the task summaries for T01 and T02.
2. Confirm they record fresh evidence for cast_id, UltInstant, turn-phase ordering, and DryRun parity.
3. **Expected:** The slice closeout is backed by explicit live proof notes, not just roadmap intent.

## Edge Cases

### Multiple dangling refs in one timeline
1. Inject more than one invalid reference into the compiled timeline.
2. **Expected:** Boot validation aggregates all missing references before panicking.

## Failure Signals
- Missing panic text for either the hook or predicate.
- App::finish() returning successfully when invalid timeline ids are present.
- Task summaries missing the required proof mapping.

## Not Proven By This UAT
- Full milestone validation.
- UI/runtime behavior outside the boot-validation and foundation-proof surface.
- Any unrelated combat systems not part of the S13 closeout.

## Notes for Tester
- This slice is intentionally narrow: it closes the deferred proof gap and does not re-validate the whole combat stack.
