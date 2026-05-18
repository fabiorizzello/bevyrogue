# Agent Testing Workflow

## Snapshot tests (`insta`)

Snapshot files are committed text artifacts under `tests/snapshots/`. Each `.snap` file records the expected observable output for a regression test. Read the `source` and `expression` metadata first, then inspect the body after the `---` separator; that body is the contract the test compares against.

Run the normal suite with snapshots checked:

```bash
cargo test
```

Regenerate snapshots non-interactively when an intentional behavior change updates the contract:

```bash
INSTA_UPDATE=always cargo test --test follow_up_triggers agumon_break_follow_up_uses_real_pilot_config
```

Review pending snapshots without interactive tooling:

```bash
# Compare generated pending output against the committed contract.
diff -u tests/snapshots/<name>.snap tests/snapshots/<name>.snap.new

# Accept after review by replacing the committed snapshot.
mv tests/snapshots/<name>.snap.new tests/snapshots/<name>.snap

# Reject by deleting the pending snapshot.
rm tests/snapshots/<name>.snap.new
```

Do not accept a snapshot just because the test changed. The `.snap` diff is a product diff: verify that every changed line is an intended behavior change, not a drift in logging, ordering, targeting, damage, or follow-up scheduling.

## Machine-readable test output (`cargo-nextest`)

The repository keeps nextest configuration in `.config/nextest.toml`. The `agent` profile sets `fail-fast = false`, so a panic in one test is reported as one failed case while the remaining discovered tests still run.

Install the runner outside the project if the command is missing:

```bash
cargo install cargo-nextest --locked
```

Produce newline-delimited libtest-style JSON for an agent parser:

```bash
mkdir -p target/nextest/agent
NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 \
  cargo nextest run --profile agent --message-format libtest-json-plus \
  > target/nextest/agent/libtest-json-plus.ndjson
```

Produce JUnit XML using the committed profile:

```bash
cargo nextest run --profile agent
# writes target/nextest/agent/junit.xml
```

Nextest runs each test case as an isolated process by default. Keep `fail-fast = false` in the agent profile: parsers should see all pass/fail/ignored outcomes from the run, not just the first panic.

## Structured headless tracing

Headless entry points use Bevy's `LogPlugin`, but route its formatter through `BEVYROGUE_TRACE_FORMAT` for agent-readable output. This avoids installing a second global tracing subscriber, which would fail after Bevy's logger is initialized.

Emit JSON logs with span metadata:

```bash
BEVYROGUE_TRACE_FORMAT=json cargo run --quiet --bin bevyrogue 2> target/headless-trace.jsonl
```

The JSON formatter includes current span data and explicit span enter/close events. Combat spans currently cover:

- `combat.resolution` — root action resolution and legality checks.
- `combat.apply` — legacy `step_app` application paths.
- `combat.apply.intent_queue` — timeline-backed intent queue application.
- `combat.follow_up.evaluate` — follow-up trigger evaluation per combat event.
- `combat.follow_up.resolve` — scheduled follow-up action resolution.

The span instrumentation is compiled only for debug builds (`debug_assertions`) so release builds do not pay runtime overhead for agent diagnostics.
