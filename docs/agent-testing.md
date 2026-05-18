# Agent Testing Workflow

## Snapshot tests (`insta`)

`.snap` files under `tests/snapshots/` are committed contracts. Read the `source`/`expression` metadata, then the body after `---` — that body is what the test compares against.

```bash
cargo test                                                    # run with snapshots checked
INSTA_UPDATE=always cargo test --test follow_up_triggers agumon_break_follow_up_uses_real_pilot_config   # regenerate on intentional change
diff -u tests/snapshots/<name>.snap tests/snapshots/<name>.snap.new   # review pending
mv  tests/snapshots/<name>.snap.new tests/snapshots/<name>.snap       # accept
rm  tests/snapshots/<name>.snap.new                                   # reject
```

A `.snap` diff is a product diff. Never auto-accept: verify every changed line is intended behavior, not drift in logging, ordering, targeting, damage, or follow-up scheduling.

## Machine-readable output (`cargo-nextest`)

Config: `.config/nextest.toml`. The `agent` profile sets `fail-fast = false` — one panic = one failed case, the rest still run. Each case runs as an isolated process.

```bash
cargo install cargo-nextest --locked            # if missing (install outside the project)

mkdir -p target/nextest/agent
NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 \
  cargo nextest run --profile agent --message-format libtest-json-plus \
  > target/nextest/agent/libtest-json-plus.ndjson   # newline-delimited JSON

cargo nextest run --profile agent               # JUnit XML → target/nextest/agent/junit.xml
```

Keep `fail-fast = false`: parsers must see all outcomes, not just the first panic.

## Structured headless tracing

Headless entry points route Bevy's `LogPlugin` formatter through `BEVYROGUE_TRACE_FORMAT` (avoids a second global subscriber, which fails after Bevy's logger inits).

```bash
BEVYROGUE_TRACE_FORMAT=json cargo run --quiet --bin bevyrogue 2> target/headless-trace.jsonl
```

JSON output carries span data + explicit enter/close. Combat spans:

- `combat.resolution` — root action resolution and legality.
- `combat.apply` — legacy `step_app` paths.
- `combat.apply.intent_queue` — timeline-backed intent queue.
- `combat.follow_up.evaluate` — follow-up trigger evaluation per event.
- `combat.follow_up.resolve` — scheduled follow-up resolution.

Spans compile only under `debug_assertions`; release pays no overhead.

## Deterministic RNG (`bevy_rand`)

No unseeded RNG in the game path. `CombatRng` wraps `bevy_rand`'s `WyRand` (`CombatEntropy`); `seed_unit_rngs` forks it into a per-entity `UnitRng`. Each random decision is rolled by the entity that owns it from its own stream:

- Timeline accuracy + legacy `single_target` → `roll_pct_for_unit_in_world(world, unit, threshold)`. `roll_accuracy_in_world` is a thin self-documenting alias.
- Tentomon block reaction rolls from the **defending** Tentomon's own `CombatEntropy` stream (reacting defender owns the decision), same primitive as accuracy.

Fixed seed → identical replay (accuracy, block procs, follow-up scheduling), so snapshots stay stable.

Contract: `cargo test --test deterministic_rng_contract`

- `seeded_combat_rng_replays_same_roll_sequence` — pins the exact `WyRand` sequence for a fixed seed; drift = a behavior/library change.
- `seeded_combat_rng_forks_replayable_entity_streams` — forked streams replay from the root seed yet diverge from each other.
- `unit_rng_streams_are_seeded_from_bevy_rand_global_entropy` — `seed_unit_rngs` attaches `UnitRng` + `CombatRngSeed` + `CombatEntropy` to every spawned `Unit`.

On an intentional sequence shift, update the expected vector deliberately — treat as a snapshot diff, not auto-accept. Default seed `DEFAULT_COMBAT_RNG_SEED`; fixed-combat tests insert `CombatRngSeed` before the entropy plugin runs.
