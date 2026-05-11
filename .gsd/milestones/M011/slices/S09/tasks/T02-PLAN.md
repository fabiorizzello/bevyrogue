---
estimated_steps: 23
estimated_files: 3
skills_used: []
---

# T02: Lock R083 TTK targets in three deterministic scenario fixtures

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

## Inputs

- `assets/data/units.ron`
- `assets/data/skills.ron`
- `src/combat/bootstrap.rs`
- `src/bin/combat_cli.rs`
- `tests/encounter_e2e.rs`
- `tests/toughness_categories.rs`
- `src/combat/events.rs`

## Expected Output

- `tests/scenario_minion_ttk.rs`
- `tests/scenario_miniboss_ttk.rs`
- `tests/scenario_boss_ttk.rs`

## Verification

cargo build --tests && cargo test --test scenario_minion_ttk scenario_miniboss_ttk scenario_boss_ttk -- --nocapture 2>&1 | tee /tmp/s09_t02.log; grep -q 'panic\|did not compile' /tmp/s09_t02.log && exit 1; echo 'Tests run; failures expected pre-rebalance'

## Observability Impact

- Signals added/changed: tests assert on existing CombatEvent kinds (OnBreak, EnergyGained, OnDamageDealt) — no new signals.
- How a future agent inspects this: failing test's assertion message names the encounter, turn count, expected band, and event counts; rerun with `-- --nocapture` to see the per-turn dump if debug prints are added.
- Failure state exposed: turn-count out-of-band → message includes actual count vs. R083 band.
