---
id: T02
parent: S09
milestone: M012
key_files:
  - docs/combat_design.md
key_decisions:
  - Placed the new section as a subsection of §12 (using ## heading) rather than a standalone §12.x or appendix — keeps it visually inside the roster chapter where the divergences originate
  - Listed divergences as MVP simplifications, not bugs, with explicit pointer to RON files as runtime source of truth
duration: 
verification_result: passed
completed_at: 2026-05-01T17:13:00.300Z
blocker_discovered: false
---

# T02: Added '## Data Alignment Notes — MVP vs Design' section to combat_design.md §12 documenting skill name/SP/effect/slot divergences between design doc and shipped RON data

**Added '## Data Alignment Notes — MVP vs Design' section to combat_design.md §12 documenting skill name/SP/effect/slot divergences between design doc and shipped RON data**

## What Happened

Read `docs/combat_design.md` §12, `assets/data/skills.ron`, and `assets/data/units.ron` to verify all divergences listed in the task plan against actual data. All findings confirmed:\n\n- Skill name mismatches: Pepper Breath vs Baby Flame, Bubble Blast vs Blue Blaster, Arctic Torrent vs Blue Blaster (ult), Nova Blast vs Baby Burner, Diamond Storm vs Koyōsetsu, Holy Breeze+patamon_revive vs Air Shot+Wing Slap, Draconic Edge only vs Metal Cannon+Dash Metal.\n- SP costs: data uses 3–4 SP for Child skills; design specifies 1 SP.\n- Missing conditional effects on Basic attacks (Burn energy gain, Slow shield, micro-heal, Self-Advance) plus Static Charge passive and DORUmon toughness conditional — all absent from RON data.\n- Skill slot counts: Agumon, Gabumon, DORUmon, Renamon have only 1 skill_id; Tentomon has 1 (petit_thunder, no Static Charge); Patamon has 2 but different names.\n\nAdded the new subsection immediately before the §13 separator in `docs/combat_design.md`, leaving all existing §12 text untouched. The section includes three tables: divergent skill names, SP cost deltas, and missing second skills — plus a prose list of absent conditional effects. All existing doc-contract tests continue to pass.

## Verification

Ran three verification commands: `grep -q 'Data Alignment' docs/combat_design.md` (exit 0), `cargo test --test ui_readiness_gap_matrix_docs` (7/7 pass), `cargo test --test skill_legality_contract_docs` (10/10 pass).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -q 'Data Alignment' docs/combat_design.md` | 0 | ✅ pass | 10ms |
| 2 | `cargo test --test ui_readiness_gap_matrix_docs` | 0 | ✅ pass — 7/7 | 350ms |
| 3 | `cargo test --test skill_legality_contract_docs` | 0 | ✅ pass — 10/10 | 350ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `docs/combat_design.md`
