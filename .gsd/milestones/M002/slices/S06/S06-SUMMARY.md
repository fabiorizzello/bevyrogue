---
id: S06
parent: M002
milestone: M002
provides:
  - Windowed smoke UAT runbook and capture helper script for Agumon-vs-Agumon-dummy full-kit session
  - R015 repomix-grounded architectural review report (7 findings, pass-with-followups, F2 medium triaged to M003/S01)
  - R016 invariant gate: full regression matrix confirming R002/R004/R005/R006/I3 all PASS at M002 closeout
requires:
  - slice: S01
    provides: Stance FSM + runtime player baseline whose FPS stability and hot-reload safety the windowed smoke runbook probes.
  - slice: S02
    provides: Two-clock cue-barrier contract exercised in the smoke session and by the R016 invariant matrix (I3 parity).
  - slice: S03
    provides: Event-driven phase strip surface verified live during the windowed smoke runbook.
  - slice: S04
    provides: Reactive detonate + flash VFX path the smoke session triggers to validate no-panic / no-corruption behavior under reactions.
  - slice: S05
    provides: Full-kit Agumon-vs-Agumon-dummy encounter bootstrap, HUD HP bars, floating damage, twin-core badge — the exact session `capture-windowed-smoke.sh` and the UAT runbook drive end-to-end.
affects:
  []
key_files:
  - docs/uat/M002-S06-windowed-smoke.md
  - scripts/capture-windowed-smoke.sh
  - scripts/repomix-review.sh
  - .gsd/milestones/M002/slices/S06/repomix-pack.xml
  - .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md
  - .gsd/milestones/M002/slices/S06/regression-matrix.md
key_decisions:
  - Verdict pass-with-followups for R015 architectural review: M002 stack structurally ready for M003-M007 and RON editor; no needs-remediation findings.
  - F2 (unsafe raw-pointer ExtRegistries dance in timeline_exec.rs) flagged medium severity — sole above-low item — triaged for M003/S01; does not block M002 closeout.
  - R005 grep hit on src/combat/runtime/mod.rs:20 treated as PASS because it is a //! doc-comment, not a use import — judgment recorded verbatim in regression matrix for auditability.
  - K001 compliance: windowed binary not launched from auto-mode; UAT runbook + capture helper delegate live smoke to user, with uat-evidence/ as the evidence drop point.
  - Capture script uses PIPESTATUS to preserve cargo exit code through tee, ensuring panic surfaces as non-zero even though tee succeeds.
patterns_established:
  - (none)
observability_surfaces:
  - Timestamped windowed smoke log: .gsd/milestones/M002/slices/S06/uat-evidence/windowed-smoke-<timestamp>.log (user-produced, captures full stdout/stderr)
  - Repomix pack: .gsd/milestones/M002/slices/S06/repomix-pack.xml (full source snapshot at M002 closeout)
  - Regression matrix: .gsd/milestones/M002/slices/S06/regression-matrix.md (records exact commands, outputs, and judgments for all R016 checks)
  - Architectural review: .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md (findings F1-F7 with triage owners)
drill_down_paths:
  - .gsd/milestones/M002/slices/S06/tasks/T01-SUMMARY.md
  - .gsd/milestones/M002/slices/S06/tasks/T02-SUMMARY.md
  - .gsd/milestones/M002/slices/S06/tasks/T03-SUMMARY.md
  - .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md
  - .gsd/milestones/M002/slices/S06/regression-matrix.md
duration: ""
verification_result: passed
completed_at: 2026-05-21T12:06:44.921Z
blocker_discovered: false
---

# S06: Windowed smoke end-to-end + repomix review gate

**Authored windowed smoke UAT runbook and capture helper, produced R015 architectural review (7 findings, pass-with-followups), and confirmed all R016 invariants green across 7 cargo commands plus structural dep/hygiene checks.**

## What Happened

S06 closes M002 with three deliverables assembled across tasks T01–T03.

**T01 — Windowed smoke UAT runbook + capture helper (docs/uat/M002-S06-windowed-smoke.md, scripts/capture-windowed-smoke.sh):**
The runbook prescribes a single continuous windowed session covering all three Agumon skills: Sharp Claws, Bouncing Fire, and Baby Burner Ultimate (with dummy pre-heated for the reactive detonate). It documents expected pass signals (§9 phase strip advancing, HP bars decrementing, floating damage numbers on impact frames, target blink/hurt states, TwinCore badge) and explicit failure signals (panic stacktrace, unbounded VFX entity growth). The capture helper tees `cargo run --features windowed` stdout/stderr into a timestamped log under `uat-evidence/` and preserves the cargo exit code via `PIPESTATUS` so a panic surfaces as non-zero. Per K001, the auto-mode agent cannot launch the windowed binary; the runbook and script exist for the user to perform and attach the UAT log.

**T02 — Repomix pack + R015 architectural review (.gsd/milestones/M002/slices/S06/repomix-pack.xml, S06-ARCHITECTURAL-REVIEW.md):**
The repomix review script packs the full source and the review was conducted against the R015 prompt ("Please review the overall structure and suggest any improvements or refactoring opportunities, focusing on maintainability, scalability and extensibility."). Seven findings (F1–F7) were recorded, with F2 (unsafe raw-pointer `ExtRegistries` dance in `timeline_exec.rs`) the sole medium-severity item — all others low. Verdict: **pass-with-followups** — the M002 stack is structurally ready for M003-M007 and a future RON editor; F2 is triaged for M003/S01. Hygiene gates R003/R005/R006/I3 are explicitly called out as PASS in the report, making the gate auditable.

**T03 — R016 invariant gate + final regression matrix (.gsd/milestones/M002/slices/S06/regression-matrix.md):**
Seven cargo commands executed and results recorded: `cargo test` (M001 suite intact), `cargo test --features windowed --test windowed_only`, `cargo build --no-default-features`, `cargo build --features windowed`, plus `cargo clippy` passes. R005 structural grep returned one hit (`src/combat/runtime/mod.rs:20`) that is a `//!` doc-comment, not an import — judgment recorded verbatim and auditable. R006 find confirms no `.md` in repo root. Three I3 timeline parity test files confirmed present on disk and green inside the timeline harness. All checks: **PASS**. The regression matrix explicitly delegates live windowed smoke to the user-driven UAT log per K001.

**Evidence summary:** All three task-defined verify commands returned exit 0. The slice-level composite verification (all T01/T02/T03 checks combined) likewise returned exit 0. The one missing artifact (`uat-evidence/windowed-smoke.log`) is by design — it is a human-produced artifact per K001 whose collection infrastructure (runbook + capture script) was delivered in T01.

## Verification

Ran composite slice-level verification via gsd_exec (run id: f60623bd-e8f9-4633-becc-a1e9a78cde36, exit 0). All checks passed:
- T01: docs/uat/M002-S06-windowed-smoke.md exists; scripts/capture-windowed-smoke.sh executable; Baby Burner / hot-reload / tee present.
- T02: scripts/repomix-review.sh executable; repomix-pack.xml non-empty; S06-ARCHITECTURAL-REVIEW.md non-empty; Maintainability / Scalability / Extensibility / Findings / Verdict sections present.
- T03: regression-matrix.md non-empty; cargo test / R005 / R006 / I3 / PASS all referenced.
- uat-evidence/ directory absent per K001 (user must run windowed binary and attach log manually).

## Requirements Advanced

None.

## Requirements Validated

- R015 — repomix-pack.xml generated; S06-ARCHITECTURAL-REVIEW.md authored with Maintainability/Scalability/Extensibility/Findings/Verdict sections; verdict pass-with-followups; findings triaged.
- R016 — regression-matrix.md records 7 cargo commands all PASS; R005 structural grep and R006 repo-root find both PASS; I3 parity tests confirmed green.

## New Requirements Surfaced

- R014 (windowed smoke operational UAT) validation is delegated to the user per K001 — evidence collection infrastructure (runbook + capture script + uat-evidence/ drop point) is in place.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

uat-evidence/windowed-smoke.log not present at slice completion — this is by design. K001 forbids auto-mode from launching the windowed binary. The collection infrastructure (runbook + capture helper) was delivered in T01; the user must execute it manually to produce the R014 operational UAT evidence.

## Known Limitations

R014 operational UAT (live windowed session) cannot be auto-verified per K001. The log evidence must be produced and attached by the user before M002 milestone closeout.

## Follow-ups

["F2 (unsafe raw-pointer ExtRegistries dance in timeline_exec.rs) — medium severity — triage in M003/S01. Architectural review provides the finding detail.", "F1-F7 follow-up table in S06-ARCHITECTURAL-REVIEW.md records suggested owners for each finding.", "User must run scripts/capture-windowed-smoke.sh and attach the produced log to uat-evidence/ to fulfill R014 operational UAT requirement."]

## Files Created/Modified

None.
