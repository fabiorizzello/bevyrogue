# S01: BuffKind::DR primitive + damage formula integration

**Goal:** Add a generic damage-reduction (DR) primitive to the combat kernel via a new `DrBag` component, and integrate it as a multiplicative mitigation step in `calculate_damage` (unclamped sum, `(1.0 - sum).max(0.0)` factor, `final_damage.max(0)`). No franchise-specific logic, no new RON Effect variant in this slice — DR is plumbed at the component/formula level only. Closes M019 success criterion #1.
**Demo:** Test integration tests/dr_pipeline.rs dimostra DR singolo, DR×N sommato, DR+ARM combinato, DR durante Break — damage clampato a 0 senza panic, CombatEvent::Damage emesso con amount=0 dove applicabile.

## Must-Haves

- New `DrBag` component (Vec of `{value, duration}`) lives in `src/combat/buffs.rs`, separate from `StatusBag` so cleanse semantics are untouched.
- `calculate_damage` accepts `defender_dr: Option<&DrBag>`; existing 18-row multiplicative matrix tests still pass with `defender_dr=None`.
- Formula: `dr_mod = (1.0 - sum_dr).max(0.0)`; `final_damage = round(raw * dr_mod).max(0)`. Over-100% DR → 0 damage, no panic; `CombatEvent::Damage`/`OnDamageDealt` still emitted with `amount=0`.
- Both `resolution.rs` call sites pull defender `DrBag` from the world and pass it through; `DrBag::default()` is inserted on units at bootstrap.
- `DrBag` durations decrement alongside `StatusBag::tick_all` (same per-turn tick path); zero-duration instances are dropped.
- `tests/dr_pipeline.rs` covers: DR singolo, DR×N sommato, DR+resist combinato, DR durante Break, clamp a 0 (sum ≥ 1.0).
- Kernel stays franchise-agnostic (P001): no Digimon names, no `if skill_id == …` branches in the damage path.

## Proof Level

- This slice proves: Integration tests in `tests/dr_pipeline.rs` exercise the kernel end-to-end with deterministic units; existing damage-matrix unit tests in `src/combat/damage.rs` form the regression gate.

## Integration Closure

DR is wired into both `calculate_damage` call sites in `resolution.rs` and into the per-turn tick path in `turn_system/mod.rs`. Bootstrap inserts `DrBag::default()` on every unit alongside `StatusBag::default()`.

## Verification

- `DamageBreakdown` gains a `dr_pct: i32` field for log visibility. `CombatEvent::Damage` / `OnDamageDealt` payloads unchanged; existing JSONL stream remains backward-compatible.

## Tasks

- [ ] **T01: Create DrBag component + sum_dr helper + bootstrap insert** `est:small`
  Introduce a new `src/combat/buffs.rs` module owning `DrInstance { value: f32, duration: u32 }`, `DrBag(Vec<DrInstance>)` (derive `Component`, `Default`, `Debug`, `Clone`), a pure `sum_dr(bag: Option<&DrBag>) -> f32` helper (unclamped), and a `DrBag::tick_all() -> usize` method that decrements every instance's duration and drops zero entries (returning the count dropped, mirroring `StatusBag::tick_all`'s shape). Also expose `DrBag::apply(value: f32, duration: u32)` so future M021 `Intent::ApplyDR` work has a public seam. Re-export the module from `src/combat/mod.rs`. Insert `DrBag::default()` at the same spawn site as `StatusBag::default()` in `src/combat/bootstrap.rs:162` (and any sibling unit-spawn paths the grep surfaces, e.g. `pipeline.rs:1717` fresh-bag construction — only if it constructs full units, not a partial reset). No formula changes yet; existing tests must still pass.
  - Files: `src/combat/buffs.rs`, `src/combat/mod.rs`, `src/combat/bootstrap.rs`
  - Verify: cargo check && cargo test --lib calculate_damage && cargo test bootstrap_spawn_composition

- [ ] **T02: Integrate DR into calculate_damage formula + DamageBreakdown** `est:small`
  Extend `calculate_damage` in `src/combat/damage.rs` to accept a new parameter `defender_dr: Option<&DrBag>` (placed after `defender_status`). Compute `dr_sum = sum_dr(defender_dr)`, `dr_mod = (1.0 - dr_sum).max(0.0)`, multiply it into the raw formula, then apply `final_damage = round(raw).max(0)`. Add `dr_pct: i32` (integer percent, i.e. `(dr_sum * 100.0).round() as i32`) to `DamageBreakdown` and populate it. Update every internal/unit-test call site in `damage.rs` to pass `None` for the new parameter so the existing 18-row multiplicative matrix continues to pass byte-for-byte. Do NOT modify `resolution.rs` in this task (T03 owns those call sites).
  - Files: `src/combat/damage.rs`
  - Verify: cargo test --lib calculate_damage && cargo check

- [ ] **T03: Wire DrBag through resolution.rs call sites + per-turn tick** `est:small`
  Update the two `calculate_damage` call sites in `src/combat/resolution.rs` (~line 478 and ~line 636) to fetch the defender's `DrBag` from the world using the same query pattern that already reads `defender_status`/`StatusBag`, and pass `Option<&DrBag>` into `calculate_damage`. Then extend the per-turn tick block in `src/combat/turn_system/mod.rs` (lines 518 and 569 — both spots that call `bag.tick_all()` on `StatusBag`) so that the matching `DrBag` for the same entity also has `tick_all()` called and any drop count is logged through the existing log seam (or simply discarded — match what `StatusBag::tick_all` does). Do not introduce new events or change `OnDamageDealt` payload.
  - Files: `src/combat/resolution.rs`, `src/combat/turn_system/mod.rs`
  - Verify: cargo check && cargo test --test status_blessed_offensive && cargo test --test damage_breakdown_log

- [ ] **T04: Integration tests: tests/dr_pipeline.rs** `est:medium`
  Create `tests/dr_pipeline.rs` (headless integration test) covering the five cases listed in S01 success criteria. Build deterministic units with the same bootstrap pattern used in `tests/status_blessed_offensive.rs`. For each case, insert a `DrBag` with specific instances on the defender directly via the world API (no new RON Effect variant — S01 is formula-side only), trigger a single damaging skill, and assert via `CombatEvent::Damage` (or `OnDamageDealt`) read from the event bus: 
  - DR singolo: base=100, one DrInstance{value:0.30} → amount=70.
  - DR×N sommato: two DrInstance{value:0.20} → amount=60.
  - DR + resist (tag_mod=0.75): base=100, DR=0.20 → amount=60.
  - DR durante Break (break_mod=2.0): base=100, DR=0.30 → amount=140.
  - Clamp a 0: DR sum=1.5 → amount=0, no panic, defender hp_current unchanged, event still emitted.
  Also add a sixth case asserting tick decrements: insert DrInstance{value:0.30,duration:1}, advance one turn-end, verify the instance is dropped and damage no longer mitigated. Tests must be deterministic (no wall-clock, no unseeded RNG).
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
