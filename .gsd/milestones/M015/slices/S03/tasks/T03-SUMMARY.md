---
id: T03
parent: S03
milestone: M015
key_files:
  - src/data/skills_ron.rs
  - assets/data/skills.ron
  - src/combat/state.rs
  - src/combat/resolution.rs
  - tests/patamon_blueprint_seam.rs
  - tests/event_stream.rs
key_decisions:
  - Used a top-level `SkillCustomSignal` wrapper around a Patamon-specific enum rather than a line/mechanic-owned enum, preserving RON as typed intent and leaving interpretation to downstream blueprint Rust logic.
  - Copied custom signals into `ResolvedAction` during `resolve_action` to avoid downstream per-action asset re-lookups.
  - Kept `apply_effects` free of per-Digimon signal interpretation so generic effect application remains branch-light.
duration: 
verification_result: passed
completed_at: 2026-05-08T16:02:50.146Z
blocker_discovered: false
---

# T03: Added Patamon-scoped custom signal declarations and copied them into resolved actions without making the effect applier authoritative.

**Added Patamon-scoped custom signal declarations and copied them into resolved actions without making the effect applier authoritative.**

## What Happened

Added a typed `SkillCustomSignal` schema that wraps a Patamon-specific `PatamonCustomSignal` enum, with `SkillDef.custom_signals` defaulting to an empty vector for older RON and fixtures. Seeded `patamon_ult` in `assets/data/skills.ron` with `Patamon(BuildHolySupportGrace(amount: 1))` while leaving `holy_breeze` and other skills signal-free. Extended `ResolvedAction` with copied `custom_signals` metadata and populated it in `resolve_action`, deliberately leaving `apply_effects` unchanged so custom signals remain data-only until the downstream Patamon blueprint seam interprets them. Created `tests/patamon_blueprint_seam.rs` through a RED/GREEN cycle to prove missing fields default empty, unknown Patamon variants are rejected by serde, tracked RON parses the seeded signal, and resolved actions carry the signal without changing ordinary effect metadata. Adapted three exhaustive `SkillDef` fixtures in `tests/event_stream.rs` with explicit empty `custom_signals` so existing S03 event-stream regressions continue to compile without changing their behavior.

## Verification

Fresh verification passed after the final code changes and formatting. The required T03 command `cargo test --test patamon_blueprint_seam custom_signal` passed 4 custom-signal tests. Existing S03 kernel/snapshot regression surfaces also passed across `event_stream`, `battery_loop_kernel`, `predator_loop_kernel`, and `validation_snapshot`; the combat authority audit script passed; and `cargo check` exited 0 for the headless app/binaries. The initial RED checks failed as expected before implementation because the test target/schema fields did not exist.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test patamon_blueprint_seam custom_signal` | 0 | ✅ pass | 158ms |
| 2 | `cargo test --test event_stream --test battery_loop_kernel --test predator_loop_kernel --test validation_snapshot` | 0 | ✅ pass | 536ms |
| 3 | `python3 scripts/verify_combat_authority_audit.py` | 0 | ✅ pass | 20ms |
| 4 | `cargo check` | 0 | ✅ pass | 2740ms |

## Deviations

Minor local adaptation: `tests/event_stream.rs` had exhaustive `SkillDef` fixtures without struct update syntax, so explicit empty `custom_signals` fields were added there to preserve the existing regression surface.

## Known Issues

Existing compiler warning backlog remains outside this task; all verification commands exited 0.

## Files Created/Modified

- `src/data/skills_ron.rs`
- `assets/data/skills.ron`
- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/event_stream.rs`
