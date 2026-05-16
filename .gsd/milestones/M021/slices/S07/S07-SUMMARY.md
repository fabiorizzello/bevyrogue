---
id: S07
parent: M021
milestone: M021
provides:
  - 6 canonical passives running through shared PassiveRunner/filter wiring
  - IncomingDamage pre-damage seam and BlockReactionTriggered post-mitigation event
  - DamageModifierLedger with ordered deterministic modifier fold
  - Deterministic Tentomon Block Reaction before DR cascade
  - Plugin-boot passive listener installation pattern
requires:
  []
affects:
  - S08
  - S09
  - S11
key_files:
  - src/combat/api/event_filter.rs
  - src/combat/api/event_bridge.rs
  - src/combat/api/passive_runner.rs
  - src/combat/api/applier.rs
  - src/combat/modifiers.rs
  - src/combat/events.rs
  - src/combat/blueprints/agumon/mod.rs
  - src/combat/blueprints/gabumon.rs
  - src/combat/blueprints/patamon/mod.rs
  - src/combat/blueprints/renamon.rs
  - src/combat/blueprints/tentomon.rs
  - src/combat/blueprints/dorumon/mod.rs
  - src/combat/blueprints/dorumon/hooks.rs
  - src/combat/battery_loop.rs
  - src/combat/kernel.rs
  - src/combat/plugin.rs
  - tests/passive_event_filters.rs
  - tests/block_reaction_pipeline.rs
  - tests/passive_canon_support.rs
  - tests/passive_reactive_canon.rs
key_decisions:
  - Represent passive subscriptions as composable runtime EventFilter predicates rather than owner/name string pairs — enables multi-condition matching without proliferating filter variants.
  - IncomingDamage is an observational seam only; armed modifier state must be written before the hit resolves so the damage pipeline can drain it synchronously.
  - Canonical modifier fold order Intrinsic→Status→Buff→Passive is enforced in DamageModifierLedger to guarantee deterministic, replayable damage across any combination of layered modifiers.
  - BlockReactionTriggered is emitted for any consumed passive mitigation modifier (not only Tentomon's proc) making it the single canonical diagnostic event for pre-damage passive mitigation.
  - Canonical passive listeners bootstrap from CombatPlugin at startup using fixed UnitId owners and shared trigger keys with per-blueprint guards, eliminating per-test scaffolding.
patterns_established:
  - Composable EventFilter predicate for passive subscriptions (all/any combinators over combat event matchers)
  - DamageModifierLedger: target-scoped ordered fold with drain-on-resolve semantics
  - IncomingDamage → modifier drain → OnDamageDealt → BlockReactionTriggered event ordering
  - PassiveRunner one-pass outer-cycle stop rule to settle state-gated timelines without spinning
  - Plugin-boot passive listener installation with canonical UnitId owners
observability_surfaces:
  - IncomingDamage event (pre-damage observational seam)
  - BlockReactionTriggered event (canonical post-mitigation surface for any passive modifier consumption)
  - OnKernelTransition::Blueprint JSONL entries for passive trigger round-trips
  - PassiveRunner circuit-breaker warnings (256-hop breaker logs)
  - Combat event stream via SignalBus
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-16T11:58:36.479Z
blocker_discovered: false
---

# S07: Modifier pipeline + Migrate 6 passive canon

**Introduced the shared passive modifier pipeline, composite passive event routing, and migrated all 6 canonical Digimon passives onto PassiveRunner with deterministic Tentomon Block Reaction.**

## What Happened

S07 delivered four interlocking subsystems across four tasks:

**T01 — Composite passive event routing and loop-safe PassiveRunner**
Added `EventFilter` as a composable runtime predicate for passive subscriptions (replacing owner/name trigger pairs), bridged all combat events into `Signal::CombatEvent` while preserving the legacy `kernel/ult_used` blueprint signal, and extended PassiveRunner with a one-pass outer-cycle stop rule. Queued intents are flushed between BeatRunner steps to prevent state-gated timelines from spinning on stale reads. Verified by `passive_event_filters`.

**T02 — Pre-damage modifier ledger and Block Reaction seam**
Introduced `DamageModifierLedger` as a target-scoped resource holding an ordered modifier fold (Intrinsic→Status→Buff→Passive). `IncomingDamage` is emitted as a pure observational seam before damage resolves; armed Block Reaction state must exist before the hit arrives. After the fold the pipeline drains the ledger, applies mitigation, and emits `BlockReactionTriggered` exactly once when a passive modifier is consumed. Verified by `block_reaction_pipeline`.

**T03 — Agumon, Gabumon, Patamon, Renamon passive bootstrap**
Wired all four support passives as plugin-boot listeners under fixed canonical UnitId owners using the shared `kernel/ult_used` trigger with per-blueprint guard keys. Twin Flame, Holy Support, Predator Loop, and Kitsune Grace now install declaratively at boot without per-test scaffolding. Verified by `passive_canon_support`.

**T04 — Dorumon and Tentomon reactive passives; deterministic Block Reaction**
Migrated Dorumon's enemy-kill listener and Tentomon's battery-loop block-arm logic onto PassiveRunner. The damage applier was updated to emit `BlockReactionTriggered` for any consumed passive mitigation modifier (not just Tentomon-specific procs), making it the canonical diagnostic surface for pre-damage passive mitigation. The canon test uses a seed search to confirm the armed path triggers and the no-proc guard path is respected. Verified by `passive_reactive_canon`.

All 6 slice-level checks passed on the final tree: `passive_event_filters`, `block_reaction_pipeline`, `passive_canon_support`, `passive_reactive_canon`, full `cargo test`, and `cargo check --features windowed`.

## Verification

All 6 slice-level verification commands passed:
- `cargo test --test passive_event_filters` → exit 0 (2428 ms)
- `cargo test --test block_reaction_pipeline` → exit 0 (393 ms)
- `cargo test --test passive_canon_support` → exit 0 (503 ms)
- `cargo test --test passive_reactive_canon` → exit 0 (395 ms)
- `cargo test` (full suite) → exit 0 (12586 ms)
- `cargo check --features windowed` → exit 0 (5982 ms)

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

["Added BeatRunner cursor/loop accessors and a one-pass outer-cycle stop in PassiveRunner (not in original T01 plan) to allow state-gated passive timelines to settle.", "BlockReactionTriggered emission was broadened to cover any consumed passive mitigation modifier, not only Tentomon-specific procs, to create a single canonical diagnostic surface.", "Fixed canonical UnitId owners chosen for passive bootstrap to avoid per-test setup (T03 deviation from original approach).", "HP inequality comparison in passive_reactive_canon test was corrected to compare remaining HP rather than absolute values."]

## Known Limitations

["S08–S10 still need roster-driven Digimon migration cleanup and final kernel digimon-free verification before the milestone is end-to-end clean.", "S11 (UI/AI consumers via SkillCtx Preview) and S12 (RosterEntry blueprint-keyed) remain."]

## Follow-ups

None.

## Files Created/Modified

None.
