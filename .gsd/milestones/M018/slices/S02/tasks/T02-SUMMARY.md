---
id: T02
parent: S02
milestone: M018
key_files:
  - src/data/skills_ron.rs
  - src/combat/resolution.rs
key_decisions:
  - Added TargetShape::Blast between Single and Row to preserve enum ordering conventions; AllEnemies stays as-is (no new variant)
  - TargetableSnapshot is a plain Rust struct (no Bevy Component) to keep resolve_targets() pure and testable without ECS
  - Used u8::abs_diff(slot) <= 1 for Blast adjacency — avoids underflow on slot 0 and naturally handles edge case without checked_sub
  - target_shape_is_executable_now left unchanged (false for Blast/AllEnemies) — fan-out execution is T03/T04 scope; opening the gate now would cause silent single-target execution of multi-target shapes
duration: 
verification_result: passed
completed_at: 2026-05-13T16:14:46.053Z
blocker_discovered: false
---

# T02: Added TargetShape::Blast variant and pure resolve_targets() helper with 6 table-driven tests covering edge slots, KO adjacents, and AllEnemies ordering

**Added TargetShape::Blast variant and pure resolve_targets() helper with 6 table-driven tests covering edge slots, KO adjacents, and AllEnemies ordering**

## What Happened

Added `Blast` as a new variant to `TargetShape` in `src/data/skills_ron.rs` (between Single and Row, with a doc comment explaining slot ±1 semantics). Did NOT add a new AoE(All) variant — the existing `AllEnemies` covers the DSL alias.

Added two new public types to `src/combat/resolution.rs`: `TargetEntry` (id, team, slot_index: u8, alive: bool) and `TargetableSnapshot` (Vec<TargetEntry>). Both are pure Rust structs with no Bevy ECS dependency, enabling deterministic testing without a running world.

Added `pub fn resolve_targets(shape, primary, snapshot) -> Vec<UnitId>` to resolution.rs. Contract: Single/Row/SelfOnly → [primary]; Blast → slot.abs_diff(primary_slot) <= 1, same team, alive, sorted slot_index asc; AllEnemies → all alive on primary's team, sorted slot_index asc. KO'd adjacents are silently absorbed (omitted from result). Determinism is enforced via explicit sort_by_key, never trusting iteration order.

Added 6 table-driven tests inline in resolution.rs `#[cfg(test)] mod tests`: single returns primary; blast edge slot 0 returns only [0,1]; blast with KO'd adjacent omits it; blast all-three inserted out-of-order is sorted; AllEnemies omits dead; AllEnemies sorted ascending. All 6 pass.

`target_shape_is_executable_now` intentionally left returning false for Blast/AllEnemies — fan-out execution wiring is deferred to T03/T04 to avoid single-primary-only execution of multi-target shapes.

## Verification

cargo test resolve_targets: 6 passed, 0 failed. cargo check: clean (warnings only, no errors).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test resolve_targets 2>&1 | grep -E '(test result|FAILED)'` | 0 | 6 passed, 0 failed (in lib + unit crates) | 4200ms |
| 2 | `cargo check 2>&1 | tail -5` | 0 | Finished dev profile, no errors | 1860ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`
