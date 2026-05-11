# S09: Doc/data alignment and UI handoff docs

**Goal:** Docs, data, and query examples agree on what shipped kits do; gap matrix reflects S02–S08 delivery; future mechanics can plug into legality without a new hardcoded path.
**Demo:** After this: docs, data, and query examples agree on what shipped kits do; future mechanics can plug into legality without a new hardcoded path.

## Must-Haves

- Gap matrix has zero `ToFixNow` entries — all reclassified to `Implemented` or `Deferred` based on S02–S08 delivery
- combat_design.md §12 annotates which design-doc skills/effects are simplified in current data vs what's actually in skills.ron/units.ron
- UI handoff doc exists and summarizes the legality query surface, implemented vs deferred vs hidden mechanics, and data-vs-design simplifications
- `cargo test-dev --test ui_readiness_gap_matrix_docs` passes
- `cargo test-dev --test skill_legality_contract_docs` passes
- `cargo test-dev` full suite green

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Reclassify gap matrix to reflect S02–S08 delivery and fix doc-contract test** `est:30m`
  Update `docs/combat_ui_readiness_gap_matrix.md` to reclassify all `ToFixNow` entries. S02–S08 resolved every `ToFixNow` item: Revive, Ally Toughness, Zero-max Toughness, Energy caps, SP/ultimate readiness, Attacker state, Commander target, Structured failure reasons, and Windowed active unit are now `Implemented`. Row/AllEnemies TargetShape and Mixed-effect (angemon_ult) are now `Deferred` (explicitly gated via `UnimplementedTargetShape`/`UnimplementedEffect`). Enemy counterplay/telegraphs is now `Deferred` with typed declarations from S08.

For each reclassified row, update the 'Current evidence' column to reflect post-S08 state and the 'UI truth risk' column to reflect the resolved status. Keep the 'Contract note' column unchanged (it describes the contract, which remains valid).

The doc-contract test `tests/ui_readiness_gap_matrix_docs.rs` has a test `gap_matrix_uses_each_required_status_family` that asserts `| \`ToFixNow\` |` appears in at least one table row. Since M012 resolved all ToFixNow items, update this test to remove `ToFixNow` from the required-in-table-rows list (but keep the vocabulary definition test unchanged, since the classification vocabulary section still defines `ToFixNow` as a concept).

Slice: S09 — Doc/data alignment and UI handoff docs
Milestone: M012

R085 relevance: The gap matrix IS the R085 contract doc. Updating it to reflect what S01–S08 actually delivered is the primary R085 validation artifact.

Key constraints:
- The vocabulary definition section must still mention all four statuses (`Implemented`, `ToFixNow`, `Deferred`, `Hidden`) so the vocabulary test passes
- All other tests in `ui_readiness_gap_matrix_docs.rs` must remain green — mechanic names, reason codes, boundary text, etc. must stay in the doc
- The `Downstream use rules` section can note that `ToFixNow` is retained as vocabulary for future milestones even though M012 resolved all current instances
  - Files: `docs/combat_ui_readiness_gap_matrix.md`, `tests/ui_readiness_gap_matrix_docs.rs`
  - Verify: cargo test-dev --test ui_readiness_gap_matrix_docs && cargo test-dev --test skill_legality_contract_docs

- [x] **T02: Add data-alignment annotations to combat_design.md §12 roster descriptions** `est:25m`
  Add a brief subsection or per-kit annotations in `docs/combat_design.md` § 12 (Roster MVP — 6 Linee) documenting which design-doc skills, effects, and conditionals are simplified or absent in the current `assets/data/skills.ron` and `assets/data/units.ron`. This prevents the next UI milestone from building against design-doc assumptions that don't match the shipped data.

Slice: S09 — Doc/data alignment and UI handoff docs
Milestone: M012

Known divergences (from S09 research, verified against actual data files):

**Skill name divergences:**
- Agumon: design 'Baby Flame' (1 SP) → data 'Pepper Breath' (4 SP)
- Agumon ult: design 'Baby Burner' → data 'Nova Blast'
- Gabumon: design 'Blue Blaster' (1 SP) → data 'Bubble Blast' (3 SP)
- Gabumon ult: design 'Blue Blaster' (100 E) → data 'Arctic Torrent'
- DORUmon: design 'Metal Cannon' + 'Dash Metal' → data 'Draconic Edge' only
- Patamon: design 'Air Shot' + 'Wing Slap' → data 'Holy Breeze' + 'patamon_revive'
- Renamon: design 'Koyōsetsu' + 'Kohenkyo' → data 'Diamond Storm' only
- SP costs differ (design 1 SP for most Child skills, data 3-4 SP)

**Missing conditional effects in data:**
- Agumon Basic: design 'if Burning, +2 Energy' → data plain damage
- Gabumon Basic: design 'shield ally if target Slowed' → data plain damage
- Tentomon: design 'Static Charge' passive → not in data
- Patamon Basic: design 'if vs Virus/debuffed, micro-heal ally' → data plain damage
- Renamon Basic: design 'Self-Advance 5 AV' → data plain damage
- DORUmon: design 'bonus toughness vs >50% Toughness' passive → not in data

**Missing second skills:**
- Agumon, Gabumon, DORUmon, Renamon, Tentomon have only 1 skill_id in units.ron vs 2+ in design

Add a new section (e.g. '## Data Alignment Notes — MVP vs Design') after §12 or at the end of §12 listing these as intentional MVP simplifications, NOT bugs. Reference `assets/data/skills.ron` and `assets/data/units.ron` as the current source of truth. Read those files first to verify the research findings before writing.

Do NOT change any existing text in §12 — only add the new annotation section.
  - Files: `docs/combat_design.md`
  - Verify: grep -q 'Data Alignment' docs/combat_design.md && cargo test-dev --test ui_readiness_gap_matrix_docs && cargo test-dev --test skill_legality_contract_docs

- [x] **T03: Write UI handoff summary doc for the next graphical UI milestone** `est:30m`
  Create `docs/ui_handoff_m012.md` — a standalone handoff document summarizing what M012 delivered for the next graphical UI milestone team/agent. This doc should be readable cold by someone who hasn't seen M012's slice history.

Slice: S09 — Doc/data alignment and UI handoff docs
Milestone: M012

The doc must cover:

**1. What the legality query surface provides:**
- Pure function query over skill data + world snapshot (no Bevy World access needed)
- Four status shapes: ActionStatus, TargetStatus, ResourceStatus, ImplementationStatus
- Machine-readable reason codes (LegalityReasonCode enum)
- Entry points: `action_query.rs` — `query_action_affordances()`, `query_target_affordances()`, `query_enemy_trait_affordances()`, `query_charged_telegraph_affordance()`
- Snapshot construction: `build_snapshot_from_ecs()` / `build_snapshot_from_ecs_with_sp()`

**2. What's Implemented vs Deferred vs Hidden:**
- Implemented: offensive single-target, revive (KO-ally targeting), SP/ult readiness, energy caps, attacker state, commander rejection, enemy-only toughness, structured failure reasons, windowed active unit, enemy counterplay declarations (TempoAnchor, BreakSeal)
- Deferred: Row/AllEnemies targeting, SelfOnly targeting, mixed-effect semantics (angemon_ult), Tamer Gauge/Commands, Child gauge boost, heal-like content (DSL exists but no executable heal skill), enemy traits (TypeTrap, ReactiveArmor), charged attack telegraphs
- Hidden: Cleanse/Silence/Guard effects

**3. Known data-vs-design simplifications:**
- Reference the data-alignment section added to combat_design.md by T02
- Summarize: most Child units have 1 skill in data vs 2+ in design, SP costs differ, conditional effects not implemented, passives not in data

**4. How to consume the query API:**
- Build a UnitQuerySnapshot from ECS state (or construct one directly in tests)
- Call pure query functions with snapshot + skill book data
- Branch on status enum variants — never on skill IDs or entity names
- The no-hardcoding contract is enforced by source-scan tests in `tests/action_affordance_consumers.rs`

**5. Key files for UI integration:**
- `src/combat/action_query.rs` — query functions
- `src/data/skills_ron.rs` — SkillDef, Effect, TargetShape, SkillTargeting
- `src/data/units_ron.rs` — UnitDef, EnemyCounterplayKind, EnemyTraitDeclaration
- `docs/skill_legality_contract.md` — full contract reference
- `docs/combat_ui_readiness_gap_matrix.md` — classification of all mechanics

Read `src/combat/action_query.rs` to verify the actual function names and snapshot types before writing. Keep the doc concise — aim for a document a new agent can scan in 2 minutes to know where to start.
  - Files: `docs/ui_handoff_m012.md`
  - Verify: test -f docs/ui_handoff_m012.md && grep -q 'action_query' docs/ui_handoff_m012.md && grep -q 'UnitQuerySnapshot' docs/ui_handoff_m012.md

## Files Likely Touched

- docs/combat_ui_readiness_gap_matrix.md
- tests/ui_readiness_gap_matrix_docs.rs
- docs/combat_design.md
- docs/ui_handoff_m012.md
