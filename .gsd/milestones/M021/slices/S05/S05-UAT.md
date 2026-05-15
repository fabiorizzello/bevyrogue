# S05: Built-in extension fns + RON → CompiledTimeline compiler — UAT

**Milestone:** M021
**Written:** 2026-05-15T18:05:07.970Z

## UAT Type
Integration / runtime path verification

## Preconditions
- `assets/data/skills.ron` contains the timeline-backed canon entries for `petit_thunder` and the live Renamon ult id `renamon_ult`.
- The compiled-timeline loader and BeatRunner routing are built into the current binary.
- The test environment is headless and deterministic.

## Steps
1. Run `cargo test --test compiled_timeline_petit_thunder --test compiled_timeline_tohakken`.
2. Confirm the Petit Thunder test resolves and executes through the compiled-timeline path.
3. Confirm the Renamon ult canon test resolves the live asset id `renamon_ult` and executes through the compiled-timeline path.
4. Confirm the runtime event stream includes the expected combat events for the compiled-timeline flow.
5. Confirm the negative bootstrap coverage still fails fast on a malformed timeline reference before any encounter can proceed.

## Expected Outcomes
- Both canon tests pass.
- Petit Thunder uses the timeline-backed runtime path.
- Renamon ult / Tohakken coverage uses the live asset id `renamon_ult` and still proves the intended semantics.
- Invalid timeline references are rejected during load with skill and site context.
- No regression appears in legacy skill execution.

## Edge Cases
- A skill with no compiled timeline should still resolve through the legacy path.
- A typo in a hook, selector, or predicate should fail at load time, not during combat.
- The test should assert combat events rather than legacy custom-signal artifacts.

## Not Proven By This UAT
- Full migration of the rest of the active canon.
- Passive runner behavior.
- UI-specific rendering or interaction paths.
- Performance characteristics under large encounter loads.
