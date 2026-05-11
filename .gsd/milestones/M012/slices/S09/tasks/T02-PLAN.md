---
estimated_steps: 24
estimated_files: 1
skills_used: []
---

# T02: Add data-alignment annotations to combat_design.md §12 roster descriptions

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

## Inputs

- ``docs/combat_design.md` — current design doc with §12 roster descriptions`
- ``assets/data/skills.ron` — current skill data (source of truth for actual skill names/costs)`
- ``assets/data/units.ron` — current unit data (source of truth for actual skill_ids per unit)`

## Expected Output

- ``docs/combat_design.md` — updated with data-alignment annotation section after §12`

## Verification

grep -q 'Data Alignment' docs/combat_design.md && cargo test-dev --test ui_readiness_gap_matrix_docs && cargo test-dev --test skill_legality_contract_docs
