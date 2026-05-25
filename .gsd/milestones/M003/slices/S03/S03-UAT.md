# S03: S03 — UAT

**Milestone:** M003
**Written:** 2026-05-22T13:32:09.830Z

# S03 UAT — VFX flash renders as visible particles

## UAT Type
Manual windowed visual verification (`cargo winx`) plus observation of the existing egui flash chip.

## Preconditions
- Build succeeds locally with the `windowed` feature.
- The Agumon-vs-Agumon dummy combat scene launches through the normal windowed entrypoint (`cargo winx` in this project).
- The user can trigger both Baby Flame and Baby Burner during the encounter.

## Steps
1. Launch the windowed combat demo and wait for both Agumon sprites to appear idle on screen.
2. Trigger **Baby Flame** from the ally Agumon.
3. Observe the cast window as the skill begins, then watch the target area during the projectile/impact portion.
4. Repeat Baby Flame from the mirrored/dummy side if that control path is available in the demo.
5. Charge and trigger **Baby Burner**.
6. Watch for the detonate moment and compare the world-space flash against the egui flash chip timing.
7. Repeat Baby Burner from the mirrored/dummy side if available.
8. Let spawned particles expire naturally and verify the battlefield returns to its normal idle presentation without stuck quads.

## Expected Outcomes
- During **Baby Flame**, a visible short-lived particle flash appears in world space instead of the effect being inert.
- The Baby Flame particle appears at the authored locus for the acting/impacted units and moves consistently with the authored motion mode.
- During **Baby Burner** detonate, a visible world-space flash appears at the target locus.
- The existing egui Baby Burner flash chip still appears; the new world particle complements it rather than replacing or breaking it.
- Particles despawn cleanly after their short TTL with no permanent artifacts left on screen.
- Both ally and mirrored dummy can show the VFX when they are the acting side.

## Edge Cases
- If a target becomes unavailable or cannot be resolved, the app should avoid crashing; the effect may be skipped and debug logging should indicate the unresolved target case.
- Re-triggering skills after prior particles have expired should spawn fresh particles with no visible drift or buildup from previous casts.
- Particle behavior should remain presentation-only: no extra gameplay payload, collisions, or combat-state side effects should appear.

## Not Proven By This UAT
- Headless structural guarantees for `VfxSpawnDescriptor`, `resolve_locus`, and the no-numeric-payload serialization contract.
- Full regression coverage for the headless suite and windowed unit tests.
- Any performance/soak characteristics beyond this manual spot check.
