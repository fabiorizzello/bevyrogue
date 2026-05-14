---
sliceId: S01
uatType: artifact-driven
verdict: PASS
date: 2026-05-14T10:45:00.000Z
---

# UAT Result — S01

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| UltimateUsed emitted exactly once per ultimate cast | runtime | PASS | `cargo test --test ultimate_event`: 3 tests pass — `ultimate_used_emitted_once_on_ult_cast`, `no_ultimate_used_on_basic_attack`, `no_ultimate_used_on_skill_cast`. 0 failures. |
| UltimateUsed carries correct unit_id | runtime | PASS | `ultimate_used_emitted_once_on_ult_cast` asserts `unit_id == attacker_id`; passed green. |
| UnitDied carries StatusBag snapshot | runtime | PASS | `cargo test --test unit_died_payload`: `unit_died_carries_defender_status_snapshot` passes — `status_remaining` contains `Heated` + `Slowed`, `heated_remaining == 2`. |
| UnitDied not emitted on survival | runtime | PASS | `unit_died_not_emitted_on_survival` passes — no `UnitDied` when defender survives. |
| Full regression suite clean | runtime | PASS | `cargo test`: 673 tests across all suites, 0 failed, 0 ignored. All test result lines show `ok`. |
| No residual OnKO references | artifact | PASS | `rg -n 'CombatEventKind::OnKO' src tests` → exit code 1, zero matches. Rename complete throughout codebase. |
| Stun-damage KO path emits empty payload with comment | artifact | PASS | `src/combat/turn_system/mod.rs:488-489` — emit site has `// No StatusBag in scope at stun-damage site; payload left empty.` comment followed by `UnitDied { status_remaining: vec![], heated_remaining: 0 }`. Documented limitation confirmed. |
| Basic action and non-Reset skills do not emit UltimateUsed | runtime | PASS | `no_ultimate_used_on_basic_attack` and `no_ultimate_used_on_skill_cast` both pass green in the ultimate_event suite. |

## Overall Verdict

PASS — All 8 automatable checks pass: 5 new tests green (3 in ultimate_event, 2 in unit_died_payload), full 673-test regression suite clean, zero residual OnKO references, and stun-damage KO path correctly emits empty UnitDied payload with documentation comment.

## Notes

- `cargo test --test ultimate_event`: 3/3 pass in 0.00s
- `cargo test --test unit_died_payload`: 2/2 pass in 0.00s
- `cargo test` (full suite): 673 passed, 0 failed across all integration and lib targets
- `rg 'CombatEventKind::OnKO' src tests`: exit 1 — confirmed zero matches
- Stun-damage KO comment at `src/combat/turn_system/mod.rs:488`: `// No StatusBag in scope at stun-damage site; payload left empty.` — known limitation documented as specified in UAT
- Warnings present (dead_code, unused_mut, deprecated StatusEffect struct) are pre-existing; none new from this slice
