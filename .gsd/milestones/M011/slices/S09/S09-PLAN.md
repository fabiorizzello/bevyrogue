# S09: Numerical rebalance pass + UAT scenarios

**Goal:** Close M011 by extending the enemy roster (minion + mini-boss), wiring the CLI to spawn three canonical encounter presets, locking the R083 TTK targets in three deterministic scenario tests, performing a numerical rebalance in RON until those tests pass, finalizing combat_design.md sez. 9, and signing off a 30-minute UAT through combat_cli.
**Demo:** tests/scenarios/ con 3 fixture verdi per TTK target; UAT manuale 30 minuti firmato dal product owner via combat_cli; combat_design.md sez. 9 finale

## Must-Haves

- After this: tests/scenarios/ holds 3 fixtures verdi for boss / mini-boss / minion TTK targets (R083 bands: 4-7, 3-5, 2-3 turns).
- After this: cargo run --bin combat_cli prompts for an encounter preset (Minion / MiniBoss / Boss) and spawns enemies accordingly.
- After this: assets/data/units.ron contains at least one minion (toughness Standard, no Form Identity) and one mini-boss; current Devimon retains boss role.
- After this: docs/combat_design.md sez. 9 is finalized (Form Identity wiring annotated as shipped in S08).
- After this: a signed S09-UAT.md captures the 30-minute manual playthrough verdict; S09-ASSESSMENT.md records the slice verdict.
- All 21+ integration binaries green; R083 marked validated.

## Proof Level

- This slice proves: - This slice proves: final-assembly (M011 closure: real CLI + real RON + real engine + manual UAT).
- Real runtime required: yes (combat_cli must drive three encounter presets end-to-end).
- Human/UAT required: yes (R083 has a subjective 30-minute play component that automated fixtures cannot capture).

## Integration Closure

- Upstream surfaces consumed: bootstrap_encounter (S04), CombatEvent bus + form_identity_listener_system (S08), DataPlugin RON loaders (S03), TempoResistance + Toughness categories (S06/S07), CLI inquire prompts + SelectedAllies resource (S04 T05).
- New wiring introduced in this slice: SelectedEncounter resource + inquire::Select preset prompt in combat_cli main(); EncounterPreset enum + bootstrap_encounter signature extension to populate enemies; tests/scenario_*_ttk.rs binaries wiring DataPlugin + full combat system chain at integration test scope.
- What remains before the milestone is truly usable end-to-end: nothing — S09 is the M011 closure slice; after sign-off the milestone moves to validation.

## Verification

- Runtime signals: existing CombatEvent bus (OnDamageDealt with damage_tag, OnBreak, OnKO, OnActionResolved, EnergyGained, TurnAdvance) — no new event kinds needed; scenario tests count these via MessageCursor<CombatEvent>.
- Inspection surfaces: BEVYROGUE_JSONL=1 stream from jsonl_logger_system; CLI dashboard already displays HP/SP/TGH/ULT per unit per tick; encounter preset choice logged at bootstrap.
- Failure visibility: scenario test failures self-diagnose via dual-surface assertions (event count + component state per MEM055 pattern).
- Redaction constraints: none.

## Tasks

- [x] **T01: Extend enemy roster + wire EncounterPreset into bootstrap and CLI** `est:3h`
  Add a minion and a mini-boss UnitDef to `assets/data/units.ron`, introduce an `EncounterPreset` enum (`MinionWave` / `MiniBossEncounter` / `BossEncounter`), extend `bootstrap_encounter` to accept a preset and populate the `enemies` field of `EncounterComposition`, and add an `inquire::Select` encounter prompt to `combat_cli` (with a non-interactive default of `BossEncounter` to keep CI smoke-tests green). This is purely additive — no engine logic changes, only data + bootstrap signature + CLI UX.

Canonical enemy fixtures (numbers are starting points; T03 will rebalance):
- **Minion** (UnitId 102, e.g. `Goblimon`): HP ~120, toughness 0 (no break bar), Standard category, attribute Virus, basic_damage_tag Physical, no form_identity, no follow_up, tempo_resistant=false, 1 basic + 1 cheap skill, low ult_cap.
- **Mini-boss** (UnitId 103, e.g. `Ogremon`): HP ~280, toughness ~6, Standard or Armored, attribute Data, no form_identity, no tempo resistance, 1 basic + 2 skills + 1 ultimate. Resists none (or 1 weak resist) so the rebalance has room.
- Devimon (UnitId 101) stays the boss anchor unchanged.

Encounter presets:
- `MinionWave` → 3 minions (UnitId 102 ×3).
- `MiniBossEncounter` → 1 mini-boss (103) + 2 minions (102).
- `BossEncounter` → Devimon (101) + 0 supporting minions (matches current behavior the closest).

**bootstrap_encounter** changes: add a second parameter `preset: EncounterPreset`. Populate `enemies` by looking up the preset's enemy ids in the roster (returning `SelectionError::UnknownRookie { id }` if missing — reuse existing error variant). All call sites must pass a preset; the unit test in `bootstrap.rs` (and any test in `tests/bootstrap_spawn_composition.rs`) must be updated.

**CLI** changes: in `src/bin/combat_cli.rs::main()`, after the party `MultiSelect`, add an `inquire::Select` for `EncounterPreset` (3 labels). Non-interactive branch defaults to `BossEncounter`. Insert a new `SelectedEncounter(EncounterPreset)` resource and have `bootstrap_system` consume it when calling `bootstrap_encounter`.

Keep this task strictly mechanical — do not retune numbers here. Verify by parsing the new units.ron and by running the full integration suite (existing tests must still pass; new minion/mini-boss must round-trip through `roster_catalog` invariants).
  - Files: `assets/data/units.ron`, `src/combat/bootstrap.rs`, `src/bin/combat_cli.rs`, `tests/bootstrap_spawn_composition.rs`, `tests/roster_catalog.rs`
  - Verify: cargo check && cargo test --test roster_catalog && cargo test --test bootstrap_spawn_composition && cargo test && echo 'OK'

- [x] **T02: Lock R083 TTK targets in three deterministic scenario fixtures** `est:4h`
  Create three integration test binaries that load the real `assets/data/units.ron` + `assets/data/skills.ron` (via `DataPlugin` or synchronous `ron::from_str` mirroring the S04 T05 pattern in `combat_cli.rs:336-344`), spawn each canonical encounter via `bootstrap_encounter(roster, request, preset)` (T01 output), and run a hardcoded scripted action sequence representing competent play. Assert the encounter ends in `CombatPhase::Victory` within the R083 turn band.

Flat-file layout (decided per research §Open Risks): three top-level test binaries, no `tests/scenarios/main.rs` driver:
- `tests/scenario_minion_ttk.rs` — `MinionWave`, expected turns 2–3, no break required, no Form Identity required.
- `tests/scenario_miniboss_ttk.rs` — `MiniBossEncounter`, expected turns 3–5, at least one `OnBreak` event.
- `tests/scenario_boss_ttk.rs` — `BossEncounter` (Devimon), expected turns 4–7, at least one `OnBreak` and at least one Form Identity `EnergyGained` event.

**Test pattern (mirror `tests/encounter_e2e.rs` + `tests/toughness_categories.rs`):**
1. Build `App::new()` with the same plugins/resources/messages/systems chain as `src/bin/combat_cli.rs::main()` (lines 393-433), minus `player_action_system`, `combat_dashboard_system`, `event_logger_system`, `jsonl_logger_system`, `timeout_system`. The system chain must include: `bootstrap_system`, `resolve_action_system`, `follow_up_listener_system`, `form_identity_listener_system`, `resolve_follow_up_action_system`, `ult_accumulation_system`, `flush_ult_gain_system`, `advance_turn_system`, `check_victory_system`.
2. Insert `SelectedAllies` (4 picks chosen for the encounter — e.g. Greymon line for boss to leverage Form Identity Fire trigger; pick non-Fire/Ice for boss because Devimon resists Fire+Ice — use Light/Electric/Physical hitters), and `SelectedEncounter(<preset>)`.
3. Pre-load RON synchronously and seed `UnitRoster` so bootstrap fires on the first tick (do not depend on `DataPlugin` async readiness — use the pre-App load pattern from `combat_cli.rs:336-344`).
4. Drive the encounter with a hardcoded `ActionIntent` script: write each turn's intended action via `MessageWriter<ActionIntent>` when `CombatPhase::WaitingAction` and the active actor is the scripted unit. Set up a small per-test driver system or a manual `app.update()` loop that injects the next intent on each `WaitingAction` tick.
5. Bound the loop with a turn counter (max 12 turns; fail with assertion message naming the encounter and turn count if exceeded).
6. Use `MessageCursor<CombatEvent>` (per MEM055 pattern in `tests/toughness_categories.rs`) to count `OnBreak` and `EnergyGained` events. Drain the cursor between updates to avoid stale-event re-counting (MEM029).

**Assertions per test:**
- `CombatPhase::Victory` reached.
- Turn count is within the per-tier R083 band (use `assert!(t >= lo && t <= hi, "...")` with explicit message including the actual count).
- Boss + mini-boss tests: `OnBreak` event count ≥ 1.
- Boss test: `EnergyGained` event count ≥ 1 (Form Identity fired at least once).
- No panics.

**Crucial:** these tests are expected to FAIL initially against current numbers — they define the rebalance target for T03. That is by design (test-first rebalance). A failing test here is acceptable as long as the test itself runs (compiles, drives the loop, fails on the turn-count assertion or victory assertion). Mark in S09-PLAN that T02 leaves these red until T03 closes them.

**Action script picks (recommendation):**
- Boss script: party = Greymon (Fire), Patamon→Angemon (Light), Tentomon→Kabuterimon (Electric), Gabumon→Garurumon (Ice — accept Devimon resist for variety), but adjust if Light/Electric is needed for TTK. Actions per turn: highest-impact attacker uses skill if SP available else basic; ultimate when ready.
- Mini-boss script: same party, target the mini-boss first (focus fire); minions die incidentally.
- Minion script: all basics; verify TTK from base damage alone.
  - Files: `tests/scenario_minion_ttk.rs`, `tests/scenario_miniboss_ttk.rs`, `tests/scenario_boss_ttk.rs`
  - Verify: cargo build --tests && cargo test --test scenario_minion_ttk scenario_miniboss_ttk scenario_boss_ttk -- --nocapture 2>&1 | tee /tmp/s09_t02.log; grep -q 'panic\|did not compile' /tmp/s09_t02.log && exit 1; echo 'Tests run; failures expected pre-rebalance'

- [x] **T03: Numerical rebalance pass — turn T02 fixtures green and finalize combat_design sez. 9** `est:5h`
  Iterate on `assets/data/units.ron` and `assets/data/skills.ron` (HP, toughness, base damage, SP costs, ultimate triggers, status durations) until all three T02 scenario fixtures pass within the R083 bands. Decide upfront whether to wire `BonusToughnessDamage` and `BonusDamageVsAttribute` into `apply_effects`, or remove the unused variants — document the call in S09-ASSESSMENT.md and (if removing) record a superseding decision in `.gsd/DECISIONS.md`.

**Recommended decision (default plan):** keep the S08 `fire-a-separate-skill` workaround for DORUgamon and Angemon. Strip `BonusToughnessDamage(i32)` and `BonusDamageVsAttribute { attribute, bonus_pct }` variants from `src/data/skills_ron.rs::Effect` and the matching round-trip test. Rationale: minimum-change path, S08 already proves the workaround feels correct, and dead variants are MVP debt. If during rebalance the workaround visibly under-delivers (e.g. DORUgamon ramp is too shallow), switch to wiring — this requires extending `ResolvedAction` and `apply_effects` in `src/combat/resolution.rs`, plus tests covering both effect paths. **Document the chosen path early in T03 execution** so the rest of the task scope is bounded.

**Rebalance levers (apply in order, lowest-blast-radius first):**
1. Skill base damage / SP cost in `skills.ron` (preferred — local, testable, no cross-unit ripple).
2. Unit HP / toughness / weakness lists in `units.ron` (per-tier; minion HP for minion-tier TTK, mini-boss for mini-boss-tier, etc.).
3. Ultimate trigger / cap / charge per event for ult-pacing.
4. Form Identity grant amounts (e.g. GrantEnergy(5) → 7 if boss tier needs more energy ramp).

**Iteration loop:** rerun the three scenario tests, read the actual turn count from the assertion message, adjust the smallest lever, retest. Avoid mass tuning across multiple files in a single edit — single-lever iterations make the cause/effect legible. Each commit-worthy intermediate state should leave `cargo test` green for non-scenario tests; only the three scenario tests are allowed to be red mid-rebalance.

**Side hazards to verify don't regress:**
- `cargo test --test triangle_matchup` — triangle multipliers (D043 ratios).
- `cargo test --test ultimate_meter` — ult charge accumulation.
- `cargo test --test form_identity` — all 6 Adults still trigger correctly.
- `cargo test --test toughness_categories` — break category requirements.
- `cargo test --test sp_economy` — SP cap and child discount.

**Final step — combat_design.md sez. 9 finalization:** Update `docs/combat_design.md` sez. 9 (Form Identity) to annotate that the framework is wired in S08 (cite the 6 wired Adults: Greymon Fire/GrantEnergy, Garurumon Ice/GrantEnergy, Kabuterimon Electric/GrantEnergy, Kyubimon Freeze-status/SelfAdvance, DORUgamon Dark-skill/separate-toughness-skill, Angemon Virus-attack/separate-light-skill). If `BonusToughnessDamage` / `BonusDamageVsAttribute` were stripped, add a short note explaining the `fire-a-separate-skill` choice and pointing at the relevant decision id.

**Definition of done:** all three scenario tests green; full `cargo test` green (29+ binaries); `docs/combat_design.md` sez. 9 has a 'wired in M011' annotation; if any decision was made (wire vs strip), `.gsd/DECISIONS.md` has a new D04X or D050+ entry recording it.
  - Files: `assets/data/units.ron`, `assets/data/skills.ron`, `src/data/skills_ron.rs`, `src/combat/resolution.rs`, `docs/combat_design.md`, `.gsd/DECISIONS.md`
  - Verify: cargo test --test scenario_minion_ttk && cargo test --test scenario_miniboss_ttk && cargo test --test scenario_boss_ttk && cargo test && grep -q -i 'wired in M011\|wired in S08\|S08 implementation' docs/combat_design.md && echo 'OK'

- [x] **T04: UAT script + 30-minute manual playthrough sign-off + slice assessment** `est:2h`
  Author `.gsd/milestones/M011/slices/S09/S09-UAT.md` as a structured 30-minute UAT script that walks the tester through three encounters via `cargo run --bin combat_cli` (one minion wave, one mini-boss, one boss) with explicit observation checkpoints. Capture the verdict in `.gsd/milestones/M011/slices/S09/S09-ASSESSMENT.md`.

**S09-UAT.md structure (use the template at `~/.gsd/agent/extensions/gsd/templates/uat.md` as a starting point):**
- Header: tester name slot, date slot, build SHA slot.
- Pre-run checklist: `cargo build --bin combat_cli` succeeds; `BEVYROGUE_JSONL=1 cargo run --bin combat_cli` produces a JSONL stream alongside the dashboard.
- Encounter 1 — MinionWave: party = any 4 Children, expected TTK 2-3 turns, expected observation = basic attacks suffice, no Form Identity firings necessary.
- Encounter 2 — MiniBossEncounter: party = mix with at least one breaker (Tentomon/Kabuterimon for Standard category if applicable), expected TTK 3-5 turns, expected observation = at least one Break event, mini-boss falls before all minions.
- Encounter 3 — BossEncounter (Devimon): party = Greymon + 3 others (Light hitter recommended — Patamon/Angemon — since Devimon resists Fire+Ice), expected TTK 4-7 turns, expected observations = (a) Form Identity fires at least once and reads cleanly in the event log, (b) Break Seal applied after Devimon's first break (Armored category, see S07), (c) ultimate fires for at least one ally.
- Subjective rubric checklist (per research §UAT subjectivity): TTK feels paced (not too fast/slow), Form Identity firings are visible and impactful, Break Seal correctness reads clearly, Tempo Resistance behavior on Devimon is observable, status effects (Slow/Burn/Freeze) have legible impact.
- Verdict slot: pass / fail / pass-with-followups + rationale.

**S09-ASSESSMENT.md:** record (a) UAT verdict from the playthrough, (b) any deviations (e.g. unwired Bonus* variant decision from T03), (c) any follow-ups for M012 (e.g. Tamer Gauge, DNA Chips, Enemy Counterplay traits — already documented in M011-ROADMAP §Out of scope but reaffirm any new ones), (d) confirmation that all 21+ integration binaries are green.

This task is human-gated: the agent prepares the script and produces an empty assessment scaffold; the actual playthrough verdict and rubric checkmarks are filled in by the human operator (the milestone owner). Auto-mode cannot sign off — the file must be left in a state ready for human completion. Document this in S09-ASSESSMENT.md by leaving the verdict field as `<awaiting human sign-off>` if running in auto-mode.

Note: research § flags that the `is_terminal` branch in `combat_cli.rs:349` defaults non-interactive runs to a fixed party; T01 already added the encounter preset prompt with a non-interactive default. This guarantees CI smoke-tests still pass without UAT input.
  - Files: `.gsd/milestones/M011/slices/S09/S09-UAT.md`, `.gsd/milestones/M011/slices/S09/S09-ASSESSMENT.md`
  - Verify: test -f .gsd/milestones/M011/slices/S09/S09-UAT.md && test -f .gsd/milestones/M011/slices/S09/S09-ASSESSMENT.md && grep -q 'MinionWave\|MiniBossEncounter\|BossEncounter' .gsd/milestones/M011/slices/S09/S09-UAT.md && grep -q -E '^## ' .gsd/milestones/M011/slices/S09/S09-UAT.md && echo 'OK'

## Files Likely Touched

- assets/data/units.ron
- src/combat/bootstrap.rs
- src/bin/combat_cli.rs
- tests/bootstrap_spawn_composition.rs
- tests/roster_catalog.rs
- tests/scenario_minion_ttk.rs
- tests/scenario_miniboss_ttk.rs
- tests/scenario_boss_ttk.rs
- assets/data/skills.ron
- src/data/skills_ron.rs
- src/combat/resolution.rs
- docs/combat_design.md
- .gsd/DECISIONS.md
- .gsd/milestones/M011/slices/S09/S09-UAT.md
- .gsd/milestones/M011/slices/S09/S09-ASSESSMENT.md
