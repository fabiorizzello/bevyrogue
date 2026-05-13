# S02: S02

**Goal:** Implement §H.1 status policy: multi-instance `StatusBag` storage per (target,kind), `refresh_max_dur` on re-apply of the same kind, and a `BuffKind`-classified cleanse that removes only Debuffs (Blessed survives). No per-status semantics (those are S03-S05), no new Effect-DSL variants (M019), no source attribution (M020).
**Demo:** Test deterministico: apply Heated(dur=2), re-apply Heated(dur=1), check dur=2. Cleanse rimuove Debuff ma non Buff cleanse-immune.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: StatusBag + BuffKind types and policy**
  - Files: `src/combat/status_effect.rs`
  - Verify: Inline `#[cfg(test)] mod tests` in `src/combat/status_effect.rs` covers: refresh-max-dur math, multi-kind coexistence, classify_buff_kind totality, cleanse_debuffs leaving Blessed intact. Run `cargo test --lib combat::status_effect` (the rest of the tree will not compile until T02-T04, which is expected).

- [x] **T02: Migrate apply pipeline to StatusBag**
  - Files: `src/combat/turn_system/pipeline.rs`, `src/combat/bootstrap.rs`
  - Verify: `cargo check` compiles cleanly for the apply path. Manual read: `OnStatusApplied` still fires on refresh; `OnStatusResisted` still gated by `roll_pct`.

- [x] **T03: Migrate tick + expiration to StatusBag**
  - Files: `src/combat/turn_system/mod.rs`
  - Verify: `cargo check` clean. The tick system emits exactly one `OnStatusExpired` per expired instance (verified later by T05 tests).

- [x] **T04: Migrate follow_up + in-tree tests to StatusBag**
  - Files: `src/combat/follow_up.rs`, `src/combat/turn_system/tests.rs`
  - Verify: `cargo check` clean across the whole tree. `cargo test --lib` green. Grep `rg 'StatusEffect\s*\{' src/` returns zero hits (all spawns go through `StatusBag::apply`).

- [x] **T05: Slice DoD tests all green: status_refresh_max_dur, status_multi_kind_coexist, status_cleanse_policy, status_accuracy (fresh), combat_coherence migrated — 0 FAILED across full suite**
  - Files: `tests/status_refresh_max_dur.rs`, `tests/status_multi_kind_coexist.rs`, `tests/status_cleanse_policy.rs`, `tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/status_effect_turn_tick.rs`, `tests/status_accuracy.rs`, `tests/follow_up_chains.rs`, `tests/combat_coherence.rs`, `tests/form_identity.rs`
  - Verify: `cargo test --test status_refresh_max_dur`, `cargo test --test status_multi_kind_coexist`, `cargo test --test status_cleanse_policy` all green individually. Full `cargo test` green with 0 ignored.

- [ ] **T06: Smoke + grep guard + SUMMARY**
  - Files: `.gsd/milestones/M017/slices/S02/S02-SUMMARY.md`
  - Verify: Smoke CLI exits 0. Grep guard clean. `cargo test` 0 failed / 0 ignored. SUMMARY.md persisted via `gsd_complete_slice`.

## Files Likely Touched

- src/combat/status_effect.rs
- src/combat/turn_system/pipeline.rs
- src/combat/bootstrap.rs
- src/combat/turn_system/mod.rs
- src/combat/follow_up.rs
- src/combat/turn_system/tests.rs
- tests/status_refresh_max_dur.rs
- tests/status_multi_kind_coexist.rs
- tests/status_cleanse_policy.rs
- tests/status_effect_apply.rs
- tests/status_effect_integration.rs
- tests/status_effect_turn_tick.rs
- tests/status_accuracy.rs
- tests/follow_up_chains.rs
- tests/combat_coherence.rs
- tests/form_identity.rs
- .gsd/milestones/M017/slices/S02/S02-SUMMARY.md
