---
sliceId: S03
uatType: manual-windowed
verdict: pass
date: 2026-05-25T00:00:00.000Z
---

# S03: S03 — UAT

**Milestone:** M003
**Written:** 2026-05-22T13:32:09.830Z
**Recorded pass:** 2026-05-25 (user manual `cargo winx` sign-off)

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

## Recorded Result — PASS (2026-05-25)
User ran the `cargo winx` Agumon-vs-dummy encounter (K001 manual loop) and confirmed all five surfaces render and animate on both actors:

| Surface | Result | User note |
|---------|--------|-----------|
| Idle (both sprites) | PASS | Smooth cycling, no freeze/flicker |
| Basic (Sharp Claws) | PASS | Damage lands on impact frame, not keypress |
| Skill (Baby Flame) | PASS | cast→impact→recover linear; world-space particle flash **visible** at authored locus; damage on impact frame |
| Ultimate (Baby Burner) | PASS | charge→launch→recovery; world-space detonate flash visible **and** egui chip still fires; damage on launch frame |
| Cleanup | PASS | Particles despawn cleanly after TTL; re-triggers spawn fresh particles, no stuck quads or buildup |

All Expected Outcomes met. The VFX seam is no longer inert; the world particle complements (does not replace) the preserved egui chip path.

**Follow-up (out of scope for S03, deferred to a fresh polish task):** the user noted the Baby Flame particle *aesthetic* is functional but plainer than envisioned — a desired future look is a swirling charge-in-mouth feeding the flame, a fast launch, and a flame-dissolve burst on impact. This is a new polish wish beyond S03's "flash renders as visible particles" contract, which is satisfied. Tracked for M004+ / a `/gsd quick` task; reuses the three existing assets (`baby_flame_charge/projectile/impact.png`) with at most one optional ember sprite.

## Edge Cases
- If a target becomes unavailable or cannot be resolved, the app should avoid crashing; the effect may be skipped and debug logging should indicate the unresolved target case.
- Re-triggering skills after prior particles have expired should spawn fresh particles with no visible drift or buildup from previous casts.
- Particle behavior should remain presentation-only: no extra gameplay payload, collisions, or combat-state side effects should appear.

## Not Proven By This UAT
- Headless structural guarantees for `VfxSpawnDescriptor`, `resolve_locus`, and the no-numeric-payload serialization contract.
- Full regression coverage for the headless suite and windowed unit tests.
- Any performance/soak characteristics beyond this manual spot check.
