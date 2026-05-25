# M004 VFX Windowed Signoff / Waiver

This runbook is the **manual-only** visual acceptance artifact for M004's Agumon VFX review. It exists so a human reviewer can launch the real windowed build, verify the intended HDR bloom / overbright look for each skill, and record an honest verdict without mixing manual UAT with automated proof.

## Top-Level Status

**WAIVED — autonomous closeout recorded without a live `cargo winx` session (K001: auto-mode cannot launch the windowed binary).**

## Launch

Use the sanctioned windowed path if a later human wants to perform a post-closeout spot check:

```bash
cargo winx
```

Notes:
- `.cargo/config.toml` defines `cargo winx` as `run --features 'dev windowed'`.
- `[env] BEVY_HEADLESS="1"` is globally configured for agent safety.
- **Do not treat that as a setup bug and do not rewrite the runbook to ask for unsetting it manually.** `cargo winx` is the intended human verification path for this milestone.
- K001 remains in force: auto-mode must **not** launch the windowed binary. This document is for a human reviewer to execute if a later manual spot-check is desired.

## Reviewer Instructions

For each skill below:
1. Launch the game with `cargo winx`.
2. Navigate to the scenario or input flow that triggers the named skill.
3. Observe the skill at least a few times to confirm timing, bloom response, and readability.
4. Compare what you see against the acceptance bar for that skill.
5. Record `PASS-with-notes`, `FAIL`, or `WAIVED` in the Signoff / Waiver section.

## D037 Caveat — Do Not Over-Fail

**D037 defers strict additive blending.**

The acceptance bar for M004 is **HDR bloom + overbright glow that reads visually better than the old flat-alpha-quad placeholder**. A reviewer must **not** fail Sharp Claws, Baby Flame, or Baby Burner solely because the effect is not rendered with a true custom additive material. Strict additive blending is deferred work; for this milestone, the intended bar is that the effect reads as real glow under HDR / bloom.

## Skill Checks

### Sharp Claws

**What should trigger it**
- Use Agumon's Sharp Claws attack in the windowed build.
- The slash is spawned from `SpawnParticle("sharp_claws_slash")` on enter of the `sharp_claws_strike` AnimGraph node.

**What to look for**
- A pale yellow-white overbright slash streak.
- A visible scale pop rather than a static flat stamp.
- A short, punchy lifetime consistent with the authored 6-tick TTL.
- HDR bloom that makes the streak read as luminous rather than as a dull textured quad.

**Acceptance bar**
- **Pass** if Sharp Claws reads as a bright, intentional slash with visible glow / bloom and clear impact energy, rather than the old flat-alpha-quad placeholder look.
- **Fail** if it looks flat, dim, placeholder-like, visually broken, or if the bloom / glow read is missing enough that the slash no longer feels like a real VFX beat.

### Baby Flame

**What should trigger it**
- Use Baby Flame in the windowed build.
- Observe the full sequence from charge to projectile to impact.

**What to look for**
- A charge phase with ember-swirl energy buildup.
- A fast, readable launch phase.
- An impact that resolves as a shard-fan / burst instead of a single flat placeholder sprite.
- A fully data-driven visual sequence that blooms under HDR and reads as an authored fire effect.

**Acceptance bar**
- **Pass** if the sequence reads as charge ember-swirl -> fast launch -> impact shard-fan, with believable glow / bloom and a clearly improved result over the old flat-alpha-quad placeholder.
- **Fail** if one of those beats is visually absent, unreadable, obviously broken, or if the overall effect still feels like a placeholder rather than a finished glow-driven fire skill.

### Baby Burner

**What should trigger it**
- Use Baby Burner in the windowed build.
- Focus on the detonate / flash moment.

**What to look for**
- A clear detonate flash at the intended moment.
- HDR bloom / overbright read that makes the flash feel energetic.
- A data-driven effect that no longer reads as a flat placeholder quad.

**Acceptance bar**
- **Pass** if the detonate flash lands clearly, reads as luminous, and feels materially better than the flat-alpha-quad placeholder baseline.
- **Fail** if the flash is visually weak, flat, broken, or fails to present as a convincing glow-driven detonation beat.

## Signoff / Waiver

### Per-Skill Verdicts

- **Sharp Claws:** WAIVED
  - Basis: automated S05/S06 evidence proves authored data, bridge wiring, HDR/Bloom proxy, and regression coverage, but no human `cargo winx` session occurred in auto-mode.
  - Notes: This is **not** a live visual PASS. A later human may replace this waiver with `PASS-with-notes` or `FAIL` after a real windowed review.

- **Baby Flame:** WAIVED
  - Basis: automated regression evidence proves the data-driven charge -> projectile -> impact chain and the windowed render/test surfaces, but no human `cargo winx` session occurred in auto-mode.
  - Notes: This is **not** a live visual PASS. A later human may replace this waiver with `PASS-with-notes` or `FAIL` after a real windowed review.

- **Baby Burner:** WAIVED
  - Basis: automated regression evidence proves the data-driven detonate -> flash chain and the windowed render/test surfaces, but no human `cargo winx` session occurred in auto-mode.
  - Notes: This is **not** a live visual PASS. A later human may replace this waiver with `PASS-with-notes` or `FAIL` after a real windowed review.

### Final Status

- **Current autonomous-execution status:** WAIVED — closeout recorded from artifact evidence only; K001 prevented any auto-mode `cargo winx` run.
- **Final reviewer-completed status field:** `WAIVED`
- Reviewer name: GSD auto-mode closeout
- Date: 2026-05-25T21:05:31Z
- Evidence location / capture notes: `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`; `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`; `.gsd/milestones/M004/slices/S06/S06-UAT.md`; no live capture artifact exists because the windowed binary was not launched.
- Overall notes: This waiver closes the milestone artifact honestly without fabricating a human visual PASS. It preserves the manual-review runbook for any later human spot-check.

## Honesty Boundary

This file is intentionally a **runbook plus tracked waiver record**, not evidence that the visual review already happened in a real windowed session. Automated proof for M004 lives in the S05 acceptance and S06 assessment/UAT artifacts; this document exists to keep the manual `cargo winx` review separate and explicit while recording that the milestone closeout used a formal waiver instead of a fabricated PASS.
