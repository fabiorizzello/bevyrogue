---
estimated_steps: 9
estimated_files: 2
skills_used: []
---

# T05: Add Blast + AoE fixture skills and combat_cli --scenario aoe-blast with JSONL determinism gate

1. **Fixture skills (`assets/data/skills.ron`):** Add ONE new Blast skill and ONE new AllEnemies skill (or flip an existing Implemented Single skill if doing so is lower-churn). Both marked `implementation: Implemented`. Keep all 11 existing non-Single deferred skills untouched (no scope creep). Ensure Effect::Damage{target} matches targeting.shape so the consistency check at skills_ron.rs:291-330 passes.

2. **CLI scenario (`src/bin/combat_cli.rs`):** Extend the `--scenario` dispatcher (which S01 just extended with advance-delay-cap) with a new `aoe-blast` branch. Load a 3-enemy encounter, cast one Blast skill (slot-1 primary), then one AllEnemies skill. Print per-step:
   - Resolved target list (UnitId + slot_index)
   - Per-target damage applied
   - Final HP gauge per enemy
  Emit one JSONL line per OnDamageDealt event (use existing jsonl_logger output, no schema change).

3. **Determinism gate:** run `cargo run --bin combat_cli -- --scenario aoe-blast > /tmp/run1.jsonl` and `> /tmp/run2.jsonl`, then `diff /tmp/run1.jsonl /tmp/run2.jsonl` MUST be empty. Repeat 10× as final smoke.

4. **Full suite gate:** `cargo test` — 554 baseline + 3 new test binaries (slot_index_tiebreak, target_shape_blast_spillover, target_shape_aoe_all_order) all green, zero failures. `cargo check --features windowed` clean.

5. **Legacy sweep:** `rg -n 'UnimplementedTargetShape' src/` should still show the gate site(s) — gates were only widened, not removed. `rg -n 'TargetShape::Blast' src/` lists exactly the 4 sites (enum decl + 3 gates) plus the resolver helper.

## Inputs

- `assets/data/skills.ron`
- `src/bin/combat_cli.rs`
- `src/data/skills_ron.rs`
- `src/combat/jsonl_logger.rs`
- `src/combat/turn_system/pipeline.rs`

## Expected Output

- `assets/data/skills.ron`
- `src/bin/combat_cli.rs`

## Verification

cargo run --bin combat_cli -- --scenario aoe-blast > /tmp/aoe1.jsonl 2>&1 && cargo run --bin combat_cli -- --scenario aoe-blast > /tmp/aoe2.jsonl 2>&1 && diff /tmp/aoe1.jsonl /tmp/aoe2.jsonl && cargo test 2>&1 | tail -5
