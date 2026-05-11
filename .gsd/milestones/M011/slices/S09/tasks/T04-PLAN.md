---
estimated_steps: 12
estimated_files: 2
skills_used: []
---

# T04: UAT script + 30-minute manual playthrough sign-off + slice assessment

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

## Inputs

- `src/bin/combat_cli.rs`
- `assets/data/units.ron`
- `.gsd/milestones/M011/slices/S09/S09-RESEARCH.md`

## Expected Output

- `.gsd/milestones/M011/slices/S09/S09-UAT.md`
- `.gsd/milestones/M011/slices/S09/S09-ASSESSMENT.md`

## Verification

test -f .gsd/milestones/M011/slices/S09/S09-UAT.md && test -f .gsd/milestones/M011/slices/S09/S09-ASSESSMENT.md && grep -q 'MinionWave\|MiniBossEncounter\|BossEncounter' .gsd/milestones/M011/slices/S09/S09-UAT.md && grep -q -E '^## ' .gsd/milestones/M011/slices/S09/S09-UAT.md && echo 'OK'
