# S02 Research — Apply / refresh_max_dur / cleanse policy

## Summary

S02 turns the §H.1 lifecycle skeleton landed in S01 into actual policy: (1) multi-instance per `(target, kind)` so the five canon statuses can coexist on one unit; (2) refresh-max-dur on re-apply of the same `kind`; (3) a `BuffKind` classification with a default-Debuff cleanse filter that leaves `Blessed` (cleanse-immune Buff) intact. No new `Effect::EmitCleanse` variant — that ships in M019. The cleanse function is exposed as a public API on the status bag so the M019 effect can call it.

The slice DoD calls out two deterministic tests: (a) `apply Heated(dur=2)` then `re-apply Heated(dur=1)` ⇒ `dur=2`, and (b) cleanse removes Debuff but Blessed survives.

## Implementation Landscape

### Current state (post-S01)

- `src/combat/status_effect.rs:23` — `StatusEffect` is a **single Bevy Component** with one `kind: StatusEffectKind` + `duration_remaining: u32`. `StatusEffect::refresh(new_dur)` already implements `max(old, new)` at unit level; inline unit tests on lines 76-82 cover the math.
- `src/combat/turn_system/pipeline.rs:721-753` — apply path uses `commands.entity(target_entity).insert(StatusEffect { kind, duration_remaining })`. **Bevy `insert` overwrites** the component, so today a unit can carry at most one status of any kind and re-applying overwrites duration unconditionally (not max). A status accuracy roll gates the insert (`triangle_modifiers().status_acc_modifier` from `src/combat/damage.rs:30`). The `OnStatusApplied` / `OnStatusResisted` events are already correctly emitted.
- `src/combat/turn_system/mod.rs:465-509` — tick path reads `Option<&'static mut StatusEffect>` (single), per-kind match arm is a placeholder for S03-S05. Expiration removes the whole component.
- `src/combat/follow_up.rs:104` — query also takes `Option<&'static mut StatusEffect>`.
- `src/combat/turn_system/tests.rs` — existing tests at lines 136, 155, 183, 234, 286, 324 spawn `StatusEffect { kind, duration_remaining }` directly and assert `app.world().get::<StatusEffect>(entity)`. These will break under any storage redesign.

### What S02 must add

1. **Storage that holds N status instances per entity.** Two viable shapes:
   - **(A) `StatusBag(Vec<StatusInstance>)` component** — single component, append+find-by-kind. Bevy-idiomatic, queryable as one borrow, minimal churn on query tuples.
   - **(B) Per-kind component** (`Heated`, `Chilled`, … as individual components) — type-safe access, but blows up query surface and forces every kind into a separate `Option<&mut _>` borrow in tick/apply/follow_up.
   - **Recommendation: A.** Keeps pipeline/follow_up query shape almost intact (`Option<&mut StatusBag>`), centralizes refresh+cleanse policy, matches the §H.2 sketch in `.gsd/spikes/spike-kernel-primitives/sketches/status_and_reactive_events.rs:48-74` (single `StatusEffect` struct with `kind`+`buff_kind`+`dur`, gathered into a collection).
2. **`BuffKind` enum** — at minimum `Buff` and `Debuff` in M017 (the spike sketch lists `Buff | Debuff | DR | Aura | Mark` but DR/Aura/Mark are M019+). Add a `fn classify(StatusEffectKind) -> BuffKind` returning `Buff` for `Blessed` and `Debuff` for the four active debuffs (Heated/Chilled/Paralyzed/Slowed). Reserved `Burn`/`Shock` should be classifiable (`Debuff`) so the enum stays total, even though they cannot be applied.
3. **`refresh_max_dur` policy in apply.** Replace `commands.entity().insert(StatusEffect{…})` at `pipeline.rs:731` with a `StatusBag::apply(kind, dur)` call that either inserts a new `StatusInstance` or updates the existing one via `existing.duration_remaining = max(existing, new)`. Behavior: single-instance per `(target, kind)`, never duplicates, never additively stacks. The `OnStatusApplied` event still fires on both insert and refresh — refresh is still an "apply" from the canon's perspective.
4. **Cleanse API.** Pub function on `StatusBag` (or free fn over `&mut StatusBag`) that drops every instance whose classified `BuffKind == Debuff`, leaving `Buff` (i.e. `Blessed`) untouched. Signature shape: `pub fn cleanse_debuffs(&mut self) -> Vec<StatusEffectKind>` returning removed kinds for future event emission. No `Effect::EmitCleanse` is wired in M017 (per `M017-CONTEXT.md` Out of Scope and the spike `gaps.md:52-70` note flagging it for M019).
5. **Tick + expiration** must iterate the bag. Per-status semantics stay zero in S02 (those are S03-S05). On any instance reaching `dur == 0`, emit `OnStatusExpired { kind }` for that kind and drop the instance; if the bag becomes empty, the component may be removed or kept empty (either is fine for S03-S05).

### Boundary map for S02 (what to touch vs leave alone)

In scope:
- `src/combat/status_effect.rs` — add `BuffKind`, `StatusInstance` (rename current `StatusEffect` payload), `StatusBag` component with `apply`/`tick_all`/`cleanse_debuffs`/`has`/`get_dur`.
- `src/combat/turn_system/pipeline.rs:721-753` — replace `insert(StatusEffect{…})` with `StatusBag::apply(…)`; ensure component exists (either guarantee via spawn bundle or `commands.entity().entry::<StatusBag>().or_default()`).
- `src/combat/turn_system/mod.rs:474-509` — iterate `StatusBag` instances; per-kind match arm still empty (S03-S05); fire `OnStatusTick`/`OnStatusExpired` per instance.
- `src/combat/follow_up.rs:104` — switch `Option<&mut StatusEffect>` to `Option<&mut StatusBag>` (or whatever the new component name is).
- `src/combat/bootstrap.rs` — if any unit-spawn bundle existed for `StatusEffect`, update it. Verify whether spawning needs to seed an empty `StatusBag` or if `entry().or_default()` at apply-time suffices.
- `tests/` — three new tests (see below).
- `src/combat/turn_system/tests.rs:136,155,183,234,286,324,338` — update existing fixtures to spawn via `StatusBag` (don't bypass the new policy; keep assertions about lifecycle on the new container shape).

Out of scope (per `M017-CONTEXT.md` and roadmap S02 demo):
- DR pipeline, `BuffKind::DR/Aura/Mark` — M019.
- `Effect::EmitCleanse` / `Effect::EmitHeal` Effect-DSL variants — M019.
- Per-status numeric effects (DoT, amp, speed, skip, delay, dealt-dmg, Ult charge) — S03-S05.
- `source_unit` / `source_blueprint` fields on instances — M020 (the context Decision explicitly defers source attribution).
- `BuffDur::UntilRoundEnd` / `BuffDur::Permanent` — M019 (M017 only needs `Turns(u8)`-equivalent `u32` duration).

## First Proof / Highest-Risk Wedge

Land the storage change first. The smallest end-to-end proof:

1. Introduce `StatusBag` component with `apply(kind, dur)` doing refresh-max-dur and `has(kind)` / `get_dur(kind)` queries. Inline unit tests (already-present pattern in `status_effect.rs`) for the policy math.
2. Switch `pipeline.rs` apply site + tick site + follow_up query to the new component. `cargo check` is the proof that the type-level migration is complete.
3. Add `tests/status_refresh_max_dur.rs`: build a minimal Bevy `App` mirroring the existing `turn_system/tests.rs` fixture pattern; apply Heated(2), apply Heated(1), assert bag holds one Heated with `dur=2`; then apply Heated(5), assert `dur=5`. (The slice's authoritative test from the roadmap.)
4. Add `tests/status_multi_kind_coexist.rs`: apply Heated + Chilled + Blessed on the same target, assert all three present with their durations. Catches the regression where two kinds collapse onto one component.
5. Add `tests/status_cleanse_policy.rs`: stage a bag with Heated+Chilled+Paralyzed+Slowed+Blessed, call `cleanse_debuffs`, assert only Blessed remains.

The biggest unblocker for S03-S06 is having a queryable, multi-instance bag. Steps 1-2 are the wedge.

## Verification

- `cargo check` — type migration compiles end-to-end (status_effect, pipeline, turn_system tick, follow_up, bootstrap, in-tree tests module).
- `cargo test` — full headless integration suite stays green. Existing tests rewritten to use `StatusBag` API (no `#[ignore]` carryover; S01 left zero ignored).
- New tests:
  - `tests/status_refresh_max_dur.rs` (authoritative — slice DoD line 1).
  - `tests/status_multi_kind_coexist.rs` (multi-instance proof — without this, refresh_max_dur reduces to "single component overwrite" by accident).
  - `tests/status_cleanse_policy.rs` (authoritative — slice DoD line 2; covers Buff/Debuff classification and Blessed cleanse-immunity).
- Grep guard from S01 stays clean (no new `Burn/Freeze/Shock/DeepFreeze` references introduced).

## Natural Seams (task decomposition hint for planner)

- **T01 — Storage + policy types** — `src/combat/status_effect.rs`: add `BuffKind`, `classify_buff_kind`, `StatusInstance`, `StatusBag` (component) with `apply`/`tick_all`/`cleanse_debuffs`/`has`/`get_dur`; inline unit tests for refresh-max-dur + cleanse filter at the unit level. Keep the old `StatusEffect` symbol as a deprecated type alias for one commit if convenient, or remove in this task.
- **T02 — Apply pipeline migration** — `src/combat/turn_system/pipeline.rs:721-753` rewires to `StatusBag::apply`; ensure bag presence via `entry().or_default()` at apply-time (or seed in `bootstrap.rs`).
- **T03 — Tick + expiration migration** — `src/combat/turn_system/mod.rs:474-509` iterates bag; per-kind match arm still empty (S03-S05 hook).
- **T04 — Follow-up + in-tree test migration** — `src/combat/follow_up.rs:104` query; `src/combat/turn_system/tests.rs` fixtures (lines 136, 155, 183, 234, 286, 324, 338).
- **T05 — Slice tests** — `tests/status_refresh_max_dur.rs`, `tests/status_multi_kind_coexist.rs`, `tests/status_cleanse_policy.rs`; `cargo test` green.
- **T06 — Smoke + summary** — `cargo run --bin combat_cli` headless smoke; grep guard re-run; SUMMARY.

T02-T04 can be one commit if churn is small, but T01 must land first since it defines the API surface.

## Risks & Watch-outs

- **`commands.entity().insert(StatusBag)` after `entry().or_default()`** — Bevy 0.18 supports `EntityCommands::entry::<C>().or_default()` for Component+Default types; verify the exact call shape against this codebase's Bevy version before T02 (alt: explicit `world.get_mut::<StatusBag>()` branch with `commands.insert(StatusBag::default())` fallback). If `entry` API is unavailable, the apply path needs a two-step `Query<Option<&mut StatusBag>>` read + conditional `insert`.
- **`Option<&mut StatusEffect>` → `Option<&mut StatusBag>` ripple in `follow_up.rs:104`.** Pure rename if API surface is preserved; if any follow-up reads `.kind` directly, those callsites now need `.has(kind)` lookups.
- **`turn_system/tests.rs` fixtures spawn raw `StatusEffect` components.** Update to `StatusBag::default().apply(kind, dur)` insertion. Don't bypass the policy by mutating private fields — keep the tests honest.
- **Expiry event semantics.** Today expiry removes the whole component and emits one `OnStatusExpired`. With a bag, each instance expires independently; ensure the tick loop emits one event per expired instance (else S03-S06 observability will look wrong).
- **Refresh accuracy gate.** `pipeline.rs:725-729` runs `roll_pct(threshold)` even when the target already carries the status. Canon §H.1 single-instance + refresh-max-dur policy is silent on whether a re-apply that fails the accuracy roll should still refresh. Conservative choice (consistent with "apply is gated by acc"): if the roll fails, do not refresh — emit `OnStatusResisted` and leave the existing duration. The slice DoD test uses a deterministic fixture, so as long as the test seeds RNG or uses same-attribute triangle (threshold ≥ 100%), this is invisible. Flag for planner: lock this choice in T01's doc comment.
- **Reserved Burn/Shock under cleanse.** `classify_buff_kind` must be total; classify both as `Debuff` (or any value) — they cannot be applied per the S01 RON validator, but the function must compile for every variant.

## Skills To Consider

Recommendations (not installs):
- **rust-best-practices** — borrowing/ownership tradeoffs in the `StatusBag` API (return `Vec<Kind>` vs in-place mutation; `&mut` vs `&` accessors).
- **rust-testing** — fixture patterns for the three new integration tests.
- **bevy** — confirm `EntityCommands::entry::<C>().or_default()` shape in Bevy 0.18 before T02.

## Don't Hand-Roll

- The refresh-max-dur math — `u32::max` is fine; the existing `StatusEffect::refresh` at `src/combat/status_effect.rs:35` is the same shape, just moved inside `StatusBag::apply`.

## Call-Site Inventory (Addendum, 2026-05-13)

Exhaustive `rg StatusEffect src/ tests/` results, classified by required action.

### `src/` — migrate to `StatusBag`

| File:Line | Form | Action |
|---|---|---|
| `src/combat/status_effect.rs:9-25` | enum + struct definitions | **T01** — keep `StatusEffectKind` enum unchanged; replace `StatusEffect` struct with `StatusInstance` (private) + `StatusBag` (Component). |
| `src/combat/status_effect.rs:50-138` | inline `#[cfg(test)] mod tests` (RON round-trip) | **T01** — rewrite tests against `StatusBag::apply` / `StatusInstance` shape; the RON round-trip tests can stay if `StatusInstance` derives Serialize/Deserialize, or be replaced by `StatusBag` round-trip. |
| `src/combat/turn_system/pipeline.rs:21,731` | `use ...StatusEffect;` + `commands.entity(...).insert(StatusEffect { ... })` | **T02** — swap import to `StatusBag`; replace insert with bag-entry-or-default + `apply(kind, dur)`. |
| `src/combat/turn_system/mod.rs:6,67` | `use StatusEffect;` + `ResolveActorsQuery` type alias query tuple | **T03** — swap import; change query tuple slot to `Option<&'static mut StatusBag>`. |
| `src/combat/turn_system/mod.rs:369` | inline query tuple in `advance_turn_system` (DISTINCT from `ResolveActorsQuery`) | **T03** — same migration; do not miss this site — it is a separate query, not the type alias. |
| `src/combat/turn_system/mod.rs:500` | `commands.entity(snap.entity).remove::<StatusEffect>()` | **T03** — delete entirely; bag persists empty after instances expire (cheap, avoids churn on next apply). If the snap-driven removal exists for a specific reason, replace with `bag.tick_all()` already handling expiry. |
| `src/combat/turn_system/mod.rs:479-485` | match arm covering all 7 `StatusEffectKind` variants in tick | **T03** — preserve totality over enum (placeholder no-op stays for S03-S05). |
| `src/combat/turn_system/tests.rs:3,136,149,155,162,183,184,193,199,206,234,235,286,287,302,324,325,337-339` | direct `StatusEffect { kind, duration_remaining }` spawns + `app.world().get::<StatusEffect>(...)` assertions | **T04** — rewrite all fixtures through `StatusBag::default().apply(...)`; rewrite assertions to `world.get::<StatusBag>(entity).map(\|b\| b.get_dur(kind))` or `b.has(kind)`. The `Some(&StatusEffect { ... })` pattern at 338 becomes a `bag.get_dur(kind) == Some(N)` check. |
| `src/combat/follow_up.rs:4,104` | `use StatusEffect;` + `ResolveActorsQuery` slot | **T04** — same migration as T03 query slots; confirmed no `.kind` read access here (pure ownership for downstream systems). |
| `src/combat/mod.rs:61` | `pub use status_effect::{StatusEffect, StatusEffectKind};` | **T01** — replace `StatusEffect` with `StatusBag` (and optionally `StatusInstance` if other crates need read access). `StatusEffectKind` stays exported. |

### `src/` — INFORMATIONAL, no migration

The following carry `StatusEffectKind` (the enum), NOT the legacy `StatusEffect` component. They do NOT change in S02:

- `src/combat/state.rs:5,44` — `status_to_apply: Option<(StatusEffectKind, u32)>` on `CombatState`. The apply pipeline still extracts a single status per action (first-match-wins, set in `resolution.rs:136`); S02 changes only *how it lands*, not *what is extracted*.
- `src/combat/resolution.rs:2,136` — `skill_apply_status()` returns `Option<(StatusEffectKind, u32)>`. Unchanged.
- `src/combat/events.rs:4,50,55,85,88-90` — event payloads carry `StatusEffectKind` only. Note: `events.rs:88` comment says "No StatusEffect component is inserted" — comment text references the old component, can be updated to "no StatusBag instance is inserted" for clarity but not a behavior change.
- `src/combat/kit.rs:4,41` — `OnStatusApplied(StatusEffectKind)` trigger payload. Unchanged.
- `src/data/skills_ron.rs:5,182,267,271,508,519,530` — RON schema for `Effect::ApplyStatus { kind, duration }`. Unchanged.
- `src/combat/observability.rs` — **grep clean, zero matches**. `ValidationSnapshot` does NOT carry per-unit status state in M017; do not chase it.

### `tests/` — files containing `StatusEffect` references

Authoritative list (verified `rg StatusEffect tests/`):

1. `tests/status_accuracy.rs` — 8 references (line 12 import, 60 spawn, 200/242/279 `world.get::<StatusEffect>(...).is_some()/is_none()` lifecycle assertions, 187/229 `OnStatusResisted/OnStatusApplied { kind: ::Paralyzed }`). All lifecycle assertions migrate to `world.get::<StatusBag>(defender).map(\|b\| b.has(Paralyzed))`.
2. `tests/combat_coherence.rs` — 8 references including a helper at line 410: `fn status_effect_kind(...) -> Option<StatusEffectKind>` that queries `Option<&StatusEffect>`. Rewrite the helper to query `Option<&StatusBag>` and return the first kind (or build a small `statuses_on(entity) -> Vec<StatusEffectKind>` helper). Callers at 926/946/967/981 only assert presence/kind, so a `bag.has(Paralyzed)` predicate works.
3. `tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/status_effect_turn_tick.rs` — **S01 drift, see below**.

### `tests/` — files NOT containing `StatusEffect`

Verified `rg StatusEffect` count = 0:

- `tests/form_identity.rs`
- `tests/follow_up_chains.rs`

These are **NOT in S02 file scope**. Their inclusion in the previous T05 file list was overcautious — drop them from the file list.

## S01 Drift: legacy test files were migrated, not deleted

`M017-CONTEXT.md` D-section "Test legacy: delete-and-rewrite-fresh" explicitly states:

> I 4 file `tests/status_effect_apply.rs`, `tests/status_effect_turn_tick.rs`, `tests/status_effect_integration.rs`, `tests/status_accuracy.rs` sono cancellati in S01. S02-S05 scrivono test fresh sulla semantica canon.

`S01-SUMMARY.md` shows the files were **migrated to canon names**, not deleted. They still exist on disk. This is a drift between the milestone canon and the executed S01.

S02 must reconcile this. Two reasonable paths:

- **Path A — Honor canon (delete now, write fresh in S02-S05):** In S02 we delete `tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/status_effect_turn_tick.rs`, and `tests/status_accuracy.rs`. Then write the three S02 DoD tests (`status_refresh_max_dur.rs`, `status_multi_kind_coexist.rs`, `status_cleanse_policy.rs`) + a fresh `tests/status_accuracy.rs` covering the same accuracy gate behavior (since accuracy is a live mechanic in M017, not a legacy concern). S03-S05 add their semantic tests.
- **Path B — Migrate in place:** Keep the 4 files, retarget their assertions to `StatusBag`. Lower churn but accumulates legacy test framing that was already supposed to be sunset.

**Recommendation: Path A.** Costs ~20 extra lines per deleted file's rewrite (the accuracy test is the only one with non-redundant coverage; the other three test lifecycle behaviors that the three new S02 DoD tests already cover). The benefit is honoring the canon decision and leaving S02-S05 with a clean test directory shape. **Decision belongs to the planner** — flagging here, not unilateral.

## Bevy 0.18 Component-or-default API (T02 unknown)

The plan as written suggests `EntityCommands::entry::<StatusBag>().or_default()`. Bevy 0.18 added `EntityCommands::entry` (commit 8a3d2c5 in 0.15+), but the exact ergonomics under deferred commands need verification. Two fallback shapes if the API is awkward in a system context:

- **Shape A — query-first:** read `Query<Option<&mut StatusBag>>` in the apply system; on `Some`, call `apply` in place; on `None`, queue `commands.entity(target).insert({ let mut b = StatusBag::default(); b.apply(kind, dur); b })`.
- **Shape B — bundle in bootstrap:** seed every unit with `StatusBag::default()` at spawn time in `src/combat/bootstrap.rs`. Removes the "create-on-first-apply" branch entirely. Costs one component slot per unit (cheap; the bag is empty until first apply).

**Recommendation: Shape B** — bootstrap-seed eliminates the branch and matches how `RoundFlags`, `BasicStreak`, and other per-unit state are already seeded. T02 becomes a pure `query.get_mut(target)` + `bag.apply(kind, dur)` call, no `commands.insert` branch needed.

## Sources

- `src/combat/status_effect.rs` (current single-component shape, refresh-max-dur math).
- `src/combat/turn_system/pipeline.rs:700-755` (apply site + accuracy gate).
- `src/combat/turn_system/mod.rs:465-509` (tick + expire site).
- `src/combat/follow_up.rs:90-108` (query tuple to migrate).
- `src/combat/turn_system/tests.rs` (in-tree fixtures to migrate).
- `.gsd/spikes/spike-kernel-primitives/sketches/status_and_reactive_events.rs` (canon `BuffKind`/`StatusEffect` shape sketch — reference, not law).
- `.gsd/spikes/spike-skill-dsl-coverage/gaps.md:52-70` (`EmitCleanse` is M019, not M017 — confirms cleanse policy lives in this slice but the Effect variant doesn't).
- `.gsd/milestones/M017/M017-CONTEXT.md` (scope/out-of-scope; D004 single-instance + refresh-max-dur; cleanse Debuff-only + Blessed cleanse-immune).
- `.gsd/milestones/M017/slices/S01/S01-SUMMARY.md` (post-S01 state — vocabulary landed, no policy yet).