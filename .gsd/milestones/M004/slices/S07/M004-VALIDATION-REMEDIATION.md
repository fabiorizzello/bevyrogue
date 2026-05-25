# M004 Validation Remediation Closeout

Milestone M004 / Slice S07 / Task T02.

This is the **canonical closeout artifact** for the remediation findings recorded in `.gsd/milestones/M004/M004-VALIDATION.md`. It consolidates the final disposition of the validation gaps that were previously split across S04 scope/boundary docs, S05 rendering acceptance, and S06 assessment/UAT artifacts.

## Reader contract

Use this file as the first answer surface for the M004 rerun of milestone validation.

- It states what M004 does and does **not** claim about requirement ownership.
- It points to the canonical producer → consumer boundary evidence.
- It distinguishes automated proof, accepted rescope, future-only seams, and external manual blockers.
- It preserves the K001 honesty boundary: **auto-mode did not run `cargo winx` and does not claim live visual PASS.**

## Requirement scope

`.gsd/REQUIREMENTS.md` currently reports:

- **Active requirements: 0**
- **Validated: 11** — `R004`, `R005`, `R006`, `R007`, `R008`, `R010`, `R011`, `R012`, `R014`, `R015`, `R016`

Therefore M004 should not be read as creating or newly validating fresh global requirement records. Instead, M004 re-verifies and documents **local / inherited constraints** around:

- headless-first behavior,
- deterministic authored VFX math,
- windowed-only rendering boundaries,
- no numeric gameplay payload in the presentation seam,
- data-driven effect loading / chaining / validation.

The validation finding that marked several requirements as PARTIAL or MISSING is resolved as a **scope-mapping issue**, not as evidence that M004 newly owns those requirements. Those requirements were already validated earlier and remain validated in `.gsd/REQUIREMENTS.md`; M004 only re-verifies local contracts that support them.

### Previously validated global requirements referenced by M004

| Requirement | Current status in `.gsd/REQUIREMENTS.md` | M004 relationship |
|---|---|---|
| R004 | validated | Re-verifies deterministic VFX eval / selection contracts locally |
| R005 | validated | Re-verifies windowed dependency-gating / presentation-boundary constraints locally |
| R006 | validated | References earlier two-clock cue-barrier proof; does not re-own full gameplay parity validation |
| R007 | validated | Preserves the no-gameplay-command presentation boundary; does not claim a new global validation event |
| R008 | validated | Not re-implemented by M004; variant-selection seam is future-consumer-only, not missing runtime integration |
| R010 | validated | Not in M004 runtime scope; prior validation stands |
| R011 | validated | M004 contributes VFX improvements used by the Agumon kit, but does not replace the earlier full-kit validation |
| R012 | validated | Re-verifies the no-numeric-gameplay-payload presentation seam locally |
| R014 | validated | S06 preserves the human-only UAT boundary and runbook framework; no live visual PASS is claimed here |
| R015 | validated | Prior architectural review validation stands; M004 does not create a new review obligation |
| R016 | validated | M004 re-verifies hygiene / boundary invariants locally without changing global status |

## Validation finding disposition table

| Validation finding | Disposition | Classification | Canonical evidence |
|---|---|---|---|
| Requirement coverage report treated R005/R006/R007/R008/R010/R011/R014/R015/R016 as partial or missing | Resolved by explicit scope mapping: these requirements are already validated globally and M004 only re-verifies local / inherited constraints | **Automated proof + scope reconciliation** | `.gsd/REQUIREMENTS.md`; `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`; this file |
| Roadmap/validation flow said boundary map was not provided | Resolved at the artifact layer: S04 already authored the dedicated boundary map and this closeout marks it as canonical evidence for rerun | **Automated proof / documentation closure** | `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`; `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` |
| Variant-selection boundary had a producer but no consumer summary | Resolved as an intentional seam classification: S03 proves a deterministic future-consumer seam, not a missing M004 runtime integration | **Future-only seam, already proven** | `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`; `tests/animation/vfx_variant_selection.rs`; S03 summary |
| S06 assessment artifact was reported missing | Resolved: the artifact exists and records the automatable evidence plus the honest manual boundary | **Automated proof / documentation correction** | `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`; `.gsd/milestones/M004/slices/S06/S06-UAT.md` |
| Strict additive rendering was not fully delivered | Resolved by accepted rescope under D037: strict custom additive material is deferred; HDR + Bloom + overbright channels are the accepted S05 proxy | **Accepted rescope** | `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`; `.gsd/DECISIONS.md` (D037) |
| Human `cargo winx` visual signoff is still not complete | Not resolved by automation; remains an honest manual boundary until a human signs off or waives it in the tracked artifact | **External blocker / manual-only** | `docs/uat/M004-vfx-signoff.md`; `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`; `.gsd/milestones/M004/slices/S06/S06-UAT.md` |

## Boundary map

The canonical producer → consumer boundary evidence remains the S04 artifact:

- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`

That file should be read as the authoritative boundary inventory for rerun validation. The key dispositions are:

| Boundary | Status | Notes |
|---|---|---|
| Owned VFX asset schema | Delivered | Typed `VfxAsset` / `EffectDef` / placement / appearance / variants schema is proven by authored asset and schema/load tests |
| Placement verb registry | Delivered | Windowed render consumes registered placement ids rather than reviving hardcoded kind dispatch |
| AnimGraph cue → owned effect-id bridge | Delivered with explicit limit | Cue-name/effect-id bridge exists, but this is not a claim of fully generic cue registration |
| Effect chaining via `on_expire` | Delivered | Projectile → impact and Baby Burner detonate → flash remain data-driven |
| Variant selection seam | Delivered as seam only | Proven deterministic selector for future callers; not a missing gameplay feature in M004 |
| Failure visibility / validation boundary | Delivered | Invalid authored data produces named validation failures or localized warnings |
| K001 manual visual boundary | Still manual-only | Human-visible quality remains outside auto-mode proof |

## Variant seam disposition

The M004 validation report flagged the variant-selection boundary because no later slice summary declared a consumer. The correct disposition is:

- **Variant selection is not an unfulfilled runtime integration promise in M004.**
- It is a **deterministic future-consumer seam** authored and proven in S03.
- The contract being validated is the existence and correctness of the selector, not live consumption by a gameplay progression system.

Evidence:

- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`
- `tests/animation/vfx_variant_selection.rs::select_variant_maps_context_to_expected_effect`
- `tests/animation/vfx_variant_selection.rs::select_variant_is_deterministic_across_repeated_calls`
- `tests/animation/vfx_variant_selection.rs::select_variant_returns_none_for_unmapped_keys`

## S06 evidence

S06 now has both of the evidence artifacts that the validation report expected:

- `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`
- `.gsd/milestones/M004/slices/S06/S06-UAT.md`

What they prove:

- the signoff runbook exists and correctly points humans at `cargo winx`,
- the capture helper exists and is explicitly human-only,
- the automated regression set passed,
- the signoff artifact remains honest about the lack of live human PASS,
- auto-mode did **not** launch the windowed binary.

Important limit:

- `S06-ASSESSMENT.md` is an **artifact-driven closeout assessment**, not a substitute for human-eye visual acceptance.

## D037 rendering rescope

D037 is the accepted rendering rescope for the remaining additive-rendering gap.

The closeout disposition is:

- **Delivered in S05:** HDR-capable camera policy, Bloom, overbright linear color channels, Sharp Claws VFX authored through the data-driven seam, and no-hardcoded-VFX-kind regression protection.
- **Deferred by D037:** strict custom additive particle material.

That means milestone rerun validation should count:

- HDR + Bloom + overbright authored channels as the accepted delivered rendering proxy,
- strict additive material as **deferred by decision**, not silently missing work.

Canonical evidence:

- `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`
- `.gsd/DECISIONS.md` (`D037`)

## UAT disposition

The current tracked UAT state is:

- `docs/uat/M004-vfx-signoff.md` says **Framework complete — human capture pending**.
- Per-skill verdicts for Sharp Claws, Baby Flame, and Baby Burner remain **PENDING**.
- S06 explicitly records that no `cargo winx` session was run by auto-mode.

Therefore the honest disposition for rerun validation is:

- **No live visual PASS is claimed.**
- **No automated artifact should be interpreted as human visual signoff.**
- Until a human completes signoff or records a formal waiver, this remains the only external manual blocker to full visual closure.

## Fresh-validator guidance

A fresh validator should use the following rules:

1. Treat `.gsd/REQUIREMENTS.md` as the source of truth for global requirement status; M004 does not create new active requirements.
2. Treat S04's scope doc and boundary map as the canonical explanation of what M004 automation does and does not prove.
3. Treat S05 as the canonical explanation of the D037 rendering rescope.
4. Treat S06 as proof that the manual signoff framework and automated regression baseline exist, while preserving the no-live-visual-PASS boundary.
5. Treat the unresolved visual signoff as an **external manual blocker**, not as a hidden defect in the automated VFX seam.

## Verification commands

This task's local verification surface is intentionally small and does not run the windowed binary:

```bash
test -s .gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md
```

Related proof surfaces cited by this document:

```bash
python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py
cargo test --test animation vfx_asset_load -- --nocapture
cargo test --test animation vfx_asset_eval -- --nocapture
cargo test --test animation render_no_vfx_kind_guard -- --nocapture
cargo check --features windowed
cargo test --features windowed --test windowed_only vfx_asset_impact_render -- --nocapture
cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture
```

None of the above constitutes a claim that `cargo winx` was run by auto-mode.
