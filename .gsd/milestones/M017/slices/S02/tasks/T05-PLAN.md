---
estimated_steps: 1
estimated_files: 8
skills_used: []
---

# T05: Reconcile S01 drift + write fresh DoD tests + migrate combat_coherence

**Destructive step — requires explicit user confirmation before deletion.**

## S01 drift reconciliation

`M017-CONTEXT.md` mandates *delete-and-rewrite-fresh* for the 4 legacy lifecycle test files in S01. `S01-SUMMARY.md` shows S01 migrated them in place instead. S02 reconciles: delete the 3 pure-lifecycle files (`tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/status_effect_turn_tick.rs`) — their coverage is subsumed by the 3 new S02 DoD tests plus the in-tree `turn_system/tests.rs` fixtures (T04). Delete `tests/status_accuracy.rs` and write a fresh replacement.

## Files NOT in scope (corrected from previous plan)

`tests/form_identity.rs` and `tests/follow_up_chains.rs` are dropped from the file list — verified `rg StatusEffect` returns 0 matches on both. The previous plan overstated their inclusion.

## Fresh `tests/status_accuracy.rs`

Mirror the 3 tests from the deleted file under the `StatusBag` API: (1) miss (Vaccine→Data, threshold=90, miss_seed) → `OnStatusResisted` + `bag.has(&Paralyzed) == false`; (2) hit (Vaccine→Data, threshold=90, hit_seed) → `OnStatusApplied` + `bag.has(&Paralyzed) == true`; (3) neutral (Vaccine→Vaccine, threshold=100, any seed) → always-apply. Use the same `hit_seed` / `miss_seed` seed-search helpers if they exist in the deleted file or `src/combat/rng.rs`; otherwise inline seed constants. The accuracy gate is a live mechanic in M017 — fresh coverage is required, not optional.

## Three new DoD tests

Each a minimal Bevy `App` fixture mirroring `turn_system/tests.rs`:

(a) `tests/status_refresh_max_dur.rs` — neutral matchup so accuracy never gates. Apply Heated(dur=2), apply Heated(dur=1), assert `bag.get_dur(&Heated) == Some(2)` and `bag.iter().count() == 1`. Then apply Heated(dur=5), assert `Some(5)`.

(b) `tests/status_multi_kind_coexist.rs` — apply Heated + Chilled + Blessed to the same target via three skill-apply events, assert all three `bag.has(...)` are true and their durations match.

(c) `tests/status_cleanse_policy.rs` — direct unit-level test (no Bevy events). Construct `StatusBag::default()`, call `apply` five times (Heated/Chilled/Paralyzed/Slowed/Blessed). Call `bag.cleanse_debuffs()`. Assert returned `Vec` contains exactly the 4 debuff kinds (sort or HashSet compare) and `bag.has(&Blessed)` is true while the 4 debuff `has(...)` are false.

## Migrate `tests/combat_coherence.rs`

Update import at line 3 to add `StatusBag` (keep `StatusEffectKind`). Rewrite the helper at lines 410-417 from `Option<&StatusEffect>` to `Option<&StatusBag>` — return `bag.iter().next().map(|i| i.kind.clone())` to preserve "first kind" semantic (callers at lines 926/946/967/981 only assert one kind at a time, so this is sufficient). Rewrite the spawn at line 62 from `StatusEffect { kind: ..., duration_remaining: ... }` to `{ let mut b = StatusBag::default(); b.apply(...); b }`.

## Inputs

- `.gsd/milestones/M017/M017-CONTEXT.md`
- `.gsd/milestones/M017/slices/S02/S02-RESEARCH.md`
- `.gsd/milestones/M017/slices/S01/S01-SUMMARY.md`
- `src/combat/status_effect.rs` (post-T01)
- `tests/status_accuracy.rs` (read before delete to capture `hit_seed`/`miss_seed`)
- `tests/combat_coherence.rs`

## Expected Output

- Deleted: `tests/status_effect_apply.rs`
- Deleted: `tests/status_effect_integration.rs`
- Deleted: `tests/status_effect_turn_tick.rs`
- Rewritten fresh: `tests/status_accuracy.rs`
- Migrated: `tests/combat_coherence.rs`
- New: `tests/status_refresh_max_dur.rs`
- New: `tests/status_multi_kind_coexist.rs`
- New: `tests/status_cleanse_policy.rs`

## Verification

`cargo test --test status_refresh_max_dur`, `cargo test --test status_multi_kind_coexist`, `cargo test --test status_cleanse_policy`, `cargo test --test status_accuracy`, `cargo test --test combat_coherence` all green individually. Full `cargo test` green with 0 failed, 0 ignored. `ls tests/status_effect_*.rs` returns empty. `rg "StatusEffect\b" tests/` returns 0 matches against the struct (only `StatusEffectKind` enum references survive).

## Observability Impact

Tests assert per-instance lifecycle semantics under the new `StatusBag` API; they do not depend on JSONL log shape. Accuracy-gate coverage is preserved via the fresh `tests/status_accuracy.rs`.
