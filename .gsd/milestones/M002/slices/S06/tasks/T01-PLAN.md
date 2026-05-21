---
estimated_steps: 6
estimated_files: 2
skills_used: []
---

# T01: Author windowed smoke UAT runbook + capture helper script

Why: R014 requires operational UAT with captured console output — not just a documented procedure. K001 forbids the auto-mode agent from launching the windowed binary, so this task produces the artifacts a human will use to perform the UAT and attach the log as S06 evidence.

Do:
1. Create `docs/uat/M002-S06-windowed-smoke.md` with: launch command (`bash scripts/capture-windowed-smoke.sh` or `cargo run --features windowed`); pre-flight checklist (`cargo build --features windowed` green; `assets/data/digimon/agumon/skills.ron` clean working tree); the smoke scenario — at least one full turn sequence using Sharp Claws, Bouncing Fire (loop hops visible), and Baby Burner Ultimate (charge → launch → recovery, reactive detonate visible against the heated dummy); hot-reload step — edit a numeric field in `assets/data/digimon/agumon/skills.ron` mid-Ultimate windup, save, observe no panic and combat state intact; expected pass signals (no panic, phase strip updates, HP bar drains smoothly, hurt tint on hits, Twin Core badge appears once on Ultimate, floating damage numbers anchor to sprites, FPS visually stable, no unbounded entity growth in egui inspector if available); fail signals (panic stacktrace in log, frozen frame, growing VFX entity count, world state desync after hot-reload).
2. Create `scripts/capture-windowed-smoke.sh`: bash script that creates `.gsd/milestones/M002/slices/S06/uat-evidence/` if missing, then runs `cargo run --features windowed --bin bevyrogue 2>&1 | tee .gsd/milestones/M002/slices/S06/uat-evidence/windowed-smoke-$(date -u +%Y%m%dT%H%M%SZ).log`. Make executable (`chmod +x`).
3. Document in the runbook that the user (not auto-mode) executes the smoke; the latest log under `uat-evidence/` is the canonical evidence.

Done when: both files exist; the script is executable; the runbook lists all three skills, the hot-reload step, and the pass/fail signal lists.

## Inputs

- `src/windowed/mod.rs`
- `src/combat/encounter/bootstrap.rs`
- `assets/data/digimon/agumon/skills.ron`
- `.gsd/milestones/M002/slices/S05/S05-SUMMARY.md`
- `.gsd/REQUIREMENTS.md`

## Expected Output

- `docs/uat/M002-S06-windowed-smoke.md`
- `scripts/capture-windowed-smoke.sh`

## Verification

test -f docs/uat/M002-S06-windowed-smoke.md && test -x scripts/capture-windowed-smoke.sh && grep -q 'Baby Burner' docs/uat/M002-S06-windowed-smoke.md && grep -q 'hot-reload' docs/uat/M002-S06-windowed-smoke.md && grep -q 'tee' scripts/capture-windowed-smoke.sh
