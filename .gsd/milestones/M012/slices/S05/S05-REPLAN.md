# S05 Replan

**Milestone:** M012
**Slice:** S05
**Blocker Task:** T03
**Created:** 2026-05-01T08:23:43.991Z

## Blocker Description

T03 discovered that Form Identity follow-ups now schedule but the current runtime pipeline still cannot reliably execute same-entity self-target effects through the cap-aware action path, and DORUgamon's canonical cast-trigger/toughness follow-up still fails. A subsequent verification attempt also exposed a compile-time regression in src/combat/turn_system/pipeline.rs where event buffering and attacker/defender component bindings were referenced outside their valid scope. The original remaining T04 was only a verification/docs sweep and is insufficient because the slice now needs explicit remediation work before final contract verification.

## What Changed

Replaced the original verification-only T04 with a remediation task that first restores compilation, then fixes the same-entity self-target/Form Identity execution path and DORUgamon canonical follow-up behavior without changing completed tasks. Added a new T05 as the final focused S05 verification and documentation-tightening task, preserving the original closure intent after remediation is complete.
