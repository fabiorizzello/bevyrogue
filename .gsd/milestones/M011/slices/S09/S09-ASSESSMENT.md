# S09: Numerical Rebalance + Enemy Roster — Slice Assessment

**Milestone:** M011
**Slice:** S09
**Date:** 2026-04-28
**Author:** GSD auto-mode (agent scaffold; verdict awaiting human sign-off)

---

## (a) UAT Verdict

**Verdict:** `<awaiting human sign-off>`

The UAT script (`S09-UAT.md`) has been authored and is ready for execution. The agent cannot perform the 30-minute subjective playthrough — this gate requires human judgment on pacing feel, Form Identity readability, Break Seal correctness, and status-effect legibility.

**To sign off:**
1. Complete all three encounters in `S09-UAT.md` (MinionWave → MiniBossEncounter → BossEncounter).
2. Fill in the subjective rubric checklist.
3. Record verdict (`pass` / `fail` / `pass-with-followups`) and rationale in `S09-UAT.md`.
4. Update this file's verdict line above.
5. Notify the milestone owner so M011 milestone closure can proceed.

---

## (b) Deviations and Decisions from S09 Execution

### D052 — BonusToughnessDamage / BonusDamageVsAttribute stripped (T03)

Both `BonusToughnessDamage(i32)` and `BonusDamageVsAttribute { attribute, bonus_pct }` were Effect enum variants added in S08 but never wired into `apply_effects`. T03 stripped them as dead code rather than wiring them. Rationale: the DORUgamon and Angemon Form Identity effects are already delivered via the "fire a separate skill" pattern (D050); wiring modifier-in-place variants would have been S09 scope creep with no TTK-observable payoff. The decision is recorded as D052 in `.gsd/DECISIONS.md`.

**Impact on UAT:** None. DORUgamon Form Identity fires a ToughnessHit follow-up skill; Angemon Form Identity fires `angemon_form_identity`. Both are visible in the event log. The stripped variants were never player-facing.

### angemon_basic ToughnessHit 8 → 20 (T03)

To guarantee the Ogremon break in the miniboss scenario, `angemon_basic` ToughnessHit was bumped from 8 to 20. This was required because of the toughness-break mechanic: the crossing hit (the one that takes toughness from >0 to ≤0) must also match a weakness tag. With Ogremon `toughness_max: 20` and `weaknesses: [Fire, Light]`, Angemon (second attacker on turn 1) with ToughnessHit(20) hits exactly toughness−20=0 while being a Light weakness hit → break guaranteed. This change is functionally correct and UAT-observable.

### Goblimon hp_max 120 → 40 (T03)

Required to bring minion TTK from ~6 turns down to the R083 target of 2–3 turns. Goblimon is a minion-tier enemy by design; 40 HP is appropriate for a one-hit-elimination feel for adults and a two-hit-elimination feel for children.

### Ogremon and Devimon toughness values tuned (T03)

- Ogremon: `toughness_max` set to 20 (from 60), `weaknesses: [Fire, Light]`, `hp_max: 200` (from 280).
- Devimon: `toughness_max` set to 35 (from 100), `weaknesses: [Fire, Light]` (Light added), `hp_max: 300` (from 500).

These changes are locked by the T02/T03 scenario tests and confirmed by the 37-binary CI suite.

---

## (c) Follow-Ups for M012

The following items were identified during M011 execution as out-of-scope deferral candidates. They are reaffirmed here for M012 planning:

### High-priority deferrals

1. **Tamer Gauge** — The Commander (Taichi, UnitId 0) is spawned by bootstrap but is purely passive in M011. M012 should wire the Tamer Gauge mechanic (charge on ally actions, discharge for Tamer skills/boosts) as the next meta-layer on top of the SP pool.

2. **DNA Chips / Equipment** — No item/equipment system exists. M012 is the earliest feasible milestone for a chip-slot model on UnitDef (passive stat modifiers, passive skill unlocks).

3. **Enemy Counterplay Traits** — Enemies currently use a minimal skill set (`enemy_skill_fire`, `enemy_ult_fire`). A proper enemy AI with tag-specific counterplay (e.g., Devimon retaliating with Dark on Light hits) would make the boss encounter feel more tactical. This is the next enemy AI milestone.

4. **Multiple Enemy Skills / AI Routing** — The `enemy_ai.rs` decision routing is currently single-path. Extending it with a multi-skill selection policy (weighted by enemy HP thresholds, player party composition) is M012+ work.

5. **Floor / Meta-Loop** — The roguelite progression layer (Slay the Spire-style floor traversal, encounter sequencing, reward selection) is entirely unimplemented. M012 should define the first floor-loop milestone.

### Lower-priority / post-M012

6. **Windowed egui UI** — The `combat_cli` is the only playtest harness. The egui UI behind `--features windowed` has not been updated to reflect S09 additions (enemy roster, encounter presets, Form Identity events). Refreshing the egui dashboard is tracked but non-blocking.

7. **Status Effect Legibility** — Burn/Slow/Freeze ticks are visible in the event log but the dashboard does not display active status effects per unit. Adding a status effect column to the dashboard would improve UAT feel for future slices.

8. **Break Seal Visual Indicator** — The `BreakSealApplied` event is in the log but the dashboard TGH bar does not visually distinguish sealed state from normal state. A `[SEALED]` annotation would improve readability.

---

## (d) Integration Test Suite Status

**Date of last run:** 2026-04-28
**Command:** `cargo test`
**Result:** 37 integration binaries, 0 failures

| Binary | Tests Passing |
|--------|---------------|
| scenario_minion_ttk | 1 (turn_count=3, band 2–3 ✓) |
| scenario_miniboss_ttk | 1 (turn_count in [3,5], break_count≥1 ✓) |
| scenario_boss_ttk | 1 (turn_count=5, band 4–7, energy≥1, break≥1 ✓) |
| All other binaries | 34 remaining, all green |

R083 TTK targets are locked by automated scenario fixtures. The UAT provides the subjective/qualitative gate that the fixtures cannot cover.

---

## Milestone Closure Pre-Conditions

Before M011 can be closed:

- [ ] UAT verdict recorded in `S09-UAT.md` by human operator.
- [ ] Verdict is `pass` or `pass-with-followups` (not `fail`).
- [ ] Any `pass-with-followups` items are triaged: either fixed before closure or explicitly deferred to M012 with a GSD requirement entry.
- [ ] `cargo test` still green at time of sign-off (run immediately before signing).
- [ ] Follow-up items in section (c) above are captured in `.gsd/REQUIREMENTS.md` or the M012 roadmap as appropriate.
