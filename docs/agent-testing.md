# Agent Testing Workflow

## Snapshot tests (`insta`)
`.snap` in `tests/snapshots/` are contracts. Diff is a product change; verify behavior, don't auto-accept.

```bash
cargo test                                                    # checked
INSTA_UPDATE=always cargo test --test follow_up_triggers <case> # update
diff -u tests/snapshots/<name>.snap tests/snapshots/<name>.snap.new # review
mv tests/snapshots/<name>.snap.new tests/snapshots/<name>.snap # accept
```

## Machine-readable output (`cargo-nextest`)
`.config/nextest.toml` profile `agent`: `fail-fast = false` (report all).

```bash
cargo nextest run --profile agent --message-format libtest-json-plus > out.ndjson
cargo nextest run --profile agent # JUnit XML -> target/nextest/agent/junit.xml
```

## Structured headless tracing
`BEVYROGUE_TRACE_FORMAT=json` routes `LogPlugin` to stderr. Spans (debug only):

- `combat.resolution`: Action legality.
- `combat.apply`: Legacy paths.
- `combat.apply.intent_queue`: Timeline queue.
- `combat.follow_up.evaluate`: Trigger evaluation.
- `combat.follow_up.resolve`: Resolution.

```bash
BEVYROGUE_TRACE_FORMAT=json cargo run --quiet --bin bevyrogue 2> trace.jsonl
```

## Deterministic RNG (`bevy_rand`)
No unseeded RNG. `CombatRng` (`WyRand`) forks into `UnitRng`. Decisions owned by entities:

- Accuracy: `roll_pct_for_unit_in_world(world, unit, threshold)`.
- Reactions: Defending entity rolls from own stream.

Contract: `cargo test --test deterministic_rng_contract`
- `seeded_combat_rng_replays_same_roll_sequence`: Pins `WyRand` vector.
- `seeded_combat_rng_forks_replayable_entity_streams`: Independent entity streams.
- `unit_rng_streams_are_seeded_from_bevy_rand_global_entropy`: World-backed seeding.

Seed: `DEFAULT_COMBAT_RNG_SEED`. Treat sequence drift as snapshot change.
