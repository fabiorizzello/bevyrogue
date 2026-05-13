---
id: T04
parent: S02
milestone: M017
key_files:
  - src/combat/turn_system/tests.rs
key_decisions:
  - follow_up.rs required no changes — T03 already completed that migration as a cascade fix
  - Dead-code OLD functions still migrated to StatusBag to satisfy the rg verification gate (no StatusEffect struct literals in src/)
duration: 
verification_result: mixed
completed_at: 2026-05-13T07:06:36.736Z
blocker_discovered: false
---

# T04: Migrated all StatusEffect struct literals in turn_system/tests.rs to StatusBag; follow_up.rs was already clean from T03

**Migrated all StatusEffect struct literals in turn_system/tests.rs to StatusBag; follow_up.rs was already clean from T03**

## What Happened

follow_up.rs was already fully migrated by T03 (import at line 4 already uses StatusBag, ResolveActorsQuery at line 104 already has Option<&'static mut StatusBag>). No changes needed there.

turn_system/tests.rs had six StatusEffect { ... } struct literals across five #[allow(dead_code)] OLD functions, plus two .get::<StatusEffect>() assertions. All were migrated:

1. Import: `StatusEffect` replaced with `StatusBag` (StatusEffectKind retained).
2. Five spawn sites: pre-constructed StatusBag via `let mut bag = StatusBag::default(); bag.apply(kind, dur);` pattern, then passed to spawn tuple.
3. Two `.is_none()` assertions: replaced with `.map_or(true, |b| b.is_empty())`.
4. One struct-comparison assertion in the stunned-skips-tick OLD function: replaced with `.and_then(|b| b.get_dur(&kind))` == Some(2).

## Verification

cargo check: clean (0 errors, warnings only from pre-existing deprecated shim). cargo test --lib: 140 passed, 0 failed. rg 'StatusEffect\s*\{' src/ returns only the shim definition in status_effect.rs — no spawn sites. rg '&amp;static mut StatusEffect\b|&amp;mut StatusEffect\b' src/combat/follow_up.rs: 0 hits.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 130ms |
| 2 | `cargo test --lib` | 0 | 140 passed, 0 failed | 1200ms |
| 3 | `grep -rn 'StatusEffect\s*{' src/` | 1 | 0 hits in spawn sites (only shim definition in status_effect.rs) | 50ms |
| 4 | `grep -n "&'static mut StatusEffect\b" src/combat/follow_up.rs` | 1 | 0 hits | 30ms |

## Deviations

follow_up.rs import and ResolveActorsQuery were already migrated by T03; task plan described them as pending but no action was required.

## Known Issues

None.

## Files Created/Modified

- `src/combat/turn_system/tests.rs`
