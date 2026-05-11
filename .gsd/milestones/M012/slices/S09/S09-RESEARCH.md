# S09 Research: Doc/data alignment and UI handoff docs

**Depth: Light** — straightforward documentation alignment using patterns established by S01–S08. No new technology, no risky integration.

## Summary

S09 closes M012 by ensuring `docs/combat_design.md`, the RON data files, and the legality/query docs agree on what shipped kits actually do. It also produces a UI handoff summary so the next graphical UI milestone knows exactly what's implemented, deferred, and hidden.

## Requirement Coverage

- **R085** (primary): S09 must update the gap matrix classifications to reflect S01–S08 delivery. Items tagged `ToFixNow` that were resolved by earlier slices must move to `Implemented`. Items that remain `Deferred`/`Hidden` must reference the typed query declarations delivered by S05/S08.
- **R084** (secondary): The skill legality contract doc is already stable — S09 verifies it matches the implemented code but likely needs no changes.

## Key Findings

### 1. Design doc ↔ data mismatches (combat_design.md § 12 vs skills.ron/units.ron)

The design doc describes rich kit behavior per unit (conditional effects, passives, multi-skill kits for Children) while the actual data has simplified MVP implementations. Known categories of drift:

**Skill name divergences (design vs data):**
- Agumon: design says "Baby Flame" (1 SP), data has "Pepper Breath" (4 SP)
- Agumon ult: design says "Baby Burner", data has "Nova Blast"
- Gabumon: design says "Blue Blaster" (1 SP), data has "Bubble Blast" (3 SP)
- Gabumon ult: design says "Blue Blaster" (100 E), data has "Arctic Torrent"
- DORUmon: design says "Metal Cannon" + "Dash Metal", data has "Draconic Edge" only
- Patamon: design says "Air Shot" + "Wing Slap", data has "Holy Breeze" + "patamon_revive"
- Renamon: design says "Koyōsetsu" + "Kohenkyo", data has "Diamond Storm" only
- SP costs differ widely (design says 1 SP for most Child skills, data has 3-4 SP)

**Missing conditional effects in data:**
- Agumon Basic: design says "if Burning, +2 Energy" — data has plain damage
- Gabumon Basic: design says "shield ally if target Slowed" — data has plain damage
- Tentomon: design says "Static Charge" passive (3rd Electric hit grants ally Energy) — not in data
- Patamon Basic: design says "if vs Virus/debuffed, micro-heal ally" — data has plain damage
- Renamon Basic: design says "Self-Advance 5 AV" — data has plain damage
- DORUmon: design says "bonus toughness vs >50% Toughness" passive — not in data

**Missing skills in data (design lists 2+ per Child, data has 1):**
- Agumon, Gabumon, Dorumon, Renamon, Tentomon all have only 1 skill in `skill_ids` vs 2 described in design

**These are intentional MVP simplifications, not bugs.** S09 should document them explicitly so the UI milestone doesn't assume design-doc kits are what's implemented.

### 2. Gap matrix update needs

The gap matrix (`docs/combat_ui_readiness_gap_matrix.md`) still uses S01-era "current evidence" language for items that S02–S08 resolved:
- **Ally Toughness** — resolved by S02 (enemy-only helpers), now `Implemented` via team-aware exposure
- **Row/AllEnemies** — resolved by S03 (skills marked `Deferred(UnimplementedTargetShape)`), confirmed in skills.ron
- **Energy caps** — resolved by S05 (pipeline wiring), validated by R073
- **SP/ultimate readiness** — resolved by S04/S06 (query + engine parity)
- **Attacker state** — resolved by S04/S06
- **Commander target** — resolved by S04/S06
- **Structured failure reasons** — resolved by S04/S06 (LegalityReasonCode enum)
- **Windowed active unit** — resolved by S07
- **Enemy counterplay/telegraphs** — resolved by S08 (typed EnemyCounterplayKit + query helpers)
- **Zero-max enemy Toughness** — Goblimon has toughness_max: 0, handled by query as not breakable
- **Tamer Gauge/Commands** — declared as deferred by S05
- **Child Tamer Gauge boost** — declared as deferred by S05
- **Heal-like** — DSL has TargetHpRule, fixture `first_aid` exists; still `Deferred` for executable content
- **Cleanse/silence/guard** — still `Hidden` (no executable content)
- **Mixed-effect (`angemon_ult`)** — marked `Deferred(UnimplementedEffect)` in skills.ron; Row shape also deferred

### 3. Existing doc-contract test infrastructure

Two compile-time doc tests exist:
- `tests/ui_readiness_gap_matrix_docs.rs` — asserts vocabulary, reason codes, mechanic names, and boundary text
- `tests/skill_legality_contract_docs.rs` — asserts status shapes, reason codes, and parity rules

These use `include_str!()` pattern (MEM065). S09 doc updates must keep these tests green.

### 4. Skill legality contract doc

`docs/skill_legality_contract.md` is already stable and comprehensive. It defines all status shapes, reason codes, target/resource/implementation semantics, and consumer rules. No changes needed unless S09 discovers a gap.

## Recommendation

S09 has three natural tasks:

**T01 — Update combat_design.md with data-alignment annotations.** Add a brief section (or per-kit annotations in § 12) noting which design-doc skills/effects are simplified in the current data and which are deferred/hidden. This prevents the next UI milestone from building against design-doc assumptions.

**T02 — Update gap matrix classifications.** Change `ToFixNow` entries that S02–S08 resolved to `Implemented` or note them as delivered. Update "current evidence" text to reflect post-S08 state. Keep doc-contract tests green.

**T03 — Write UI handoff summary.** A new doc (`docs/ui_handoff_m012.md` or similar) that summarizes:
- What the legality query surface provides (action/target/resource/implementation status)
- What's implemented vs deferred vs hidden
- Known data-vs-design simplifications the UI should not assume are working
- How to consume the query API (snapshot + pure functions, not Bevy world access)
- Entry points for UI integration

## Implementation Landscape

**Files to modify:**
- `docs/combat_design.md` — add data-alignment notes in § 12 or a new section
- `docs/combat_ui_readiness_gap_matrix.md` — update classifications and evidence text
- `tests/ui_readiness_gap_matrix_docs.rs` — may need test adjustments if matrix vocabulary changes

**Files to create:**
- `docs/ui_handoff_m012.md` — UI integration handoff doc

**Files to read (reference only):**
- `assets/data/units.ron`, `assets/data/skills.ron` — current data truth
- `src/combat/action_query.rs` — query API surface
- `src/data/skills_ron.rs`, `src/data/units_ron.rs` — schema/type definitions

**Verification:**
- `cargo test-dev --test ui_readiness_gap_matrix_docs` — doc-contract tests stay green
- `cargo test-dev --test skill_legality_contract_docs` — contract doc tests stay green
- `cargo test-dev` — full suite green (no code changes expected, only docs)
- `cargo check --features "dev windowed"` — windowed compile (should be unaffected by doc-only changes)

## Skill Discovery

No external libraries or technologies involved — this is a documentation-only slice.

## Risks

Minimal. Doc-only changes carry near-zero regression risk. The only constraint is keeping the compile-time doc-contract tests green, which the existing `include_str!()` pattern makes easy to verify.
