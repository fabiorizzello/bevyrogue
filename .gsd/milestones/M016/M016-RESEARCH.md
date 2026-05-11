# M016 — Research

**Date:** 2025-02-28

## Summary

M016 focuses on migrating the revised combat identity from shared mechanic primitives into deliberate per-Digimon Rust blueprint ownership, while preserving the M015 authority boundaries. The core objective is to expand the seed concept established by Patamon/Holy Support into the full roster, preventing unique mechanics from leaking into shared system branching.

The research confirms that the current architecture successfully bounds RON to declarative intent (`custom_signals`), leaving gameplay authority to the Rust kernels and blueprints. The migration strategy should tackle one high-risk primitive at a time, moving them into the `custom_signal` -> Rust blueprint -> kernel transition -> event/snapshot pipeline.

## Recommendation

Follow the prioritized candidate slices from the milestone strategy:
1. **Tentomon/Kabuterimon Battery loop:** A stateful mechanic that will prove the blueprint seam can handle complex internal state and resource accumulation without leaking into generic resolution.
2. **Dorumon/DORUgamon Predator loop:** A status-driven conditional damage logic that tests how blueprints interact with status effects and specific target states.
3. **Renamon/Kyubimon Precision loop:** Tests accuracy, evasion modifiers, and hit-chance manipulation via kernel transitions.
4. **Agumon/Gabumon Twin Core refinement:** Refines established mechanics into clean separated blueprints (`agumon.rs`, `gabumon.rs`).

For each, we must ensure:
- `assets/data/skills.ron` uses `custom_signals`.
- Logic is moved to a specific `src/combat/blueprints/<digimon>.rs` file.
- Action queries and real-binary tests (CLI) continue to pass.

## Implementation Landscape

### Key Files

- `src/combat/blueprints/mod.rs` — The registry for blueprints. Needs to expand to include `tentomon.rs`, `dorumon.rs`, `renamon.rs`, `agumon.rs`, `gabumon.rs`.
- `assets/data/skills.ron` — Needs updates to replace ad-hoc metadata fields with `custom_signals` for the migrated roster.
- `src/combat/turn_system.rs` & `src/combat/resolution.rs` — Must remain generic. Any character-specific branching found here during migration must be extracted to the respective blueprint.
- `src/combat/battery_loop.rs` (or similar shared mechanic modules) — Must expose state and generic transitions (e.g. `BatteryLoopTransition`) that blueprints can emit.

### Build Order

1. **Tentomon Blueprint (Battery Loop):** Proves stateful complex mechanics can be isolated.
2. **Dorumon Blueprint (Predator Loop):** Proves conditional status-driven logic isolation.
3. **Renamon Blueprint (Precision Loop):** Proves stat-modifier and roll manipulation isolation.
4. **Agumon/Gabumon Blueprints:** Refines standard damage/buff mechanics.

### Verification Approach

- **CLI Proof:** `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` must continue to pass, proving the shared query vocabulary and event surfaces remain intact.
- **Headless Tests:** `cargo test --no-fail-fast` must remain deterministic and green.
- **Validation Snapshots:** Ensure `ValidationSnapshot` structures still accurately reflect the state changes introduced by the new blueprints.

## Constraints

- **RON is Declarative Only:** Do not encode execution logic in `skills.ron`. Use `custom_signals` to tag intent.
- **Generic Kernel:** `turn_system.rs` and `resolution.rs` must not contain any Digimon-specific `match` arms or logic.

## Common Pitfalls

- **Authority Leakage:** Moving logic to a blueprint but leaving a specific check in `resolution.rs`. **How to avoid:** Ensure blueprints emit generic `CombatKernelTransition` values that the resolution system blindly applies.
- **Breaking CLI Legality:** Changing how a skill's legality is determined such that the CLI cannot query it generically. **How to avoid:** Respect the `skill_legality_contract.md`.

## Open Risks

- **Blueprint Seam Erosion:** As more complex mechanics are migrated, there may be a temptation to add just "one more field" to the generic state to support a specific Digimon. This must be strongly resisted in favor of generic mechanics and custom signals.