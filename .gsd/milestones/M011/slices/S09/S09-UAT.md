# S09: Numerical rebalance pass + UAT scenarios — UAT

**Milestone:** M011
**Written:** 2026-04-28T12:18:29.303Z

# S09: Numerical Rebalance + Enemy Roster — UAT

**Milestone:** M011
**Written:** 2026-04-28

**Tester name:** ___________________________
**Date of playthrough:** ___________________________
**Build SHA:** `git rev-parse --short HEAD` → ___________________________

---

## UAT Type

- UAT mode: human-experience (live-runtime)
- Why this mode is sufficient: the three TTK bands are already locked by automated scenario tests (T02/T03); UAT verifies *feel* — pacing, readability of Form Identity/Break Seal events, status effect legibility, and encounter variety — which automated tests cannot capture.

---

## Preconditions

- [ ] `cargo build --bin combat_cli` exits 0.
- [ ] `BEVYROGUE_JSONL=1 cargo run --bin combat_cli` launches without panic and prints a JSONL stream alongside the dashboard text output.
- [ ] `cargo test` exits 0 (all 37+ integration binaries green).
- [ ] Terminal is at least 80 columns wide and supports UTF-8.
- [ ] You have ~30 minutes of uninterrupted focus time.

---

## Smoke Test

```bash
cargo run --bin combat_cli
```

At the encounter-selection prompt, choose **MinionWave**. At the party prompt, pick any 4 allies. The combat dashboard should appear within 2 seconds and the encounter should resolve to `[VICTORY]` within ~3 turns (no more than 4). If this smoke test fails, stop — do not proceed to the three-encounter script below.

---

## Test Cases

### 1. Encounter 1 — MinionWave (minion-tier, ~5 minutes)

**Setup:**
```bash
cargo run --bin combat_cli
```
- Encounter: **MinionWave** (3 × Goblimon)
- Recommended party: Agumon · Gabumon · Renamon · Patamon (all Children for purity; any 4 Children work)

**Steps:**
1. Select `MinionWave` at the encounter prompt.
2. Select the 4 recommended Children at the party prompt.
3. Play each ally's turn using **Basic Attack** on the highest-HP Goblimon.
4. Continue until `[VICTORY]` is shown.

**Expected observations:**
- [ ] The combat dashboard refreshes per turn showing HP bars for both teams.
- [ ] Encounter resolves in **2–3 turns** (R083 minion band).
- [ ] No Form Identity events are visible in the event log (Children don't have Form Identity).
- [ ] No `OnBreak` event appears (Goblimon has `toughness_max: 0`).
- [ ] `[VICTORY]` text is printed, game exits cleanly (exit code 0).

**Fail signals:**
- Encounter takes 4+ turns → numbers regressed, recheck `units.ron` Goblimon `hp_max`.
- Panic or hang → engineering escalation.
- Dashboard shows enemies with full HP after 3 turns → bootstrap wiring issue.

---

### 2. Encounter 2 — MiniBossEncounter (mini-boss tier, ~10 minutes)

**Setup:**
```bash
cargo run --bin combat_cli
```
- Encounter: **MiniBossEncounter** (Ogremon + 2 × Goblimon)
- Recommended party: Greymon · Angemon · DORUgamon · Patamon
  - Greymon (Fire, `breaker`) and Angemon (Light, `angemon_basic` ToughnessHit=20) are the designated breakers — Ogremon `weaknesses: [Fire, Light]`.

**Steps:**
1. Select `MiniBossEncounter` at the encounter prompt.
2. Select the recommended party at the party prompt.
3. Focus first attacks on Goblimons to clear them, then concentrate on Ogremon.
4. When Angemon acts, use `angemon_basic` targeting Ogremon — watch for `[EVENT] OnBreak` in the log.
5. After Ogremon breaks, chain Greymon's follow-up if it fires.
6. Continue until `[VICTORY]`.

**Expected observations:**
- [ ] At least **one `OnBreak` event** appears in the event log for Ogremon.
- [ ] Mini-boss (Ogremon) is KO'd **before all Goblimons** clear (Ogremon has more HP, but the combination of 2 Goblimons should survive at least one Ogremon break cycle).
- [ ] Encounter resolves in **3–5 turns** (R083 mini-boss band).
- [ ] DORUgamon's Form Identity (`OnFirstSkillCastWithTag(Dark)`) fires at least once: look for `FormIdentityTriggered` or `EnergyGained` event on DORUgamon's first Dark-tagged skill cast.
- [ ] `[VICTORY]` exits cleanly.

**Fail signals:**
- Ogremon never breaks → Angemon not hitting with Light tag / T03 rebalance regression.
- Encounter takes 6+ turns → recheck Ogremon/Goblimon HP values in `units.ron`.
- Goblimons are KO'd before Ogremon → acceptable if Ogremon still breaks during the encounter.

---

### 3. Encounter 3 — BossEncounter / Devimon (boss tier, ~15 minutes)

**Setup:**
```bash
cargo run --bin combat_cli
```
- Encounter: **BossEncounter** (Devimon, Armored, tempo-resistant, `resists: [Fire, Ice]`)
- Recommended party: Greymon · Angemon · DORUgamon · Patamon
  - Greymon deals Fire toughness hits (Fire IS a Devimon weakness for toughness).
  - Angemon deals Light hits (Light IS a Devimon weakness for both HP and toughness).
  - DORUgamon Form Identity fires ToughnessHit on first Dark skill — contributes to toughness drain.
  - Patamon provides healing support and Light secondary pressure.
  - **Avoid** Gabumon / Garurumon (Ice) as primary attackers — Devimon `resists: [Ice]` for HP damage.

**Steps:**
1. Select `BossEncounter` at the encounter prompt.
2. Select the recommended party.
3. Turn 1: use DORUgamon's skill (`power_metal` or `cannonball`) → Form Identity should fire, draining Devimon toughness.
4. Turn 2: use Greymon's Fire attack or Angemon's `angemon_basic` targeting Devimon → watch for `[EVENT] OnBreak` on toughness crossover.
5. After first break: verify `BreakSealApplied` event appears (Devimon is Armored → Break Seal mechanic, see S07).
6. Continue attacking; use skills on SP-positive turns; fire ultimate when `ULT` gauge is ready.
7. Continue until `[VICTORY]` or `[DEFEAT]`.

**Expected observations:**
- [ ] **Form Identity fires at least once**: look for `FormIdentityTriggered` (or `EnergyGained` on DORUgamon) in the event log within the first 2 turns.
- [ ] **Break Seal applied after first break**: look for `BreakSealApplied` event after Devimon's first `OnBreak` event. The dashboard TGH bar should read `0/35` after break, then recover.
- [ ] **At least one ultimate fires**: any ally whose ULT gauge (`ULT: X/100`) reaches the trigger threshold should fire their ultimate; verify `UltimateUsed` or `OnActionResolved` event with ultimate intent.
- [ ] **Tempo Resistance behavior is observable**: applying Slow/Freeze via Garurumon or Kyubimon (if in party) should show reduced effect on Devimon's speed (tempo-resistant units resist Delay stacking).
- [ ] Encounter resolves in **4–7 turns** (R083 boss band).
- [ ] `[VICTORY]` exits cleanly (or `[DEFEAT]` — note if defeat occurred).

**Fail signals:**
- No `OnBreak` within 4 turns → toughness-break mechanic regression or wrong weakness tags.
- No `BreakSealApplied` after break → S07 Break Seal wiring regression.
- Form Identity never fires after turn 1 DORUgamon skill → Form Identity listener regression.
- Encounter resolved in <4 turns → Devimon HP too low, rebalance needed.
- Encounter still active after 7 turns → Devimon HP too high or damage output too low.

---

## Subjective Rubric Checklist

After completing all three encounters, rate each dimension (tick if satisfactory):

- [ ] **TTK pacing feels right**: minion wave feels quick/cleanup; mini-boss has a satisfying break moment; boss feels like a sustained tactical challenge with escalating pressure.
- [ ] **Form Identity firings are visible and impactful**: the event log clearly shows when Form Identity triggers; the follow-up action visibly changes the combat state (extra damage, energy granted, etc.).
- [ ] **Break Seal correctness reads clearly**: after Devimon's first break, the dashboard's TGH bar clearly shows the sealed state; subsequent turns show the re-arming behavior.
- [ ] **Tempo Resistance behavior is observable**: if status effects (Slow/Freeze) are applied to Devimon, the resistance is legible in the dashboard or event log.
- [ ] **Status effects have legible impact**: applying Burn/Slow/Freeze to regular enemies shows visible HP/speed changes over turns; the event log is not flooded with noise.
- [ ] **Event log is readable under pressure**: in the boss encounter with 4+ allies, the `[EVENT]` log output is parseable — events are not truncated or interleaved in a confusing way.

---

## Verdict

**Verdict (circle one):** `pass` / `fail` / `pass-with-followups`

**Rationale:**

_______________________________________________________________________________

_______________________________________________________________________________

**Tester signature / initials:** ___________________________

---

## Failure Signals (global)

- Any `cargo run --bin combat_cli` panic with a Rust backtrace → engineering blocker.
- Dashboard shows no enemies after encounter starts → bootstrap wiring issue (T01 regression).
- `[DEFEAT]` on boss encounter with recommended party and competent play → balance regression.
- `cargo test` fails after the playthrough → test suite regression.

---

## Requirements Proved By This UAT

- R083 — TTK targets (minion 2–3, mini-boss 3–5, boss 4–7) feel balanced in real play, not just in automated fixtures.
- Form Identity (S08) — fires correctly and is visible in the event log during a live CLI session.
- Break Seal (S07) — applied on Armored Devimon's first break; observable by human operator.
- Enemy roster (T01) — Goblimon/Ogremon/Devimon all spawn correctly under the three preset compositions.

## Not Proven By This UAT

- DNA Chips / Tamer Gauge (out of scope for M011 — planned for M012).
- Enemy counterplay traits / status immunity interactions beyond tempo resistance.
- Multi-floor roguelite meta-loop (not yet implemented).
- Windowed egui UI correctness (UAT is CLI-only).
- Multiplayer or network correctness (not applicable).

---

## Notes for Tester

- **JSONL stream**: run `BEVYROGUE_JSONL=1 cargo run --bin combat_cli` if you want machine-readable event output alongside the dashboard. Each JSONL line is a `CombatEvent` record.
- **Non-interactive default**: if you run the CLI piped (non-terminal), it auto-selects the first 4 allies and defaults to `BossEncounter`. This is by design for CI.
- **Devimon Fire/Ice quirk**: Devimon both *resists* Fire/Ice (reduced HP damage) and is *weak* to Fire (toughness break). Greymon hits hard on toughness but sub-optimal on HP — Light (Angemon/Patamon) hits both. Intentional design.
- **angemon_basic ToughnessHit**: bumped from 8 → 20 during T03 rebalance to guarantee the mini-boss break. Deliberate numerical choice, not a bug.
