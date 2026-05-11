# T04: Interactive Action Selection via Inquire

When `CombatPhase::WaitingAction` is active, use `inquire::Select` to present available actions (Basic, Skill, Ultimate) and targets to the user. Emit an `ActionIntent` based on the selection.

**Must-haves:**
- User can choose action and target using arrow keys/enter.
- Selection is correctly translated to ECS components.

## Inputs

- ``src/bin/combat_cli.rs``

## Expected Output

- ``src/bin/combat_cli.rs``

## Verification

Manual run allows completing a full turn.

## Observability Impact

None
