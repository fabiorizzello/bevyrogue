---
id: T02
parent: S04
milestone: M002
key_files:
  - src/combat/blueprints/agumon/baby_burner.rs
  - src/combat/blueprints/agumon/mod.rs
  - src/combat/turn_system/pipeline/paths/single_target.rs
  - src/combat/turn_system/pipeline/application.rs
  - src/combat/turn_system/resolve.rs
  - src/combat/mechanics/follow_up/resolve.rs
  - tests/agumon_baby_burner_reactive.rs
key_decisions:
  - Agumon Baby Burner reactive detonate is implemented as post-action runtime intents: per-adjacent `DealDamage` plus `BlueprintSignal(owner=agumon,name=baby_burner_detonate,payload=UnitTarget(target))` so combat mutation stays in the shared applier while observability uses the existing generic blueprint envelope.
  - Legacy single-target post-action outputs now use the live `IntentQueue`/`IntentExecutionMeta` resources when present, preserving cast/follow-up metadata for reactive intents without adding digimon-specific logic to shared runtime code.
duration: 
verification_result: passed
completed_at: 2026-05-20T21:56:49.706Z
blocker_discovered: false
---

# T02: Registered Agumon Baby Burner reactive detonate through the post-action runtime seam and verified deterministic adjacent-target damage plus blueprint transition observability.

**Registered Agumon Baby Burner reactive detonate through the post-action runtime seam and verified deterministic adjacent-target damage plus blueprint transition observability.**

## What Happened

Added a new Agumon-owned `baby_burner` post-action module that watches the T01 KO context seam, gates on `agumon_ult` + lethal primary hit + `heated_remaining > 0`, resolves adjacent alive enemy targets with existing `TargetShape::Blast` semantics from roster snapshots, and emits deterministic Fire detonate damage plus targeted `baby_burner_detonate` blueprint signals. Registered the reaction in Agumon runtime setup and added the detonate signal to `SignalTaxonomy`. To make the reaction hit the real combat surface, threaded live `IntentQueue`/`IntentExecutionMeta` resources through the legacy `step_app` root and follow-up callers so post-action reactions can enqueue shared runtime intents directly when the queue is present. Added `tests/agumon_baby_burner_reactive.rs` covering the positive lethal-Heated case, primary-target exclusion, non-lethal/no-Heated/non-Baby-Burner negatives, no duplicate detonate on extra updates, and exact `OnKernelTransition::Blueprint` targeting for real detonate recipients.

## Verification

Verified the new headless Baby Burner reactive coverage and the existing UnitDied payload contract with `cargo test --test agumon_baby_burner_reactive --test unit_died_payload`; the new suite passed lethal-Heated adjacent detonate, negative no-op cases, duplicate-update protection, and exact `baby_burner_detonate` transition targeting while the older test still proved Heated payload preservation on KO.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test agumon_baby_burner_reactive --test unit_died_payload` | 0 | ✅ pass | 667ms |

## Deviations

Expanded the legacy `step_app` plumbing beyond the original Agumon file list so root and follow-up action paths can hand live `IntentQueue`/`IntentExecutionMeta` resources into `single_target::run`; this lets post-action reactions enqueue real runtime intents directly when the queue already exists instead of relying only on deferred world access.

## Known Issues

None.

## Files Created/Modified

- `src/combat/blueprints/agumon/baby_burner.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/turn_system/pipeline/paths/single_target.rs`
- `src/combat/turn_system/pipeline/application.rs`
- `src/combat/turn_system/resolve.rs`
- `src/combat/mechanics/follow_up/resolve.rs`
- `tests/agumon_baby_burner_reactive.rs`
