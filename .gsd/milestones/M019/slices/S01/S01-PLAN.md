# S01: S01

**Goal:** Add a generic damage-reduction (DR) primitive to the combat kernel via a new `DrBag` component, and integrate it as a multiplicative mitigation step in `calculate_damage` (unclamped sum, `(1.0 - sum).max(0.0)` factor, `final_damage.max(0)`). No franchise-specific logic, no new RON Effect variant in this slice — DR is plumbed at the component/formula level only. Closes M019 success criterion #1.
**Demo:** Test integration tests/dr_pipeline.rs dimostra DR singolo, DR×N sommato, DR+ARM combinato, DR durante Break — damage clampato a 0 senza panic, CombatEvent::Damage emesso con amount=0 dove applicabile.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: DrBag component + sum_dr helper + bootstrap insert already fully implemented in prior commit (2c09b85)**
  - Files: `src/combat/buffs.rs`, `src/combat/mod.rs`, `src/combat/bootstrap.rs`
  - Verify: cargo check && cargo test --lib calculate_damage && cargo test bootstrap_spawn_composition

- [x] **T02: Integrate DR into calculate_damage formula + DamageBreakdown**
  - Files: `src/combat/damage.rs`
  - Verify: cargo test --lib calculate_damage && cargo check

- [x] **T03: Wire DrBag through resolution.rs call sites + per-turn tick**
  - Files: `src/combat/resolution.rs`, `src/combat/turn_system/mod.rs`
  - Verify: cargo check && cargo test --test status_blessed_offensive && cargo test --test damage_breakdown_log

- [ ] **T04: Integration tests: tests/dr_pipeline.rs**
  - Files: `tests/dr_pipeline.rs`
  - Verify: cargo test --test dr_pipeline && cargo test

## Files Likely Touched

- src/combat/buffs.rs
- src/combat/mod.rs
- src/combat/bootstrap.rs
- src/combat/damage.rs
- src/combat/resolution.rs
- src/combat/turn_system/mod.rs
- tests/dr_pipeline.rs
