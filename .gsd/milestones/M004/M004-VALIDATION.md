---
verdict: needs-remediation
remediation_round: 0
---

# Milestone Validation: M004

## Success Criteria Checklist
## Reviewer C — Assessment & Acceptance Criteria

## Acceptance Criteria

- [x] Zero hardcoded VFX-kind paths remain in `render.rs` | Evidence: S02 summary states `VfxParticleKind`, `kind_from_name`, `vfx_particle_kind`, and per-kind helpers were deleted from `src/windowed/render.rs`; S02 verification passed `render_no_vfx_kind_guard::render_rs_has_no_vfx_kind_dispatch`. S05 strengthened the same guard with positive boundary assertions; S05 verification shows `cargo test --test animation render_no_vfx_kind_guard` passed with 2 tests.

- [x] Every Agumon effect is expressed in `assets/digimon/agumon/vfx.ron` | Evidence: S02 summary records all five Baby Flame effects authored in `vfx.ron` and verified by `vfx_asset_load::agumon_vfx_contains_all_five_effects`. S03 summary records `baby_burner.detonate` plus `baby_burner.flash` authored via RON-only reuse and tested. S05 summary records `sharp_claws.slash` authored in `vfx.ron`, triggered from `anim_graph.ron`, and verified by `vfx_asset_load::agumon_vfx_contains_sharp_claws_slash`.

- [x] Adding an effect that reuses existing verbs is RON-only; a novel motion is one `register("ns/name", fn)` in the Digimon blueprint with no core change | Evidence: S02 created the `PlacementExt` registry axis and Agumon placement verb registration pattern, with `all_four_verbs_resolve_via_freshly_built_registries` passing. S03 Baby Burner detonate enrichment reused existing verbs with no `register_agumon_ext` change. S05 Sharp Claws reused `agumon/baby_flame/static` with no new verb registration or core change, documented in `M004-RENDERING-ACCEPTANCE.md`.

- [x] All placement/appearance/variation verb math is headless-tested and deterministic | Evidence: S01 verification passed 11 deterministic curve-eval tests including 1000-call bit-identical checks. S02 verification passed placement verb determinism and registry-resolution tests. S03 verification passed `cargo test --test animation` with 110 tests covering deterministic `VfxContext` variant selection. S05 verification passed `vfx_asset_eval`, including `sharp_claws_slash_curves_evaluate_deterministically`.

- [x] The `vfx.ron` schema is editor-ready: typed, introspectable, `Serialize` + `Deserialize` + `Reflect`, not a stringly-typed map | Evidence: S01 summary records typed `VfxAsset`, `EffectDef`, `Placement`, `Appearance`, `ScaleCurve`, and `ColorCurve` deriving serialization and `Reflect`, plus schema introspection tests. S02 summary records typed `PlacementParams` enum variants and Reflect tests: `placement_is_reflectable_with_typed_params_and_anchor`, `appearance_is_reflectable_with_expected_fields`, and `all_authored_effects_round_trip`.

- [ ] HDR + bloom + additive rendering is in place and the user signs off that all three skills look good in `cargo winx` | Evidence: S05 delivered HDR + Bloom + overbright color proof, verified by `vfx_windowed_contracts` tests and documented in `M004-RENDERING-ACCEPTANCE.md`. However, strict custom additive particle material is explicitly deferred by D037, and S06 / `docs/uat/M004-vfx-signoff.md` leaves Sharp Claws, Baby Flame, and Baby Burner visual verdicts as `PENDING`. This criterion is not fully covered.

Verdict: NEEDS-ATTENTION — automated contract evidence is strong, but the milestone still has uncovered manual visual signoff, and strict additive rendering was deferred rather than delivered.

## Slice Delivery Audit
| Slice | Claimed output | Delivered output | Status |
|---|---|---|---|
| S01 | Typed VfxAsset schema, pure curve eval, Agumon vfx.ron tracer, Baby Flame impact data path | SUMMARY, ASSESSMENT, and UAT artifacts present; automated tests/builds passed; visual checks marked human follow-up | PASS with human follow-up |
| S02 | Registry-resolved placement verbs, remove VfxParticleKind/kind_from_name, Baby Flame RON effects | SUMMARY, ASSESSMENT, and UAT artifacts present; reviewer evidence says grep guard and registry tests passed | PASS |
| S03 | Baby Burner detonate from RON, variant selection seam, no hardcoded VFX paths | SUMMARY, ASSESSMENT, and UAT artifacts present; reviewer evidence says deterministic variant and detonate contract tests passed | PASS with integration note |
| S04 | Requirements and boundary documentation cleanup | SUMMARY, ASSESSMENT, and UAT artifacts present, but roadmap still says `Boundary Map: Not provided`; boundary artifact exists outside roadmap | NEEDS-ATTENTION |
| S05 | Sharp Claws VFX or rescope; HDR bloom/additive criteria or rescope; automated evidence | SUMMARY, ASSESSMENT, and UAT artifacts present; Sharp Claws and HDR/Bloom automated evidence present; strict additive material deferred by D037 | NEEDS-ATTENTION |
| S06 | Human cargo winx UAT captured or formally waived | SUMMARY and UAT artifacts present, but no S06-ASSESSMENT.md found in artifact inventory; reviewer says visual verdicts remain `PENDING` | NEEDS-ATTENTION |
| S07 | Remediation slice added by validation | Added to roadmap via `gsd_reassess_roadmap` to close validation gaps | PENDING |

## Cross-Slice Integration
## Reviewer B — Cross-Slice Integration

## M004 Cross-Slice Boundary Review

Note: `.gsd/milestones/M004/M004-ROADMAP.md` still says **“Boundary Map: Not provided.”** I used the dedicated boundary artifact referenced by S04: `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`.

| Boundary | Producer Summary | Consumer Summary | Status |
|---|---|---|---|
| Owned VFX asset schema | S01 summary explicitly provides typed `VfxAsset` schema, deterministic eval/load seams, and `assets/digimon/agumon/vfx.ron`. | S02 requires S01 `VfxAsset` resolver/eval API and RonAssetPlugin wiring; S03 requires S01 typed schema/eval/load path; S05 requires S01 typed VfxAsset/eval/load contracts. | Honored |
| Placement verb registry | S02 summary explicitly provides Registry-resolved placement dispatcher, `PlacementExt`, registered Agumon placement verbs, and validation. | S03 requires S02 `PlacementExt`, registered verbs, validation, and render-dispatch seam; S05 requires S02 placement registry/windowed render-dispatch contracts. | Honored |
| AnimGraph presentation cue to owned effect ids | S03 summary provides Baby Burner detonate data path and windowed detonate spawn contract; S05 summary provides Sharp Claws AnimGraph `on_enter` trigger and render bridge. | S05 requires S03 AnimGraph cue/effect bridge patterns and extends them for Sharp Claws. | Honored |
| Effect chaining via `on_expire` | S02 summary provides projectile-to-impact chain via data; S03 summary provides Baby Burner detonate fan-out chaining flash. | S03 consumes S02 registry/render-dispatch path; S06 summary re-consumes the authored Baby Flame two-stage `impact -> impact_flash` chain during closeout verification. | Honored |
| Variant selection seam | S03 summary explicitly provides `VfxContext` + deterministic `select_variant` seam and dangling variant validation. | No later slice summary declares consumption of variant selection; boundary map itself labels this as a seam for future callers, not full feature wiring. | Needs attention: producer exists, but no M004 consumer summary consumes it. |
| Failure visibility and validation boundary | S01 summary provides load/missing-effect warnings; S02 summary provides `validate_effects` and warn-and-skip invalid data behavior; S03 extends validation to dangling variants. | S04 consumes these into validation/boundary documentation and executable doc checks; S05/S06 consume regression guards and validation proof during acceptance/closeout. | Honored |
| K001 manual visual boundary | S04 summary provides validation-scope and boundary documentation that preserves K001/manual-only visual signoff. | S06 summary explicitly consumes the K001 boundary by creating human-only runbook/capture helper and refusing to claim visual PASS. | Honored |
| S03 consumes S01 contract | S01 summary provides typed schema, `resolve_effect`, deterministic `eval_scale`/`eval_color`, and owned `vfx.ron` load path. | S03 summary frontmatter explicitly requires S01 typed `VfxAsset` schema, resolve/eval API, and owned load path. | Honored |
| S03 consumes S02 contract | S02 summary provides `PlacementExt`, registered placement verbs, `validate_effects`, registry-resolved render path, and no legacy VFX-kind dispatch. | S03 summary frontmatter explicitly requires S02 `PlacementExt`, registered verbs, validation, and registry-resolved windowed VFX data path. | Honored |

Verdict: NEEDS-ATTENTION — the roadmap still lacks an inline boundary map, and the variant-selection boundary has a producer summary but no M004 consumer summary.

## Requirement Coverage
## Reviewer A — Requirements Coverage

| Requirement | Status | Evidence |
|---|---|---|
| R004 | COVERED | S01 delivered pure deterministic `eval_scale` / `eval_color` with 11 green tests and explicitly lists R004 as validated in the slice summary. S05 re-verified deterministic Sharp Claws curves and authored color evaluation. |
| R005 | PARTIAL | M004 touches AnimGraph presentation triggers and effect bridging in S03/S05, but slice summaries do not show coverage of the broader stance FSM / SkillGraphRegistry / ult-energy loop described by R005. |
| R006 | PARTIAL | S04/S05 document and prove AnimGraph cue-to-effect presentation bridges, but the summaries do not show the R006 gameplay two-clock damage cue barrier or headless/windowed damage parity. |
| R007 | PARTIAL | S04/S05 preserve opaque presentation cues and no numeric gameplay payload, but the summaries do not cite the original `GameplayCommandForbidden` / no `EmitDamage`-style guard for AnimGraph gameplay commands. |
| R008 | MISSING | No M004 slice summary mentions skill-id-to-graph resolution, SkillGraphRegistry extensibility, or instant fallback behavior. |
| R010 | MISSING | No M004 slice summary mentions the phase strip UI, `EventReader<CombatEvent>`, or UI non-mutation proof. |
| R011 | PARTIAL | S05/S06 cover Agumon VFX for Sharp Claws, Baby Flame, and Baby Burner, but the summaries do not cover the full Agumon kit loop: HP bars, damage numbers, dummy death, Twin Core badge, and ult gauge end-to-end. |
| R012 | COVERED | S01 explicitly re-confirms no numeric gameplay payload in serialized command surfaces; S03 keeps `VfxContext` render-free; S04 validation scope and boundary map document the no-gameplay-payload seam; S05 extends VFX through RON/AnimGraph without new gameplay payload. |
| R014 | PARTIAL | S06 provides a tracked manual signoff runbook and capture helper, plus fresh regression proof, but visual signoff remains explicitly pending under K001 and no live windowed visual PASS is claimed. |
| R015 | MISSING | No M004 slice summary provides or updates the repomix-grounded architectural review report or finding triage described by R015. |
| R016 | COVERED | S01/S02/S03/S05/S06 repeatedly verify headless builds/tests, windowed feature builds/tests, no windowed dependency leak, no hardcoded VFX kind dispatch, and boundary regression guards. |

Verdict: FAIL if any missing.

Synthesis note: The system context says only R004 was validated by M004 and no requirements were advanced, so several reviewer findings may be scope-mapping issues rather than implementation defects. Remediation must explicitly reconcile M004's requirement scope with `.gsd/REQUIREMENTS.md` and document whether R005/R006/R007/R008/R010/R011/R014/R015/R016 are out-of-scope, already covered elsewhere, or require additional evidence.

## Verification Class Compliance
## Verification Classes

| Class | Planned Check | Evidence | Verdict |
|---|---|---|---|
| Contract | Headless tests load `assets/digimon/agumon/vfx.ron` into typed `VfxAsset`, evaluate placement/appearance/curve verb math deterministically, prove variant selection maps `VfxContext` to an effect-tree variant, and assert no hardcoded VFX-kind paths remain in `render.rs`. | S01: schema, RON load, curve eval, and deterministic tests passed. S02: placement registry, `validate_effects`, on-expire chain, and no-kind grep guard passed. S03: `VfxContext` + `select_variant` deterministic tests passed. S05/S06: fresh regression runs passed `vfx_asset_load`, `vfx_asset_eval`, and `render_no_vfx_kind_guard`. | PASS |
| Integration | Existing FSM cue/barrier release path still drives spawns; generic Registry id dispatcher replaces kind resolution only; all three Agumon skills render end-to-end in `cargo winx`. | Automated integration evidence is partial: S02/S03/S05 windowed-only contract tests prove registry-resolved dispatch, on-expire chains, Baby Burner detonate spawn contract, Sharp Claws `on_enter` bridge, and HDR/bloom camera policy. But live `cargo winx` end-to-end rendering for all three skills is human-only and remains pending in `docs/uat/M004-vfx-signoff.md`. | NEEDS-ATTENTION |
| Operational | No operational checks planned beyond contract/integration because this is a presentation/data milestone with no unattended lifecycle. | S01-S06 summaries list no operational readiness requirements. S06 adds a human-only capture helper and evidence-log path for future manual review, but does not introduce unattended runtime obligations. | PASS |
| UAT | Human visual review in `cargo winx` for Sharp Claws, Baby Flame, and Baby Burner; visual quality cannot be simulated and must be signed off or waived by a human. | S01-S03 UAT files define manual visual checks. S05 explicitly excludes human visual signoff. S06 creates `docs/uat/M004-vfx-signoff.md` and `scripts/capture-windowed-m004-vfx.sh`, but the signoff artifact currently says “Framework complete — human capture pending” and per-skill verdicts are `PENDING`. | NEEDS-ATTENTION |

Verdict: NEEDS-ATTENTION — automated contract evidence is strong, but the milestone still has uncovered manual visual signoff, and strict additive rendering was deferred rather than delivered.


## Verdict Rationale
Parallel reviewers did not all pass: requirements coverage returned FAIL, while cross-slice integration and acceptance verification returned NEEDS-ATTENTION. The core data-driven VFX implementation appears well covered by automated contract tests, but milestone closure is blocked by unresolved requirement-scope mapping, pending human UAT or waiver, a roadmap boundary-map mismatch, an unconsumed variant-selection seam, missing S06 assessment artifact, and a deferred additive-rendering criterion.

## Remediation Plan
A remediation slice S07 was added via `gsd_reassess_roadmap`. S07 must: (1) reconcile M004 requirement coverage against `.gsd/REQUIREMENTS.md`, explicitly marking non-M004 requirements out of scope or adding missing evidence; (2) move or link the S04 boundary map into the canonical roadmap validation surface and disposition the variant-selection seam as consumed or future-only; (3) create or repair the S06 ASSESSMENT artifact; (4) complete human `cargo winx` visual signoff for Baby Flame, Baby Burner, and Sharp Claws or formally waive it; and (5) document D037 additive-material deferral against the success criteria so validation can distinguish accepted rescope from missing work.
