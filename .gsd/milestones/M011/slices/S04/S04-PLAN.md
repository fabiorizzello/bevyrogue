# S04: Headless interactive CLI (combat playtest harness)

**Goal:** Build an interactive text-based CLI (`combat_cli`) using `inquire` or `dialoguer` to serve as a playtest harness for the combat engine, fulfilling R082.
**Demo:** cargo run --bin combat_cli permette di selezionare 4 alleati dal roster, di scegliere azioni interattivamente, di vedere lo stato post-azione e gli eventi del bus

## Must-Haves

- `cargo test --bin combat_cli` passes.
- Manual execution via `cargo run --bin combat_cli` launches successfully, allows 4-unit roster selection using `inquire`, and permits interactive turn-by-turn play without crashing.

## Proof Level

- This slice proves: Operational / UAT

## Integration Closure

Connects Bevy ECS directly to standard terminal input/output via synchronous prompts in a dedicated thread or standard execution flow, providing a human-playable loop for the headless engine.

## Verification

- The CLI acts as a deep observability surface for playtesting, logging internal combat state (SP, Toughness, Energy, Team composition) and streaming the CombatEvent bus in real-time to stdout.

## Tasks

- [x] **T01: Setup combat_cli binary and inquire input loop** `est:30m`
  Add `inquire` as a dependency in `Cargo.toml`. Create the entry point `src/bin/combat_cli.rs` with a Bevy `ScheduleRunnerPlugin` configured at 60Hz. Set up a basic input loop that allows exiting the application gracefully (e.g. typing 'quit'). Ensure the application doesn't block the Bevy executor while waiting for input.
  - Files: `Cargo.toml`, `src/bin/combat_cli.rs`
  - Verify: `cargo check --bin combat_cli` compiles, and running it allows graceful exit.

- [x] **T02: Integrate Headless Engine & Combat Event Logging** `est:45m`
  Register the core combat systems and `DataPlugin` inside the `combat_cli` Bevy app. Auto-bootstrap a fixed party temporarily to populate state. Set up an event listener that prints incoming `CombatEvent` messages to stdout so the player can see what is happening in the combat engine.
  - Files: `src/bin/combat_cli.rs`, `src/headless.rs`
  - Verify: `cargo run --bin combat_cli` successfully prints combat initialization events to stdout.

- [x] **T03: Combat Dashboard Text Rendering** `est:45m`
  Create a system that queries `Unit`, `Team`, `Toughness`, `SpPool`, and `TurnOrder` from the ECS and prints a formatted text summary (dashboard) of the current combat state before prompting the player for an action. Include HP/MaxHP, SP pool, Energy/Ultimate charge, Toughness, and Turn Queue.
  - Files: `src/bin/combat_cli.rs`
  - Verify: `cargo check --bin combat_cli` passes. Manual run verifies stats are printed clearly to the terminal.

- [x] **T04: Interactive Action Selection Menu with inquire** `est:1h`
  Implement an interactive prompt using `inquire::Select` to present the player with available actions (Basic, Skill, Ultimate) and targets when `CombatPhase::WaitingAction` is active. Emit an `ActionIntent` upon selection to drive the combat engine forward. Add a unit test verifying the menu logic.
  - Files: `src/bin/combat_cli.rs`
  - Verify: `cargo test --bin combat_cli` passes. Manual test allows choosing actions.

- [x] **T05: Interactive Roster Selection Phase with inquire** `est:45m`
  Implement a pre-combat drafting phase (`Phase::Drafting`). Query `UnitRoster`, present a list of all available characters using `inquire::MultiSelect` (or sequential `Select`), and allow the user to select 4 allies interactively. Build a `SelectionRequest` and transition to the combat dashboard, removing the fixed-party bootstrap.
  - Files: `src/bin/combat_cli.rs`
  - Verify: `cargo run --bin combat_cli` enforces 4-ally selection and enters combat successfully.

## Files Likely Touched

- Cargo.toml
- src/bin/combat_cli.rs
- src/headless.rs
