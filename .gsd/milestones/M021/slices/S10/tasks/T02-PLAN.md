---
estimated_steps: 4
estimated_files: 5
skills_used: []
---

# T02: Move Renamon precision runtime ownership behind the blueprint envelope

Skills used: bevy, rust-best-practices, verify-before-complete.

Why: Renamon still dispatches raw `PrecisionMindGame` kernel transitions and the runtime is still registered from shared combat code, so the kernel is not yet owner-clean.

Do: refactor `src/combat/blueprints/renamon.rs` to emit Blueprint owner envelopes for the momentum/commit/reveal/resolve signals, fold the precision runtime state/hook/applier ownership into the Renamon blueprint module, and register that ownership through a Renamon plugin so `register_combat_kernel_runtime()` keeps existing call sites working without shared Renamon-specific state setup; preserve foreign-owner/malformed-envelope no-op behavior and current momentum-window ordering guarantees; update Renamon dispatch/runtime tests to prove the same behavior through the blueprint path.

Done when: Renamon custom signals no longer use the shared precision kernel variant directly, kernel registration no longer owns Renamon runtime state, and the Renamon dispatch/timeline/runtime tests pass through blueprint-owned decoding.

## Inputs

- `src/combat/blueprints/renamon.rs`
- `src/combat/kernel.rs`
- `src/combat/precision_mind_game.rs`
- `tests/digimon_signal_registry.rs`
- `tests/compiled_timeline_tohakken.rs`
- `tests/renamon_precision_runtime.rs`
- `assets/data/skills.ron`

## Expected Output

- `src/combat/blueprints/renamon.rs`
- `src/combat/kernel.rs`
- `tests/digimon_signal_registry.rs`
- `tests/compiled_timeline_tohakken.rs`
- `tests/renamon_precision_runtime.rs`

## Verification

cargo test --test digimon_signal_registry
cargo test --test compiled_timeline_tohakken
cargo test --test renamon_precision_runtime

## Observability Impact

Renamon precision diagnostics move behind the blueprint-owned runtime resource, keeping failure inspection owner-scoped and removing shared registration ambiguity.
