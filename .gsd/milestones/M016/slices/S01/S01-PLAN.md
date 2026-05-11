# S01: Tentomon/Kabuterimon Battery Loop Blueprint

**Goal:** Migrate the Battery loop into a discrete per-Digimon blueprint, proving the M015 seam architecture scales to stateful mechanics.
**Demo:** Battery mechanics operate entirely through `custom_signals` and the Tentomon blueprint, with CLI proof passing.

## Must-Haves

- `TentomonCustomSignal` is defined in `skills_ron.rs`.\n- `assets/data/skills.ron` uses these signals for Tentomon and Kabuterimon skills.\n- `src/combat/blueprints/tentomon.rs` maps these signals to generic `BatteryLoop` kernel transitions.\n- `src/combat/blueprints/mod.rs` correctly routes `SkillCustomSignal::Tentomon` to the new blueprint.\n- A new integration test proves the seam works from action to kernel transition.

## Proof Level

- This slice proves: integration

## Integration Closure

Tentomon blueprint is hooked into the main action resolution via `transitions_for_action`, emitting generic BatteryLoop kernel transitions.

## Verification

- Battery loop kernel transitions will now be driven by custom signals, maintaining visibility in ValidationSnapshots and CLI event streams.

## Tasks

- [ ] **T01: Define Tentomon custom signals in RON schema and data** `est:15m`
  Define `TentomonCustomSignal` enum in `src/data/skills_ron.rs` with variants matching the battery loop capabilities (e.g. `BuildStaticCharge`, `BuildCircuitCharge`, `SpendCircuitCharge`). Expand `SkillCustomSignal` to include `Tentomon(TentomonCustomSignal)`. Then update `assets/data/skills.ron` to inject these signals into Tentomon and Kabuterimon's skills (e.g. `tentomon_basic`, `petit_thunder`, `mega_blaster`).
  - Files: `src/data/skills_ron.rs`, `assets/data/skills.ron`
  - Verify: cargo check && cargo test --no-run

- [ ] **T02: Implement Tentomon blueprint logic** `est:20m`
  Create `src/combat/blueprints/tentomon.rs`. Implement `transitions_for_signal` matching `TentomonCustomSignal` to `CombatKernelTransition::BatteryLoop(...)` wrapping the respective `BatteryLoopTransition`. Update `src/combat/blueprints/mod.rs` to include the new module and dispatch `SkillCustomSignal::Tentomon` to it.
  - Files: `src/combat/blueprints/tentomon.rs`, `src/combat/blueprints/mod.rs`
  - Verify: cargo check && cargo test --no-run

- [ ] **T03: Verify blueprint integration and CLI proof** `est:20m`
  Create `tests/tentomon_blueprint.rs` to verify that executing a skill with the `TentomonCustomSignal` correctly triggers the expected `BatteryLoopState` transitions through the kernel. Ensure no regressions occur in headless testing or the CLI proof.
  - Files: `tests/tentomon_blueprint.rs`
  - Verify: cargo test --test tentomon_blueprint && BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli

## Files Likely Touched

- src/data/skills_ron.rs
- assets/data/skills.ron
- src/combat/blueprints/tentomon.rs
- src/combat/blueprints/mod.rs
- tests/tentomon_blueprint.rs
