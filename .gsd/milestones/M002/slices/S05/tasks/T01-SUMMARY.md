---
id: T01
parent: S05
milestone: M002
key_files:
  - src/combat/encounter/bootstrap.rs
  - src/windowed/mod.rs
  - src/windowed/render.rs
  - tests/bootstrap_encounter/encounter_bootstrap_windowed.rs
  - tests/bootstrap_encounter.rs
key_decisions:
  - Enemy dummy assembled by cloning the ally Agumon def and overriding id/team/name — no unit.ron edits needed
  - Stable AGUMON_DUMMY_ID = UnitId(9001) exported from bootstrap.rs for test assertions
  - Sprite placement: ally at x=-200, enemy at x=+200 with flip_x=true for enemy mirroring
  - SP capped to 999 at bootstrap time so resource shortfalls don't block the demo
duration: 
verification_result: passed
completed_at: 2026-05-21T10:22:33.100Z
blocker_discovered: false
---

# T01: Windowed Agumon-vs-Agumon-dummy encounter bootstrap is fully wired: two on-screen sprites, SP/talent seeded, PartySelected+TurnOrderSeeded fired, and test assertions confirmed green.

**Windowed Agumon-vs-Agumon-dummy encounter bootstrap is fully wired: two on-screen sprites, SP/talent seeded, PartySelected+TurnOrderSeeded fired, and test assertions confirmed green.**

## What Happened

All work for this task was already present in the codebase from the prior session (compacted before summary). Inspection confirmed:

1. `src/combat/encounter/bootstrap.rs` — `EncounterPreset::AgumonTrainingDummy` variant is defined (line 56), `Display` handles it (line 67), `bootstrap_encounter` early-returns for it (lines 77-79), and `bootstrap_agumon_training_dummy` clones the ally Agumon def, overrides `id` to `AGUMON_DUMMY_ID` (UnitId 9001), `team` to `Team::Enemy`, and `name` to "Agumon (Dummy)". `AGUMON_DUMMY_ID` const is exported.

2. `src/windowed/mod.rs` — `windowed_bootstrap_system` uses `EncounterPreset::AgumonTrainingDummy`, runs `apply_composition`, caps SP to 999, seeds `agumon::bouncing_fire` rank=1 on `TalentRanks`, and emits `PartySelected` + `TurnOrderSeeded` via `MessageWriter<CombatEvent>`.

3. `src/windowed/render.rs` — `spawn_unit_sprites` iterates `Query<(&Unit, &Team)>`, sets `flip_x = true` for enemies, places ally at x=-200 and enemy at x=+200, spawns one `AgumonSprite` per unit using the shared `AGUMON_STANCE_GRAPH_ID` entry node. `advance_agumon_presentation` iterates all `AgumonSprite` entities (unit-scoped, not singleton).

4. CLI binaries (`src/bin/combat_cli.rs`, `src/bin/combat_cli/config.rs`) — both compile cleanly with the new variant due to the exhaustive match in bootstrap.rs; no behavior change in CLI paths.

5. `tests/bootstrap_encounter/encounter_bootstrap_windowed.rs` — assertions for one-ally-one-enemy, distinct UnitIds, `AGUMON_DUMMY_ID`, `Team::Enemy` on dummy, and graceful failure on empty roster. Included in `tests/bootstrap_encounter.rs` harness.

## Verification

Three verification commands run:
- `cargo test --test bootstrap_encounter --features windowed`: 16 passed, 1 ignored (subprocess test), 0 failed. Both `encounter_bootstrap_windowed` cases passed.
- `cargo test --test windowed_only --features windowed`: 7 passed, 0 failed. All `windowed_preview_cache` and `phase_strip_readonly` tests green.
- `cargo build --features windowed`: compiled successfully in ~57s.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test bootstrap_encounter --features windowed` | 0 | pass — 16 passed, 1 ignored | 360ms |
| 2 | `cargo test --test windowed_only --features windowed` | 0 | pass — 7 passed | 180ms |
| 3 | `cargo build --features windowed` | 0 | pass — compiled successfully | 57170ms |

## Deviations

None. All implementation was already present; the session resumed after compaction and found everything complete.

## Known Issues

None.

## Files Created/Modified

- `src/combat/encounter/bootstrap.rs`
- `src/windowed/mod.rs`
- `src/windowed/render.rs`
- `tests/bootstrap_encounter/encounter_bootstrap_windowed.rs`
- `tests/bootstrap_encounter.rs`
