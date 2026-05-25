---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Author the M004 VFX windowed UAT runbook and signoff/waiver artifact

Why: M004's Final Integrated Acceptance has a human-eye half — Sharp Claws, Baby Flame, and Baby Burner must read as real HDR glow in the real windowed build, which auto-mode cannot verify (K001). The reviewer needs an explicit, per-skill runbook and a place to record the verdict, so the milestone's last unchecked visual item can be closed honestly. Do: Create docs/uat/M004-vfx-signoff.md (tracked, outside gitignored .gsd/). Structure it as: (a) a Launch section instructing the human to run `cargo winx` (the .cargo/config.toml alias = `run --features 'dev windowed'`; note [env] BEVY_HEADLESS="1" is global and the human run is the intended exception — do NOT tell them to unset it, cargo winx is the sanctioned path); (b) one subsection per skill — Sharp Claws (pale yellow-white overbright streak, ttl 6 ticks, scale pop, blooms under HDR; triggered by on_enter SpawnParticle("sharp_claws_slash") on the sharp_claws_strike AnimGraph node), Baby Flame (charge ember-swirl -> fast launch -> impact shard-fan, data-driven), Baby Burner (detonate flash, data-driven) — each with trigger steps and an acceptance bar ('reads as real glow vs the flat-alpha-quad placeholder'); (c) an explicit D037 caveat: strict additive blending is deferred — the reviewer must NOT fail a skill for absent true-additive blending, HDR bloom + overbright is the intended bar; (d) a Signoff / Waiver section with a per-skill verdict line (PASS-with-notes / FAIL / WAIVED) and a top-level status field. Since this runs in autonomous auto-mode with no human available, fill the status as 'Framework complete — human capture pending (K001: auto-mode cannot launch the windowed binary)' and mark each skill verdict PENDING, mirroring M002/S06's honest 'environment-limited' terminal state. Done-when: docs/uat/M004-vfx-signoff.md exists, is non-empty, names all three skills with acceptance bars, carries the D037 no-strict-additive caveat, and has a per-skill verdict + top-level status section that does not overclaim a signoff that did not occur.

## Inputs

- `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`
- `.gsd/milestones/M004/slices/S05/S05-UAT.md`
- `.gsd/milestones/M002/slices/S06/S06-UAT.md`
- `.cargo/config.toml`

## Expected Output

- `docs/uat/M004-vfx-signoff.md`

## Verification

test -s docs/uat/M004-vfx-signoff.md && grep -qi 'cargo winx' docs/uat/M004-vfx-signoff.md && grep -qi 'sharp claws' docs/uat/M004-vfx-signoff.md && grep -qi 'baby flame' docs/uat/M004-vfx-signoff.md && grep -qi 'baby burner' docs/uat/M004-vfx-signoff.md && grep -qi 'D037' docs/uat/M004-vfx-signoff.md && grep -qiE 'waiver|pending|signoff' docs/uat/M004-vfx-signoff.md
