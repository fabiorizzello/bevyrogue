# S02: Apply / refresh_max_dur / cleanse policy

**Goal:** Implement §H.1 status policy: multi-instance `StatusBag` storage per (target,kind), `refresh_max_dur` on re-apply of the same kind, and a `BuffKind`-classified cleanse that removes only Debuffs (Blessed survives). No per-status semantics (those are S03-S05), no new Effect-DSL variants (M019), no source attribution (M020).
**Demo:** Test deterministico: apply Heated(dur=2), re-apply Heated(dur=1), check dur=2. Cleanse rimuove Debuff ma non Buff cleanse-immune.

## Must-Haves

- `cargo check` + full `cargo test` headless suite green
- New `StatusBag` component holds N `StatusInstance`s per entity; `apply(kind,dur)` upserts with `max(old,new)` duration; never duplicates same-kind instances
- `BuffKind { Buff, Debuff }` with `classify_buff_kind(StatusEffectKind)` total over the enum (Blessed=Buff; Heated/Chilled/Paralyzed/Slowed/Burn/Shock=Debuff)
- `StatusBag::cleanse_debuffs` removes every Debuff instance and leaves Buff (Blessed) intact
- Apply pipeline (`turn_system/pipeline.rs`), tick path (`turn_system/mod.rs`), and `follow_up.rs` query migrated to `StatusBag`; existing in-tree `turn_system/tests.rs` fixtures updated
- New deterministic integration tests: `tests/status_refresh_max_dur.rs`, `tests/status_multi_kind_coexist.rs`, `tests/status_cleanse_policy.rs` all green
- Tick emits one `OnStatusExpired { kind }` per expired instance; `OnStatusApplied` still fires on both insert and refresh
- Grep guard from S01 still clean — no `Burn|Freeze|Shock|DeepFreeze` regressions in src/ or tests/ beyond reserved Burn/Shock

## Proof Level

- This slice proves: Deterministic integration tests over a minimal Bevy `App` fixture (mirroring `turn_system/tests.rs` pattern) — three new `tests/status_*.rs` files exercise refresh-max-dur, multi-kind coexistence, and Debuff-only cleanse. No RNG, no wall-clock.

## Integration Closure

`StatusBag` is the new shared substrate for S03-S05 per-status semantics; tick loop already iterates instances so S03+ can plug per-kind effect arms. `cleanse_debuffs` is exposed as a public API on the bag so M019's `Effect::EmitCleanse` can call it without further refactor.

## Verification

- No new event variants. `OnStatusApplied` continues to fire on apply and on refresh (both are "apply" per canon). `OnStatusExpired` now fires once per expired instance instead of once per component removal. JSONL log surface unchanged in shape.

## Tasks

- [x] **T01: StatusBag + BuffKind types and policy** `est:S`
  Add `BuffKind { Buff, Debuff }` enum, `classify_buff_kind(StatusEffectKind) -> BuffKind` (total: Blessed=Buff, all others=Debuff including reserved Burn/Shock), `StatusInstance { kind, duration_remaining }`, and `StatusBag(Vec<StatusInstance>)` as a Bevy Component (derive `Default`, `Component`). Methods: `apply(kind, dur)` upserts with `max(old, new)` duration; `tick_all()` decrements every instance and returns the kinds that expired (so the tick system can emit `OnStatusExpired`); `cleanse_debuffs() -> Vec<StatusEffectKind>` drains every Debuff-classified instance and returns the kinds removed; `has(kind) -> bool`; `get_dur(kind) -> Option<u32>`. Keep the inline `#[cfg(test)] mod tests` style established in the file: add unit tests for refresh-max-dur math (apply 2 then 1 -> 2; apply 2 then 5 -> 5), multi-kind coexistence at the unit level, classify_buff_kind totality, and cleanse_debuffs leaving Blessed intact. Lock the doc-comment policy decision: a re-apply that fails the accuracy roll does NOT refresh — the apply pipeline gates entry to `StatusBag::apply` behind the existing `roll_pct(threshold)` check at `src/combat/turn_system/pipeline.rs:725-729`, so `StatusBag::apply` itself does not see resisted re-applies. Remove the old single-component `StatusEffect` shape (do not keep as alias — S01's policy was delete-and-rewrite-fresh). Note: this task only adds the types — call-sites are migrated in T02/T03/T04; expect compile errors at those sites until those tasks land.
  - Files: `src/combat/status_effect.rs`
  - Verify: Inline `#[cfg(test)] mod tests` in `src/combat/status_effect.rs` covers: refresh-max-dur math, multi-kind coexistence, classify_buff_kind totality, cleanse_debuffs leaving Blessed intact. Run `cargo test --lib combat::status_effect` (the rest of the tree will not compile until T02-T04, which is expected).

- [x] **T02: Migrate apply pipeline to StatusBag** `est:S`
  Rewrite the status apply site in `src/combat/turn_system/pipeline.rs:721-753`: replace `commands.entity(target_entity).insert(StatusEffect { kind, duration_remaining })` with a path that ensures a `StatusBag` exists on the target and then calls `bag.apply(kind, duration_remaining)`. Use Bevy 0.18 `EntityCommands::entry::<StatusBag>().or_default()` if the API matches in this codebase; if not, fall back to a two-step `Query<Option<&mut StatusBag>>` read at the system level plus a conditional `commands.insert(StatusBag::default())` for first-application units (verify the exact shape against `Cargo.toml` Bevy version before committing — research notes this as a Bevy-version risk). Preserve the accuracy gate at lines 725-729 exactly: the `roll_pct(threshold)` check stays in front of `apply`, so a resisted re-apply still fires `OnStatusResisted` and leaves the existing duration. Continue emitting `OnStatusApplied { target, kind }` on both insert and refresh (refresh is still an apply per canon). If `bootstrap.rs` spawns units with a `StatusEffect` component in a bundle, swap to empty `StatusBag::default()` (or omit and let apply-time `entry().or_default()` handle it). Do not change event payload shapes.
  - Files: `src/combat/turn_system/pipeline.rs`, `src/combat/bootstrap.rs`
  - Verify: `cargo check` compiles cleanly for the apply path. Manual read: `OnStatusApplied` still fires on refresh; `OnStatusResisted` still gated by `roll_pct`.

- [x] **T03: Migrate tick + expiration to StatusBag** `est:S`
  Rewrite the tick path in `src/combat/turn_system/mod.rs:465-509`: switch the query from `Option<&'static mut StatusEffect>` to `Option<&'static mut StatusBag>`. For each unit's bag, call `bag.tick_all()` and iterate the returned expired kinds, emitting one `OnStatusExpired { unit, kind }` event per expired instance. The per-kind match arm stays empty (S03-S05 will hook DoT/amp/skip/delay/dealt-buff here). Leave the bag component in place even if empty after expiry (cheap, avoids re-insert churn on next apply). Do not change event payload shapes.
  - Files: `src/combat/turn_system/mod.rs`
  - Verify: `cargo check` clean. The tick system emits exactly one `OnStatusExpired` per expired instance (verified later by T05 tests).

- [ ] **T04: Migrate follow_up + in-tree tests to StatusBag** `est:M`
  Update `src/combat/follow_up.rs:90-108` query tuple from `Option<&'static mut StatusEffect>` to `Option<&'static mut StatusBag>`. If any read accessed `.kind` directly, replace with `bag.has(kind)` / `bag.get_dur(kind)` lookups. Update every fixture in `src/combat/turn_system/tests.rs` (lines 136, 155, 183, 234, 286, 324, 338 per research) that currently spawns `StatusEffect { kind, duration_remaining }` directly: replace with `let mut bag = StatusBag::default(); bag.apply(kind, dur); commands.spawn((..., bag))`. Update every assertion that reads `app.world().get::<StatusEffect>(entity)` to `app.world().get::<StatusBag>(entity)` plus `.has(kind)` / `.get_dur(kind)`. Do not bypass the policy by constructing instances with private-field access — go through `apply`. After this task `cargo check` and `cargo test --lib` must be fully green; integration tests in `tests/` may still have stale references (handled in T05).
  - Files: `src/combat/follow_up.rs`, `src/combat/turn_system/tests.rs`
  - Verify: `cargo check` clean across the whole tree. `cargo test --lib` green. Grep `rg 'StatusEffect\s*\{' src/` returns zero hits (all spawns go through `StatusBag::apply`).

- [ ] **T05: Slice DoD tests + integration test migration** `est:M`
  Add three new integration tests under `tests/`, each spinning up a minimal Bevy `App` mirroring the fixture pattern already used in `src/combat/turn_system/tests.rs`: (a) `tests/status_refresh_max_dur.rs` — apply Heated(dur=2), apply Heated(dur=1), assert exactly one Heated instance with `dur=2`; then apply Heated(dur=5), assert `dur=5`. Use the same triangle/threshold setup so accuracy is 100% (otherwise the second apply could be resisted and skew the test). (b) `tests/status_multi_kind_coexist.rs` — apply Heated + Chilled + Blessed to the same target, assert all three present with their durations via `bag.has(kind)` and `bag.get_dur(kind)`. (c) `tests/status_cleanse_policy.rs` — stage a bag with Heated+Chilled+Paralyzed+Slowed+Blessed; call `cleanse_debuffs`; assert returned `Vec` is the four debuff kinds (order-insensitive) and only Blessed remains. Also migrate any remaining `tests/*.rs` files that still reference the old single-component `StatusEffect` shape — research lists `tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/status_effect_turn_tick.rs`, `tests/status_accuracy.rs`, `tests/follow_up_chains.rs`, `tests/combat_coherence.rs`, `tests/form_identity.rs` as candidates — update them to the `StatusBag` API. Lifecycle assertions remain; per-status semantic assertions stay deleted (S03-S05).
  - Files: `tests/status_refresh_max_dur.rs`, `tests/status_multi_kind_coexist.rs`, `tests/status_cleanse_policy.rs`, `tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/status_effect_turn_tick.rs`, `tests/status_accuracy.rs`, `tests/follow_up_chains.rs`, `tests/combat_coherence.rs`, `tests/form_identity.rs`
  - Verify: `cargo test --test status_refresh_max_dur`, `cargo test --test status_multi_kind_coexist`, `cargo test --test status_cleanse_policy` all green individually. Full `cargo test` green with 0 ignored.

- [ ] **T06: Smoke + grep guard + SUMMARY** `est:S`
  Run the headless smoke CLI: `cargo run --bin combat_cli` and confirm exit 0 with no panics. Re-run the S01 grep guard `grep -rEn '\b(Burn|Freeze|Shock|DeepFreeze)\b' src/ tests/` and confirm only the reserved Burn/Shock variant declarations remain (no new legacy references introduced by S02). Confirm `cargo check` and full `cargo test` both green (0 failed, 0 ignored). Produce `.gsd/milestones/M017/slices/S02/S02-SUMMARY.md` via `gsd_complete_slice` describing the migration, the `StatusBag` API surface for S03-S05, and the cleanse hook for M019.
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
