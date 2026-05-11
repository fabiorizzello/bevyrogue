---
sliceId: S08
uatType: artifact-driven
verdict: PASS
date: 2026-04-28T12:35:30Z
---

# UAT Result — S08

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| UAT-S08-01: Depth-2 chain progression | runtime | PASS | `depth_chain_progresses_to_depth_two ... ok` |
| UAT-S08-02: Chain terminates naturally | runtime | PASS | `chain_terminates_when_follow_up_cannot_retrigger ... ok` |
| UAT-S08-03: OneHopSuppressed fully purged | artifact | PASS | `grep -rn 'OneHopSuppressed' src/ tests/` → exit code 1 (zero matches in worktree) |
| UAT-S08-04: Ultimate charging guard preserved | artifact | PASS | `src/combat/ultimate.rs:77: && event.follow_up_depth >= 1` — exactly one match |
| UAT-S08-05: Greymon first Fire hit grants +5 Energy | runtime | PASS | `greymon_first_fire_hit_grants_energy ... ok` |
| UAT-S08-06: Greymon once-per-round enforcement | runtime | PASS | `greymon_second_fire_hit_blocked ... ok` |
| UAT-S08-07: Greymon form_identity resets next turn | runtime | PASS | `greymon_resets_next_turn ... ok` |
| UAT-S08-08: Garurumon first Ice hit grants +5 Energy | runtime | PASS | `garurumon_first_ice_hit_grants_energy ... ok` |
| UAT-S08-09: Kabuterimon first Electric hit grants +5 Energy | runtime | PASS | `kabuterimon_first_electric_hit_grants_energy ... ok` |
| UAT-S08-10: Tag cross-contamination impossible | runtime | PASS | `greymon_fire_trigger_does_not_fire_on_garurumon_ice_hit ... ok` |
| UAT-S08-11: Kyubimon Freeze application self-advances | runtime | PASS | `kyubimon_freeze_application_self_advances ... ok` |
| UAT-S08-12: DORUgamon first Dark skill grants bonus toughness | runtime | PASS | `dorugamon_first_dark_skill_grants_bonus_toughness ... ok` |
| UAT-S08-13: Angemon bonus fires vs Virus target | runtime | PASS | `angemon_attack_vs_virus_grants_bonus ... ok` |
| UAT-S08-14: Angemon bonus does NOT fire vs Data target | runtime | PASS | `angemon_attack_vs_data_no_bonus ... ok` |
| UAT-S08-15: Skills catalog at 65 | runtime | PASS | `parse_canonical_skills_ron ... ok` (1 passed, 0 failed) |
| UAT-S08-16: All 6 Adults have form_identity in units.ron | artifact | PASS | `grep -c 'form_identity:' assets/data/units.ron` → `6` |
| UAT-S08-17: Full integration suite green | runtime | PASS | 34 test result blocks, 0 failures, 0 ignored (30 integration binaries + lib/bin/doc; see notes) |
| UAT-S08-18: EnergyGained visible in JSONL stream | human-follow-up | NEEDS-HUMAN | Marked optional in UAT. Automated smoke not run (requires interactive party selection). Manual follow-up: run `echo '' \| BEVYROGUE_JSONL=1 cargo run --bin combat_cli 2>&1 \| grep EnergyGained` with a Greymon party. |

## Overall Verdict

PASS — all 17 automatable checks passed; 1 optional observability smoke check deferred to human reviewer.

## Notes

**Binary count deviation (UAT-S08-17):** The UAT spec expected "29 binaries" but the suite now contains 30 integration test files (`tests/*.rs`) plus lib unit tests, bin tests, and doc-tests (34 `test result:` lines total). The additional tests were added during S08 (`form_identity.rs`, `follow_up_chains.rs`) and earlier slices. All pass with 0 failures; the count deviation is additive only and does not indicate regressions.

**OneHopSuppressed note:** Running the grep from the main project checkout (`master` branch) shows residual matches in `src/combat/follow_up.rs` and `tests/follow_up_reentrancy.rs`. From the worktree (`milestone/M011` branch) the symbol is fully absent — exit code 1, zero matches. UAT-S08-03 verdict stands as PASS on the correct branch.

**Known deviations carried forward to S09 sign-off** (per UAT file):
1. DORUgamon `modifier-in-place`: `dorugamon_form_identity` fires as separate ToughnessHit(10) rather than modifying the triggering skill in-place.
2. Angemon `modifier-in-place`: `angemon_form_identity` fires as separate Damage(15 Light) rather than modifying base attack damage.
3. DORUgamon tag-scope: OnFirstSkillCastWithTag(Dark) matches any OnDamageDealt with Dark tag; skill-vs-basic distinction not enforced.
4. DORUgamon Light-skill negative test omitted — no Light skills in kit.
