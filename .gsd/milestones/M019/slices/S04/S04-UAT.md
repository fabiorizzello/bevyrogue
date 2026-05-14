# S04: DamageCurve::PerHop runtime length guard (chiude follow-up #3 M018) — UAT

**Milestone:** M019
**Written:** 2026-05-14T09:37:16.229Z

# UAT — S04: DamageCurve::PerHop Runtime Length Guard

**UAT Type:** Integration (direct `apply_effects` call, no Bevy world spin-up, per MEM003)

## Preconditions
- Codebase at commit containing S04 changes (pipeline.rs guard + tests/perhop_guard.rs)
- `cargo test` toolchain available (rust-toolchain.toml satisfied)
- No external dependencies required

## Test Steps

1. **Run the dedicated guard test:**
   ```
   cargo test --test perhop_guard
   ```
   Expected: `test perhop_length_guard_emits_diagnostic_and_clamps_loop ... ok` — exit 0.

2. **Verify diagnostic event emitted exactly once:**
   The test asserts `events.iter().filter(|e| matches!(e.kind, CombatEventKind::OnActionFailed { .. })).count() == 1`. Confirm this assertion passes without modification.

3. **Verify exactly 2 OnDamage events (no ghost hop):**
   The test asserts exactly 2 `OnDamage` events corresponding to the two available coefficients (30, 20). Confirm no third event is present.

4. **Verify no panic:**
   The test completes without a panic. The pre-loop clamp prevents index-out-of-bounds; `apply_effects` returns normally.

5. **Run full regression suite:**
   ```
   cargo test
   ```
   Expected: all tests pass, 0 failures. Covers target_shape_bounce_chain.rs (well-formed PerHop), dr_pipeline.rs, heal_effect.rs, cleanse_effect.rs, and all other suites.

## Expected Outcomes

| Check | Expected |
|-------|----------|
| perhop_guard test | 1 passed, 0 failed |
| OnActionFailed count | exactly 1 |
| OnDamage count | exactly 2 |
| Ghost hop (3rd damage) | absent |
| Panic | none |
| Full suite | all pass, 0 failures |

## Edge Cases

- **Well-formed PerHop (coeffs.len == hops_planned):** guard does not fire; no OnActionFailed emitted. Covered by existing target_shape_bounce_chain.rs.
- **coeffs.len == 0, hops_planned > 0:** guard fires, 0 OnDamage events emitted, 1 OnActionFailed. Not covered by UAT but structurally handled by the clamp (loop bound = 0).
- **DamageCurve::Flat or Falloff:** guard not reached (only executes in PerHop arm); no effect.

## Not Proven By This UAT
- Load-time validation of PerHop length in skills_ron.rs (deferred to M021)
- Behaviour when the diagnostic event is consumed by the egui UI panel (windowed feature, untested headless)
- Multi-target Bounce interactions with PerHop length mismatch (covered implicitly by clamp but not explicitly tested)

