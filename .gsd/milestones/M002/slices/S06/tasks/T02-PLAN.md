---
estimated_steps: 13
estimated_files: 3
skills_used: []
---

# T02: Generate repomix pack + author architectural review report (R015 gate)

Why: R015 requires a repomix-grounded architectural review at M002 closeout, using the prompt 'Please review the overall structure and suggest any improvements or refactoring opportunities, focusing on maintainability, scalability and extensibility.' The produced report is attached as S06 evidence; findings are triaged before milestone completion.

Do:
1. Create `scripts/repomix-review.sh`: invokes `npx --yes repomix@1.14.0 --style xml --output .gsd/milestones/M002/slices/S06/repomix-pack.xml --ignore 'target/**,.gsd/**,.planning/**,.audits/**,assets/**,*.lock'` so the pack covers source + tests + config without binary noise. Make executable.
2. Run the script to produce the pack file at `.gsd/milestones/M002/slices/S06/repomix-pack.xml`.
3. Author `.gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md` covering:
   - **Prompt + scope**: quote the R015 prompt; list files reviewed (the pack); the M002 objective (first on-screen combat) and CONTEXT references.
   - **Maintainability**: assess module boundaries (combat/runtime, combat/blueprints, combat/turn_system/pipeline, windowed, ui), test layout under R003, decision discipline (D025/D026 followed), comment/dead-code hygiene.
   - **Scalability**: assess kernel hop cost, timeline runner allocation patterns, asset-driven kit data, M003-M007 readiness (adding more Digimon, more skills, more passives without touching combat core).
   - **Extensibility**: assess the animation/skill seam (AnimGraph + timeline + cue barrier), the two-clock contract (D025), the post-application reaction seam (D026), readiness for a future RON editor, blueprint envelope generic-vs-typed pattern (P003), modifier ledger fold order (P004), typed observability contracts (P005).
   - **Findings**: numbered list, each item has Severity (info/low/medium/high), Location (file:line or module), Rationale, and Suggested action. At minimum scan for: cross-module coupling that would block M003, presentation-vs-truth leakage (R016/I3), missed-extension hooks, test-layout violations of R003, .md files in repo root (R006), winit/wgpu/egui leakage outside windowed (R005).
   - **Verdict**: pass / pass-with-followups / needs-remediation; if followups, list them with suggested milestone/slice owners.
4. Keep the report concise and grounded — every finding must reference a real path in the pack.

Done when: pack file exists and is non-empty; review markdown exists with all five sections plus a Findings list (>=3 items, even if all info-level), and a Verdict.

## Inputs

- `src`
- `tests`
- `assets/data`
- `Cargo.toml`
- `.gsd/REQUIREMENTS.md`
- `.gsd/DECISIONS.md`
- `.gsd/milestones/M002/M002-CONTEXT.md`
- `.gsd/milestones/M002/M002-ROADMAP.md`

## Expected Output

- `scripts/repomix-review.sh`
- `.gsd/milestones/M002/slices/S06/repomix-pack.xml`
- `.gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md`

## Verification

test -x scripts/repomix-review.sh && test -s .gsd/milestones/M002/slices/S06/repomix-pack.xml && test -s .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md && grep -q 'Maintainability' .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md && grep -q 'Scalability' .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md && grep -q 'Extensibility' .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md && grep -q 'Findings' .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md && grep -q 'Verdict' .gsd/milestones/M002/slices/S06/S06-ARCHITECTURAL-REVIEW.md
