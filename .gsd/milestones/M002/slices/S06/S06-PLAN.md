# S06: Windowed smoke end-to-end + repomix review gate

**Goal:** Close M002 with a launchable proof: a documented windowed smoke UAT (Agumon vs Agumon dummy at full kit, no panic, stable FPS, hot-reload mid-skill intact) with captured console output attached as evidence, plus a repomix-grounded architectural review report covering maintainability/scalability/extensibility. R016 invariants (headless-first, determinism, dep-gating, repo hygiene, I3 parity) remain green.
**Demo:** A windowed session with no panic, stable FPS, hot-reload mid-skill not corrupting world state, captured console output; plus a repomix-grounded architectural review report.

## Must-Haves

- A windowed smoke UAT runbook exists under `docs/uat/` with explicit steps for: launch, full-kit Agumon-vs-Agumon-dummy turns (Sharp Claws / Bouncing Fire / Baby Burner Ultimate), hot-reload of `assets/data/digimon/agumon/skills.ron` mid-skill, and expected pass/fail signals.
- A capture helper script (`scripts/capture-windowed-smoke.sh`) runs `cargo run --features windowed` with stdout/stderr teed into a timestamped log under `.gsd/milestones/M002/slices/S06/uat-evidence/`.
- The captured UAT log (produced by the user manually per K001) is attached as S06 evidence at `.gsd/milestones/M002/slices/S06/uat-evidence/windowed-smoke.log` and referenced from the S06 summary.
- A repomix pack is generated via `scripts/repomix-review.sh` and an architectural review report exists at `.gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md` answering the R015 prompt against M002 CONTEXT + objectives.
- Full headless verification matrix is green: `cargo test`, `cargo test --features windowed --test windowed_only`, `cargo build --no-default-features`, `cargo build --features windowed`; I3 two-clock parity and timeline_loop_hop_cue_parity remain green; no new `.md` in repo root; no winit/wgpu/egui leakage outside `windowed`.

## Proof Level

- This slice proves: final-assembly — operational UAT (manual, user-executed per K001) combined with architectural review gate and full regression matrix. Real runtime required: yes (user). Human/UAT required: yes.

## Integration Closure

Upstream surfaces consumed: windowed bootstrap (`src/windowed/mod.rs`), encounter bootstrap (`src/combat/encounter/bootstrap.rs`), full kit timeline + cue handshake (S02/S04/S05), phase strip + HUD + Twin Core badge (S03/S05). No new wiring is introduced — S06 only adds evidence collection, an architectural review, and final regression coverage. After S06, M002 is end-to-end usable: a user can launch the windowed binary, fight Agumon-vs-Agumon-dummy at full kit, hot-reload skills mid-fight, and the architectural review confirms the assembled stack is maintainable/scalable/extensible for M003-M007 and a future RON editor.

## Verification

- No runtime observability added in src/. Evidence-level observability only: timestamped windowed smoke log, repomix pack file, and architectural review markdown. Existing in-runtime signals (PhaseStripDisplay, HpBarView, FloatingDamageView, TargetHurtState, TwinCoreBadgeState, CueBarrierStatus.hop_index) are exercised by the UAT and named in the runbook as inspection surfaces. Failure visibility: panic stacktrace or unbounded VFX entity growth would surface in the captured stdout/stderr log; the runbook calls these out explicitly.

## Tasks

- [x] **T01: Author windowed smoke UAT runbook + capture helper script** `est:45m`
  Why: R014 requires operational UAT with captured console output — not just a documented procedure. K001 forbids the auto-mode agent from launching the windowed binary, so this task produces the artifacts a human will use to perform the UAT and attach the log as S06 evidence.
  - Files: `docs/uat/M002-S06-windowed-smoke.md`, `scripts/capture-windowed-smoke.sh`
  - Verify: test -f docs/uat/M002-S06-windowed-smoke.md && test -x scripts/capture-windowed-smoke.sh && grep -q 'Baby Burner' docs/uat/M002-S06-windowed-smoke.md && grep -q 'hot-reload' docs/uat/M002-S06-windowed-smoke.md && grep -q 'tee' scripts/capture-windowed-smoke.sh

- [x] **T02: Generate repomix pack + author architectural review report (R015 gate)** `est:1h30m`
  Why: R015 requires a repomix-grounded architectural review at M002 closeout, using the prompt 'Please review the overall structure and suggest any improvements or refactoring opportunities, focusing on maintainability, scalability and extensibility.' The produced report is attached as S06 evidence; findings are triaged before milestone completion.
  - Files: `scripts/repomix-review.sh`, `.gsd/milestones/M002/slices/S06/repomix-pack.xml`, `.gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md`
  - Verify: test -x scripts/repomix-review.sh && test -s .gsd/milestones/M002/slices/S06/repomix-pack.xml && test -s .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md && grep -q 'Maintainability' .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md && grep -q 'Scalability' .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md && grep -q 'Extensibility' .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md && grep -q 'Findings' .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md && grep -q 'Verdict' .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md

- [x] **T03: R016 invariant gate + final M002 regression matrix** `est:45m`
  Why: R016 requires that R002 (headless-first), R004 (determinism), R005 (windowed dep-gating), R006 (no .md in repo root), and I3 two-clock parity all hold at M002 closeout. This task is the mechanical proof.
  - Files: `.gsd/milestones/M002/slices/S06/regression-matrix.md`
  - Verify: test -s .gsd/milestones/M002/slices/S06/regression-matrix.md && grep -q 'cargo test' .gsd/milestones/M002/slices/S06/regression-matrix.md && grep -q 'R005' .gsd/milestones/M002/slices/S06/regression-matrix.md && grep -q 'R006' .gsd/milestones/M002/slices/S06/regression-matrix.md && grep -q 'I3' .gsd/milestones/M002/slices/S06/regression-matrix.md && grep -q 'PASS' .gsd/milestones/M002/slices/S06/regression-matrix.md

## Files Likely Touched

- docs/uat/M002-S06-windowed-smoke.md
- scripts/capture-windowed-smoke.sh
- scripts/repomix-review.sh
- .gsd/milestones/M002/slices/S06/repomix-pack.xml
- .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md
- .gsd/milestones/M002/slices/S06/regression-matrix.md
