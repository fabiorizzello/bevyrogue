# S02: Damage Tag rename + matchup ±25% + Attribute Triangle v5.3 in-line — UAT

**Milestone:** M011
**Written:** 2026-04-27T14:34:19.406Z

# S02 UAT Script — Damage Tag + Attribute Triangle v5.3

## Preconditions
- `cargo test --no-fail-fast` passes (28 binaries, 0 failed).
- No `Element::|Resistances|thread_rng` in src/, tests/, assets/.
- Feature `windowed` not required; all tests are headless.

---

## TC01 — Rename completeness
**Command:** `grep -rn 'Element::\|: Element\|basic_element\|Resistances' src/ tests/ assets/data/`
**Expected:** zero matches (exit 1 from grep = clean)
**Pass criteria:** command exits non-zero (no matches found)

---

## TC02 — Full suite green
**Command:** `cargo test --no-fail-fast 2>&1 | grep 'test result'`
**Expected:** all lines show `ok. N passed; 0 failed`; ≥27 binaries present
**Pass criteria:** no line contains `FAILED`

---

## TC03 — DamageTag weak matchup (tag_mod_pct=125)
**Test file:** `tests/damage_breakdown_log.rs` second scenario
**Setup:** Attacker with Fire tag attacks Defender whose Toughness.weaknesses includes Fire
**Expected:** `OnDamageDealt { tag_mod_pct: 125, ... }`
**Pass criteria:** `cargo test --test damage_breakdown_log` exits 0

---

## TC04 — DamageTag resist matchup (tag_mod_pct=75) + triangle attacker-loses (triangle_mod_pct=111)
**Test file:** `tests/damage_breakdown_log.rs` primary scenario
**Setup:** Devimon (Virus attr, Fire damage_tag, base 100) attacks Greymon (Vaccine attr, resists Fire)
**Expected:** `OnDamageDealt { amount: 83, tag_mod_pct: 75, triangle_mod_pct: 111 }`
**Calculation:** round(100 × 0.75 × 1.11) = round(83.25) = 83
**Pass criteria:** `cargo test --test damage_breakdown_log` exits 0; assertion on all three fields

---

## TC05 — Attribute Triangle all 16 pairs
**Test file:** `tests/triangle_matchup.rs`
**Expected:** triangle_all_16_pairs passes; each (attacker_attr, defender_attr) combination yields correct TriangleMods triple:
- Vaccine vs Virus (attacker wins): dmg_modifier=1.0, tough_modifier=1.0, status_acc=1.0
- Virus vs Vaccine (attacker loses): dmg_modifier=1.11, tough_modifier=1.0, status_acc=0.90
- Vaccine vs Data (defender wins): dmg_modifier=0.87, tough_modifier=0.87, status_acc=1.0
- Free vs any: dmg_modifier=1.0, tough_modifier=1.0, status_acc=1.0
**Pass criteria:** `cargo test --test triangle_matchup` exits 0

---

## TC06 — Status accuracy miss (seeded)
**Test file:** `tests/status_accuracy.rs` scenario 1
**Setup:** Vaccine attacker → Data defender (attacker loses, status_acc_modifier=0.90); seed chosen so roll ≥ 90
**Expected:** `OnStatusResisted { kind }` emitted; `OnStatusApplied` not emitted; StatusEffect component absent from defender entity
**Pass criteria:** `cargo test --test status_accuracy` exits 0

---

## TC07 — Status accuracy hit (seeded)
**Test file:** `tests/status_accuracy.rs` scenario 2
**Setup:** same matchup, different seed so roll < 90
**Expected:** `OnStatusApplied` emitted; StatusEffect component present on defender
**Pass criteria:** `cargo test --test status_accuracy` exits 0

---

## TC08 — Status accuracy neutral (always hits)
**Test file:** `tests/status_accuracy.rs` scenario 3
**Setup:** Vaccine attacker → Vaccine defender (neutral, status_acc_modifier=1.0)
**Expected:** status always applied regardless of seed; `OnStatusApplied` emitted
**Pass criteria:** `cargo test --test status_accuracy` exits 0

---

## TC09 — CombatRng replaces thread_rng (determinism)
**Command:** `grep -rn 'thread_rng' src/combat/`
**Expected:** zero matches
**Pass criteria:** command exits non-zero (no matches = grep exit 1 = clean)

---

## TC10 — S01 pipeline lifecycle contract preserved
**Test file:** `tests/pipeline_dispatch.rs`
**Expected:** OnStatusResisted is positioned between OnActionPreApp and OnActionApplied in the event stream when status misses; existing pipeline_dispatch tests continue to pass
**Pass criteria:** `cargo test --test pipeline_dispatch` exits 0

---

## Edge Cases

- **Base damage = 0:** damage_tests.rs covers this case; result must be 0 regardless of modifiers.
- **Free attribute vs all:** triangle_matchup covers all Free pairings; dmg_modifier must be 1.0 in all cases.
- **Physical tag, no weakness/resist:** tag_mod_pct=100 (neutral); covered in damage_tests.rs.
- **Break multiplier stacking:** damage_tests.rs covers tag=weak × triangle=win × break=true case.
