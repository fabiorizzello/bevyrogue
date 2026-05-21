# M002 / S06 — Windowed Smoke UAT Runbook

This runbook drives the manual windowed smoke for slice S06. Auto-mode does
**not** launch the windowed binary (K001); the human operator executes this
runbook and the resulting log under
`.gsd/milestones/M002/slices/S06/uat-evidence/` is the canonical evidence.

## Launch

Preferred:

```bash
bash scripts/capture-windowed-smoke.sh
```

This wraps `cargo run --features windowed --bin bevyrogue` and tees stdout/stderr
to a timestamped log under `.gsd/milestones/M002/slices/S06/uat-evidence/`.

Fallback (no log capture):

```bash
cargo run --features windowed
```

## Pre-flight checklist

- [ ] `cargo build --features windowed` is green from a cold or warm cache.
- [ ] `assets/data/digimon/agumon/skills.ron` has a clean working tree
      (`git status -- assets/data/digimon/agumon/skills.ron` reports no changes).
      We need a known-good baseline so the hot-reload step is meaningful.
- [ ] Evidence directory exists (the capture script handles this; verify the
      latest log filename after the run for the slice ASSESSMENT).

## Smoke scenario — at least one full turn sequence

Execute a single combat encounter against the Agumon training dummy that
exercises **all three** Agumon abilities end-to-end:

1. **Sharp Claws** — basic strike. Confirm phase strip ticks through
   windup → active → recovery, target HP bar drains, hurt tint flashes on
   the dummy sprite, and a floating damage number anchors to the dummy.
2. **Bouncing Fire** — multi-hop projectile. Confirm the loop hops are
   visually distinct (each hop produces its own impact / VFX burst) and that
   `CueBarrierStatus.hop_index` advances per hop in the inspector if open.
   Each hop should produce its own floating damage number.
3. **Baby Burner Ultimate** — charge → launch → recovery. During windup the
   phase strip must clearly show the charge segment; the launch frame should
   spawn the Twin Core badge exactly once; recovery returns control. The
   dummy must already be "heated" (after a prior Bouncing Fire) so the
   **reactive detonate** triggers — confirm the detonate VFX visually fires
   against the heated dummy rather than silently skipping.

A full smoke is one continuous combat that touches all three skills; repeat
the encounter if needed but a single uninterrupted sequence is the canonical
pass.

## Hot-reload step

While the **Baby Burner Ultimate windup** is in flight (phase strip showing
charge segment):

1. Open `assets/data/digimon/agumon/skills.ron` in an editor.
2. Edit a single numeric field (e.g. nudge a damage or duration value by a
   small amount — do **not** rename keys or change structure).
3. Save the file.
4. Observe:
   - No panic in the log.
   - Combat state remains intact (HP bars, phase strip, target tint,
     pending cues all survive the reload).
   - The Ultimate completes its launch / recovery without desync.
5. Revert the edit so the working tree is clean again for the next run.

## Pass signals

- No panic / no stack trace anywhere in the captured log.
- `PhaseStripDisplay` advances smoothly through every phase of every skill.
- `HpBarView` drains smoothly — no teleporting / negative values.
- `TargetHurtState` hurt tint flashes on each landed hit.
- `TwinCoreBadgeState` shows the Twin Core badge **exactly once** at the
  Ultimate launch frame, then clears.
- `FloatingDamageView` damage numbers anchor to the correct sprite and
  travel/fade without orphaning.
- `CueBarrierStatus.hop_index` advances correctly across Bouncing Fire hops.
- FPS visually stable (no perceptible stutter); if the egui inspector is
  available, entity counts for VFX / cue / damage-number archetypes return
  to baseline between skills (no unbounded growth).
- Hot-reload step completes with combat state intact.

## Fail signals

- Panic stack trace in the log (any thread).
- Frozen frame / hung main loop.
- Growing VFX / cue / floating-damage entity count that never returns to
  baseline in the egui inspector.
- World state desync after the hot-reload (HP snaps, phase strip resets,
  pending cues lost, Twin Core badge re-fires, etc.).
- Bouncing Fire hops collapse into a single impact (loop did not run).
- Baby Burner reactive detonate fails to fire against a confirmed-heated
  dummy.

## Inspection surfaces (named in S06-PLAN)

Use the egui inspector (if compiled in) or log-side telemetry to inspect:

- `PhaseStripDisplay`
- `HpBarView`
- `FloatingDamageView`
- `TargetHurtState`
- `TwinCoreBadgeState`
- `CueBarrierStatus.hop_index`

## Evidence

The capture script writes
`.gsd/milestones/M002/slices/S06/uat-evidence/windowed-smoke-<UTC>.log`.
The **latest** log in that directory is the canonical evidence consumed by
the slice ASSESSMENT — do not delete older logs, append-only is fine.
