---
id: T05
parent: S02
milestone: M018
key_files:
  - assets/data/skills.ron
  - src/bin/combat_cli.rs
  - src/data/skills_ron.rs
key_decisions:
  - aoe-blast scenario is pure-Rust (no Bevy app) — same pattern as advance-delay-cap; calls resolve_targets() directly on a hand-built TargetableSnapshot for determinism and speed
  - Blast primary = slot-1 (GobB) to exercise ±1 coverage hitting all three slots; validates edge-slot behavior covered by unit tests is also observable via CLI
  - parse_canonical_skills_ron count bumped from 72→74 to track the two new fixture skills
duration: 
verification_result: passed
completed_at: 2026-05-13T20:05:54.446Z
blocker_discovered: false
---

# T05: Added nova_burst (Blast) and dark_flood (AllEnemies) fixture skills to skills.ron; added pure-Rust aoe-blast scenario to combat_cli with JSONL output, 10x determinism verified, all 162+ tests green

**Added nova_burst (Blast) and dark_flood (AllEnemies) fixture skills to skills.ron; added pure-Rust aoe-blast scenario to combat_cli with JSONL output, 10x determinism verified, all 162+ tests green**

## What Happened

Step 1 — Fixture skills: Added two new `Implemented` skills to `assets/data/skills.ron`: `nova_burst` (Blast, Fire, sp_cost:3, damage:20) and `dark_flood` (AllEnemies, Dark, sp_cost:4, damage:15). Effect targets match targeting shape (Blast/AllEnemies respectively), satisfying the skills_ron.rs consistency check. Updated the `parse_canonical_skills_ron` count assertion from 72 → 74.

Step 2 — CLI scenario: Added `run_aoe_blast_scenario()` as a pure-Rust function (no Bevy app) in `src/bin/combat_cli.rs`. It builds a deterministic TargetableSnapshot with 3 enemies at slots 0/1/2, calls `resolve_targets(&TargetShape::Blast, UnitId(2), &snapshot)` (slot-1 primary → hits all 3 slots ±1), prints resolved target list sorted by slot_index, per-target damage, and one JSONL `OnDamageDealt` line per hit. Then casts AllEnemies, same reporting. Final HP gauge printed. Wired the branch into the `--scenario` dispatcher alongside `advance-delay-cap`.

Step 3 — Imports: Added `use bevyrogue::combat::resolution::{TargetEntry, TargetableSnapshot, resolve_targets}` and `use bevyrogue::data::skills_ron::TargetShape` (removed accidental duplicate `Team` import on first compile).

Step 4 — Verification: `cargo build` clean, `cargo check --features windowed` clean, scenario stdout identical across 10 consecutive runs (diff empty), all 162+ lib+integration tests pass, legacy sweep confirms UnimplementedTargetShape gates intact and TargetShape::Blast appears at exactly the expected sites.

## Verification

1. `cargo run --bin combat_cli -- --scenario aoe-blast` prints correct Blast (3 targets slot 0/1/2) and AllEnemies (3 targets) target lists with JSONL lines per hit and final HP gauge. 2. 10x determinism: all 10 stdout captures are identical (diff empty). 3. `cargo test` → all results ok, 0 failures. 4. `cargo check --features windowed` → clean. 5. `rg -n 'UnimplementedTargetShape' src/` → gate sites still present. 6. `rg -n 'TargetShape::Blast' src/` → enum + 3 gates + resolver + test + cli.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --bin combat_cli` | 0 | pass | 1590ms |
| 2 | `cargo run --bin combat_cli -- --scenario aoe-blast 2>/dev/null > /tmp/aoe1.jsonl && cargo run --bin combat_cli -- --scenario aoe-blast 2>/dev/null > /tmp/aoe2.jsonl && diff /tmp/aoe1.jsonl /tmp/aoe2.jsonl` | 0 | pass — diff empty | 4000ms |
| 3 | `10x determinism loop` | 0 | pass — all 10 runs identical | 20000ms |
| 4 | `cargo test 2>&1 | grep 'test result'` | 0 | pass — 0 failures across all test suites | 12000ms |
| 5 | `cargo check --features windowed` | 0 | pass | 3880ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `assets/data/skills.ron`
- `src/bin/combat_cli.rs`
- `src/data/skills_ron.rs`
