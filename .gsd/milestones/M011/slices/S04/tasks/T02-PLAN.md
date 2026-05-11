# T02: Integrate Headless Engine & Combat Event Logging

Register the core combat systems and `DataPlugin` inside the `combat_cli` Bevy app. Create a system that prints `CombatEvent` logs to stdout as they occur.

**Must-haves:**
- Headless systems are registered.
- Events from the bus are printed to the terminal.

## Inputs

- ``src/bin/combat_cli.rs``

## Expected Output

- ``src/bin/combat_cli.rs``

## Verification

`cargo run --bin combat_cli` shows initialization logs.

## Observability Impact

Streams events to the terminal for manual inspection.
