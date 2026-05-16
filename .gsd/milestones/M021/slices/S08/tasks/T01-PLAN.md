---
estimated_steps: 16
estimated_files: 13
skills_used: []
---

# T01: Extract TwinCore to blueprints/twin_core mini-plugin and remove kernel coupling

**Why:** The M021 success criterion requires `rg TwinCore src/combat/ --glob '!blueprints/**'` → 0 lines. Currently `TwinCoreSignal`, `TwinCoreTransition`, and the `CombatKernelTransition::TwinCore(...)` variant live in `kernel.rs`, and `TwinCoreHook`, `TwinCoreState`, `apply_twin_core_transitions_system` live in `blueprints/agumon/identity.rs`. All must consolidate into a new `blueprints/twin_core/` module. The `CombatKernelTransition::TwinCore(TwinCoreTransition)` variant is replaced by `CombatKernelTransition::Blueprint { owner: "twin_core", name: "<signal>", payload: Amount(amount) }` per M021 CONTEXT M5.

**Do:**
1. Create `src/combat/blueprints/twin_core/mod.rs` — move `TwinCoreDesignTag`, tag constants, helper fns (`twin_core_design_tag`, `classify_twin_core_tag`, `twin_core_added_tag_transition`, `twin_core_design_tag_name`), `TwinCoreState`, `TwinCoreHook`, `apply_twin_core_transitions_system`, and the private `apply_twin_core_transition` fn from `agumon/identity.rs`.
2. Move `TwinCoreSignal` enum and `TwinCoreTransition` struct + constructors from `kernel.rs` into `twin_core/mod.rs`.
3. Remove `CombatKernelTransition::TwinCore(TwinCoreTransition)` variant from the enum in `kernel.rs`. All match arms in `TwinCoreHook::on_transition` that previously pushed `CombatKernelTransition::TwinCore(...)` now push `CombatKernelTransition::Blueprint { owner: "twin_core".into(), name: signal_name.into(), payload: SignalPayload::Amount(amount) }`.
4. Update `apply_twin_core_transitions_system` to match on `CombatKernelTransition::Blueprint { owner, name, .. }` where `owner == "twin_core"` and decode the signal from `name`+`payload`.
5. Create `pub struct TwinCorePlugin;` in `twin_core/mod.rs` that owns `init_resource::<TwinCoreState>()`, `add_systems(Update, apply_twin_core_transitions_system)`, and `registry.register(TwinCoreHook)`.
6. In `kernel.rs` `register_combat_kernel_runtime`: replace `crate::combat::blueprints::agumon::AgumonPlugin` with `crate::combat::blueprints::twin_core::TwinCorePlugin`.
7. Add `pub mod twin_core;` to `src/combat/blueprints/mod.rs`.
8. Update `agumon/mod.rs` to re-export from `super::twin_core` instead of `identity` for the TwinCore public API items. Shrink `agumon/identity.rs` to only Agumon-specific identity if any remains (likely remove the file entirely if all content moved).
9. Update `src/combat/observability.rs` — replace `TwinCoreSignal`/`TwinCoreTransition` imports from `kernel` with imports from `blueprints::twin_core`. Update snapshot formatting to handle the Blueprint variant for twin_core.
10. Update `tests/twin_core_integration.rs` and `tests/twin_core_mechanics.rs` imports: `blueprints::twin_core::` instead of `blueprints::agumon::`.
11. Update `tests/event_stream.rs` match on `CombatKernelTransition::TwinCore(_)` → match on Blueprint variant.
12. Update `tests/validation_snapshot.rs` references.
13. Update `tests/status_observability_canon.rs`, `tests/holy_support_mechanics.rs`, `tests/holy_support_affordance.rs` — change `blueprints::agumon::TwinCoreState` to `blueprints::twin_core::TwinCoreState`.

**Done when:** `cargo test` passes; `rg "TwinCore" src/combat/ --glob '!blueprints/**'` → 0 lines; `rg "CombatKernelTransition::TwinCore" src/` → 0 lines.

## Inputs

- `src/combat/blueprints/agumon/identity.rs`
- `src/combat/kernel.rs`
- `src/combat/observability.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/mod.rs`
- `tests/twin_core_integration.rs`
- `tests/twin_core_mechanics.rs`
- `tests/event_stream.rs`
- `tests/validation_snapshot.rs`
- `tests/status_observability_canon.rs`
- `tests/holy_support_mechanics.rs`
- `tests/holy_support_affordance.rs`

## Expected Output

- `src/combat/blueprints/twin_core/mod.rs`

## Verification

cargo test && rg "TwinCore" src/combat/ --glob '!blueprints/**'
