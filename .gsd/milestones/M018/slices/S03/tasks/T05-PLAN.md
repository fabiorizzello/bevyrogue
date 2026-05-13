---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T05: combat_cli bounce-chain scenario + determinism gate + final regression sweep

Add `run_bounce_chain_scenario()` to `src/bin/combat_cli.rs` mirroring `run_aoe_blast_scenario` (line 921). Build a deterministic 3-enemy mock encounter where HP values force a meaningful chain (e.g. primary slot-1 at HP 60, two others at 50 and 40 — first hop lands on user-chosen primary, then next_bounce_hop visits lowest HP% remaining). At hop 2, deal enough damage to KO the current target so the chain naturally recomputes on the remaining survivor. Emit one JSONL line per hop with fields `{event:"BounceHop", hop_index, source_id, target_id, target_slot, target_hp_pre, target_hp_post, ko, skill_id:"chain_bolt"}` — wrap the existing per-hop damage events with the hop counter at print-time only (no new CombatEvent variant). Add `Some("bounce-chain")` arm to the dispatcher around line 1050. Run determinism gate: invoke `cargo run --bin combat_cli -- --scenario bounce-chain` twice, capture stdout to two files, byte-diff must be empty. Then run final regression sweep: `cargo test` full suite (must be all green, S02 + M017 suites included), `cargo check --features windowed` clean. Document the determinism result in T05 verification evidence.

## Inputs

- ``src/bin/combat_cli.rs` — `run_aoe_blast_scenario` (line 921) as template; dispatcher line 1050`
- ``src/combat/resolution.rs` (T01-T03 outputs) — apply_damage_only and next_bounce_hop available`
- ``assets/data/skills.ron` (T04 output) — chain_bolt fixture`
- ``tests/target_shape_bounce_chain.rs` (T03 output) — integration tests must still pass`

## Expected Output

- ``src/bin/combat_cli.rs` — `run_bounce_chain_scenario()` fn and `Some("bounce-chain")` dispatcher arm`

## Verification

cargo build --bin combat_cli && bash -c 'cargo run --quiet --bin combat_cli -- --scenario bounce-chain > /tmp/bounce1.txt 2>&1 && cargo run --quiet --bin combat_cli -- --scenario bounce-chain > /tmp/bounce2.txt 2>&1 && diff -q /tmp/bounce1.txt /tmp/bounce2.txt' && cargo test && cargo check --features windowed
