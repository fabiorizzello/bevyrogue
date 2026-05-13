# S04 RESEARCH — Paralyzed (skip-turn) + Slowed (delay-on-apply)

## Summary

S04 wires two distinct semantics on top of the S02 `StatusBag` lifecycle:

1. **Paralyzed = skip-turn**, deterministic (always-skip while present, canon §H.1). The actor's `TurnAdvanced` handler must short-circuit *like Stunned does*, emit a "skipped" signal, and still tick the bag (so dur counts down). RNG-based "cancel probability" wording in old code is legacy (Shock); per M017-CONTEXT line 128 Paralyzed is plain skip.
2. **Slowed = one-shot +30% gauge delay AT APPLY TIME** (M017-CONTEXT lines 51–58 + 102): on a successful apply, push the target's AV/gauge backwards once. The instance still lives in `StatusBag` for dur tracking + observability but produces no recurring tick. No clamp (M018 territory). Re-apply does NOT re-push (refresh_max_dur is the only side-effect of re-apply).

DoD: `tests/status_paralyzed_skip.rs` (100 iterations, deterministic skip count); `tests/status_slowed_delay.rs` (AV before/after apply shows visible push).

## Implementation Landscape

| File | Purpose / role for S04 |
|------|------------------------|
| `src/combat/turn_system/mod.rs` lines 425–550 | Per-`TurnAdvanced` handler. Stunned short-circuit at L465 (`if let Some(mut s) = stunned_opt`) is the model: Paralyzed skip goes here, *before or after* status-tick block (L474–513). Enemy AI dispatch is at L516–550 (`intents_out.write`) — must be gated on `!is_paralyzed` like it already is on `!is_stunned` and `!shock_cancelled`. |
| `src/combat/turn_system/pipeline.rs` lines 721–759 | Status apply site (`if outcome.succeeded { if let Some((kind, duration)) = ... status_to_apply { ... bag.apply(...) ; emit OnStatusApplied }`). This is where the Slowed AV-push must fire — **only on the apply branch (`passes`), not on resist**, and **only when the bag did not already contain Slowed** (first-apply-only semantic). Detect with `defender_bag.as_ref().map_or(true, \|b\| !b.has(&Slowed))` *before* the `bag.apply` call. |
| `src/combat/av.rs` | `ActionValue::delay(amount)` already exists (L31) and clamps to ≥0. `MAX_AV = 10000`. "+30% gauge delay" = `(MAX_AV as f32 * 0.30) as i32 = 3000`. Apply via `av.delay(3000)` on the defender entity. |
| `src/combat/events.rs` L94–97 | `CombatEventKind::TurnAdvance { target, amount_pct: i32 }` already exists and is consumed by `apply_turn_advance_system` (mod.rs L607–627) which routes through `resistance::apply_av_change` (handles TempoResistance for *negative* amounts = delays). This is the canonical path — **emit `TurnAdvance { target, amount_pct: -30 }`** rather than mutating AV directly. Reuses M010 plumbing, picks up TempoResistance for free, observable in event log. |
| `src/combat/status_effect.rs` | `StatusBag::has(kind)` (L92) lets us detect first-apply. No schema change needed. |
| `src/combat/turn_system/tests.rs` L209–302 | Legacy `_OLD` tests for Paralyzed/Shock cancel — already `#[allow(dead_code)]`, do NOT resurrect. New tests live in `tests/`. |

## Recommendation — build order

1. **T01 — Paralyzed skip in turn dispatch.** In `turn_system/mod.rs` add a `is_paralyzed` flag computed from the snapshot (extend `Snap` with `is_paralyzed: bool` read from `StatusBag`). After the Stunned short-circuit (L465–472), add an analogous `if snap.is_paralyzed { /* still run status tick, then continue */ }` block. Order matters: tick the bag first so dur decrements, then `continue` to skip both action dispatch and AV-advancement-on-skip is N/A (AV already reset on `TurnAdvanced`). Gate enemy AI dispatch at L517 with `&& !snap.is_paralyzed`. Emit `OnActionFailed { reason: "paralyzed" }` (or a dedicated `OnTurnSkipped`; reuse `OnActionFailed` to avoid event-bus churn).
2. **T02 — Slowed delay-on-apply in pipeline.** In `pipeline.rs` around L730, just before `bag.apply(kind.clone(), duration)`, compute `let is_first_apply_slowed = kind == StatusEffectKind::Slowed && defender_bag.as_ref().map_or(true, |b| !b.has(&StatusEffectKind::Slowed));`. After the `OnStatusApplied` emission, if `is_first_apply_slowed`, emit `CombatEvent { kind: TurnAdvance { target: target_id, amount_pct: -30 }, ... }`. `apply_turn_advance_system` will pick it up next tick. (Alternative: direct `av.delay(3000)` via `commands.entity(target_entity)` — less observable, skips TempoResistance. Prefer event route.)
3. **T03 — Integration tests.** `tests/status_paralyzed_skip.rs`: spawn 1 ally + 1 enemy, apply Paralyzed dur=100, drive 100 `TurnAdvanced` cycles, assert 100 `OnActionFailed`/skip events and zero `ActionIntent`. `tests/status_slowed_delay.rs`: spawn target with `ActionValue(5000)`, apply Slowed via a skill resolution, assert `TurnAdvance { amount_pct: -30 }` event emitted exactly once + AV decreases by 3000 (modulo TempoResistance) on next `apply_turn_advance_system` tick; assert re-apply does NOT emit a second `TurnAdvance`.

## First Proof

After T01: `cargo test --test status_paralyzed_skip` deterministically passes — 100 turns Paralyzed yields 100 skips, 0 intents.

## Verification commands

```bash
cargo check
cargo test --test status_paralyzed_skip --test status_slowed_delay
cargo test                                  # full suite must stay green
```

Plus grep guards already in place from S01 (no `Burn`/`Freeze`/`Shock`/`DeepFreeze` in src/tests outside reserved variants).

## Risks

- **Slowed first-apply-only semantic** is the subtle bit. Re-apply on an already-Slowed target must refresh duration via `StatusBag::apply` (max-dur) but NOT re-push gauge. Putting the `has(&Slowed)` check *before* `bag.apply` is mandatory — after `apply` the check would always return true. Add an explicit test case for re-apply no-double-push.
- **No clamp in M017:** if two attackers each apply Slowed to two different targets before next turn, no problem (single-instance per (target, kind)). But if M018 clamp regression appears, recurring `TurnAdvance` from this slice cannot push AV<0 because `ActionValue::delay` already clamps `.max(0)` (av.rs L33). Safe.
- **Paralyzed + Stunned interaction:** Stunned check runs first and `continue`s — Paralyzed branch will never reach if both present. Acceptable (Stunned is the harder lock). Document, don't fix.
- **Enemy AI dispatch:** the existing `!shock_cancelled` flag at L517 is dead code from legacy Shock; reuse the pattern but keep a clean `is_paralyzed` name. Do not delete `shock_cancelled` here (out of slice scope).
- **Event ordering:** `TurnAdvance` must be emitted AFTER `OnStatusApplied` so the JSONL log reads "applied → delayed" in the natural order S06 will verify.

Slice S04 researched.
