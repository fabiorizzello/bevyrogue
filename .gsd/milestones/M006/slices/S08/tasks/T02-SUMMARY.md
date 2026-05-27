---
id: T02
parent: S08
milestone: M006
key_files:
  - /home/fabio/dev/bevyrogue/tests/windowed_only/agumon_module_extraction.rs
key_decisions:
  - Used source-contract (include_str!) approach rather than live behavior test because src/windowed/ is a binary crate unreachable from tests/ and K001 forbids launching the windowed binary
  - Added AGUMON_ANIM_GRAPH const to also pin the authored RON token, making the proof cover the full three-link chain including the animation-graph → cue-name boundary
  - Placed the new test in the existing agumon_module_extraction.rs file rather than creating a new file, consistent with project test layout conventions (R003)
duration: 
verification_result: passed
completed_at: 2026-05-27T08:40:00.143Z
blocker_discovered: false
---

# T02: Added source-contract test `agumon_cast_cue_resolves_to_registered_enoki_effects` to `tests/windowed_only/agumon_module_extraction.rs`, locking the baby_flame cast→effect chain end-to-end.

**Added source-contract test `agumon_cast_cue_resolves_to_registered_enoki_effects` to `tests/windowed_only/agumon_module_extraction.rs`, locking the baby_flame cast→effect chain end-to-end.**

## What Happened

The goal was to prove Agumon's Baby Flame cast cue resolves to its two registered enoki effects through the per-species seam. The chain has three links: (1) the authored `baby_flame_cast` animation-graph node fires a `SpawnParticle` named `"baby_flame_charge"` (confirmed in `assets/digimon/agumon/anim_graph.ron`); (2) the Agumon module's `on_enter_effect_specs()` maps `"baby_flame_charge"` → `["baby_flame.charge", "baby_flame.ember"]`; (3) both effect ids are registered in `register_agumon_enoki_vfx` with `baby_flame_charge.particle.ron` and `baby_flame_ember.particle.ron` paths, and `skill_start_node_specs()` wires `BABY_FLAME_SKILL_ID` → `BABY_FLAME_CAST_NODE`.

Since `src/windowed/` is binary-crate code unreachable from `tests/`, the proof is source-contract-shaped (K001 forbids launching the windowed binary). Two `include_str!` consts are used: the existing `AGUMON_SRC` for the Rust module, and a new `AGUMON_ANIM_GRAPH` const for the RON asset. The new test `agumon_cast_cue_resolves_to_registered_enoki_effects` pins all three chain links with precise token assertions and clear failure messages. No production code was modified; only `tests/windowed_only/agumon_module_extraction.rs` was extended.

## Verification

Ran `cargo test --features windowed --test windowed_only`. All 70 tests passed (0 failed), including the new `agumon_cast_cue_resolves_to_registered_enoki_effects` test. The test is sensitive to token changes — renaming any of the canonical string literals in the source would cause it to fail with a descriptive message.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only` | 0 | All 70 tests pass including agumon_cast_cue_resolves_to_registered_enoki_effects | 8200ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `/home/fabio/dev/bevyrogue/tests/windowed_only/agumon_module_extraction.rs`
