---
id: S09
parent: M012
milestone: M012
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["Gap matrix ToFixNow vocabulary retained in definition section even though no table rows use it — allows future milestones to reuse the status without re-defining it", "Data alignment notes added as additive section only — no existing §12 text modified — to prevent breaking doc-contract tests that assert specific strings", "UI handoff doc kept to scanning density (~2 min read target) rather than exhaustive — detailed contract lives in skill_legality_contract.md which the handoff references"]
patterns_established:
  - ["Doc-contract tests (ui_readiness_gap_matrix_docs.rs, skill_legality_contract_docs.rs) as machine-readable alignment between implementation and documentation — pattern reusable for future milestone handoffs", "Additive annotation sections in design docs preserve existing string-grep contracts while extending documentation — safe pattern for doc updates under test coverage"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-01T17:17:00.109Z
blocker_discovered: false
---

# S09: Doc/data alignment and UI handoff docs

**Gap matrix fully reclassified, data-vs-design divergences documented, and UI handoff doc written — all doc/data artifacts agree on what M012 shipped.**

## What Happened

S09 closed the documentation loop for M012. Three tasks delivered the documentation layer that makes the legality query surface usable by the next milestone team.

**T01 — Gap matrix reclassification:** All `ToFixNow` entries in `docs/combat_ui_readiness_gap_matrix.md` were reclassified to either `Implemented` or `Deferred` based on S02–S08 delivery. Revive, Ally Toughness, Zero-max Toughness, Energy caps, SP/ultimate readiness, Attacker state, Commander target, Structured failure reasons, and Windowed active unit are now `Implemented`. Row/AllEnemies TargetShape and mixed-effect (angemon_ult) are `Deferred` (explicitly gated via `UnimplementedTargetShape`/`UnimplementedEffect`). Enemy counterplay/telegraphs is `Deferred` with typed declarations from S08. The doc-contract test `gap_matrix_uses_each_required_status_family` was updated to remove `ToFixNow` from the required-in-table-rows assertion (retained in vocabulary definition for future milestones).

**T02 — Data alignment annotations:** A new `## Data Alignment Notes — MVP vs Design` section was added to `docs/combat_design.md` §12, cataloguing all known divergences between design-doc descriptions and actual RON data. Skill name mismatches (Baby Flame→Pepper Breath, Blue Blaster→Bubble Blast, etc.), SP cost differences, missing conditional effects (Burning/Slowed conditionals, micro-heal, Self-Advance), absent passives (Static Charge, bonus toughness), and missing second skills for five Child units are documented as intentional MVP simplifications, not bugs.

**T03 — UI handoff document:** `docs/ui_handoff_m012.md` was created as a standalone cold-start reference for the next graphical UI milestone. It covers: the four query function entry points in `action_query.rs`, the four status shapes and machine-readable reason codes, the full Implemented/Deferred/Hidden classification, data-vs-design simplification summary (referencing T02), API consumption patterns (never branch on skill IDs), and key file references. The no-hardcoding contract and source-scan guard tests are explicitly called out.

All three tasks verified green: `cargo test-dev --test ui_readiness_gap_matrix_docs` (7 tests), `cargo test-dev --test skill_legality_contract_docs` (10 tests), and full `cargo test-dev` suite (all binaries, 0 failures).

## Verification

- `cargo test-dev --test ui_readiness_gap_matrix_docs`: 7/7 pass — gap matrix vocabulary, mechanic names, reason codes, boundary text, no-placeholder, downstream contract, status family assertions all green
- `cargo test-dev --test skill_legality_contract_docs`: 10/10 pass — contract shapes, status families, reason codes, no-placeholder, engine parity, no-skill-ID boundary all green
- `cargo test-dev` full suite: all binaries, 0 failures
- `test -f docs/ui_handoff_m012.md && grep -q 'action_query' docs/ui_handoff_m012.md && grep -q 'UnitQuerySnapshot' docs/ui_handoff_m012.md`: PASS
- `grep -q 'Data Alignment' docs/combat_design.md`: PASS
- Gap matrix contains no remaining `ToFixNow` table rows; vocabulary section retains definition for future use

## Requirements Advanced

- R085 — Gap matrix updated to show zero ToFixNow entries; all M012-in-scope mechanics either Implemented or Deferred with explicit query declarations — primary R085 validation artifact complete

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

None.

## Files Created/Modified

- `docs/combat_ui_readiness_gap_matrix.md` — Reclassified all ToFixNow entries to Implemented or Deferred; updated evidence and UI truth risk columns
- `tests/ui_readiness_gap_matrix_docs.rs` — Removed ToFixNow from required-in-table-rows assertion; vocabulary definition test unchanged
- `docs/combat_design.md` — Added '## Data Alignment Notes — MVP vs Design' section documenting skill name, SP cost, conditional effect, passive, and second-skill divergences
- `docs/ui_handoff_m012.md` — Created standalone UI handoff document summarizing legality query surface, Implemented/Deferred/Hidden classification, data simplifications, API consumption patterns, and key file references
