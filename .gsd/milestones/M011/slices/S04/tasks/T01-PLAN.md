# T01: Setup combat_cli binary and interactive prompts

Add `inquire` or `dialoguer` as dependencies in `Cargo.toml`. Create the entry point `src/bin/combat_cli.rs` with a Bevy `ScheduleRunnerPlugin` configured to run once or at a low rate. Implement a basic "Press Enter to start" prompt.

**Must-haves:**
- Dependencies added to `Cargo.toml`.
- Binary compiles and runs.

## Inputs

- ``Cargo.toml``

## Expected Output

- ``src/bin/combat_cli.rs``
- ``Cargo.toml``

## Verification

`cargo check --bin combat_cli` compiles.

## Observability Impact

None
