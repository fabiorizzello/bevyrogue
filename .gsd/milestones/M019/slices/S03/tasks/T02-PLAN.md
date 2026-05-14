---
estimated_steps: 18
estimated_files: 3
skills_used: []
---

# T02: StatusBag::cleanse_n + apply_cleanse_only + ResolvedAction.cleanse_count + extractor

Implement the cleanse primitive end-to-end except for pipeline dispatch. Steps:

1) src/combat/status_effect.rs — Add `pub fn cleanse_n(&mut self, count: Option<u8>) -> Vec<StatusEffectKind>` beside the existing `cleanse_debuffs()` (leave cleanse_debuffs untouched — additive only). Selection algorithm:
   - Enumerate `self.0.iter().enumerate()`, keep only entries where `classify_buff_kind(inst.kind) == BuffKind::Debuff`.
   - Stable sort the candidate (idx, &inst) tuples by key `(Reverse(inst.duration_remaining), idx)` so duration-DESC wins, with idx-ASC tiebreak.
   - Truncate to `count.map(|c| c as usize).unwrap_or(usize::MAX)`.
   - Collect kinds in selection order, then remove the entries from the inner Vec by their original indices (descending swap_remove or filter-rebuild — choose whichever preserves the non-removed order; rebuild is simpler and the bag is small).

Add inline `#[cfg(test)] mod tests` covering: ordering by duration-DESC, idx-ASC tiebreak, count=None removes all non-immune, count=Some(0) no-op, Blessed never removed, count > available removes all without panic.

2) src/combat/state.rs — Add `pub cleanse_count: Option<Option<u8>>` to ResolvedAction. Outer None = not a cleanse skill; Some(inner) = cleanse with that count. Locked decision per research §Open Decisions #1.

3) src/combat/resolution.rs — Add extractor `fn skill_cleanse_count(effects: &[Effect]) -> Option<Option<u8>>` returning Some(count) when the skill carries Effect::Cleanse, else None. Populate ResolvedAction.cleanse_count from this extractor inside resolve_action.

4) src/combat/resolution.rs — Add `apply_cleanse_only(action: &ResolvedAction, bag: &mut StatusBag, defender_alive: bool) -> (ResolutionOutcome, Vec<CombatEventKind>)` mirroring apply_heal_only:
   - If !defender_alive (KO): return (success with sp_ok=true, empty events) — silent no-op, no OnCleansed emitted (per research §Open Decisions #5).
   - Otherwise: read action.cleanse_count.expect (outer Some required when this is called), call bag.cleanse_n(inner_count), and emit a single OnCleansed { kinds } event — emit even when kinds is empty for telemetry parity (per §Open Decisions #3).

5) Do NOT wire into apply_effects or pipeline yet. apply_cleanse_only is reachable only from T03. Keep `pub(crate)` visibility so T03's pipeline call site compiles.

Locked decisions:
- ResolvedAction shape: `cleanse_count: Option<Option<u8>>` (typesafe over flat-bool variant).
- count=Some(0) emits empty OnCleansed (telemetry parity).
- KO target: silent no-op (no event), mirrors heal KO policy.
- count=None == 'remove all non-immune' (semantically equivalent to cleanse_debuffs but without forcing the call site to branch).

## Inputs

- `.gsd/milestones/M019/slices/S03/S03-RESEARCH.md`
- `.gsd/milestones/M019/slices/S02/S02-SUMMARY.md`
- `src/combat/status_effect.rs`
- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/follow_up.rs`

## Expected Output

- `src/combat/status_effect.rs`
- `src/combat/state.rs`
- `src/combat/resolution.rs`

## Verification

cargo check --tests clean. cargo test --lib runs the inline #[cfg(test)] mod tests for cleanse_n — all ordering / tiebreak / count edge cases pass. Full integration suite (cargo test) remains green since apply_cleanse_only is not yet reachable from the pipeline.

## Observability Impact

apply_cleanse_only emits CombatEventKind::OnCleansed once per non-KO call (empty kinds vector on no-op cleanses). KO branch is silent.
