# S01: S01: Kernel framework primitives + CombatPlugin extract — UAT

**Milestone:** M021
**Written:** 2026-05-15T07:23:17.676Z

## UAT: S01 — Kernel framework primitives + CombatPlugin extract

**UAT Type:** Static analysis + Integration test suite (headless, no DISPLAY required)

### Preconditions
- Rust toolchain per `rust-toolchain.toml` installed
- `cranelift` dev profile active (default)
- Working directory: `/home/fabio/dev/bevyrogue`
- No DISPLAY required (all tests headless)

### Steps and Expected Outcomes

1. **cargo check (headless)**
   ```
   cargo check
   ```
   Expected: exit 0, no new errors, only pre-existing dead-code warnings (104 warnings bin, 100 windowed).

2. **cargo check --features windowed**
   ```
   cargo check --features windowed
   ```
   Expected: exit 0, no new errors.

3. **Full test suite**
   ```
   cargo test
   ```
   Expected: 0 failures across ALL test binaries. Includes 208+209 lib tests, all integration test files (follow_up_triggers, status_effects, sp_mechanics, intent_applier_canary, cast_id_propagation, etc.).

4. **Gate: no forbidden imports in src/combat/ (excluding blueprints/)**
   ```
   rg "use bevy::winit|use bevy::render|use bevy_egui" src/combat/ --glob '!blueprints/**'
   ```
   Expected: 0 actual import matches. (A doc comment in api/mod.rs mentions these strings but is not an import — acceptable.)

5. **Gate: pub mod api wired**
   ```
   rg 'pub mod api' src/combat/mod.rs
   ```
   Expected: ≥1 match.

6. **Gate: CombatPlugin re-exported from lib.rs**
   ```
   rg 'CombatPlugin' src/lib.rs
   ```
   Expected: ≥1 match (`pub use combat::CombatPlugin`).

7. **Gate: CombatPlugin mounted in main.rs**
   ```
   rg 'add_plugins.*CombatPlugin' src/main.rs
   ```
   Expected: exactly 1 match.

8. **Gate: old registration removed from main.rs**
   ```
   rg 'register_combat_kernel_runtime' src/main.rs
   ```
   Expected: 0 matches.

9. **Gate: intent_applier present**
   ```
   rg 'fn intent_applier' src/combat/api/applier.rs
   ```
   Expected: 1 match.

10. **api/ module file inventory**
    ```
    ls src/combat/api/
    ```
    Expected: exactly 8 files: `applier.rs`, `clock.rs`, `intent.rs`, `mod.rs`, `registry.rs`, `rng.rs`, `signal.rs`, `skill_ctx.rs`.

11. **Canary integration test**
    ```
    cargo test --test intent_applier_canary
    ```
    Expected: 2 tests pass — DealDamage reduces target HP, CombatEvent::OnDamageDealt emitted.

12. **CastId propagation integration test**
    ```
    cargo test --test cast_id_propagation
    ```
    Expected: 3 tests pass — cast-scoped events share cast_id, cast_id ≠ ROOT during cast, pre-cast events use ROOT.

### Edge Cases

- Multi-line `CombatEvent { ... }` struct literals: grep gate 2 (`rg 'CombatEvent \{' | rg -v 'cast_id'`) produces false positives for multi-line literals. Verified via `-A` context that every instance includes `cast_id` on a subsequent line.
- `CastId::ROOT` (NonZeroU32::new(1)) used for pre-cast events — allows downstream filtering of pre-cast vs cast-scoped events.
- `src/combat/api/` types are bevy-agnostic where possible; Bevy dependencies are minimal (`Resource` derive, `bevy::log`).

### Not Proven By This UAT

- Intent variants beyond `DealDamage` are no-ops (log::warn!) — full wiring arrives in S05+.
- `ExtRegistries` created empty — built-in extension functions arrive in S05.
- `SignalBus` and `Clock` are Resource scaffolds with no consumers yet (S04).
- Windowed smoke test (requires DISPLAY) not run; cargo check --features windowed passes statically.
- `CastRng` (SplitMix64) used in applier but not yet exercised by any skill execution path beyond the canary.
