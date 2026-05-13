# S04: Paralyzed + Slowed — turn skip + delay-on-apply

**Goal:** Wire Paralyzed (always-skip turn, deterministic — canon §H.1) and Slowed (one-shot −30% gauge push at apply time only, via TurnAdvance event reusing M010 plumbing) on top of the S02 StatusBag lifecycle. No RNG-based skip, no recurring tick for Slowed, no clamp (M018 territory). Re-apply of Slowed refreshes duration via existing refresh_max_dur but does NOT re-push the gauge.
**Demo:** Test `status_paralyzed_skip.rs`: scenario con seed fisso, 100 turn iterations Paralyzed, skip count nel range deterministico atteso. Test `status_slowed_delay.rs`: applicare Slowed pusha la timeline visibilmente.

## Must-Haves

- `tests/status_paralyzed_skip.rs` green: 100 turns with Paralyzed dur=100 yields 100 skips and 0 ActionIntents (deterministic, fixed seed).
- `tests/status_slowed_delay.rs` green: applying Slowed emits exactly one `TurnAdvance { amount_pct: -30 }` event; defender AV decreases by 3000 (modulo TempoResistance) on next `apply_turn_advance_system` tick; re-apply does NOT emit a second `TurnAdvance` (refresh_max_dur only).
- Full `cargo test` headless suite stays green; no regression in `combat_coherence`, `follow_up_chains`, `form_identity`, `status_amp_pipeline`.
- Grep guard from S01 still clean (no `Burn`/`Freeze`/`Shock`/`DeepFreeze` outside reserved variants).

## Proof Level

- This slice proves: Integration tests under `tests/` with deterministic seeds; emitted CombatEvents asserted via existing event-bus harness.

## Integration Closure

Slowed delay path goes through canonical `TurnAdvance` event → `apply_turn_advance_system` → `resistance::apply_av_change`, automatically picking up TempoResistance handling. Paralyzed skip mirrors the existing Stunned short-circuit pattern in `process_turn_advanced_system`.

## Verification

- Reuses existing `OnActionFailed` (or `OnStatusTick` already emitted at turn-start) for Paralyzed skip signaling; reuses existing `TurnAdvance` event for Slowed gauge push (already routed through `apply_turn_advance_system` and `resistance::apply_av_change`). No new event variants. JSONL log naturally orders `OnStatusApplied → TurnAdvance` for Slowed.

## Tasks

- [x] **T01: Paralyzed skip-turn in process_turn_advanced_system** `est:M`
  Add Paralyzed always-skip semantic to the per-TurnAdvanced handler in `src/combat/turn_system/mod.rs`. Mirror the existing Stunned short-circuit at L495–502: detect Paralyzed presence in the unit's StatusBag inside the mutable-borrow block (via `bag.has(&StatusEffectKind::Paralyzed)`), tick the bag normally so duration decrements, then `continue` to skip action dispatch. Extend the `Snap` struct with `is_paralyzed: bool` populated from the StatusBag at snapshot time so the enemy-AI dispatch gate at L547 can be widened to `&& !shock_cancelled && !snap.is_stunned && !snap.is_paralyzed`. Emit `CombatEventKind::OnActionFailed { reason: "paralyzed" }` (or the closest existing variant if `OnActionFailed` does not carry a reason — fall back to the generic skip-emission used by Stunned) so tests can count skips deterministically. Order: Heated DoT (existing) runs first, then Stunned check, then Paralyzed check + tick, then `continue`. Do NOT alter the Stunned branch.
  - Files: `src/combat/turn_system/mod.rs`
  - Verify: cargo check && cargo test --lib turn_system

- [x] **T02: Slowed delay-on-apply via TurnAdvance event in pipeline** `est:M`
  In `src/combat/turn_system/pipeline.rs` around L723–760 (the `if outcome.succeeded { if let Some((kind, duration)) = ... status_to_apply` branch), add the first-apply gauge push for Slowed. Before calling `bag.apply(kind.clone(), duration)`, compute `let is_first_apply_slowed = matches!(kind, StatusEffectKind::Slowed) && defender_bag.as_ref().map_or(true, |b| !b.has(&StatusEffectKind::Slowed));`. After the existing `OnStatusApplied` emission (so JSONL log order reads applied → delayed), if `is_first_apply_slowed` emit `CombatEventKind::TurnAdvance { target: target_id, amount_pct: -30 }` via `emit_combat_event` so `apply_turn_advance_system` consumes it next tick (this routes through `resistance::apply_av_change` for free TempoResistance handling). Do NOT mutate `ActionValue` directly — keep the event-bus path. Re-apply must NOT re-emit: the `has(&Slowed)` check guarantees first-apply-only. Resist branch (`!passes`) must NOT emit either.
  - Files: `src/combat/turn_system/pipeline.rs`
  - Verify: cargo check && cargo test --lib

- [x] **T03: Integration test: status_paralyzed_skip** `est:M`
  Create `tests/status_paralyzed_skip.rs`. Spawn 1 ally + 1 enemy via the standard test bootstrap; apply `StatusEffectKind::Paralyzed` with duration=100 to the enemy via direct `StatusBag::apply` insertion (or via a skill resolution if simpler). Drive 100 `TurnAdvanced` cycles for the enemy by writing the event and running the schedule. Assert: zero `ActionIntent` written for the enemy across all 100 cycles; the matching skip-signal events (e.g. `OnActionFailed { reason: "paralyzed" }` or whatever T01 settled on) appear at the expected count (100 minus expirations once duration ticks to 0 — pick the count consistent with T01's tick ordering). Use a fixed seed for `CombatRng`. Keep the test deterministic and headless. Naming follows the functional convention from `CLAUDE.md` (no `s##_` prefix).
  - Files: `tests/status_paralyzed_skip.rs`
  - Verify: cargo test --test status_paralyzed_skip

- [x] **T04: Integration test: status_slowed_delay** `est:M`
  Create `tests/status_slowed_delay.rs`. Spawn a defender unit with a known starting `ActionValue` (e.g. 5000). Apply Slowed via the skill-resolution path (so `pipeline.rs` runs and the first-apply branch executes) — use a deterministic seed so the status-accuracy roll passes. Assert: exactly one `CombatEventKind::TurnAdvance { target, amount_pct: -30 }` is emitted, sourced after `OnStatusApplied`. Run `apply_turn_advance_system` and assert defender AV decreased by 3000 (or matches the expected value once `resistance::apply_av_change` is applied for a unit with no TempoResistance, which equals 3000). Then apply Slowed a second time on the same target and assert NO additional `TurnAdvance` event is emitted (refresh_max_dur path only, gauge already pushed). Keep deterministic and headless.
  - Files: `tests/status_slowed_delay.rs`
  - Verify: cargo test --test status_slowed_delay

- [ ] **T05: Full-suite verification and grep guard** `est:S`
  Run the full headless verification: `cargo check`, `cargo test` (entire integration suite). Confirm zero failures, zero ignored. Re-run the S01 grep guard: ensure no occurrences of `Burn|Freeze|Shock|DeepFreeze` in `src/` and `tests/` outside the reserved-variant declarations in `src/combat/status_effect.rs` and `src/data/skills_ron.rs`. Capture exit codes and a brief evidence summary into the task verification record. Pure verification task — no source files modified.
  - Verify: cargo check && cargo test && grep -rn -E 'Burn|Freeze|Shock|DeepFreeze' src/ tests/ | grep -v 'reserved' | wc -l

## Files Likely Touched

- src/combat/turn_system/mod.rs
- src/combat/turn_system/pipeline.rs
- tests/status_paralyzed_skip.rs
- tests/status_slowed_delay.rs
