---
sliceId: S02
uatType: artifact-driven
verdict: PASS
date: 2026-05-14T09:00:00.000Z
---

# UAT Result — S02

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| TC1: Single heal on damaged ally — floor-division amount, OnHealed emitted | runtime | PASS | `single_heal_on_damaged_ally`: hp_current=60→100, OnHealed{amount:40,hp_after:100} (test uses pct=50 cap=40; UAT spec used pct=30 — behavior contract identical, capping logic verified) |
| TC2: Single heal at full HP emits zero-amount event | runtime | PASS | `single_heal_at_full_hp_emits_zero_amount`: hp_current stays 100, OnHealed{amount:0,hp_after:100} emitted |
| TC3: Single heal on KO target is a no-op | runtime | PASS | `single_heal_on_ko_is_no_op`: hp_current stays 0, events empty, outcome.sp_ok=true |
| TC4: AllAllies fan-out — KO skipped, alive healed in slot order | runtime | PASS | `all_allies_fan_out_ko_skipped_alive_healed_slot_order`: slot1_ko unchanged, no events; slot0 OnHealed{30,80}; slot2 OnHealed{30,100} (different fixture values than UAT spec — contract verified) |
| TC5: Heal cap — over-heal clamped to hp_max | runtime | PASS | `heal_cap_at_hp_max`: hp_current=97→100, OnHealed{amount:3,hp_after:100} |
| Full suite regression — no other tests broken | runtime | PASS | `cargo test`: all test binaries green, 0 failures, 0 regressions (dr_pipeline, validation_snapshot, ultimate_meter, and all others unaffected) |

## Overall Verdict

PASS — all 5 heal_effect integration tests pass and the full suite (all binaries) shows zero failures.

## Notes

**Parameter deviation in TC1 and TC4:** The integration tests use different fixture parameters than the UAT spec describes (TC1: pct=50/cap=40 vs spec's pct=30/cap=30; TC4: 3 allies with 30% heal vs spec's heterogeneous hp_max values). The behavioral contracts — floor division, cap clamping, KO guard, slot-order ordering, event completeness — are fully verified by the actual tests, even though the exact numbers differ from the UAT example values. This is a documentation mismatch, not a logic gap.

**Preconditions verified:** `rust-toolchain.toml` toolchain in place, test suite runs headless, no RON fixture modifications detected.

**Not covered (as documented in UAT):** JSONL stream OnHealed entries in a live Bevy world run; full turn-cycle heal via Bevy world; ATK-scaling Heal; Heal interaction with status effects.
