---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T05: combat_cli scenarios for each fixture + determinism gate × N + final regression sweep

Add `run_bounce_chain_scenario()` and `run_arc_bolt_scenario()` to src/bin/combat_cli.rs (mirroring run_aoe_blast_scenario structure). Each spins up a deterministic 3-enemy mock encounter with HP values engineered to drive a meaningful chain through the fixture's selector + curve combo (chain_bolt: HP gradient surfacing LowestHpPct progression; arc_bolt: slot-walking with falloff visible in per-hop damage). Emit one JSONL line per hop with `{event:"BounceHop", hop_index, source_id, target_id, target_slot, target_hp_pre, target_hp_post, damage_dealt, selector, repeat_policy, ko, skill_id}` — wrap existing OnDamageDealt events at print time, no engine schema churn. Add `Some("bounce-chain")` and `Some("arc-bolt")` arms to the dispatcher. Run determinism gate per scenario: invoke twice, capture stdout, byte-diff must be empty. Final sweep: `cargo test` full suite green (S02 + M017 suites included), `cargo check --features windowed` clean. Document determinism diff results in verification evidence.

## Inputs

- `chain_bolt + arc_bolt fixtures from T04`
- `generic kernel hop loop from T03`

## Expected Output

- `run_bounce_chain_scenario + run_arc_bolt_scenario in combat_cli.rs`
- `byte-for-byte deterministic JSONL per scenario across 2 runs`
- `full cargo test + cargo check --features windowed green`

## Verification

cargo build --bin combat_cli && bash -c 'for s in bounce-chain arc-bolt; do cargo run --quiet --bin combat_cli -- --scenario $s > /tmp/${s}1.txt 2>&1 && cargo run --quiet --bin combat_cli -- --scenario $s > /tmp/${s}2.txt 2>&1 && diff -q /tmp/${s}1.txt /tmp/${s}2.txt; done' && cargo test && cargo check --features windowed
