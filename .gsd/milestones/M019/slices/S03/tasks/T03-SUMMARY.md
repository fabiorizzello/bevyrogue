---
id: T03
parent: S03
milestone: M019
key_files:
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - tests/cleanse_effect.rs
key_decisions:
  - Changed apply_cleanse_only from pub(crate) to pub so integration tests in tests/ can import it directly — mirrors apply_heal_only visibility
  - AllAllies branch uses either-or dispatch (heal_pct > 0 XOR cleanse_count.is_some()); T01 validator enforces this at the DSL level so no runtime collision possible
  - Missing-StatusBag fallback emits OnCleansed { kinds: [] } (alive) or no event (KO) — mirrors the fresh-bag fallback at the status_to_apply site
duration: 
verification_result: passed
completed_at: 2026-05-14T09:18:55.127Z
blocker_discovered: false
---

# T03: Pipeline wiring (Single/SelfOnly + AllAllies fan-out) + tests/cleanse_effect.rs integration suite — all 8 cases green, full suite clean.

**Pipeline wiring (Single/SelfOnly + AllAllies fan-out) + tests/cleanse_effect.rs integration suite — all 8 cases green, full suite clean.**

## What Happened

Wired apply_cleanse_only into two pipeline sites in pipeline.rs and added the 8-case integration test file.

**Pipeline changes:**

1. **Import** — added `apply_cleanse_only` to the resolution import in pipeline.rs (changed from `pub(crate)` to `pub` in resolution.rs so integration tests can reach it).

2. **AllAllies fan-out** (pipeline.rs ~line 341) — extended the existing AllAllies branch from heal-only to heal-or-cleanse. The branch now destructures `def_ko` and `mut def_bag` from the query row and dispatches: `heal_pct > 0` → `apply_heal_only`, `cleanse_count.is_some()` → `apply_cleanse_only` (or emits empty `OnCleansed` if no StatusBag component). The T01 validator's Heal+Cleanse exclusion makes the either-or contract sound.

3. **SelfOnly cleanse hook** (pipeline.rs ~line 1424) — inside `if outcome.succeeded {}` of the self-target path, declared `attacker_bag` as `mut`, then added cleanse dispatch using `attacker_bag` (the "defender" bag in a self-target action). Missing-bag case emits empty `OnCleansed { kinds: [] }` when alive, no event when KO.

4. **Single cleanse hook** (pipeline.rs ~line 1776) — inside `if outcome.succeeded {}` of the main attacker≠target path, appended cleanse dispatch after the status_to_apply sub-block. Uses the already-in-scope `defender_bag`. Guard: `cleanse_count.is_some() && !outcome.ko`.

**MEM001 check:** No tuple-arity change was needed in follow_up.rs — StatusBag was already present in the ResolveActorsQuery; no new component was added.

**Test file (tests/cleanse_effect.rs):** 8 deterministic cases using the `apply_cleanse_only` direct-call pattern (MEM003). All use inline fixtures, no Bevy world, no RNG, no wall-clock.

## Verification

cargo test --test cleanse_effect — all 8 cases passed. cargo test (full suite) — all test files green, zero failures. cargo check --tests — clean (only pre-existing warnings).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test cleanse_effect` | 0 | 8/8 pass | 3350ms |
| 2 | `cargo test` | 0 | full suite green, 0 failures | 8200ms |
| 3 | `cargo check --tests` | 0 | clean | 2070ms |

## Deviations

None — plan followed exactly. MEM001 (follow_up.rs tuple-arity) verified: StatusBag was already in the query, no change needed.

## Known Issues

none

## Files Created/Modified

- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/cleanse_effect.rs`
