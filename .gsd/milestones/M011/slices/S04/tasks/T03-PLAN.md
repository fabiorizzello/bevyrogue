# T03: Combat State Display

Implement a system that prints a formatted "Dashboard" (HP/SP/Energy/Toughness) for all units whenever the turn order advances or an action completes.

**Must-haves:**
- Clear text representation of unit vitals.
- Display of current turn order.

## Inputs

- ``src/bin/combat_cli.rs``

## Expected Output

- ``src/bin/combat_cli.rs``

## Verification

`cargo run --bin combat_cli` prints unit stats clearly.

## Observability Impact

Provides real-time visibility into the exact mathematical state of all active units via terminal output.
