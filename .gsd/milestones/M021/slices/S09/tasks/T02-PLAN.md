---
estimated_steps: 13
estimated_files: 5
skills_used: []
---

# T02: Move Tentomon Battery Loop onto Blueprint owner transitions

Skills: `bevy`, `rust-best-practices`, `tdd`.

Why: Tentomon still emits kernel-local `CombatKernelTransition::BatteryLoop` writes even though its deterministic block-reaction behavior is already correct. This task migrates only the transport envelope so Battery Loop stays deterministic while its raw event path matches Twin Core and Dorumon.

Do:
1. Rewrite Tentomon custom-signal dispatch to emit Blueprint transitions owned by `tentomon` with stable signal names and amount payloads.
2. Update `apply_battery_loop_transitions_system` and the wrapped-cycle hook to decode/emit Tentomon-owned Blueprint transitions instead of `CombatKernelTransition::BatteryLoop`.
3. Preserve the passive mitigation path and `BlockReactionTriggered` surface exactly as-is; only adjust imports/wiring needed for the new transition envelope.
4. Refresh Tentomon/Battery Loop tests so they assert the Blueprint raw transition shape, then prove state mutation, charge caps, cycle reset, and deterministic passive behavior still hold.

Done when: Tentomon no longer emits `CombatKernelTransition::BatteryLoop`, Battery Loop runtime state still converges exactly once per event, and the passive deterministic coverage remains green.

Failure modes / negative checks:
- Non-Tentomon Blueprint transitions must not mutate Battery Loop state.
- Zero/underflow/cap behaviors must still report the same typed blocked reasons.
- Passive block reaction must not regress duplicate-cast guarding or shared mitigation diagnostics.

Observability impact: Battery Loop raw kernel writes become Blueprint owner events while `BatteryLoopResolved` and `BlockReactionTriggered` stay the canonical typed diagnostics.

## Inputs

- `src/combat/blueprints/tentomon.rs`
- `src/combat/battery_loop.rs`
- `tests/tentomon_blueprint.rs`
- `tests/battery_loop_kernel.rs`
- `tests/passive_reactive_canon.rs`

## Expected Output

- `src/combat/blueprints/tentomon.rs`
- `src/combat/battery_loop.rs`
- `tests/tentomon_blueprint.rs`
- `tests/battery_loop_kernel.rs`
- `tests/passive_reactive_canon.rs`

## Verification

cargo test --test tentomon_blueprint
cargo test --test battery_loop_kernel
cargo test --test passive_reactive_canon

## Observability Impact

Preserves Tentomon’s shared passive mitigation diagnostics while changing the raw event envelope to the Blueprint owner path.
