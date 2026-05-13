# S05: Blessed — buff dealt + Ult charge + cleanse-immune — UAT

**Milestone:** M017
**Written:** 2026-05-13T10:09:38.442Z

# UAT — S05: Blessed — buff dealt + Ult charge + cleanse-immune

**UAT Type:** Integration — real damage and Ult-charge pipelines exercised through `apply_effects` in headless test harness.

## Preconditions

- `cargo test` suite fully green before this UAT run.
- Three new integration test files present: `tests/status_blessed_cleanse_immune.rs`, `tests/status_blessed_offensive.rs`, `tests/status_blessed_ult_charge.rs`.

---

## Test Cases

### TC-1: Cleanse does not remove Blessed

**Steps:**
1. Run `cargo test --test status_blessed_cleanse_immune`.

**Expected outcomes:**
- `blessed_survives_cleanse_when_alone` — PASS: Blessed remains in StatusBag after `cleanse_debuffs()`.
- `blessed_survives_cleanse_alongside_debuffs` — PASS: Debuffs are removed; Blessed survives.

---

### TC-2: Blessed attacker deals ×1.15 damage

**Steps:**
1. Run `cargo test --test status_blessed_offensive`.

**Expected outcomes:**
- `blessed_attacker_deals_115_pct_damage` — PASS: damage event value equals `round(base×tag×tri×break×1.15)`.
- `no_blessed_attacker_deals_baseline_damage` — PASS: damage event equals baseline (×1.0).
- `empty_bag_attacker_deals_baseline_damage` — PASS: empty StatusBag yields baseline damage.
- `heated_attacker_does_not_get_blessed_bonus` — PASS: Heated-but-not-Blessed attacker yields baseline damage (orthogonality check).

---

### TC-3: Blessed attacker gains +1 Ult charge per Basic action; no leak on Ultimate cast

**Steps:**
1. Run `cargo test --test status_blessed_ult_charge`.

**Expected outcomes:**
- `baseline_no_blessed_basic_action` — PASS: Ult charge delta equals baseline (0 bonus).
- `blessed_basic_action_gains_extra_charge` — PASS: Ult charge delta equals baseline + 1.
- `blessed_ult_cast_no_charge_leak` — PASS: After Ultimate cast (Reset branch), charge does not exceed reset value by +1.

---

### TC-4: No regressions in full suite

**Steps:**
1. Run `cargo test`.

**Expected outcomes:**
- All test binaries report `0 failed`. Tests in `combat_coherence`, `follow_up_chains`, `form_identity`, `damage_tests`, `resolution_tests`, `holy_support_resolution` all pass.

---

## Edge Cases

- Blessed attacker with base_damage=0 (self-target actions): ×1.15 of 0 = 0 — no phantom damage.
- Blessed Ultimate cast: Reset branch skipped, no self-charge leak (TC-3 third case).
- Starting Ult charge below cap confirmed so the +1 is not clamped away in TC-3.

---

## Not Proven By This UAT

- JSONL log emission of Blessed events under canon naming — owned by S06.
- ValidationSnapshot.statuses_per_unit Blessed entry — owned by S06.
- Interaction between Blessed ×1.15 and Heated ×1.15 amp stacking (not yet specified in §H.1 scope for this milestone).

