---
id: T01
parent: S02
milestone: M011
key_files:
  - src/combat/follow_up_tests.rs
  - src/combat/turn_system/tests.rs
  - src/combat/turn_system/pipeline.rs
  - tests/roster_smoke.rs
  - tests/pipeline_dispatch.rs
  - tests/follow_up_triggers.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-27T12:37:09.347Z
blocker_discovered: false
---

# T01: Atomic rename Element→DamageTag completed across all source, test, and RON files; suite fully green

**Atomic rename Element→DamageTag completed across all source, test, and RON files; suite fully green**

## What Happened

Most of the rename had already been applied in prior sessions: `types.rs` already defined `enum DamageTag { Physical, Fire, Ice, Electric, Light, Dark }`, `SkillDef.damage_tag`, `UnitDef.basic_damage_tag`, `OnBreak { damage_tag }`, and all RON files were already updated. The remaining stale `Element::` references were in five files:

1. `src/combat/follow_up_tests.rs` — import, `spawn_combatant` signature (`Vec<Element>`), `skill()` helper signature and struct literal field (`element:`→`damage_tag:`), plus all call-sites: `Element::Fire/Light/Dark/Water` remapped to `DamageTag::Fire/Light/Dark/Ice` respectively. `CombatEventKind::OnBreak { element: ... }` pattern updated to `{ damage_tag: ... }`.
2. `src/combat/turn_system/tests.rs` — import, `skill()` helper signature and field, call-sites (`Element::Fire`→`DamageTag::Fire`, `Element::Water`→`DamageTag::Ice`, `element: Element::Light`→`damage_tag: DamageTag::Light`).
3. `src/combat/turn_system/pipeline.rs` — `CombatEventKind::OnBreak { element }` match arm + `LogEntry::Break { element: *element }` emit — both renamed to `damage_tag`.
4. `tests/roster_smoke.rs` — import, `basic_element:`→`basic_damage_tag:` (×2), `weaknesses: vec![Element::Fire]`→`DamageTag::Fire`.
5. `tests/pipeline_dispatch.rs` — fully-qualified `bevyrogue::combat::types::Element::Fire`→`DamageTag::Fire`.
6. `tests/follow_up_triggers.rs` — `LogEntry::Break { target, element }` match arm renamed to `damage_tag`.

Variant remapping applied: Water→Ice, Electro→Electric (already done), Plant→Physical (already done in bootstrap_spawn_composition.rs). `Resistances([i8;6])` struct preserved as specified — T02 will replace it.

## Verification

cargo test --no-fail-fast: 0 failed across all test binaries. grep for 'Element::|: Element|basic_element' in src/ tests/ assets/data/ returns no matches.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --no-fail-fast 2>&1 | grep 'test result'` | 0 | ✅ pass — all test bins report 0 failed | 45000ms |
| 2 | `! grep -rn 'Element::|: Element|basic_element' src/ tests/ assets/data/ && echo CLEAN` | 0 | ✅ pass — output: CLEAN | 200ms |

## Deviations

pipeline.rs had an undocumented OnBreak match arm with the old field name that the task plan did not list — fixed as part of normal sweep. follow_up_triggers.rs had a LogEntry::Break { element } match arm not listed in the plan — also fixed.

## Known Issues

none

## Files Created/Modified

- `src/combat/follow_up_tests.rs`
- `src/combat/turn_system/tests.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/roster_smoke.rs`
- `tests/pipeline_dispatch.rs`
- `tests/follow_up_triggers.rs`
