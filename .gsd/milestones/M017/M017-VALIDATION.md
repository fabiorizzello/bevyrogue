---
verdict: needs-attention
remediation_round: 0
---

# Milestone Validation: M017

## Success Criteria Checklist
## Success Criteria Checklist

| # | Criterion | Evidence |
|---|-----------|----------|
| [x] | `cargo check` + `cargo test` (full headless integration suite) green at milestone end | Live run: 40 test binaries, all `0 failed; 0 ignored`. `cargo check` exit 0. Confirmed by S01-T06, S02-T06, S03-T05, S04-T05, S05-T03, S06-SUMMARY verification block. |
| [x] | Zero references to `Burn`/`Freeze`/`Shock`/`DeepFreeze` in code or tests (except `Burn`/`Shock` as reserved variants per §H.1) | `Freeze` and `DeepFreeze`: zero hits in src/ and tests/. `Burn`/`Shock`: present only in the 7 canonical exempted locations (enum declaration, validator guard, no-op match arms, ordinal map, one inline unit test). S01-ASSESSMENT grep guard confirmed. |
| [x] | 5 canon statuses (`Heated`/`Chilled`/`Paralyzed`/`Slowed`/`Blessed`) implemented with correct §H.1 semantics | S03: Heated DoT +4 Fire bypassing stun, amp% pipeline; Chilled −20% AV-gain. S04: Paralyzed always-skip (100-turn deterministic loop), Slowed −30% AV first-apply. S05: Blessed ×1.15 damage dealt + +1 Ult charge per action (Reset-branch exempt). All tests pass. |
| [~] | Chilled −20% AV: unit-test only | S03: `chilled_speed_delta` helper verifies AV reduction at unit-test level only. No integration scenario asserts a visible turn-order shift for Chilled units. Marked PARTIAL by Reviewer A. |
| [x] | Single-instance policy + `refresh_max_dur` verified by deterministic tests | S02: `tests/status_refresh_max_dur.rs` (1/1). `tests/status_multi_kind_coexist.rs` (3-kind coexist 1/1). S02-ASSESSMENT PASS. |
| [x] | Cleanse removes only `BuffKind::Debuff`; `Blessed` (cleanse-immune) survives cleanse | S02: `tests/status_cleanse_policy.rs` (1/1). S05: `tests/status_blessed_cleanse_immune.rs` (2/2). Both PASS. |
| [x] | JSONL log + `ValidationSnapshot` emit canon names (no leak of old taxonomy) | S06: `tests/status_observability_canon.rs` asserts canon kind strings present and legacy strings absent. `tests/validation_snapshot.rs` 6/6 including `per_unit_statuses_populated_deterministically`. |
| [x] | RON loader rejects non-canon status ids at load-time with clear error | S01: `validate_skill_book_on_load` wired into `DataPlugin`. Injecting `kind: Burn` panics with descriptive error listing 5 valid canon ids. S01-ASSESSMENT edge case PASS. |
| [x] | No regressions on existing non-status tests (`combat_coherence`, `follow_up_chains`, `form_identity`) | Live run: `combat_coherence` 3/3, `follow_up_chains` 2/2, `form_identity` 10/10. Confirmed across every slice's verification block. |

## Slice Delivery Audit
## Slice Delivery Audit

| Slice | SUMMARY | UAT/Assessment | Verdict | Notes |
|-------|---------|----------------|---------|-------|
| S01 | ✅ exists | ✅ S01-ASSESSMENT.md PASS | PASS | Enum rewrite + RON migration + tests cascade. All 6 UAT checks + 2 edge cases pass. |
| S02 | ✅ exists | ✅ S02-ASSESSMENT.md PASS | PASS | StatusBag + refresh_max_dur + BuffKind cleanse. All 7 checks pass. |
| S03 | ✅ exists | ✅ S03-ASSESSMENT.md PASS | PASS | Heated DoT + amp% + Chilled −20% AV. All 8 checks pass. Chilled integration-level turn-order assertion noted as optional/deferred. |
| S04 | ✅ exists | ✅ S04-ASSESSMENT.md PASS | PASS | Paralyzed skip-turn + Slowed delay-on-apply. All 5 checks pass. |
| S05 | ✅ exists | ✅ S05-ASSESSMENT.md PASS | PASS | Blessed ×1.15 damage + +1 Ult charge + cleanse-immune. All 9 checks + regression suite pass. |
| S06 | ✅ exists | ✅ S06-UAT.md exists; S06-ASSESSMENT.md absent | PASS (from SUMMARY `verification_result: passed`) | Observability closure: JSONL + ValidationSnapshot canon naming. ASSESSMENT.md not created as separate artifact — verdict captured in SUMMARY front-matter only (documentation gap, no correctness impact). |

**No outstanding follow-ups or blocking known limitations across any slice.**

## Cross-Slice Integration
## Cross-Slice Integration

All cross-slice boundaries are honored. Every artifact produced by S01–S05 is confirmed delivered in the producer SUMMARY and consumed in the downstream consumer SUMMARY.

| Boundary | Producer Summary | Consumer Summary | Status |
|----------|-----------------|-----------------|--------|
| S01 provides `StatusKind enum` (Heated/Chilled/Paralyzed/Slowed/Blessed + reserved) | S01 confirmed: `StatusEffectKind` rewritten, all legacy removed, `verification_result: passed` | S02 built `StatusBag` on canon enum; S03/S04/S05/S06 reference canon kinds throughout | PASS |
| S01 provides `RON validator` (load-time allow-list) | S01 confirmed: `skills_ron.rs` rejects non-canon ids at load-time | S02/S03/S04 add `Effect::ApplyStatus` call sites with canon ids, all pass validator | PASS |
| S01 provides clean test suite baseline for S02–S05 | S01 confirmed: semantic assertions removed, lifecycle scaffolding preserved | S03 uses `StatusBag::apply` in test setup; S04/S05 built on clean substrate | PASS |
| S02 provides `StatusBag` component + `refresh_max_dur` | S02 confirmed: `StatusBag` introduced, `refresh_max_dur` in apply pipeline | S04 explicitly: "on top of the S02 StatusBag lifecycle"; S05 T01 confirms "already in place from S02" | PASS |
| S02 provides `BuffKind`-classified cleanse | S02 confirmed: `BuffKind` enum on `StatusEffect`, cleanse drains Debuff only | S05 T01: cleanse-immune test needed no src/ changes — S02 had already wired `BuffKind::Buff` immunity | PASS |
| S03 provides `status_amp_pct` lookup + `calculate_damage` extension | S03 confirmed: `status_amp_pct(bag, tag)` in `status_effect.rs`, `DamageBreakdown.status_amp_pct`, `calculate_damage` updated | S05 T02 threads `attacker_dmg_mult: f32` built on the same extended signature | PASS |
| S04 provides Paralyzed skip + Slowed delay-on-apply | S04 confirmed: `process_turn_advanced_system` Paralyzed block, `TurnAdvance{amount_pct:-30}`, `verification_result: passed` | S06 `status_observability_canon` drives `advance_turn_system` over all 5 canon statuses, asserts `"kind":"Paralyzed"` and `"kind":"Slowed"` in JSONL | PASS |
| S05 provides Blessed ×1.15 + +1 Ult charge + `affects: [S06]` delegation | S05 confirmed: three integration tests green; Known Limitation explicitly delegates JSONL/ValidationSnapshot naming to S06 | S06 confirms: `ValidationUnitSnapshot` extended with `statuses` field; `"kind":"Blessed"` asserted in JSONL — delegation fully consumed | PASS |
| S01–S05 → S06 observability closure | All five slices `verification_result: passed`, full `cargo test` green at each checkpoint | S06 `status_observability_canon.rs` exercises all 5 canon statuses on 5 distinct units; `validation_snapshot.rs` 6/6 deterministic fixture | PASS |

## Requirement Coverage
## Requirement Coverage

M017 did not register new R-numbered entries in `.gsd/REQUIREMENTS.md` (all prior requirements R086–R100 were validated under M015 and are in the baseline). All 8 success criteria live in the milestone roadmap only — this is a traceability gap (no R-numbered requirements to cross-reference), but not a coverage gap.

| Requirement (Success Criterion) | Status | Evidence |
|---|---|---|
| SC-1: `cargo check` + `cargo test` green at end of milestone | COVERED | All 6 slice summaries report `verification_result: passed`; each confirms `cargo test: 0 failed, 0 ignored` |
| SC-2: Zero legacy taxonomy references in code or tests (except reserved Burn/Shock) | COVERED | S01-T06 grep guard confirmed; live check: no Freeze/DeepFreeze in src/ or tests/; Burn/Shock only in exempt canonical locations |
| SC-3: 5 canon statuses with §H.1 semantics (Heated/Chilled/Paralyzed/Slowed/Blessed) | PARTIAL | S01 (enum), S03 (Heated DoT + amp%, Chilled −20% AV unit-test only), S04 (Paralyzed/Slowed), S05 (Blessed) — Chilled has no integration-level turn-order assertion |
| SC-4: Single-instance + `refresh_max_dur` verified by deterministic tests | COVERED | S02: `status_refresh_max_dur.rs` (1/1), `status_multi_kind_coexist.rs` (1/1) |
| SC-5: Cleanse removes only `BuffKind::Debuff`; Blessed survives cleanse | COVERED | S02: `status_cleanse_policy.rs` (1/1); S05: `status_blessed_cleanse_immune.rs` (2/2) |
| SC-6: JSONL log + `ValidationSnapshot` emit canon names | COVERED | S06: `status_observability_canon.rs` (zero legacy hits + all 5 canon kinds asserted); `validation_snapshot.rs` 6/6 |
| SC-7: RON loader rejects non-canon ids at load-time | COVERED | S01: `validate_skill_book_on_load` wired into `DataPlugin`; inject-Burn test panics with descriptive error |
| SC-8: No regression on non-status tests | COVERED | `combat_coherence` 3/3, `follow_up_chains` 2/2, `form_identity` 10/10 — confirmed across all slices |

**Gaps:**
- SC-3 partial: Chilled −20% AV verified at unit-test level only. No end-to-end integration test asserting a visible turn-order shift when Chilled is applied.
- No R-numbered entries in REQUIREMENTS.md for M017 criteria — traceability gap for future audits.

## Verification Class Compliance
## Verification Classes

| Class | Planned Check | Evidence | Verdict |
|-------|--------------|----------|---------|
| **Contract** | Enum vocabulary: 5 active canon variants + 2 reserved declared; RON validator allows only 5 at load time; `BuffKind` classification correct for Blessed vs Debuffs | S01-ASSESSMENT: enum rewrite verified, validator wired into `DataPlugin`, reserved Burn/Shock rejected with clear error listing 5 valid canon ids. S02-ASSESSMENT: `BuffKind::Buff` for Blessed, `BuffKind::Debuff` for debuffs confirmed by cleanse tests. | PASS |
| **Integration** | Per-status pipelines exercised end-to-end through headless Bevy systems: amp%, DoT, speed-mod, skip-turn, delay-on-apply, damage-mult, Ult-charge, cleanse-filter | S03-ASSESSMENT: `status_amp_pipeline.rs` 4/4. S04-ASSESSMENT: `status_paralyzed_skip.rs` 1/1, `status_slowed_delay.rs` 1/1. S05-ASSESSMENT: `status_blessed_offensive.rs` 4/4, `status_blessed_ult_charge.rs` 3/3, `status_blessed_cleanse_immune.rs` 2/2. S02-ASSESSMENT: `status_refresh_max_dur.rs` 1/1, `status_multi_kind_coexist.rs` 1/1, `status_cleanse_policy.rs` 1/1. Partial gap: Chilled turn-order shift integration test absent (unit-test only). | PASS (minor gap: Chilled) |
| **Operational** | `cargo check` + `cargo test` full suite green; `cargo run --bin bevyrogue` (headless smoke) exits 0; grep guard stable (zero uncontrolled legacy refs) | All 40 test binaries: 0 failed, 0 ignored. `cargo check` exit 0. Smoke exit 0 (tick budget reached). Freeze/DeepFreeze: zero hits in src/ and tests/. Burn/Shock: only in 7 exempted canonical locations. | PASS |
| **UAT** | Artifact-driven checks: each slice UAT documents scenario steps, expected outcomes, edge cases, and recorded verdicts in `S0N-ASSESSMENT.md` | S01-ASSESSMENT PASS (6/6 checks + 2 edge cases). S02-ASSESSMENT PASS (7/7 checks). S03-ASSESSMENT PASS (8/8 checks). S04-ASSESSMENT PASS (5/5 checks). S05-ASSESSMENT PASS (9/9 checks + regression suite). S06: UAT plan written (S06-UAT.md); SUMMARY records `verification_result: passed`; S06-ASSESSMENT.md absent as standalone artifact (documentation gap only). | PASS (minor: S06-ASSESSMENT.md absent) |


## Verdict Rationale
All 6 slices have passing verification (cargo check + full cargo test suite: 0 failed, 0 ignored across 40 test binaries). Seven of 8 success criteria are fully covered by integration-level evidence. One partial exists: Chilled's −20% AV reduction is verified only at the unit-test level (chilled_speed_delta helper) with no end-to-end integration scenario asserting a visible turn-order shift — this was flagged as optional in S03 and not closed by S04 or S06. Additionally, S06-ASSESSMENT.md is absent as a standalone artifact (UAT verdict captured only in S06-SUMMARY front-matter). These are documentation and test-completeness gaps, not functional failures — all runtime behavior is correct and the full test suite passes.
