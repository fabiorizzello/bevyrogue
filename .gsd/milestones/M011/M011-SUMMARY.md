---
id: M011
title: "Combat Architecture & Synergistic Roster (v5.3 alignment)"
status: complete
completed_at: 2026-04-30T17:22:52.156Z
key_decisions:
  - M011 combat core is accepted as headless-tested and merged; UI-readiness gaps are intentionally moved to M012 rather than blocking M011 closure.
  - M012 owns data-driven legality and UI-affordance truthfulness before graphical UI work.
key_files:
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/events.rs
  - src/combat/damage.rs
  - src/combat/toughness.rs
  - src/combat/resistance.rs
  - src/combat/energy.rs
  - src/combat/follow_up.rs
  - src/bin/combat_cli.rs
  - assets/data/units.ron
  - assets/data/skills.ron
  - tests/pipeline_dispatch.rs
  - tests/form_identity.rs
  - tests/tempo_resistance.rs
  - tests/toughness_categories.rs
  - tests/scenario_boss_ttk.rs
  - tests/scenario_miniboss_ttk.rs
  - tests/scenario_minion_ttk.rs
lessons_learned:
  - Auto-mode milestone completion can miss implementation changes when the branch/worktree state was merged manually; use GSD milestone completion after verifying merged code on the integration branch.
  - Windowed/UI code can compile-drift from headless combat changes; future combat milestones that affect turn order or targeting should include a windowed compile check if UI is in scope.
---

# M011: Combat Architecture & Synergistic Roster (v5.3 alignment)

**M011 combat core and roster architecture are merged, headless-tested, and closed; UI-readiness gaps are carried forward into M012.**

## What Happened

M011 delivered the combat architecture and synergistic roster alignment planned around combat_design.md v5.3. The action lifecycle now emits declared/pre-apply/applied/resolved events; Damage Tags and Attribute Triangle modifiers are implemented and tested; EvoStage schema, SP/Child mechanics, AV/Tempo Resistance, Toughness categories, Break Seal, Form Identity, roster data, scenario TTK tests, and the CLI playtest harness are present in code and assets. All nine slices are complete in GSD and their task counts are fully done. During post-merge review we identified UI-readiness gaps that do not invalidate the headless combat core but must be solved before a player-facing UI: legality/target query, enemy-only Toughness, TargetShape truthfulness, Energy cap wiring, Tamer/Commands declarations, and windowed adapter drift. Those are captured in M012 rather than reopening M011.

## Success Criteria Results

- Integration tests green: fresh `cargo test-dev` passed with unit tests, integration tests, and doc tests all successful.
- Requirements R070, R071, R073, R075-R083 are validated or intentionally carried forward with caveats captured in M012.
- `combat_design.md` sections 1, 2, 5, 6, and 9 are materially represented in code/tests; known mismatches are documented for M012 doc/data alignment.
- Decisions are recorded through D054, including M012 follow-up scope.
- CLI playtest harness exists; full human UAT is not re-run here, but deterministic scenario fixtures pass.

## Definition of Done Results

- All M011 slices S01-S09 are complete in GSD DB with all tasks done.
- Combat implementation exists in merged code on `master` via commit `97ba0fe merge: integrate M011 combat architecture` and follow-up snapshot `5cd3f48`.
- Fresh verification: `cargo test-dev` completed successfully in this message.
- M012 was created to handle UI-readiness gaps found after M011: data-driven legality, enemy-only Toughness, TargetShape truthfulness, Energy cap wiring, Tamer/Command affordance declarations, and windowed compile alignment.

## Requirement Outcomes

- R070 validated — action lifecycle events declared/pre/apply/resolved covered by `tests/pipeline_dispatch.rs`.
- R071 validated — follow-up/reaction ordering covered by pipeline/follow-up tests.
- R073 validated with caveat — SP cap is enforced; Energy cap wiring is explicitly re-audited in M012 via R085.
- R075/R076 validated — DamageTag and Attribute Triangle tests pass.
- R077 validated — EvoStage/evo_line/evolves_to schema and asset parsing tests pass.
- R078 validated — TempoResistance and AV threshold tests pass.
- R079 validated with UI-readiness caveat — Toughness categories/Break Seal tests pass; enemy-only display/semantics are carried to M012.
- R080 validated — Form Identity tests for all Adult forms pass.
- R081 validated with caveat — Child SP discount exists; Tamer Gauge boost dependency is carried to M012 because Tamer Gauge is not implemented.
- R082 validated — CLI harness exists and builds.
- R083 validated — deterministic minion/miniboss/boss TTK scenario tests pass.
- R084/R085 introduced for M012 to cover legality/UI-readiness follow-up scope.

## Deviations

None.

## Follow-ups

Proceed to M012 — Data-driven skill legality and UI-readiness query surface. M012 should address known gaps before polished UI: shared legality/target query, windowed compile, enemy-only Toughness, TargetShape behavior/gating, Energy caps in pipeline, and declarative status for Tamer/Enemy counterplay affordances.
