---
id: S07
parent: M011
milestone: M011
provides:
  - ["ToughnessCategory enum (Standard/Armored/Shielded) on Toughness component, threaded through UnitDef/RON/bootstrap", "RoundFlags component with break_sealed field, spawned on every unit, reset on TurnAdvanced", "Break Seal pipeline integration: set on break in pipeline.rs, blocked in apply_hit when sealed, reset in advance_turn_system", "tests/toughness_categories.rs: 4 integration tests covering all category behaviors and seal lifecycle"]
requires:
  []
affects:
  - ["S08", "S09"]
key_files:
  - ["src/combat/toughness.rs", "src/combat/round_flags.rs", "src/combat/mod.rs", "src/combat/resolution.rs", "src/data/units_ron.rs", "assets/data/units.ron", "src/combat/bootstrap.rs", "src/combat/turn_system/mod.rs", "src/combat/turn_system/pipeline.rs", "src/combat/follow_up.rs", "tests/toughness_categories.rs"]
key_decisions:
  - ["ToughnessCategory #[derive(Default)] on Standard variant preserves all existing Toughness::new() call sites without changes", "Armored ceiling division (amount+1)/2 prevents zero-damage edge case on 1-point toughness hits", "break_sealed short-circuits before category dispatch — single noop path for all categories", "Shielded clamps current at 0 via saturating_sub.max(0); bar drains but never transitions to broken", "follow_up.rs has a local ResolveActorsQuery alias structurally identical to turn_system/mod.rs — both must be kept in sync on every query extension", "Seal reset is placed at the start of the TurnAdvanced iteration in advance_turn_system, making it the owner's turn-start event"]
patterns_established:
  - ["RoundFlags component (spawned on every unit) is the canonical per-round flag store — S08 Form Identity once-per-round triggers should reuse this component", "MessageCursor<CombatEvent> between action dispatches is the standard dual-surface verification pattern in integration tests", "App::new() without MinimalPlugins is the headless test setup convention — advance_turn_system Part 1 fires correctly in this context"]
observability_surfaces:
  - ["CombatEventKind::OnBreak is the single source of truth for break events — absence of OnBreak after a ToughnessHit signals suppression by Shielded or Break Seal", "RoundFlags.break_sealed queryable in tests and debug builds to inspect seal state", "ActionLog::Break entries absent after sealed attempts — diagnostic signal for seal enforcement"]
drill_down_paths:
  - [".gsd/milestones/M011/slices/S07/tasks/T01-SUMMARY.md", ".gsd/milestones/M011/slices/S07/tasks/T02-SUMMARY.md", ".gsd/milestones/M011/slices/S07/tasks/T03-SUMMARY.md"]
duration: ""
verification_result: passed
completed_at: 2026-04-28T09:16:37.283Z
blocker_discovered: false
---

# S07: Toughness 3 categorie (Standard/Armored/Shielded) + Break Seal

**Differentiated enemy defensive archetypes via ToughnessCategory enum and introduced RoundFlags-backed Break Seal preventing repeated breaks within the same round, validated by 4 integration tests.**

## What Happened

## What This Slice Delivered

S07 introduced the Toughness archetype system that differentiates enemies on the offensive-pressure axis: Standard enemies break normally, Armored enemies require ~2x cumulative toughness damage (ceiling-halving incoming hits), and Shielded enemies never break from toughness hits regardless of cumulative damage. A Break Seal mechanism prevents chain-breaking the same defender twice within a single round.

### T01 — Foundational types
Introduced `ToughnessCategory` (Standard/Armored/Shielded) in `src/combat/toughness.rs` as a `#[derive(Default)]` enum so Standard is the zero-cost default and all existing `Toughness::new()` call sites compile unchanged. Added `Toughness::with_category(max, weaknesses, category)` for explicit construction. Extended `Toughness::apply_hit` with a `break_sealed: bool` parameter and per-category dispatch:
- `break_sealed=true` short-circuits before any mutation (noop for all categories)
- `Shielded`: `saturating_sub(...).max(0)` clamps current at 0, never sets broken
- `Armored`: `(amount + 1) / 2` ceiling division before standard break logic
- `Standard`: exact prior semantics preserved

Created `src/combat/round_flags.rs` with `RoundFlags { break_sealed: bool }` (Bevy Component, Default=false). Updated `resolution.rs` with a `false` placeholder to keep the codebase compiling.

### T02 — End-to-end wiring
Threaded T01 primitives through the full data and ECS pipeline:
- `ToughnessCategory` gained Serde derives for RON round-trip; added to `UnitDef` with `#[serde(default)]`
- Devimon (id 101) set to `toughness_category: Armored` in `units.ron` as the canonical integration fixture
- `bootstrap.rs` uses `Toughness::with_category(...)` and inserts `RoundFlags::default()` on every spawned unit
- `turn_system/mod.rs` extended `ResolveActorsQuery` to element 12 (`Option<&'static mut RoundFlags>`) and reset `break_sealed=false` on the unit whose `TurnAdvanced` fires
- `pipeline.rs` reads `defender_round_flags`, passes `defender_break_sealed` into `apply_effects`, and sets `break_sealed=true` when `outcome.broke==true`
- `follow_up.rs` local `ResolveActorsQuery` alias updated in parallel (Bevy structural query matching requirement)
- ~9 test files updated to pass `false` for the new `defender_break_sealed` parameter and `Default::default()` for `toughness_category` in UnitDef literals

### T03 — Integration test suite
Created `tests/toughness_categories.rs` with 4 headless Bevy tests (App::new pattern, fixed RNG seed, inline SkillBook fixture with Fire ToughnessHit(20)):
1. `standard_breaks_in_one_full_hit` — one hit on Standard (toughness_max=20, Fire weakness) → 1 OnBreak, broken=true
2. `armored_requires_two_full_hits` — Armored halves to 10 effective; first hit → 0 OnBreak, current=10; second hit → 1 OnBreak, broken=true
3. `shielded_never_breaks` — three hits on Shielded → 0 OnBreak, broken=false, current=0 (floor-clamped)
4. `break_seal_blocks_repeat_break_in_same_round_then_lifts_on_next_turn` — break Standard (1 OnBreak, seal set), restore toughness, second hit sealed (0 OnBreak), send TurnAdvanced+update (seal reset via advance_turn_system), third hit (1 OnBreak again)

Used `MessageCursor<CombatEvent>` for dual-surface verification (event count + component state).

## Patterns Established for S08+
- `RoundFlags` component is spawned on every unit and is available for S08's once-per-round Form Identity triggers — the per-turn reset hook in `advance_turn_system` is already in place
- The `(amount + 1) / 2` ceiling division pattern is the canonical formula for damage halving in this engine
- The MessageCursor + component dual-surface pattern is the standard for integration tests that need to assert "event fired / did not fire"

## Verification

**Slice-level verification — all pass:**

| Command | Result |
|---------|--------|
| `cargo test --test toughness_categories` | 4/4 pass (standard_breaks_in_one_full_hit, armored_requires_two_full_hits, shielded_never_breaks, break_seal_blocks_repeat_break_in_same_round_then_lifts_on_next_turn) |
| `cargo test` (full suite) | All 33 test groups pass, 0 failures |

**Per-task verification:**
- T01: `cargo test --lib combat::toughness` — 9/9 pass (6 original + 3 new); `cargo check` — 0 errors
- T02: `cargo test` — full suite green after wiring; `defender_break_sealed` path live
- T03: `cargo test --test toughness_categories` — 4/4 pass; `cargo test` full suite green

**R079 validated:** ToughnessCategory enum differentiated via integration tests. Break Seal lifecycle (set on break, blocks same-round re-break, resets on TurnAdvanced) exercised end-to-end through the real ECS pipeline.

## Requirements Advanced

None.

## Requirements Validated

- R079 — 4 integration tests in tests/toughness_categories.rs pass covering Standard/Armored/Shielded break behaviors and Break Seal set/block/reset lifecycle. Full cargo test suite green.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

Shielded current floors at 0 (not preserved at toughness_max as the plan description implied). The plan note "(clamped)" referred to floor-clamping at 0 and the implementation confirms this. Tests assert current==0 for Shielded, which is correct per the T01 implementation.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

None.
