# S09: Doc/data alignment and UI handoff docs — UAT

**Milestone:** M012
**Written:** 2026-05-01T17:17:00.109Z

# S09 UAT Script — Doc/data alignment and UI handoff docs

## Preconditions
- Working directory: `/home/fabio/dev/bevyrogue/.gsd/worktrees/M012`
- Toolchain: `rust-toolchain.toml` (cranelift dev profile)
- All prior slices S01–S08 complete

## TC-S09-01: Gap matrix has no ToFixNow table rows

**Steps:**
1. `grep '| \`ToFixNow\` |' docs/combat_ui_readiness_gap_matrix.md`

**Expected:** No output (exit code 1 — no matches). The table must contain only `Implemented` and `Deferred` status rows.

**Edge case:** The vocabulary definition section may still contain the word `ToFixNow` as a concept definition — verify this is in the vocabulary block, not a table row.

## TC-S09-02: Gap matrix vocabulary section defines all four statuses

**Steps:**
1. `grep -E 'Implemented|ToFixNow|Deferred|Hidden' docs/combat_ui_readiness_gap_matrix.md | head -20`

**Expected:** All four status names appear in the vocabulary definition section. The `| \`ToFixNow\` |` table row pattern is absent.

## TC-S09-03: Doc-contract tests pass

**Steps:**
1. `cargo test-dev --test ui_readiness_gap_matrix_docs`
2. `cargo test-dev --test skill_legality_contract_docs`

**Expected:** Both test binaries report all tests passed (7 and 10 respectively), exit code 0.

## TC-S09-04: Data alignment section present in combat_design.md

**Steps:**
1. `grep -n 'Data Alignment Notes' docs/combat_design.md`

**Expected:** At least one match, pointing to the new subsection. The section must not be empty.

## TC-S09-05: Data alignment documents known skill name divergences

**Steps:**
1. `grep -E 'Pepper Breath|Nova Blast|Bubble Blast|Arctic Torrent|Holy Breeze' docs/combat_design.md`

**Expected:** All five data-side skill names appear in the alignment section.

## TC-S09-06: Data alignment documents missing conditional effects

**Steps:**
1. `grep -E 'conditional|passive|Static Charge|Self-Advance' docs/combat_design.md`

**Expected:** References to missing conditionals and passives appear in the alignment section, flagged as intentional MVP simplifications.

## TC-S09-07: UI handoff doc exists and covers query entry points

**Steps:**
1. `test -f docs/ui_handoff_m012.md && echo EXISTS`
2. `grep -E 'query_action_affordances|query_target_affordances|query_enemy_trait_affordances|query_charged_telegraph_affordance' docs/ui_handoff_m012.md`

**Expected:** File exists; all four query function names appear.

## TC-S09-08: UI handoff doc covers UnitQuerySnapshot construction

**Steps:**
1. `grep -E 'UnitQuerySnapshot|build_snapshot_from_ecs' docs/ui_handoff_m012.md`

**Expected:** Both `UnitQuerySnapshot` and `build_snapshot_from_ecs` appear, documenting how to construct the snapshot for query calls.

## TC-S09-09: UI handoff doc covers Implemented/Deferred/Hidden classification

**Steps:**
1. `grep -E 'Implemented|Deferred|Hidden' docs/ui_handoff_m012.md | head -10`

**Expected:** All three status categories appear with representative mechanic examples.

## TC-S09-10: UI handoff doc states no-hardcoding contract

**Steps:**
1. `grep -i 'hardcod\|skill.id\|entity.name' docs/ui_handoff_m012.md`

**Expected:** The doc explicitly states that UI must not branch on skill IDs or entity names, referencing the source-scan guard tests.

## TC-S09-11: Full test suite green

**Steps:**
1. `cargo test-dev 2>&1 | grep -E "FAILED|^error"` 

**Expected:** No output — zero failures across all test binaries.

## TC-S09-12: Key file references present in handoff doc

**Steps:**
1. `grep -E 'action_query\.rs|skills_ron\.rs|units_ron\.rs|skill_legality_contract\.md|gap_matrix' docs/ui_handoff_m012.md`

**Expected:** All five key file/doc references appear, giving a next-agent entry point into the codebase.
