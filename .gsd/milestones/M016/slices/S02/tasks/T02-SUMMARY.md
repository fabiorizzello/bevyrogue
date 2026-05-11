---
id: T02
parent: S02
milestone: M016
key_files:
  - src/combat/blueprints/dorumon.rs
  - src/combat/blueprints/mod.rs
  - src/combat/turn_system/pipeline.rs
  - assets/data/skills.ron
  - tests/dorumon_blueprint.rs
key_decisions:
  - Kept Dorumon-specific Predator Loop decoding inside `src/combat/blueprints/dorumon.rs` and emitted only generic kernel transitions from the plugin boundary.
duration: 
verification_result: passed
completed_at: 2026-05-09T14:43:31.472Z
blocker_discovered: false
---

# T02: Moved Dorumon Predator Loop decoding into a dedicated blueprint plugin.

**Moved Dorumon Predator Loop decoding into a dedicated blueprint plugin.**

## What Happened

I added a dedicated Dorumon blueprint interpreter that accepts the generic owner-keyed custom signal envelope, decodes Predator Loop requests, and emits only generic `CombatKernelTransition::PredatorLoop(...)` values. The module now covers build-exploit, prey-lock, payoff, berserk, tick, and expire routing with malformed payload and unknown-signal rejection, while the shared combat systems remain branch-free and generic. I also seeded Dorumon and DORUgamon RON declarations with the new blueprint-addressed format and added direct tests that prove exploit/prey-lock transitions, kernel cap/expiry/strain-block behavior, and snapshot readability through the generic validation surface.

## Verification

Verified with `cargo test --test dorumon_blueprint --no-fail-fast`; all 3 tests passed, covering exploit/prey-lock transition mapping, malformed-payload rejection, and cap/expiry/strain-block paths through the generic kernel state.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test dorumon_blueprint --no-fail-fast` | 0 | ✅ pass | 531ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/blueprints/dorumon.rs`
- `src/combat/blueprints/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `assets/data/skills.ron`
- `tests/dorumon_blueprint.rs`
