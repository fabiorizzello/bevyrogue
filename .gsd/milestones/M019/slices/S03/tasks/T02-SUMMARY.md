---
id: T02
parent: S03
milestone: M019
key_files:
  - src/combat/status_effect.rs
  - src/combat/state.rs
  - src/combat/resolution.rs
key_decisions:
  - ResolvedAction.cleanse_count uses Option<Option<u8>>: outer None = not a cleanse skill, Some(inner) = cleanse count — typesafe over flat-bool
  - apply_cleanse_only emits OnCleansed even when kinds is empty (telemetry parity with OnHealed amount=0)
  - KO target on cleanse: silent no-op, no event emitted — mirrors apply_heal_only policy
  - cleanse_n rebuild uses std::mem::take + binary_search on sorted indices — no extra imports needed
  - count=None semantically equivalent to remove-all (usize::MAX truncation)
duration: 
verification_result: passed
completed_at: 2026-05-14T09:08:32.875Z
blocker_discovered: false
---

# T02: Added StatusBag::cleanse_n + apply_cleanse_only + ResolvedAction.cleanse_count + skill_cleanse_count extractor; cargo check --tests clean, full suite green.

**Added StatusBag::cleanse_n + apply_cleanse_only + ResolvedAction.cleanse_count + skill_cleanse_count extractor; cargo check --tests clean, full suite green.**

## What Happened

Implemented the cleanse primitive end-to-end (excluding pipeline dispatch, which is T03).

**status_effect.rs — StatusBag::cleanse_n**: Added `pub fn cleanse_n(&mut self, count: Option<u8>) -> Vec<StatusEffectKind>` beside the existing `cleanse_debuffs` (left untouched). The algorithm: enumerates debuff-classified instances, stable-sorts by `(Reverse(duration_remaining), idx)` for duration-DESC with insertion-idx-ASC tiebreak, truncates to `count.map(|c| c as usize).unwrap_or(usize::MAX)`, rebuilds the bag via `std::mem::take` + `enumerate` + `binary_search` on a sorted removal-idx vec (no HashSet import needed). Six inline `#[cfg(test)] mod tests` added: ordering by duration-DESC, idx-ASC tiebreak on equal durations, count=None removes all non-immune, count=Some(0) is no-op, Blessed never removed, count > available removes all without panic.

**state.rs — ResolvedAction.cleanse_count**: Added `pub cleanse_count: Option<Option<u8>>` as the final field. Outer None = not a cleanse skill; Some(inner) = cleanse with that count. Nine integration-test files that construct `ResolvedAction` with struct literal syntax needed `cleanse_count: None` added — done via targeted sed passes.

**resolution.rs — skill_cleanse_count extractor**: Added `fn skill_cleanse_count(effects: &[Effect]) -> Option<Option<u8>>` alongside the other per-effect extractors; wired into `resolve_action` as `cleanse_count: skill_cleanse_count(&skill.effects)`.

**resolution.rs — apply_cleanse_only**: Added `pub(crate) fn apply_cleanse_only(action: &ResolvedAction, bag: &mut StatusBag, defender_alive: bool) -> (ResolutionOutcome, Vec<CombatEventKind>)` mirroring `apply_heal_only`. KO branch (defender_alive=false) returns silent no-op with sp_ok=true, no event. Alive branch: unwraps cleanse_count (panics in debug if caller forgot to check), calls bag.cleanse_n(inner_count), emits exactly one OnCleansed { kinds } event — always emitted, even when kinds is empty (telemetry parity with OnHealed amount=0).

## Verification

cargo check --tests: clean (exit 0, only pre-existing warnings). cargo test --lib cleanse_n: 6/6 new inline tests pass. cargo test (full suite): all test bins pass, 0 failures across the integration suite. apply_cleanse_only not yet wired into the pipeline (T03 scope), hence the dead_code warning — expected.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --tests` | 0 | clean compile, no new errors | 1330ms |
| 2 | `cargo test --lib cleanse_n` | 0 | 6 cleanse_n tests pass | 800ms |
| 3 | `cargo test` | 0 | full integration suite green | 3000ms |

## Deviations

Nine integration tests constructing ResolvedAction directly needed cleanse_count: None added — not mentioned in the plan but required by the struct change. Straightforward additive fix.

## Known Issues

apply_cleanse_only triggers a dead_code warning until T03 wires it into the pipeline — expected and intentional per the task plan.

## Files Created/Modified

- `src/combat/status_effect.rs`
- `src/combat/state.rs`
- `src/combat/resolution.rs`
