---
id: M017
title: "Status taxonomy v0 rewrite (canon §H.1)"
status: complete
completed_at: 2026-05-13T11:14:36.120Z
key_decisions:
  - Reserved Burn/Shock as enum variants but rejected at RON validator allow-list (fail-fast, not silent no-op) — prevents future enum-space collisions while blocking runtime use
  - Delete legacy test assertions rather than #[ignored] during taxonomy migration — fresh rewrites per slice are more honest about coverage
  - StatusBag per-unit component with single-instance-per-kind enforcement at apply() — eliminates stacking bugs at the structural level
  - BuffKind-classified cleanse (Buff/Debuff polarity at creation time) — makes cleanse a one-liner, Blessed immune with no bespoke flag
  - Chilled −20% AV via derived-read at AV-gain site, not SpeedModifier component mutation — avoids Bevy ECS ordering hazards
  - Paralyzed skip-turn via action-dispatch gating in process_turn_advanced_system — turn advances, action suppressed, keeps pipeline deterministic
  - Slowed delay-on-apply via TurnAdvance { amount_pct: -30 } event emission — observable in JSONL, unidirectional pipeline
  - Blessed ×1.15 threaded through existing attacker_dmg_mult extension from S03 — no separate pipeline branch
  - BuffKind::Buff for Blessed provides cleanse-immunity without a bespoke flag — S05 needed zero src/ changes beyond test writing
key_files:
  - src/combat/status_effect.rs
  - src/combat/damage.rs
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/sp.rs
  - src/combat/ultimate.rs
  - src/combat/observability.rs
  - src/combat/jsonl_logger.rs
  - src/data/skills_ron.rs
  - assets/data/skills.ron
  - tests/status_amp_pipeline.rs
  - tests/status_paralyzed_skip.rs
  - tests/status_slowed_delay.rs
  - tests/status_blessed_offensive.rs
  - tests/status_blessed_ult_charge.rs
  - tests/status_blessed_cleanse_immune.rs
  - tests/status_refresh_max_dur.rs
  - tests/status_multi_kind_coexist.rs
  - tests/status_cleanse_policy.rs
  - tests/status_observability_canon.rs
  - tests/validation_snapshot.rs
lessons_learned:
  - Deferring an integration test without naming a follow-up owner creates a silent coverage gap: Chilled turn-order shift was deferred in S03 and never closed, landing SC-3 as PARTIAL
  - Load-time RON id allow-list is the correct pattern for data-driven status gating: surfaces migration misses before any system runs and prints actionable error messages
  - BuffKind-classified cleanse at creation time requires zero downstream logic: S05 Blessed cleanse-immune test needed no src/ changes because S02 had already wired the necessary protection
  - Derived-read at calculation site (vs component mutation) is the right approach for conditional speed modifiers in Bevy ECS — discovered mid-S03 when SpeedModifier mutation created ordering hazards
  - Each slice UAT should produce a standalone S0N-ASSESSMENT.md artifact; S06 captured verdict only in SUMMARY front-matter — acceptable one-off but should not become convention
---

# M017: Status taxonomy v0 rewrite (canon §H.1)

**Replaced the legacy Burn/Freeze/Shock/DeepFreeze status taxonomy with the 5 canon §H.1 variants (Heated/Chilled/Paralyzed/Slowed/Blessed), wired all per-status semantics into the combat pipeline, and verified canon naming in JSONL logs and ValidationSnapshots — full test suite green across 40 binaries.**

## What Happened

M017 rewrote the combat kernel's status effect vocabulary from the ground up in 6 sequential slices.

**S01** replaced the legacy StatusEffectKind enum (Burn/Freeze/Shock/DeepFreeze) with the 5 canon §H.1 variants plus 2 reserved gas-era variants (Burn/Shock declared but rejected at load-time). All src/, tests/, and assets/ were migrated; legacy semantic test assertions were deleted entirely rather than #[ignored] to ensure fresh rewrites in S03–S05 were honest about coverage. A load-time RON validator (`validate_skill_book_on_load`) was wired into DataPlugin to reject non-canon ids with a clear error listing the 5 valid ids.

**S02** introduced StatusBag as a per-unit consolidated component with single-instance-per-kind enforcement at apply(), refresh_max_dur policy, and BuffKind-classified cleanse (Buff entries immune, Debuffs drainable). This set the structural invariant that all downstream slices relied on.

**S03** wired Heated DoT (4 Fire HP at turn-end, bypassing stun) and the status_amp_pct lookup into calculate_damage (fire amp% for Heated, ice amp% for Chilled). Chilled −20% AV was implemented as a derived read at the AV-gain site rather than a SpeedModifier component mutation, avoiding Bevy ECS ordering hazards. Integration test `status_amp_pipeline.rs` (4/4) verified the amp% pipeline end-to-end.

**S04** implemented Paralyzed (action-dispatch gating in process_turn_advanced_system — turn still advances, action is suppressed) and Slowed (first-apply emits TurnAdvance { amount_pct: -30 }). Integration tests `status_paralyzed_skip.rs` (100-turn deterministic loop) and `status_slowed_delay.rs` both passed.

**S05** implemented Blessed: ×1.15 damage dealt (threaded through the S03 attacker_dmg_mult extension), +1 Ult charge per action in apply_effects (Reset-branch exempt), and cleanse-immunity via S02's BuffKind::Buff classification (no src/ changes needed beyond S02). Three integration test files (4/4, 3/3, 2/2) all passed.

**S06** closed the observability loop: `tests/status_observability_canon.rs` drives all 5 canon statuses on 5 distinct units and asserts canon kind strings present and legacy strings absent in the JSONL event stream. `tests/validation_snapshot.rs` (6/6) verified ValidationUnitSnapshot.statuses field is deterministically populated.

Final state: `cargo check` + `cargo test` green across 40 test binaries (0 failed, 0 ignored). Zero Freeze/DeepFreeze references in src/ or tests/. Burn/Shock present only in 7 canonical exempt locations. One minor gap: Chilled integration-level turn-order shift test was deferred in S03 and never picked up — Chilled runtime behavior is correct but the end-to-end turn-order assertion remains at unit-test level only.

## Success Criteria Results

| # | Criterion | Result |
|---|-----------|--------|
| SC-1 | `cargo check` + `cargo test` green at milestone end | PASS — 40 test binaries, 0 failed, 0 ignored. `cargo check` exit 0. |
| SC-2 | Zero refs to Burn/Freeze/Shock/DeepFreeze (except reserved) | PASS — Freeze/DeepFreeze: 0 hits in src/ and tests/. Burn/Shock: only in 7 exempt canonical locations. |
| SC-3 | 5 canon statuses with §H.1 semantics | PARTIAL — All 5 implemented and runtime-correct. Chilled −20% AV verified at unit-test level only; no integration-level turn-order shift assertion (deferred in S03, not closed). |
| SC-4 | Single-instance + refresh_max_dur by deterministic tests | PASS — `status_refresh_max_dur.rs` (1/1), `status_multi_kind_coexist.rs` (1/1). |
| SC-5 | Cleanse removes only BuffKind::Debuff; Blessed survives cleanse | PASS — `status_cleanse_policy.rs` (1/1), `status_blessed_cleanse_immune.rs` (2/2). |
| SC-6 | JSONL log + ValidationSnapshot emit canon names | PASS — `status_observability_canon.rs` asserts 5 canon kinds present, 4 legacy names absent. `validation_snapshot.rs` 6/6. |
| SC-7 | RON loader rejects non-canon ids at load-time | PASS — `validate_skill_book_on_load` wired into DataPlugin; inject-Burn test panics with descriptive error. |
| SC-8 | No regressions on non-status tests | PASS — `combat_coherence` 3/3, `follow_up_chains` 2/2, `form_identity` 10/10. |

## Definition of Done Results

| Item | Status |
|------|--------|
| S01 complete (enum rewrite + RON migration) | ✅ — 6/6 tasks done, ASSESSMENT PASS |
| S02 complete (StatusBag + refresh_max_dur + cleanse policy) | ✅ — 6/6 tasks done, ASSESSMENT PASS |
| S03 complete (Heated DoT + amp% + Chilled −20% AV) | ✅ — 5/5 tasks done, ASSESSMENT PASS |
| S04 complete (Paralyzed skip-turn + Slowed delay-on-apply) | ✅ — 5/5 tasks done, ASSESSMENT PASS |
| S05 complete (Blessed ×1.15 + Ult charge + cleanse-immune) | ✅ — 3/3 tasks done, ASSESSMENT PASS |
| S06 complete (JSONL + ValidationSnapshot observability) | ✅ — 2/2 tasks done, verdict in SUMMARY front-matter (ASSESSMENT.md absent — documentation gap only) |
| All slice summaries exist | ✅ |
| Cross-slice integrations verified | ✅ — All 8 boundary crossings confirmed in M017-VALIDATION.md |
| Full test suite green | ✅ — 40 binaries, 0 failed, 0 ignored |

## Requirement Outcomes

M017 did not register new R-numbered entries in REQUIREMENTS.md — all success criteria lived in the milestone roadmap only. No requirement status transitions needed. The validated baseline is now: M017 Status taxonomy v0 rewrite (canon §H.1), superseding the M015/M016 combat authority baseline for status-related work.

## Deviations

["SC-3 (Chilled −20% AV) closes PARTIAL: end-to-end turn-order shift integration test was deferred in S03 as optional and never picked up by S04 or S06. Runtime behavior is correct; the gap is test coverage only.", "S06-ASSESSMENT.md absent as standalone artifact: UAT verdict was captured in S06-SUMMARY front-matter only. Documentation gap, no correctness impact.", "157 files changed across the branch (6858 insertions, 2939 deletions) — larger than anticipated for a vocabulary + apply/tick scope, due to the S01 test cascade migration and per-status pipeline integrations across multiple files per slice."]

## Follow-ups

["M018: AdvanceTurn/DelayTurn split + cap ±50% + gauge clamp [0,200] + TargetShape resolver expansion (boundary map delegatee)", "M019: DR pipeline BuffKind::DR + clamp 0.5 + Heal/Cleanse Effects as Effect variants (boundary map delegatee)", "M020: Reactive event variants (StatusApplied as typed event, UltimateUsed, UnitDied payload extension) (boundary map delegatee)", "M018 opportunity: close the Chilled integration-level turn-order shift test if the turn pipeline is refactored"]
