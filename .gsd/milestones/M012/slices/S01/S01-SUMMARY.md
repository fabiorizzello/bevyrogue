---
id: S01
parent: M012
milestone: M012
provides:
  - ["legality-contract-vocabulary", "ui-readiness-gap-matrix", "doc-contract-test-pattern"]
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["Status vocabulary (ActionStatus/TargetStatus/ResourceStatus/ImplementationStatus with Enabled/Disabled/Deferred/Hidden) locked before DSL work", "24+ reason codes defined as stable machine-readable identifiers separate from display strings", "Hard boundary: no CLI/windowed skill-ID-specific legality rules — enforced by both docs and tests", "Doc-contract tests use include_str! for compile-time missing-doc detection rather than runtime file reads"]
patterns_established:
  - ["Doc-contract tests use include_str! for compile-time enforcement of tracked doc presence", "Executable doc tests assert on named substrings so failures identify exactly which mechanic/status/reason is missing", "Contract artifacts (docs/) are written before implementation slices begin to prevent vocabulary drift"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-30T19:22:04.007Z
blocker_discovered: false
---

# S01: UI-readiness gap matrix and legality contract

**Produced tracked, executable-proof docs defining the M012 UI-readiness gap matrix and shared legality/affordance contract, guarded by 17 green doc-contract tests.**

## What Happened

S01 delivered the two foundational contract artifacts that all downstream M012 slices depend on: the UI-readiness gap matrix and the legality/affordance contract.

**T01 — UI-readiness gap matrix** (`docs/combat_ui_readiness_gap_matrix.md`): Produced a comprehensive classification table covering 18+ UI-affecting mechanics including offensive single-target damage, revive, heal-like examples, cleanse/silence/guard, Row/AllEnemies TargetShape, SelfOnly, mixed-effect targets (angemon_ult), ally Toughness, zero-max enemy Toughness, Energy caps, SP/ultimate readiness, attacker state, commander target, Tamer Gauge/Commands, Child Tamer Gauge boost, enemy counterplay/telegraphs, structured failure reasons, and windowed active unit. Each mechanic is classified as `Implemented`, `ToFixNow`, `Deferred`, or `Hidden`, with owning downstream slice and UI truth risk noted. The hard boundary (no CLI/windowed skill-ID-specific legality rules) is stated explicitly. A companion test (`tests/ui_readiness_gap_matrix_docs.rs`, 7 tests) enforces presence of all required mechanics, classification vocabulary, requirement/decision links, and absence of placeholder text at CI time.

**T02 — Legality/affordance contract** (`docs/skill_legality_contract.md`): Defined the exact status vocabulary (`ActionStatus`, `TargetStatus`, `ResourceStatus`, `ImplementationStatus` with `Enabled/Disabled/Deferred/Hidden` variants), 24+ stable reason-code families covering attacker constraints, resource shortfalls, target legality, implementation gaps, and deferred systems (TamerGaugeDeferred, TamerCommandDeferred, ChargedTelegraphDeferred, EnergyCapReached, etc.). Specified that the pure query API consumes skill data + world snapshot, keeps display strings separate from reason codes, and mandates that S06 engine rejection derives `CombatEventKind::OnActionFailed` text from the same preflight reason. A companion test (`tests/skill_legality_contract_docs.rs`, 10 tests) enforces all status types, reason codes, R084/D053 links, engine parity requirement, and the no-skill-ID-specific-UI-rule boundary at CI time.

**Verification**: Both doc-contract test binaries pass — 17 tests total, 0 failures. No runtime code was added or changed; this slice is purely contract and governance artifacts.

## Verification

cargo test --test ui_readiness_gap_matrix_docs --test skill_legality_contract_docs: 17 tests, 0 failed. Both test binaries verified on disk. All four artifact files confirmed present (docs/combat_ui_readiness_gap_matrix.md, docs/skill_legality_contract.md, tests/ui_readiness_gap_matrix_docs.rs, tests/skill_legality_contract_docs.rs). No placeholder text (TBD/TODO) in either doc. All required mechanics, status families, reason codes, and requirement/decision links (R084, R085, D053, D054) asserted and passing.

## Requirements Advanced

- R084 — Defined the exact status vocabulary and reason-code families that S03-S07 must implement; doc-contract test enforces coverage at CI
- R085 — Produced executable gap matrix classifying all UI-affecting mechanics with downstream slice ownership; test enforces no mechanic is dropped

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

S02 must consume docs/combat_ui_readiness_gap_matrix.md to determine which Toughness/TargetShape semantics to fix first (ToFixNow items). S03-S07 should reference docs/skill_legality_contract.md as the authoritative vocabulary when naming DSL fields, query API return types, and CombatEventKind reason strings.

## Files Created/Modified

- `docs/combat_ui_readiness_gap_matrix.md` — R085 gap matrix: 18+ mechanics classified as Implemented/ToFixNow/Deferred/Hidden with downstream slices and UI truth risk
- `tests/ui_readiness_gap_matrix_docs.rs` — 7-test doc-contract test enforcing gap matrix vocabulary, requirement links, mechanics coverage, and no placeholder text
- `docs/skill_legality_contract.md` — R084 legality contract: status types, 24+ reason codes, engine parity requirement, no-skill-ID-specific-rule boundary
- `tests/skill_legality_contract_docs.rs` — 10-test doc-contract test enforcing status vocabulary, reason codes, R084/D053 links, engine parity, hard boundary
