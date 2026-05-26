# S07 Research: Validation remediation close scope and signoff gaps

## Summary

S07 is a documentation/validation-remediation slice, not a runtime feature slice. The implementation is largely complete through S06, but the milestone validation still fails because its validation surface is stale or over-broad:

- `.gsd/milestones/M004/M004-VALIDATION.md` verdict is `needs-remediation` with five named gaps: requirement-scope reconciliation, canonical/inline boundary map, variant seam disposition, S06 assessment evidence, human visual UAT or waiver, and D037 additive-material rescope.
- `.gsd/milestones/M004/M004-ROADMAP.md` still has `## Boundary Map\n\nNot provided.` even though S04 created `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`.
- S04 docs are stale after S05/S06. `python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py` currently fails because it expects the old test token `fn projectile_on_expire_chains_the_impact_fan()`, but current `tests/windowed_only/vfx_asset_impact_render.rs` has `fn projectile_on_expire_chains_the_impact_then_flash_fan()`.
- `docs/uat/M004-vfx-signoff.md` is still honest but unresolved: Sharp Claws, Baby Flame, and Baby Burner are all `PENDING`. Auto-mode must not run `cargo winx`, so milestone visual acceptance can only close if a human updates this file to `PASS-with-notes`/`WAIVED`, or the executor records an explicitly authorized formal waiver.
- D037 is already documented in `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`: strict custom additive material is deferred; S05 delivered HDR + bloom + overbright color as the visual-intent proxy. Validation needs to count this as accepted rescope, not missing implementation.
- The validation report's S06-assessment complaint appears stale: `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md` exists and records artifact-driven PASS while explicitly not claiming live visual signoff. `.gsd/milestones/M004/slices/S06/S06-UAT.md` also exists.

Depth: targeted research. No unfamiliar library/API work; the risk is validation honesty and stale artifact alignment.

## Skills Discovered

Installed skills already relevant in the prompt:

- `rust-development`, `rust-skills`, `rust-best-practices`: only for running/understanding Rust test surfaces; no new Rust feature work expected.
- `bevy`: background context for D037/HDR/bloom, but S07 should not change Bevy code unless validation finds a regression.
- `write-docs`: most relevant; S07 artifacts must be fresh-reader validation docs, not implicit session context.
- `observability`: relevant pattern is executable doc guards (`verify_s04_docs.py`-style) that fail with specific missing tokens.
- `design-an-interface`: relevant to separating canonical validation surfaces instead of mixing requirement scope, boundary map, UAT, and rendering rescope in one ambiguous blob.
- `grill-me`: relevant as a checklist discipline: every validation objection should have a specific answer and proof path.

No external `npx skills find` installs are needed: core technologies are local Rust/Bevy/GSD docs and already covered by installed skills. The `Skill` tool was not available in this tool namespace, so the above skill rules were applied manually.

## Recommendation

Plan S07 as a docs-plus-guard remediation with one explicit external/manual gate:

1. Create a canonical S07 closeout artifact, likely `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md`, that directly answers each M004 validation finding with status, disposition, evidence path, and whether it is a non-claim/manual-only boundary.
2. Update the canonical roadmap boundary surface. The least risky option is to replace `Boundary Map: Not provided.` in `.gsd/milestones/M004/M004-ROADMAP.md` with either an inline short boundary map or a canonical pointer to the S04/S07 boundary map. Because the validation complaint says “roadmap still lacks an inline boundary map,” prefer a compact inline table plus links to the full S04/S07 artifacts.
3. Update or supersede the S04 boundary/scope docs for post-S05/S06 truth: Sharp Claws and HDR/bloom are now delivered; strict additive is D037-deferred; visual signoff remains pending/waived; variant selection is seam-only/future-consumer, not an M004 integration blocker.
4. Repair `verify_s04_docs.py` or add a new S07 verification script so doc proof tokens match current test names and current scope. The failing first token is `projectile_on_expire_chains_the_impact_fan` → current `projectile_on_expire_chains_the_impact_then_flash_fan`.
5. Reconcile requirements coverage against `.gsd/REQUIREMENTS.md`: Active requirements are 0; R004/R005/R006/R007/R008/R010/R011/R012/R014/R015/R016 are already `validated`, mostly in M002/M003. M004 should not claim new validation for them. It can re-verify local constraints only, especially deterministic VFX math, no gameplay payload, and windowed-gating.
6. Handle visual UAT honestly. If a human has not updated `docs/uat/M004-vfx-signoff.md`, S07 cannot honestly produce a visual PASS. Either collect human input outside auto-mode and update the doc, or record a formal waiver in the signoff artifact. Do not mark the milestone fully visually accepted while all three skills remain `PENDING`.

## Implementation Landscape

### Validation report to satisfy

File: `.gsd/milestones/M004/M004-VALIDATION.md`

Key findings to close:

- Requirements coverage: reviewer scanned all validated requirements and over-attributed them to M004. S07 should explicitly state M004 has no active global requirement ownership and only re-verifies local/inherited constraints.
- Cross-slice boundary: roadmap says “Boundary Map: Not provided”; dedicated S04 artifact exists but is not canonical enough for validation.
- Variant selection: S03 proves `VfxContext` + `select_variant` as deterministic seam only; no live M004 consumer exists. Disposition should be “future-only seam intentionally proven, not required to have a later M004 consumer.”
- S06 assessment: validation inventory missed/preceded `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`. Executor should cite it and ensure any checker includes it.
- Additive material: D037 accepted strict additive deferral. S05 proof is HDR camera + Bloom + overbright channels; validation should not require custom additive material.
- UAT: `docs/uat/M004-vfx-signoff.md` remains `PENDING`; this is the only non-automatable closure item.

### Current proof surfaces

Files/tests that should be cited exactly:

- Requirements state: `.gsd/REQUIREMENTS.md` → Active requirements: 0; validated requirements are already closed elsewhere.
- Scope/boundary docs: `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`, `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`.
- Rendering rescope: `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`.
- Manual signoff framework: `docs/uat/M004-vfx-signoff.md`, `scripts/capture-windowed-m004-vfx.sh`, `.gsd/milestones/M004/slices/S06/S06-UAT.md`, `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`.
- Current test function names:
  - `tests/animation/vfx_asset_load.rs::agumon_vfx_contains_sharp_claws_slash`
  - `tests/animation/vfx_asset_load.rs::validate_effects_accepts_the_real_asset`
  - `tests/animation/vfx_asset_load.rs::projectile_on_expire_chains_the_impact_burst`
  - `tests/animation/vfx_asset_load.rs::baby_burner_detonate_is_fan_out_burst_chaining_flash`
  - `tests/animation/vfx_asset_eval.rs::sharp_claws_slash_curves_evaluate_deterministically_and_overbright`
  - `tests/animation/render_no_vfx_kind_guard.rs::render_rs_has_no_vfx_kind_dispatch`
  - `tests/animation/render_no_vfx_kind_guard.rs::render_rs_keeps_the_data_driven_effect_id_boundary`
  - `tests/windowed_only/vfx_asset_impact_render.rs::projectile_on_expire_chains_the_impact_then_flash_fan`
  - `tests/windowed_only/vfx_rendering_acceptance.rs::setup_camera_enables_hdr_bloom_tonemapping_and_deband_dither`
  - `tests/windowed_only/vfx_rendering_acceptance.rs::agumon_vfx_keeps_bloom_capable_overbright_color_channels`

### Existing executable guard status

Command run during research:

```bash
python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py
```

Result: exit 1, failing with:

```text
FAIL: missing token in tests/windowed_only/vfx_asset_impact_render.rs: fn projectile_on_expire_chains_the_impact_fan()
```

This is the best first proof for S07 because it establishes the validation-doc surface is stale before adding new closure docs.

## Natural Seams / Suggested Task Split

### Task A — Scope reconciliation and canonical remediation doc

Files:

- `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md` (new)
- Possibly `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` (update or supersede)

Purpose:

- Directly answer each validation finding.
- Explicitly separate “validated global requirements” from “M004 re-verified/local constraints.”
- Cite `.gsd/REQUIREMENTS.md` Active=0 and previously validated records.
- Record D037 accepted rescope and variant seam future-only disposition.

### Task B — Boundary map canonicalization

Files:

- `.gsd/milestones/M004/M004-ROADMAP.md`
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md` and/or S07 remediation doc

Purpose:

- Replace the roadmap’s `Boundary Map: Not provided.` with a compact producer→consumer table or a canonical pointer plus summary.
- Update stale S04 limit statements that say Sharp Claws/HDR are pending; after S05 they are delivered/automated-proof complete, while K001 visual quality remains manual-only.
- Add explicit variant row disposition: delivered deterministic seam; no M004 runtime unlock consumer intended.

### Task C — S06 evidence and UAT disposition

Files:

- `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`
- `.gsd/milestones/M004/slices/S06/S06-UAT.md`
- `docs/uat/M004-vfx-signoff.md`

Purpose:

- Ensure S06 assessment/UAT artifacts are cited and discoverable.
- If human visual verdict/waiver is available, update `docs/uat/M004-vfx-signoff.md` from PENDING to `PASS-with-notes` or `WAIVED` and record reviewer/date/evidence.
- If no human verdict/waiver exists, do not pretend this is closed. Planner should mark this as external input/blocker for milestone completion.

### Task D — Executable validation guard

Files:

- `.gsd/milestones/M004/slices/S04/verify_s04_docs.py` (repair existing guard) or `.gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py` (preferred new closeout guard)

Purpose:

- Fail clearly if roadmap lacks boundary map, S07 remediation doc is missing key dispositions, D037 is not cited, S06 assessment/UAT docs are missing, or signoff is still PENDING when the chosen closeout path requires signed/waived visual UAT.
- Update stale proof-token names to match current tests.

### Task E — Fresh automated regression proof

Commands:

```bash
python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py
# or the new S07 guard
cargo test --test animation vfx_asset_load
cargo test --test animation vfx_asset_eval
cargo test --test animation render_no_vfx_kind_guard
cargo check --features windowed
cargo test --features windowed --test windowed_only vfx_asset_impact_render
cargo test --features windowed --test windowed_only vfx_rendering_acceptance
```

Do not run `cargo winx` or `scripts/capture-windowed-m004-vfx.sh` in auto-mode.

## First Proof

First unblocker: repair the executable doc guard and make it pass against current source/test names. This immediately proves the validation surface is no longer stale and gives later doc edits a safety net.

Suggested first command after edits:

```bash
python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py
```

If the executor chooses a new S07 guard, run both the old repaired S04 guard and the new S07 guard so validation can trust either historical or canonical surfaces.

## Verification Plan

Minimum S07 verification, excluding manual UAT:

```bash
python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py
python3 .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py  # if created
cargo test --test animation vfx_asset_load
cargo test --test animation vfx_asset_eval
cargo test --test animation render_no_vfx_kind_guard
cargo check --features windowed
cargo test --features windowed --test windowed_only vfx_asset_impact_render
cargo test --features windowed --test windowed_only vfx_rendering_acceptance
```

Manual/waiver verification:

- Inspect `docs/uat/M004-vfx-signoff.md`.
- Passing milestone closure requires each skill verdict to be `PASS-with-notes` or `WAIVED`, plus final status `PASS` or `WAIVED`, reviewer/date/evidence fields filled.
- If the file remains `PENDING`, S07 can close only as “validation docs remediated, manual visual signoff still external,” not as full milestone validation PASS.

## Risks and Constraints

- K001 is the hard constraint: auto-mode must not launch the windowed binary. Do not run `cargo winx` or the capture helper.
- The validation report over-scoped requirements by treating already-validated historical requirements as M004-owned. S07 should not mutate requirements unless the human explicitly wants new requirement records; current `.gsd/REQUIREMENTS.md` has Active=0.
- Roadmap files are rendered from GSD DB in some flows. If editing `.gsd/milestones/M004/M004-ROADMAP.md` directly is overwritten by tooling, planner/executor should prefer the appropriate GSD roadmap/reassessment tool or record the boundary map in the S07 remediation artifact and ensure validation reads that canonical artifact.
- S04 artifacts currently contain statements that were true before S05/S06 (“Sharp Claws not proven,” “HDR pending”). Either update them carefully or mark them as S04 point-in-time and make S07 the current canonical closeout artifact. Avoid contradictory current validation surfaces.

## Open Questions / External Decisions

1. Does the project accept a formal `WAIVED` visual verdict for M004, or must a human run `cargo winx` and record `PASS-with-notes`? This is the only true closure blocker found.
2. Should the roadmap boundary map be physically inline in `M004-ROADMAP.md`, or is a canonical linked S07 artifact acceptable to the validator? The previous validation specifically complained the roadmap still says “Not provided,” so changing that text is safest.
3. Should `verify_s04_docs.py` remain the long-term guard, or should S07 add a new closeout-specific guard and leave S04 as historical? Recommendation: repair S04’s stale token and add S07 guard for current closeout semantics.
