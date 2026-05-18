# S10 Replan

**Milestone:** M021
**Slice:** S10
**Blocker Task:** T03
**Created:** 2026-05-17T06:33:10.055Z

## Blocker Description

cargo test --test dorumon_predator_runtime still fails after T03 because the final predator runtime transition is rejected with CapReached { cap: PreyLock } instead of producing the expected applied prey-lock transition, so the remaining observability/grep proof should not proceed until the Dorumon runtime contract is reconciled and re-verified.

## What Changed

Split the remaining work into a runtime follow-up and a final observability/grep pass. T04 now resolves and re-verifies the Dorumon predator runtime expectation after the generic blueprint transition changes from T03; a new T05 then genericizes validation/CLI observability and runs the slice exit grep/build proof once the runtime contract is stable.
