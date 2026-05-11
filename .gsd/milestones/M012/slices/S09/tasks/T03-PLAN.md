---
estimated_steps: 29
estimated_files: 1
skills_used: []
---

# T03: Write UI handoff summary doc for the next graphical UI milestone

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

## Inputs

- ``docs/combat_ui_readiness_gap_matrix.md` — updated gap matrix from T01`
- ``docs/combat_design.md` — updated design doc from T02`
- ``docs/skill_legality_contract.md` — existing contract doc (reference only)`
- ``src/combat/action_query.rs` — query API surface (read for actual function names)`
- ``src/data/skills_ron.rs` — skill DSL types (read for actual type names)`
- ``src/data/units_ron.rs` — unit data types (read for EnemyCounterplayKit types)`

## Expected Output

- ``docs/ui_handoff_m012.md` — new UI handoff summary doc`

## Verification

test -f docs/ui_handoff_m012.md && grep -q 'action_query' docs/ui_handoff_m012.md && grep -q 'UnitQuerySnapshot' docs/ui_handoff_m012.md
