# S03: Renamon/Kyubimon Precision Loop Blueprint — Research

## Summary

This slice migrates the **Precision Loop** (internally `PrecisionMindGame`) mechanic into a dedicated Renamon blueprint. Currently, the precision mechanic exists as a shared kernel primitive (`src/combat/precision_mind_game.rs`) but lacks a formal blueprint seam for roster-specific ownership. Renamon and Kyubimon are the primary users of this loop, utilizing "Momentum" and "Mind-Game" windows to resolve skill payoffs.

The recommendation is to follow the established pattern from S01 (Tentomon) and S02 (Dorumon): update `assets/data/skills.ron` to emit `custom_signals` for Renamon/Kyubimon skills, implement a new `src/combat/blueprints/renamon.rs` to decode these signals into `PrecisionMindGameTransition` values, and verify the loop through headless runtime tests that observe the `ValidationSnapshot.precision` surface.

## Recommendation

Implement a single blueprint (`renamon.rs`) that handles both Renamon (Child) and Kyubimon (Adult) precision signals. While they use different aspects of the loop (Renamon focuses on Momentum windows, Kyubimon on Counterplay/Traps), they share the underlying state machine. Routing both through a unified Renamon-line blueprint maintains the "per-Digimon" (or per-line) ownership boundary established in M016.

## Implementation Landscape

### Key Files

- `src/combat/blueprints/renamon.rs` — **New**. Logic to map `custom_signals` (e.g., `open_momentum_window`, `commit_press`) to `PrecisionMindGameTransition`.
- `src/combat/blueprints/mod.rs` — Register the new renamon blueprint in the `BLUEPRINTS` constant.
- `assets/data/skills.ron` — Update Renamon/Kyubimon skills to replace or augment ad-hoc metadata with `custom_signals`.
- `src/combat/precision_mind_game.rs` — Core state machine and transitions (already exists, but may need minor adjustments if signal requirements deviate).
- `tests/renamon_precision_runtime.rs` — **New**. Headless integration test proving the full loop from RON signal to state mutation.

### Build Order

1. **Signal Registration**: Add "renamon" owner and basic signals to `src/combat/blueprints/renamon.rs` and register in `mod.rs`.
2. **Data Wiring**: Update `skills.ron` for Renamon basic/skill/ult to include the signals.
3. **Logic Implementation**: Implement the dispatch logic in `renamon.rs` to emit the correct kernel transitions.
4. **Runtime Proof**: Create `tests/renamon_precision_runtime.rs` to verify that executing a Renamon skill correctly advances the `PrecisionMindGameState` and is visible in `ValidationSnapshot`.

### Verification Approach

- `cargo test --test renamon_precision_runtime` — Main proof of runtime state mutation.
- `python3 scripts/verify_combat_authority_audit.py` — Confirm no character-specific branching was added to shared systems.
- `cargo test --test digimon_signal_registry` — Verify the new signals are correctly parsed and routed.

## Constraints

- **Generic Kernel Transitions**: The renamon blueprint MUST only emit `CombatKernelTransition::PrecisionMindGame(...)`. It must not mutate `PrecisionMindGameState` directly.
- **RON Declarative Boundary**: `skills.ron` should declare the intent (e.g., "open window") but the blueprint decides the specifics of the transition based on the action context.

## Common Pitfalls

- **State Desync**: Ensure `PrecisionMindGameState` is reset or advanced correctly at the end of a resolution. The current implementation uses a `Resolved` phase; ensure skills don't get "stuck" in a window if they fail.
- **Signal Ownership**: All precision-related signals for Renamon and Kyubimon must use `owner: "renamon"` in RON to ensure correct routing.
