# S04 — Research

**Date:** 2026-04-27

## Summary

The goal of S04 is to build an interactive headless CLI (`combat_cli`) to playtest the combat engine and validate numerical rebalances. The user explicitly requested an evaluation of whether to use `ratatui` to build a full Terminal User Interface (TUI) versus relying on a simpler prompt loop like `inquire` or `dialoguer`. 

While `ratatui` is an excellent fit for rendering complex game states (keeping HP, SP, Energy, Toughness, and buffs in fixed, non-scrolling panes while streaming the `CombatEvent` log in another pane), it introduces a heavy architectural burden. A full TUI requires managing terminal raw mode, manual UI layouts, and most importantly, asynchronous or multi-threaded event bridging, because Bevy's `ScheduleRunnerPlugin` and `ratatui`'s `crossterm` event loop both want to own the main thread.

## Recommendation

**I recommend using a plain terminal loop (`inquire` or `dialoguer` + `println!`) over `ratatui`.** `ratatui` is **excessive** for a playtest harness in M011.

While a scrolling terminal output is less elegant than a fixed-pane `ratatui` dashboard, the engineering cost of a TUI is too high for this slice. With `inquire`, we can simply invoke blocking prompts (e.g., "Select Skill", "Select Target") directly inside a Bevy system when the combat state requires input. Blocking the thread pauses the Bevy ECS tick, which is perfectly safe and deterministic for our turn-based, headless simulation. If we used `ratatui`, we would be forced to run the UI on a separate thread and shuttle inputs/state to Bevy via crossbeam channels, distracting from the core goal of validating combat balance. 

## Implementation Landscape

### Key Files

- `src/bin/combat_cli.rs` — The new binary entry point. It should construct a Bevy app similar to `src/headless.rs` but register interactive input systems instead of the hardcoded `CombatScript`.
- `Cargo.toml` — Needs to declare the new binary and add the chosen CLI dependency (`inquire` or `dialoguer`).
- `src/headless.rs` — May need light refactoring to expose common setup logic so `combat_cli.rs` can reuse the headless plugins without the 120-tick automatic exit and hardcoded script.

### Build Order

1. Add the new CLI dependency (`inquire`) to `Cargo.toml`.
2. Create `src/bin/combat_cli.rs` and replicate the basic headless Bevy setup, ensuring it runs infinitely (removing the `HEADLESS_TICK_BUDGET` exit condition).
3. Implement a blocking Bevy system that runs when it's the player's turn, using `inquire` to select an action and target, and then writes an `ActionIntent` to the bus.
4. Implement a state-dump system that prints a compact summary of all units (HP/SP/Energy/Toughness) and recent `CombatEvent`s to the terminal after each action resolves.

### Verification Approach

Run `cargo run --bin combat_cli`. The console should prompt for an initial roster selection (4 allies), print the initial encounter state, and successfully block the game loop to ask for the first unit's action. Choosing an action should advance the turn, print the combat log and new state, and prompt for the next turn.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Interactive prompts (select target/skill) | `inquire` or `dialoguer` | Handles arrow keys, validation, and terminal states without raw-mode boilerplate. |
| Complex TUI rendering | `ratatui` + `crossterm` | Evaluated but rejected as excessive. If we pivot to a full TUI later, this is the standard Rust ecosystem choice, but it requires manual layout and event loop management. |

## Constraints

- **Concurrency & ECS Blocking**: Bevy's `App::run()` owns the main thread. A plain CLI using `inquire` inside a Bevy system will block the thread and halt the ECS tick. For our turn-based headless game, this is acceptable and preserves determinism. If we used `ratatui`, we would be forced to run the terminal event loop on a separate thread and send commands to Bevy via an MPSC channel, significantly increasing complexity.
- **Dependencies**: Both approaches require adding new crates to `Cargo.toml`. Since `combat_cli` is a development tool, these should ideally be optional or dev dependencies, but Cargo doesn't cleanly support dependencies specifically for `src/bin/` targets without creating a workspace crate. Adding it as a standard dependency is the simplest path.

## Common Pitfalls

- **Terminal State Corruption**: Crashing while the terminal is in raw mode (common with custom TUI loops) leaves the user's shell unusable. Libraries like `inquire` handle transient raw mode per-prompt, which is safer for a quick playtest script.
- **Log Spam**: With a plain CLI, dumping the entire `CombatState` and all `CombatEvent`s every turn will cause aggressive terminal scrolling. We must format the post-action state compactly (e.g., a single-line summary per unit) and filter events to make it readable, otherwise the CLI becomes useless for balance validation.