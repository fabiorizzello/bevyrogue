# S02: Placement verbs in Registry + generic render dispatcher — UAT

**Milestone:** M004
**Written:** 2026-05-25T11:11:53.463Z

# S02 UAT — Placement verbs in Registry + generic render dispatcher

**UAT Type:** K001 — Human visual verification (windowed only; auto-mode cannot certify)

**Preconditions:**
- Clean build: `cargo build --features windowed` exits 0
- Headless suite green: `cargo test --test animation` and `cargo test --features windowed --test windowed_only` both exit 0

---

## Steps

1. Launch: `cargo run --features windowed` (alias: `cargo winx`)
2. Navigate Agumon into a Baby Flame charge sequence (hold the charge input)
3. **Observe charge phase:** confirm ember particles spiral inward toward Agumon (converge_inward verb)
4. Release and trigger fast launch
5. **Observe launch phase:** confirm the projectile arcs from Agumon toward the target (arc_launch verb)
6. **Observe impact phase:** confirm the impact fan-out burst and flash appear at the projectile endpoint (on_expire chain → impact + impact_shard effects)
7. Repeat 2–6 twice; confirm all three phases are visually consistent across repetitions

---

## Expected Outcomes

- Charge: ember particles orbit inward (not static, not outward) — converge_inward verb active
- Launch: projectile follows a smooth arc from caster toward target — arc_launch verb active
- Impact: fan-out burst + brief flash at impact point upon projectile expiry — on_expire chain fires
- No rendering artifacts: no missing particles, no freezes, no panics in terminal output
- No hardcoded VFX-kind branch visible (already CI-proven by grep-guard test)

---

## Edge Cases

- Interrupt the charge before release: no projectile or impact particles should appear
- Multiple Baby Flames in rapid succession: each should complete independently without bleed-over

---

## Not Proven By This UAT

- Baby Burner detonate and skill-tree variant selection (S03)
- Headless determinism (proven by `placement_verbs::verbs_are_bit_identical_across_1000_calls`)
- VfxParticleKind absence in render.rs (proven by `render_no_vfx_kind_guard::render_rs_has_no_vfx_kind_dispatch`)
- Load-time validation warnings (proven by `validate_effects_*` headless tests)
