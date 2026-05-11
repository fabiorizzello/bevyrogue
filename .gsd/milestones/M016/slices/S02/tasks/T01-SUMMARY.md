---
id: T01
parent: S02
milestone: M016
key_files:
  - src/data/skills_ron.rs
  - src/combat/blueprints/mod.rs
  - src/combat/turn_system/pipeline.rs
  - assets/data/skills.ron
  - tests/digimon_signal_registry.rs
key_decisions:
  - Kept legacy Patamon/Tentomon signal variants as compatibility shims while adding a generic blueprint envelope for new owner-keyed signals.
duration: 
verification_result: passed
completed_at: 2026-05-09T14:43:12.327Z
blocker_discovered: false
---

# T01: Added owner-keyed custom signal envelopes and registry dispatch for blueprint routing.

**Added owner-keyed custom signal envelopes and registry dispatch for blueprint routing.**

## What Happened

I replaced the ad hoc per-Digimon dispatch path with a generic `SkillCustomSignal::Blueprint { owner, signal, payload }` envelope while keeping the legacy Patamon/Tentomon variants as compatibility shims. `src/combat/blueprints/mod.rs` now routes by owner key through a registry and exposes a checked dispatch path that rejects unknown owners, unknown signals, and malformed payloads instead of silently dropping them. I also updated the combat turn pipeline to surface dispatch errors as action failures and added the canonical Dorumon/DORUgamon RON entries using the new blueprint-addressed format, backed by a focused registry/envelope test target.

## Verification

Verified with `cargo test --test digimon_signal_registry --no-fail-fast`; all 4 tests passed, covering Dorumon envelope parsing, registry routing, unknown-owner rejection, and malformed-payload rejection.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test digimon_signal_registry --no-fail-fast` | 0 | ✅ pass | 5999ms |

## Deviations

Kept compatibility variants during the migration instead of deleting the old Patamon/Tentomon enum forms outright.

## Known Issues

None.

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/combat/blueprints/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `assets/data/skills.ron`
- `tests/digimon_signal_registry.rs`
