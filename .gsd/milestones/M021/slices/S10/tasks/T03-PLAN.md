---
estimated_steps: 4
estimated_files: 9
skills_used: []
---

# T03: Collapsed shared combat event/kernel surfaces to generic blueprint seams and updated owner-module routing, but Dorumon runtime verification still needs one follow-up pass

Skills used: bevy, rust-best-practices, verify-before-complete.

Why: even after Patamon/Renamon transport migration, `src/combat/kernel.rs`, `src/combat/events.rs`, `src/combat/mod.rs`, and the standalone Tentomon/Dorumon/Renamon runtime files still carry digimon-named transition variants and resolved-event seams, so the roadmap grep gate cannot pass.

Do: apply D003 by collapsing shared combat surfaces to generic/shared names only; remove digimon-specific raw transition variants from `CombatKernelTransition`; retire digimon-named resolved variants from `CombatEventKind`; move or inline remaining Tentomon/Dorumon/Renamon runtime decoding so owner modules are the only places that understand their transition names; update shared re-exports/imports and any direct runtime resource lookups (`api/applier`, kernel registration, module exports) so shared code depends on blueprint ownership boundaries rather than mechanic names; update regression tests for Tentomon/Dorumon/shared event-stream behavior to assert the generic shared seam plus owner-owned state inspection.

Done when: shared combat modules outside `blueprints/**` no longer define digimon-specific runtime/event variants, Tentomon/Dorumon/Renamon behavior still resolves deterministically, and the shared event/regression tests pass.

## Inputs

- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `src/combat/mod.rs`
- `src/combat/api/applier.rs`
- `src/combat/battery_loop.rs`
- `src/combat/precision_mind_game.rs`
- `src/combat/blueprints/tentomon.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `tests/battery_loop_kernel.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/event_stream.rs`

## Expected Output

- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `src/combat/mod.rs`
- `src/combat/api/applier.rs`
- `src/combat/blueprints/tentomon.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `tests/battery_loop_kernel.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/event_stream.rs`

## Verification

cargo test --test battery_loop_kernel
cargo test --test dorumon_predator_runtime
cargo test --test event_stream

## Observability Impact

Shared event-bus diagnostics become generic; future failure analysis relies on blueprint-owned resources/snapshots rather than digimon-named event variants.
