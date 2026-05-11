# S04: Headless interactive CLI (combat playtest harness) — UAT

**Milestone:** M011
**Written:** 2026-04-27T19:01:40.625Z

# S04: Headless interactive CLI — UAT

**Milestone:** M011
**Written:** 2026-04-27

## UAT Type

- UAT mode: live-runtime + human-experience
- Why this mode is sufficient: The slice's primary deliverable is an interactive terminal harness. Artifact-driven tests verify compilation and non-interactive paths; human-experience UAT is required to verify the `inquire` menus render and respond correctly to real keyboard input.

## Preconditions

- `cargo build --bin combat_cli` succeeds.
- Terminal supports ANSI output (any modern shell).
- `assets/data/units.ron` exists and contains ally units.

## Smoke Test

Run `cargo run --bin combat_cli` and confirm the roster selection prompt appears listing available ally units.

## Test Cases

### 1. Roster selection enforces exactly 4 allies

1. Run `cargo run --bin combat_cli`.
2. At the MultiSelect prompt, select fewer than 4 allies and confirm.
3. **Expected:** An error message appears ("You must select exactly 4 allies") and the prompt re-displays.
4. Select exactly 4 allies and confirm.
5. **Expected:** "Bootstrap successful. Party: [...]" prints and combat dashboard appears.

### 2. Combat dashboard renders on each turn

1. Complete roster selection (4 allies).
2. Observe the initial dashboard before the first action prompt.
3. **Expected:** Dashboard shows SP pool (e.g. "SP: 3/10"), turn order, and per-unit rows with HP/MaxHP, ULT charge, and Toughness.

### 3. Action selection — Basic attack

1. At the action prompt ("Choose action for <unit>"), select "Basic".
2. At the target prompt, select any unit.
3. **Expected:** CombatEvent stream shows `OnActionDeclared`, `OnDamageDealt`, `OnActionResolved`. Dashboard updates on next turn showing changed HP.

### 4. Action selection — Skill

1. When SP ≥ skill cost, select "Skill(s)" at the action prompt.
2. Select a skill and a target.
3. **Expected:** `OnSkillCast` event appears in the log with the correct skill ID.

### 5. Ultimate — gated by charge

1. At the action prompt before ULT charge reaches threshold, confirm "Ultimate" option is absent.
2. Allow several turns of UltGain events to accumulate.
3. When charge ≥ threshold, confirm "Ultimate (charge/max)" option appears.
4. Select it and pick a target.
5. **Expected:** `OnUltimateUsed` (or equivalent) event appears in the stream.

### 6. Turn cycling

1. Let 3+ full turns pass.
2. **Expected:** Turn order in the dashboard rotates each turn; all 4 selected allies appear in the queue.

## Edge Cases

### Cancellation during roster selection

1. Run `cargo run --bin combat_cli`.
2. Press Escape at the MultiSelect prompt (cancellation).
3. **Expected:** Falls back to first 4 allies in roster order; "Bootstrap successful" prints and combat begins.

### Non-interactive CI path

1. Run `echo "" | timeout 12 cargo run --bin combat_cli 2>/dev/null`.
2. **Expected:** Exit code 0; output contains "Non-interactive mode", a bootstrap line, and at least one rendered dashboard with SP/TurnOrder/HP/ULT/TGH values.

## Failure Signals

- No roster prompt appears → TTY detection misconfigured.
- "Bootstrap successful" never prints → `SelectedAllies` resource not consumed by `bootstrap_system`.
- Dashboard shows all HP as 0/0 → ECS component queries broken.
- No CombatEvent lines appear → `event_logger_system` ordering issue (check chain order vs `combat_dashboard_system`).

## Not Proven By This UAT

- Combat against enemies (current bootstrap spawns allies only; ally-on-ally damage is a harness artifact).
- Correct numerical balance (verified in S09).
- Victory/Defeat condition with real enemy party.

## Notes for Tester

The encounter currently has no enemies — all attacks target allies. This produces "Target is a Commander" log entries for Taichi (the commander slot). This is expected and does not indicate a bug in the CLI machinery. Enemy encounter wiring is deferred to S09.
