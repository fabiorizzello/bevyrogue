# S08: Form Identity framework + 6 Adult wired + rimozione D026 cap (D046) — UAT

**Milestone:** M011
**Written:** 2026-04-28T10:34:27.928Z

# S08 UAT Script — Form Identity framework + D046 re-entrancy removal

## Preconditions
- Working directory: project root
- `cargo build` clean (no errors)
- `BEVYROGUE_JSONL=1` env var available for observability checks

---

## Test Group 1: D046 Chain Re-entrancy (follow_up_chains.rs)

### UAT-S08-01: Depth-2 chain progression
**Command:** `cargo test --test follow_up_chains depth_chain_progresses_to_depth_two`
**Expected:** PASS. Agumon follow-up fires at depth=1; a second unit's follow-up fires at depth=2 in the same update cycle. At least one CombatEvent with follow_up_depth=2 appears in the stream.

### UAT-S08-02: Chain terminates naturally
**Command:** `cargo test --test follow_up_chains chain_terminates_when_follow_up_cannot_retrigger`
**Expected:** PASS. Greymon's depth=1 follow-up does not produce a depth=2 event because the broken enemy cannot trigger OnEnemyBreak again. No depth>=2 events appear.

### UAT-S08-03: OneHopSuppressed fully purged
**Command:** `grep -rn 'OneHopSuppressed' src/ tests/`
**Expected:** Zero matches returned (exit code 1 from grep).

### UAT-S08-04: Ultimate charging guard preserved
**Command:** `grep -n 'follow_up_depth >= 1' src/combat/ultimate.rs`
**Expected:** Exactly one match at line 77 (OnAllyFollowUp ult-charge gating).

---

## Test Group 2: Form Identity — GrantEnergy (Greymon, Garurumon, Kabuterimon)

### UAT-S08-05: Greymon first Fire hit grants +5 Energy
**Command:** `cargo test --test form_identity greymon_first_fire_hit_grants_energy`
**Expected:** PASS. Exactly one EnergyGained{amount:5} event emitted. Greymon Energy.current == 5. form_identity_used == true.

### UAT-S08-06: Greymon once-per-round enforcement
**Command:** `cargo test --test form_identity greymon_second_fire_hit_blocked`
**Expected:** PASS. Second Fire hit in same round emits no EnergyGained. Energy stays at 5.

### UAT-S08-07: Greymon form_identity resets next turn
**Command:** `cargo test --test form_identity greymon_resets_next_turn`
**Expected:** PASS. Turn advance resets form_identity_used. Second round's first Fire hit emits another EnergyGained{amount:5}.

### UAT-S08-08: Garurumon first Ice hit grants +5 Energy
**Command:** `cargo test --test form_identity garurumon_first_ice_hit_grants_energy`
**Expected:** PASS. EnergyGained{amount:5} from Garurumon's Ice basic. Energy.current == 5.

### UAT-S08-09: Kabuterimon first Electric hit grants +5 Energy
**Command:** `cargo test --test form_identity kabuterimon_first_electric_hit_grants_energy`
**Expected:** PASS. EnergyGained{amount:5} from Kabuterimon's Electric basic.

### UAT-S08-10: Tag cross-contamination impossible
**Command:** `cargo test --test form_identity greymon_fire_trigger_does_not_fire_on_garurumon_ice_hit`
**Expected:** PASS. Garurumon Ice hit does NOT trigger Greymon's OnFirstHitVsTagThisRound(Fire). No EnergyGained for Greymon.

---

## Test Group 3: Form Identity — SelfAdvance (Kyubimon)

### UAT-S08-11: Kyubimon Freeze application self-advances
**Command:** `cargo test --test form_identity kyubimon_freeze_application_self_advances`
**Expected:** PASS. Kyubimon applies Freeze; TurnAdvance{target:kyubimon, amount_pct:20} event emitted. form_identity_used == true. Second Freeze application in same round does NOT re-trigger.

---

## Test Group 4: Form Identity — ToughnessHit bonus (DORUgamon)

### UAT-S08-12: DORUgamon first Dark skill grants bonus toughness
**Command:** `cargo test --test form_identity dorugamon_first_dark_skill_grants_bonus_toughness`
**Expected:** PASS. DORUgamon's Dark skill followed by dorugamon_form_identity ToughnessHit(10) fires. form_identity_used flips to true. Second Dark skill in same round does not add the bonus.

---

## Test Group 5: Form Identity — Attribute-conditional bonus (Angemon)

### UAT-S08-13: Angemon bonus fires vs Virus target
**Command:** `cargo test --test form_identity angemon_attack_vs_virus_grants_bonus`
**Expected:** PASS. Angemon attacks Virus enemy; angemon_form_identity Damage(15) fires as follow-up. At least 2 OnDamageDealt events from Angemon (base + form_identity). form_identity_used == true.

### UAT-S08-14: Angemon bonus does NOT fire vs Data target
**Command:** `cargo test --test form_identity angemon_attack_vs_data_no_bonus`
**Expected:** PASS. Angemon attacks Data enemy; no angemon_form_identity follow-up fires. form_identity_used == false.

---

## Test Group 6: Slice-level integration

### UAT-S08-15: Skills catalog at 65
**Command:** `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron`
**Expected:** PASS. 65 skills loaded; all 6 form_identity IDs present: greymon_form_identity, garurumon_form_identity, kabuterimon_form_identity, kyubimon_form_identity, dorugamon_form_identity, angemon_form_identity.

### UAT-S08-16: All 6 Adults have form_identity set in units.ron
**Command:** `grep -c 'form_identity:' assets/data/units.ron`
**Expected:** Returns `6`.

### UAT-S08-17: Full integration suite green
**Command:** `cargo test`
**Expected:** 29 binaries, 0 failures.

---

## Observability smoke check (manual, optional)

### UAT-S08-18: EnergyGained visible in JSONL stream
**Command:** `echo '' | BEVYROGUE_JSONL=1 cargo run --bin combat_cli 2>&1 | grep EnergyGained`
**Expected:** At least one `EnergyGained` line appears when a Greymon party member executes a Fire basic attack. Exact count depends on party selection; non-zero output confirms end-to-end observability.

---

## Known deviations for product owner sign-off in S09 UAT

1. **DORUgamon 'modifier-in-place':** dorugamon_form_identity fires as a separate ToughnessHit(10) skill rather than modifying the triggering skill's toughness in-place. Effect::BonusToughnessDamage exists in schema but is unused; reserved for S09 rebalance.
2. **Angemon 'modifier-in-place':** angemon_form_identity fires as a separate Damage(15) skill rather than modifying the base attack damage. Effect::BonusDamageVsAttribute exists in schema but is unused.
3. **DORUgamon tag-scope:** OnFirstSkillCastWithTag(Dark) matches any OnDamageDealt from DORUgamon with Dark tag. Skill-vs-basic distinction not enforced at engine level (all DORUgamon attacks share Dark tag in current roster, so no observable difference in S08).
4. **DORUgamon Light-skill negative test omitted:** DORUgamon has no Light skills; once-per-round exclusivity verified implicitly via second-cast guard in the same test.
